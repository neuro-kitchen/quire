//! The hub's side of a session, over any byte stream (a QUIC stream from [`crate::server`], a
//! pipe in tests): `Hello` first (the device's certificate, checked against the device the
//! connection authenticated), then requests until `Bye`. A person without an account may only
//! register with a code. Only sealed data is handled; nothing is decrypted here.

use std::sync::Arc;
use std::time::Duration;

use quire_sync::Certificate;
use quire_sync::hub::protocol::{
    self, ErrorCode, FileHeader, FileIds, Hello, Pushed, Reply, ReplyBody, Request, RequestBody, Welcome,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::store::{Account, HubError, Store};

/// How long a session may stay silent before the hub drops it.
const IDLE: Duration = Duration::from_secs(120);

/// How the hub runs.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Storage limit of new accounts in bytes (0: none).
    pub quota: u64,
}

/// What one session did (for the hub's log: counts only, never content).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub account: Option<i64>,
    pub batches_in: usize,
    pub batches_out: usize,
    pub files_in: usize,
    pub files_out: usize,
    /// The device ended the session (`Bye`, or it closed its side); `false`: the hub ended it (a
    /// refusal that stops the session, or silence), and the device may still be reading its reply.
    pub ended_by_device: bool,
}

/// The connecting device, once it said `Hello`.
struct Caller {
    user: String,
    device: String,
    library: String,
    author: String,
    account: Option<Account>,
}

