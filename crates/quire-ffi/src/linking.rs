//! Linking a device by QR code (`quire_sync::pairing`, `.tasks/T14-security.md`): this device
//! offers ([`LinkOffer`]: a code to show, then the person allows or refuses the device that asks),
//! or joins another's code ([`LinkJoin`]: check words to show, then what the other gives is taken).
//! After a join the app reopens the library (its user key, hub and relay changed).

use std::sync::Arc;

use quire_engine::activity::kind;
use quire_engine::{Engine, LinkGrant, LinkJoin as Join, LinkOffer as Offer, LinkRequest as Request};
use tokio::sync::{Mutex, Notify};

use crate::{QuireError, QuireLibrary, Result, on_runtime};

/// A device asking to be linked, with the right code: its name and the words both screens show.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct LinkRequestRecord {
    pub name: String,
    pub words: Vec<String>,
}

#[derive(uniffi::Object)]
pub struct LinkOffer {
    engine: Arc<Engine>,
    offer: Arc<Offer>,
    grant: LinkGrant,
    pending: Mutex<Option<Request>>,
    cancelled: Notify,
    /// The offer ended (its notifications were written once).
    ended: std::sync::atomic::AtomicBool,
}

impl LinkOffer {
    /// Closes the offer and notes, once, any wrong codes tried while it was open.
    async fn end(&self) {
        self.offer.close().await;
        if !self.ended.swap(true, std::sync::atomic::Ordering::Relaxed) {
            let wrong = self.offer.wrong_codes();
            if wrong > 0 {
                let _ = self.engine.record_activity(kind::WRONG_CODE, None, &wrong.to_string()).await;
            }
        }
    }
}

fn cancelled() -> QuireError {
    QuireError::Invalid("linking was cancelled".into())
}

#[uniffi::export]
impl LinkOffer {
    /// The `quire://device?…` link to show as a QR code (valid two minutes, once).
    pub fn uri(&self) -> String {
        self.offer.link().to_uri()
    }

    /// Waits for a device with the right code (others are refused); fails when the code expires
    /// or the offer is cancelled.
    pub async fn next_request(self: Arc<Self>) -> Result<LinkRequestRecord> {
        on_runtime(async move {
            let request = tokio::select! {
                request = self.offer.request() => request,
                () = self.cancelled.notified() => return Err(cancelled()),
            };
            let request = match request {
                Ok(request) => request,
                Err(error) => {
                    self.end().await;
                    return Err(quire_engine::EngineError::from(error).into());
                }
            };
            let record = LinkRequestRecord { name: request.name.clone(), words: request.words.clone() };
            *self.pending.lock().await = Some(request);
            Ok(record)
        })
        .await
    }

    /// Gives the waiting device this person's user key, hub and relay; the offer then closes.
    pub async fn allow(self: Arc<Self>) -> Result<()> {
        on_runtime(async move {
            let request =
                self.pending.lock().await.take().ok_or_else(|| QuireError::Invalid("no device is waiting".into()))?;
            // The hub lets no unknown device in: introduce it first, then hand over the keys.
            if let Err(error) = self.engine.hub_introduce(&request.device).await {
                let _ = request.refuse().await;
                self.end().await;
                return Err(error.into());
            }
            let name = request.name.clone();
            let given = request.allow(&self.grant).await.map_err(quire_engine::EngineError::from);
            self.end().await;
            given?;
            self.engine.record_activity(kind::LINK_ALLOWED, Some(&name), "").await?;
            Ok(())
        })
        .await
    }

    /// Turns the waiting device away; the offer then closes.
    pub async fn refuse(self: Arc<Self>) -> Result<()> {
        on_runtime(async move {
            if let Some(request) = self.pending.lock().await.take() {
                let name = request.name.clone();
                let _ = request.refuse().await;
                self.engine.record_activity(kind::LINK_REFUSED, Some(&name), "").await?;
            }
            self.end().await;
            Ok(())
        })
        .await
    }

    /// Stops the offer: the code no longer works, and a wait for a device ends.
    pub async fn cancel(self: Arc<Self>) {
        on_runtime(async move {
            self.cancelled.notify_one();
            if let Some(request) = self.pending.lock().await.take() {
                let _ = request.refuse().await;
            }
            self.end().await;
        })
        .await;
    }
}

#[derive(uniffi::Object)]
pub struct LinkJoin {
    engine: Arc<Engine>,
    join: Mutex<Option<Join>>,
    words: Vec<String>,
}

#[uniffi::export]
impl LinkJoin {
    /// The words to compare with the other device's screen.
    pub fn words(&self) -> Vec<String> {
        self.words.clone()
    }

    /// Waits for the person to allow this device on the other one, then takes this person's user
    /// key, hub and relay; returns the user ID now in use. Reopen the library afterwards.
    pub async fn finish(self: Arc<Self>) -> Result<String> {
        on_runtime(async move {
            let join = self.join.lock().await.take().ok_or_else(cancelled)?;
            let grant = join.finish().await.map_err(quire_engine::EngineError::from)?;
            let (user, _) = self.engine.apply_grant(&grant).await?;
            Ok(user.to_string())
        })
        .await
    }

    pub async fn cancel(self: Arc<Self>) {
        on_runtime(async move {
            if let Some(join) = self.join.lock().await.take() {
                join.close().await;
            }
        })
        .await;
    }
}

#[uniffi::export]
impl QuireLibrary {
    /// Offers to link a new device to this person's devices: show its code, then wait for a request.
    pub async fn link_offer(&self) -> Result<Arc<LinkOffer>> {
        self.with(|engine| async move {
            let (offer, grant) = engine.link_offer().await?;
            Ok(Arc::new(LinkOffer {
                engine,
                offer: Arc::new(offer),
                grant,
                pending: Mutex::new(None),
                ended: Default::default(),
                cancelled: Notify::new(),
            }))
        })
        .await
    }

    /// Joins through another device's code (`quire://device?…`, scanned or pasted), named `name`
    /// there; the words then show on both screens.
    pub async fn link_join(&self, link: String, name: String) -> Result<Arc<LinkJoin>> {
        self.with(|engine| async move {
            let join = engine.link_join(&link, &name).await?;
            let words = join.words.clone();
            Ok(Arc::new(LinkJoin { engine, join: Mutex::new(Some(join)), words }))
        })
        .await
    }
}
