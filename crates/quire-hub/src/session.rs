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
/// The same before the caller has an account: anyone may connect, and silent strangers must not
/// hold the hub's few session places (a code is typed before connecting, so this is plenty).
const IDLE_STRANGER: Duration = Duration::from_secs(15);

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
    /// The device, once it said `Hello` with an account here, and the name it sent, sealed.
    pub device: Option<String>,
    /// It synced (pulled) in this session: only then is the session its "last sync".
    pub synced: bool,
    pub sealed_name: Vec<u8>,
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
    /// Its box key and signature, checked against its device key (`None`: none sent, or forged).
    box_key: Option<(Vec<u8>, Vec<u8>)>,
    /// It used a code in this session (a new device is then let in without an introduction).
    admitted: bool,
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
    let summary = serve_session(store, settings, authenticated, reader, writer).await?;
    // For the person's device list: when each device last synced and what it moved.
    if let (Some(account), Some(device), true) = (summary.account, &summary.device, summary.synced) {
        let moved = [summary.batches_in, summary.batches_out, summary.files_in, summary.files_out];
        if let Err(error) = store.record_session(account, device, &summary.sealed_name, moved) {
            tracing::warn!(%error, "could not record the session");
        }
    }
    Ok(summary)
}

async fn serve_session<Rd, Wr>(
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
        let idle = if caller.as_ref().is_some_and(|c| c.account.is_some()) { IDLE } else { IDLE_STRANGER };
        let request: Option<Request> = match tokio::time::timeout(idle, protocol::receive(reader)).await {
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
        let sealed_name = hello.sealed_name.clone();
        let mut next = greet(store, authenticated, hello).await?;
        if !sealed_name.is_empty() {
            summary.sealed_name = sealed_name;
        }
        let welcome = welcome(store, &mut next).await?;
        if next.account.is_some() {
            summary.device = Some(next.device.clone());
        }
        *caller = Some(next);
        return Ok(Some(welcome));
    }
    let Some(who) = caller.as_mut() else {
        return Err(HubError::refused(ErrorCode::Protocol, "say Hello first"));
    };
    if let RequestBody::Register(register) = body {
        let (user, code, quota) = (who.user.clone(), register.code, settings.quota);
        // A code admits a new device only if it was used now: a person already known registers
        // again without one being taken, and a device with only their key is not let in that way.
        let known = who.account.is_some();
        let account = blocking(store, move |s| s.register(&user, &code, quota)).await?;
        if !known {
            tracing::info!(account = account.id, "a person registered");
            who.admitted = true;
        }
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
            summary.synced = true;
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
            // Written to disk as it comes (a file may be 2 GB; many sessions run at once).
            let file_id = put.id.clone();
            let partial = blocking(store, move |s| s.incoming_file(id, &file_id)).await?;
            let mut file = tokio::fs::File::create(&partial).await?;
            let copied = tokio::io::copy(&mut (&mut *reader).take(put.size), &mut file).await;
            let written = file.sync_all().await.and(copied);
            drop(file);
            if !written.as_ref().is_ok_and(|n| *n == put.size) {
                let _ = tokio::fs::remove_file(&partial).await;
                return Err(HubError::Io(match written {
                    Err(error) => error,
                    Ok(_) => std::io::Error::from(std::io::ErrorKind::UnexpectedEof),
                }));
            }
            blocking(store, move |s| s.commit_file(id, &put.id, &partial)).await?;
            summary.files_in += 1;
            ReplyBody::Ok(protocol::Empty {})
        }
        RequestBody::GetFile(get) => {
            let wanted = get.id.clone();
            let Some((path, size)) = blocking(store, move |s| s.stored_file(id, &wanted)).await? else {
                return Err(HubError::refused(ErrorCode::NotFound, "the hub does not have this file"));
            };
            // Sent from disk as it is read, not loaded whole.
            let file = tokio::fs::File::open(&path).await?;
            let header = Reply::of(ReplyBody::File(FileHeader { id: get.id, size }));
            protocol::send(writer, &header).await.map_err(io)?;
            tokio::io::copy(&mut file.take(size), writer).await?;
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
        RequestBody::Introduce(introduce) => {
            blocking(store, move |s| s.introduce(id, &introduce.device)).await?;
            ReplyBody::Ok(protocol::Empty {})
        }
        RequestBody::PutKeys(put) => {
            let library = who.library.clone();
            blocking(store, move |s| s.put_keys(id, &library, put.epoch, &put.envelopes)).await?;
            tracing::info!(account = id, epoch = put.epoch, "the library key rotated");
            ReplyBody::Ok(protocol::Empty {})
        }
        RequestBody::GetKeys(_) => {
            let (library, device) = (who.library.clone(), who.device.clone());
            ReplyBody::Keys(blocking(store, move |s| s.keys_for(id, &library, &device)).await?)
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
    let box_key = (quire_sync::keys::box_key_is_devices(&certificate.device, &hello.box_key, &hello.box_signature))
        .then_some((hello.box_key, hello.box_signature));
    Ok(Caller {
        user,
        device: certificate.device,
        library: hello.library,
        author: hello.author,
        account,
        box_key,
        admitted: false,
    })
}

/// The `Welcome` for `caller` (noting its device on its account, refused if revoked).
async fn welcome(store: &Arc<Store>, caller: &mut Caller) -> Result<Reply, HubError> {
    let Some(account) = caller.account.clone() else {
        return Ok(Reply::of(ReplyBody::Welcome(Welcome { registered: false, ..Welcome::default() })));
    };
    let (device, library, author) = (caller.device.clone(), caller.library.clone(), caller.author.clone());
    let (admitted, box_key) = (caller.admitted, caller.box_key.clone());
    let (last_number, hub_seq, used, libraries, key_epoch) = blocking(store, move |s| {
        s.see_device(account.id, &device, admitted)?;
        s.note_author(account.id, &device, &author)?;
        if let Some((key, signature)) = &box_key {
            s.set_box_key(account.id, &device, key, signature)?;
        }
        let (last, seq) = s.positions(account.id, &library, &author)?;
        Ok((last, seq, s.used(account.id)?, s.libraries(account.id)?, s.key_epoch(account.id, &library)?))
    })
    .await?;
    Ok(Reply::of(ReplyBody::Welcome(Welcome {
        registered: true,
        last_number,
        hub_seq,
        used,
        quota: account.quota,
        libraries,
        key_epoch,
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

#[cfg(test)]
mod tests {
    use quire_sync::hub::protocol::{Request, RequestBody};

    use super::*;

    /// Anyone may connect and say `Hello`; someone without an account once held one of the hub's
    /// few session places for the full idle time (two minutes), so a handful of idle connections
    /// kept paying users out. Without an account the hub now waits briefly.
    #[tokio::test(start_paused = true)]
    async fn a_silent_stranger_is_dropped_quickly() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path()).unwrap());
        let settings = Settings { quota: 0 };
        let (hub_side, mut device_side) = tokio::io::duplex(1 << 16);
        let device = quire_sync::transport::device_id(&[1; 32]);
        let hello = Hello {
            protocol: protocol::PROTOCOL,
            certificate: Some((&Certificate::issue(&[2; 32], &device)).into()),
            library: "lib_x".into(),
            author: device.clone(),
            ..Hello::default()
        };
        protocol::send(&mut device_side, &Request::of(RequestBody::Hello(hello))).await.unwrap();
        let started = tokio::time::Instant::now();
        let (mut reader, mut writer) = tokio::io::split(hub_side);
        let summary = serve(&store, &settings, None, &mut reader, &mut writer).await.unwrap();
        assert!(started.elapsed() <= Duration::from_secs(30), "held for {:?}", started.elapsed());
        assert!(!summary.ended_by_device);
        drop(device_side);
    }

    /// A file cut off mid-upload (the device went away) leaves nothing: no file, no partial one.
    /// Files are now written to disk as they arrive instead of held whole in memory.
    #[tokio::test]
    async fn a_cut_upload_leaves_nothing() {
        use quire_sync::hub::protocol::{PutFile, Register};
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path()).unwrap());
        let code = store.owner_code().unwrap().unwrap();
        let settings = Settings { quota: 0 };
        let (hub_side, mut device_side) = tokio::io::duplex(1 << 16);
        let device = quire_sync::transport::device_id(&[1; 32]);
        let id = "ab".repeat(32);
        let hello = Hello {
            protocol: protocol::PROTOCOL,
            certificate: Some((&Certificate::issue(&[2; 32], &device)).into()),
            library: "lib_x".into(),
            author: device.clone(),
            ..Hello::default()
        };
        let device_task = {
            let id = id.clone();
            tokio::spawn(async move {
                protocol::send(&mut device_side, &Request::of(RequestBody::Hello(hello))).await.unwrap();
                protocol::send(&mut device_side, &Request::of(RequestBody::Register(Register { code }))).await.unwrap();
                let put = PutFile { id, size: 100 };
                protocol::send(&mut device_side, &Request::of(RequestBody::PutFile(put))).await.unwrap();
                device_side.write_all(&[7; 10]).await.unwrap();
                device_side.flush().await.unwrap();
                // Read the replies to Hello and Register, then go away mid-file.
                for _ in 0..2 {
                    let _: Option<Reply> = protocol::receive(&mut device_side).await.unwrap();
                }
            })
        };
        let (mut reader, mut writer) = tokio::io::split(hub_side);
        let _ = serve(&store, &settings, None, &mut reader, &mut writer).await;
        device_task.await.unwrap();
        let account = store.account_of(&quire_sync::identity::user_id(&[2; 32])).unwrap().unwrap();
        assert!(store.stored_file(account.id, &id).unwrap().is_none());
        let leftovers: Vec<_> = walk(&dir.path().join("files"));
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() { found.extend(walk(&path)) } else { found.push(path) }
        }
        found
    }
}