/// Serves one session; `authenticated` is the device's network ID from the connection (`None`
/// over a pipe in tests: the certificate is then checked for its own device).
pub async fn serve<Rd, Wr>(
    store: &Arc<Store>,
    settings: &Settings,
    authenticated: Option<&str>,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<Summary, HubError>
where
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    let mut summary = Summary::default();
    let mut caller: Option<Caller> = None;
    loop {
        let request: Option<Request> = match tokio::time::timeout(IDLE, protocol::receive(reader)).await {
            Ok(Ok(request)) => request,
            Ok(Err(error)) => {
                tracing::debug!(%error, "unreadable request");
                return Ok(summary);
            }
            Err(_) => return Ok(summary),
        };
        let Some(Request { body: Some(body) }) = request else {
            summary.ended_by_device = true;
            return Ok(summary);
        };
        if matches!(body, RequestBody::Bye(_)) {
            summary.ended_by_device = true;
            return Ok(summary);
        }
        let reply = match handle(store, settings, authenticated, &mut caller, &mut summary, body, reader, writer).await
        {
            Ok(Some(reply)) => reply,
            Ok(None) => continue,
            Err(HubError::Refused(code, message)) => Reply::error(code, message),
            Err(error) => {
                tracing::warn!(%error, "hub failure");
                Reply::error(ErrorCode::Busy, "the hub failed; try again later")
            }
        };
        // A revoked device, or a stream that can no longer be read in step: the session ends.
        let stop = matches!(&reply.body, Some(ReplyBody::Error(e))
            if e.code == ErrorCode::Revoked as i32 || e.code == ErrorCode::TooLarge as i32);
        protocol::send(writer, &reply).await.map_err(io)?;
        writer.flush().await?;
        summary.account = caller.as_ref().and_then(|c| c.account.as_ref()).map(|a| a.id);
        if stop {
            return Ok(summary);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle<Rd, Wr>(
    store: &Arc<Store>,
    settings: &Settings,
    authenticated: Option<&str>,
    caller: &mut Option<Caller>,
    summary: &mut Summary,
    body: RequestBody,
    reader: &mut Rd,
    writer: &mut Wr,
) -> Result<Option<Reply>, HubError>
where
    Rd: AsyncRead + Unpin,
    Wr: AsyncWrite + Unpin,
{
    if let RequestBody::Hello(hello) = body {
        let mut next = greet(store, authenticated, hello).await?;
        let welcome = welcome(store, &mut next).await?;
        *caller = Some(next);
        return Ok(Some(welcome));
    }
    let Some(who) = caller.as_mut() else {
        return Err(HubError::refused(ErrorCode::Protocol, "say Hello first"));
    };
    if let RequestBody::Register(register) = body {
        let (user, code, quota) = (who.user.clone(), register.code, settings.quota);
        let account = blocking(store, move |s| s.register(&user, &code, quota)).await?;
        tracing::info!(account = account.id, "a person registered");
        who.account = Some(account);
        return Ok(Some(welcome(store, who).await?));
    }
    let Some(account) = who.account.clone() else {
        return Err(HubError::refused(ErrorCode::NotRegistered, "register with a code first"));
    };
    let id = account.id;
    let reply = match body {
        RequestBody::Push(push) => {
            let mut seq = 0;
            for batch in push.batches {
                // A copy pushes its own batches only (its author ID is in each batch's sealing too).
                if batch.author != who.author {
                    return Err(HubError::refused(ErrorCode::Protocol, "a batch of another library copy"));
                }
                check_quota(store, &account, batch.sealed.len() as u64).await?;
                let (library, author) = (who.library.clone(), who.author.clone());
                seq = blocking(store, move |s| s.push(id, &library, &author, &batch)).await?;
                summary.batches_in += 1;
            }
            ReplyBody::Pushed(Pushed { hub_seq: seq })
        }
        RequestBody::Pull(pull) => {
            let (library, author) = (who.library.clone(), who.author.clone());
            let batches = blocking(store, move |s| s.pull(id, &library, &author, pull.after, pull.limit)).await?;
            summary.batches_out += batches.batches.len();
            ReplyBody::Batches(batches)
        }
        RequestBody::HasFiles(FileIds { ids }) => {
            let present = blocking(store, move |s| s.has_files(id, &ids)).await?;
            ReplyBody::Present(FileIds { ids: present })
        }
        RequestBody::PutFile(put) => {
            if put.size > protocol::MAX_FILE {
                // The stream cannot be kept in step without reading it all: end the session.
                return Err(HubError::refused(ErrorCode::TooLarge, "this file is larger than the hub accepts"));
            }
            // The bytes follow the request: when refusing, they are read and dropped (never kept in
            // memory), so the stream stays in step.
            if let Err(refusal) = check_quota(store, &account, put.size).await {
                tokio::io::copy(&mut (&mut *reader).take(put.size), &mut tokio::io::sink()).await?;
                return Err(refusal);
            }
            let sealed = protocol::receive_bytes(reader, put.size).await.map_err(io)?;
            blocking(store, move |s| s.put_file(id, &put.id, &sealed)).await?;
            summary.files_in += 1;
            ReplyBody::Ok(protocol::Empty {})
        }
        RequestBody::GetFile(get) => {
            let wanted = get.id.clone();
            let Some(sealed) = blocking(store, move |s| s.get_file(id, &wanted)).await? else {
                return Err(HubError::refused(ErrorCode::NotFound, "the hub does not have this file"));
            };
            let header = Reply::of(ReplyBody::File(FileHeader { id: get.id, size: sealed.len() as u64 }));
            protocol::send(writer, &header).await.map_err(io)?;
            writer.write_all(&sealed).await?;
            writer.flush().await?;
            summary.files_out += 1;
            return Ok(None);
        }
        RequestBody::Devices(_) => {
            ReplyBody::Devices(protocol::DeviceList { devices: blocking(store, move |s| s.devices(id)).await? })
        }
        RequestBody::Revoke(revoke) => {
            if !blocking(store, move |s| s.revoke(id, &revoke.device)).await? {
                return Err(HubError::refused(ErrorCode::NotFound, "no such device on your account"));
            }
            ReplyBody::Ok(protocol::Empty {})
        }
        RequestBody::Hello(_) | RequestBody::Register(_) | RequestBody::Bye(_) => unreachable!("handled above"),
    };
    Ok(Some(Reply::of(reply)))
}

/// Checks a `Hello`: the protocol, and the certificate against the device the connection
/// authenticated; finds the person's account (none yet: they may register).
async fn greet(store: &Arc<Store>, authenticated: Option<&str>, hello: Hello) -> Result<Caller, HubError> {
    if hello.protocol != protocol::PROTOCOL {
        return Err(HubError::refused(
            ErrorCode::Protocol,
            format!("this hub speaks protocol {}, the device {}", protocol::PROTOCOL, hello.protocol),
        ));
    }
    let certificate: Certificate = hello
        .certificate
        .as_ref()
        .map(Into::into)
        .ok_or_else(|| HubError::refused(ErrorCode::Protocol, "no certificate"))?;
    let user = certificate
        .verify(authenticated.unwrap_or(&certificate.device))
        .map_err(|e| HubError::refused(ErrorCode::Protocol, e.to_string()))?;
    if hello.library.is_empty() || hello.author.is_empty() {
        return Err(HubError::refused(ErrorCode::Protocol, "no library or author"));
    }
    let lookup = user.clone();
    let account = blocking(store, move |s| s.account_of(&lookup)).await?;
    Ok(Caller { user, device: certificate.device, library: hello.library, author: hello.author, account })
}

/// The `Welcome` for `caller` (noting its device on its account, refused if revoked).
async fn welcome(store: &Arc<Store>, caller: &mut Caller) -> Result<Reply, HubError> {
    let Some(account) = caller.account.clone() else {
        return Ok(Reply::of(ReplyBody::Welcome(Welcome { registered: false, ..Welcome::default() })));
    };
    let (device, library, author) = (caller.device.clone(), caller.library.clone(), caller.author.clone());
    let (last_number, hub_seq, used, libraries) = blocking(store, move |s| {
        s.see_device(account.id, &device)?;
        let (last, seq) = s.positions(account.id, &library, &author)?;
        Ok((last, seq, s.used(account.id)?, s.libraries(account.id)?))
    })
    .await?;
    Ok(Reply::of(ReplyBody::Welcome(Welcome {
        registered: true,
        last_number,
        hub_seq,
        used,
        quota: account.quota,
        libraries,
    })))
}

async fn check_quota(store: &Arc<Store>, account: &Account, adding: u64) -> Result<(), HubError> {
    if account.quota == 0 {
        return Ok(());
    }
    let id = account.id;
    let used = blocking(store, move |s| s.used(id)).await?;
    if used + adding > account.quota {
        return Err(HubError::refused(ErrorCode::Quota, "your storage on the hub is full"));
    }
    Ok(())
}

/// Runs a store call on the blocking threads.
async fn blocking<T: Send + 'static>(
    store: &Arc<Store>,
    f: impl FnOnce(&Store) -> Result<T, HubError> + Send + 'static,
) -> Result<T, HubError> {
    let store = Arc::clone(store);
    tokio::task::spawn_blocking(move || f(&store)).await.map_err(|e| HubError::Io(std::io::Error::other(e)))?
}

fn io(error: quire_sync::SyncError) -> HubError {
    match error {
        quire_sync::SyncError::Io(e) => HubError::Io(e),
        quire_sync::SyncError::TooLarge => HubError::refused(ErrorCode::TooLarge, "too large"),
        other => HubError::refused(ErrorCode::Protocol, other.to_string()),
    }
}
