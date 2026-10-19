//! The API, all JSON:
//!
//! | Request | Does |
//! |---|---|
//! | `GET /v1/status` | app, version, library |
//! | `POST /v1/add` `{"input", "project"?}` | `quire add <input>` |
//! | `POST /v1/page` `{"url", "html", "project"?}` | add from the page the browser shows |
//! | `POST /v1/pdf?reference=<key>` (PDF body) | attach the PDF to a reference |
//! | `POST /v1/pdf?url=<page>&project=<path>` (PDF body) | add the paper the PDF is, attach it |
//!
//! Errors are `{"error": "…"}` with 400 (bad input), 401 (token), 404 (no such reference or
//! project), 422 (not a PDF, nothing found in it), 502 (a source failed) or 500.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use quire_engine::{Added, CoreError, Engine, EngineError, Reference, SlugPath};
use serde::Deserialize;
use serde_json::{Value, json};

/// Largest request body: a PDF.
const MAX_BODY: usize = 200 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    engine: Arc<Engine>,
    token: Arc<str>,
}

pub fn router(engine: Arc<Engine>, token: String) -> Router {
    let state = AppState { engine, token: token.into() };
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/add", post(add))
        .route("/v1/page", post(page))
        .route("/v1/pdf", post(pdf))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .with_state(state)
}

/// Answers CORS preflights, checks the token, and adds CORS headers (browser extensions call
/// from their own origin).
async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let mut response = if request.method() == Method::OPTIONS {
        StatusCode::NO_CONTENT.into_response()
    } else if authorized(request.headers(), &state.token) {
        next.run(request).await
    } else {
        error(
            StatusCode::UNAUTHORIZED,
            "missing or wrong token (Authorization: Bearer <token>; see `quire serve --token`)",
        )
    };
    if let Some(origin) = origin {
        let headers = response.headers_mut();
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("authorization, content-type"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, OPTIONS"));
    }
    response
}

/// Compares the bearer token in constant time.
fn authorized(headers: &HeaderMap, token: &str) -> bool {
    let Some(given) =
        headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "))
    else {
        return false;
    };
    given.len() == token.len() && given.bytes().zip(token.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

async fn status(State(state): State<AppState>) -> Json<Value> {
    let library = state.engine.library();
    Json(
        json!({ "app": "quire", "version": env!("CARGO_PKG_VERSION"), "library": library.label(), "path": library.path }),
    )
}

#[derive(Deserialize)]
struct AddBody {
    input: String,
    project: Option<String>,
}

async fn add(State(state): State<AppState>, Json(body): Json<AddBody>) -> Response {
    let result = async {
        let project = parse_project(body.project.as_deref())?;
        state.engine.add(&body.input, project.as_ref()).await
    };
    added(&state, result.await).await
}

#[derive(Deserialize)]
struct PageBody {
    url: String,
    html: String,
    project: Option<String>,
}

async fn page(State(state): State<AppState>, Json(body): Json<PageBody>) -> Response {
    let result = async {
        let project = parse_project(body.project.as_deref())?;
        state.engine.add_from_page(&body.url, &body.html, project.as_ref()).await
    };
    added(&state, result.await).await
}

#[derive(Deserialize)]
struct PdfQuery {
    reference: Option<String>,
    /// Where the browser downloaded it from.
    url: Option<String>,
    project: Option<String>,
}

async fn pdf(State(state): State<AppState>, Query(query): Query<PdfQuery>, body: Bytes) -> Response {
    let engine = &state.engine;
    let source = query.url.clone().unwrap_or_else(|| "browser".into());
    let result = async {
        match &query.reference {
            Some(reference) => {
                let (reference, _) = engine.show(reference).await?;
                engine.attach_bytes(&reference, &body, &source, false).await
            }
            None => {
                let project = parse_project(query.project.as_deref())?;
                Ok(engine.import_pdf_bytes(&body, &source, project.as_ref()).await?.1)
            }
        }
    };
    match result.await {
        Ok(attached) => {
            exports(&state).await;
            let a = &attached.attachment;
            let value = json!({
                "cite_key": attached.reference.cite_key, "new": attached.new, "pages": a.pages, "warning": attached.verdict.reason(),
            });
            Json(value).into_response()
        }
        Err(e) => engine_error(&e),
    }
}

fn parse_project(text: Option<&str>) -> quire_engine::Result<Option<SlugPath>> {
    Ok(text.filter(|t| !t.is_empty()).map(str::parse).transpose()?)
}

/// The reply to an add: what was added (or found), and its issues.
async fn added(state: &AppState, result: quire_engine::Result<Added>) -> Response {
    match result {
        Ok(added) => {
            exports(state).await;
            let (status, reference) = match &added {
                Added::New(r) => ("added", r),
                Added::Existing(r) => ("existing", r),
            };
            let issues = state.engine.issues(reference).await.unwrap_or_default();
            Json(json!({ "status": status, "reference": summary(reference), "issues": issues })).into_response()
        }
        Err(e) => engine_error(&e),
    }
}

fn summary(reference: &Reference) -> Value {
    json!({ "id": reference.id, "cite_key": reference.cite_key, "title": reference.data.title(), "type": reference.data.kind })
}

/// Rewrites automatic exports after a change; failures are logged, not sent to the extension.
async fn exports(state: &AppState) {
    if let Err(error) = state.engine.run_auto_exports(false).await {
        tracing::warn!(%error, "automatic exports");
    }
}

fn engine_error(e: &EngineError) -> Response {
    let status = match e {
        EngineError::Core(CoreError::ReferenceNotFound(_) | CoreError::ProjectNotFound(_)) => StatusCode::NOT_FOUND,
        EngineError::Model(_) | EngineError::Core(CoreError::Model(_)) => StatusCode::BAD_REQUEST,
        EngineError::Format(_) | EngineError::Pdf(_) => StatusCode::UNPROCESSABLE_ENTITY,
        EngineError::Fetch(_) => StatusCode::BAD_GATEWAY,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    error(status, &e.to_string())
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}
