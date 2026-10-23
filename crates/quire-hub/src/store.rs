//! What the hub keeps, in its data folder: `hub.sqlite` (accounts, devices, one-time codes, each
//! library's log of sealed batches and its latest snapshot) and `files/` (sealed files). Nothing
//! here is readable without the users' keys: batches and files arrive sealed by their devices.
//!
//! Calls are blocking (SQLite); the session runs them off the async threads.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use quire_sync::hub::protocol::{Batch, BatchKind, Batches, DeviceInfo, ErrorCode, KeyEnvelope, Keys};
use rusqlite::{Connection, OptionalExtension, params};

/// A refusal to report to the device, or a failure of the hub itself.
#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("{1}")]
    Refused(ErrorCode, String),
    #[error("hub database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("hub files: {0}")]
    Io(#[from] std::io::Error),
}

impl HubError {
    pub fn refused(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Refused(code, message.into())
    }
}

pub type Result<T> = std::result::Result<T, HubError>;

/// What a one-time code does when a device registers with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// The first person of a self-hosted hub (printed when the hub starts with no account).
    Owner,
    /// Another person, invited by the hub's administrator.
    Invite,
    /// Binds a person to an account made elsewhere (the cloud's account page).
    Link,
}

impl Purpose {
    fn as_str(self) -> &'static str {
        match self {
            Purpose::Owner => "owner",
            Purpose::Invite => "invite",
            Purpose::Link => "link",
        }
    }
}

/// A person's account: an ID, their user key (once registered) and their storage limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: i64,
    /// Bytes (0: no limit).
    pub quota: u64,
}

pub struct Store {
    conn: Mutex<Connection>,
    files: PathBuf,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS account (
    id INTEGER PRIMARY KEY,
    user TEXT UNIQUE,
    quota INTEGER NOT NULL DEFAULT 0,
    created INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS device (
    device TEXT PRIMARY KEY,
    account INTEGER NOT NULL REFERENCES account(id),
    first_seen INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS device_activity (
    device TEXT PRIMARY KEY REFERENCES device(device),
    sealed_name BLOB,
    last_session INTEGER NOT NULL,
    batches_in INTEGER NOT NULL,
    batches_out INTEGER NOT NULL,
    files_in INTEGER NOT NULL,
    files_out INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS device_box (
    account INTEGER NOT NULL,
    device TEXT PRIMARY KEY,
    box_key BLOB NOT NULL,
    box_signature BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS device_author (
    account INTEGER NOT NULL,
    device TEXT NOT NULL,
    author TEXT NOT NULL,
    PRIMARY KEY (device, author)
);
CREATE TABLE IF NOT EXISTS expected_device (
    account INTEGER NOT NULL,
    device TEXT PRIMARY KEY,
    expires INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS key_envelope (
    account INTEGER NOT NULL,
    library TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    device TEXT NOT NULL,
    sealed BLOB NOT NULL,
    PRIMARY KEY (account, library, epoch, device)
);
CREATE TABLE IF NOT EXISTS code (
    code TEXT PRIMARY KEY,
    purpose TEXT NOT NULL,
    account INTEGER REFERENCES account(id),
    created INTEGER NOT NULL,
    expires INTEGER,
    used INTEGER
);
CREATE TABLE IF NOT EXISTS library (
    account INTEGER NOT NULL REFERENCES account(id),
    library TEXT NOT NULL,
    hub_seq INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account, library)
);
CREATE TABLE IF NOT EXISTS author (
    account INTEGER NOT NULL,
    library TEXT NOT NULL,
    author TEXT NOT NULL,
    last_number INTEGER NOT NULL,
    PRIMARY KEY (account, library, author)
);
CREATE TABLE IF NOT EXISTS batch (
    account INTEGER NOT NULL,
    library TEXT NOT NULL,
    seq INTEGER NOT NULL,
    author TEXT NOT NULL,
    number INTEGER NOT NULL,
    sealed BLOB NOT NULL,
    PRIMARY KEY (account, library, seq)
);
CREATE TABLE IF NOT EXISTS snapshot (
    account INTEGER NOT NULL,
    library TEXT NOT NULL,
    covers INTEGER NOT NULL,
    author TEXT NOT NULL,
    number INTEGER NOT NULL,
    sealed BLOB NOT NULL,
    PRIMARY KEY (account, library)
);
CREATE TABLE IF NOT EXISTS file (
    account INTEGER NOT NULL,
    id TEXT NOT NULL,
    size INTEGER NOT NULL,
    created INTEGER NOT NULL,
    PRIMARY KEY (account, id)
);
";

/// Columns added after the first schema, to hubs that started before them.
fn migrate(conn: &Connection) -> Result<()> {
    for table in ["batch", "snapshot"] {
        let has: bool = conn.query_row(
            &format!("SELECT EXISTS (SELECT 1 FROM pragma_table_info('{table}') WHERE name = 'epoch')"),
            [],
            |r| r.get(0),
        )?;
        if !has {
            conn.execute(&format!("ALTER TABLE {table} ADD COLUMN epoch INTEGER NOT NULL DEFAULT 0"), [])?;
        }
    }
    Ok(())
}

/// How long an introduction lets a new device in (linking takes a minute or two).
const INTRODUCTION: Duration = Duration::from_secs(10 * 60);

/// Largest sealed device name kept (a name of 64 characters, sealed, is far below it).
const MAX_SEALED_NAME: usize = 512;
/// Largest key envelope (a chain of a thousand rotations is about 36 KB).
const MAX_ENVELOPE: usize = 64 * 1024;

/// Letters of codes: no 0/O, 1/I/L (read aloud or typed from a phone).
const CODE_ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";

impl Store {
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir.join("files"))?;
        let conn = Connection::open(dir.join("hub.sqlite"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Safe with WAL (fsync when the WAL is checkpointed); Litestream recommends it.
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
        migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn), files: dir.join("files") })
    }

    fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut conn = self.conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut conn)
    }

    pub fn has_accounts(&self) -> Result<bool> {
        self.with(|c| Ok(c.query_row("SELECT EXISTS(SELECT 1 FROM account WHERE user IS NOT NULL)", [], |r| r.get(0))?))
    }

    /// A new one-time code; `ttl` `None`: valid until used.
    pub fn new_code(&self, purpose: Purpose, account: Option<i64>, ttl: Option<Duration>) -> Result<String> {
        let code = new_code()?;
        let now = now();
        let expires = ttl.map(|t| now + i64::try_from(t.as_millis()).unwrap_or(i64::MAX));
        self.with(|c| {
            c.execute(
                "INSERT INTO code (code, purpose, account, created, expires) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![code, purpose.as_str(), account, now, expires],
            )?;
            Ok(())
        })?;
        Ok(code)
    }

    /// The setup code of a hub nobody registered on yet (made once, shown at every start).
    pub fn owner_code(&self) -> Result<Option<String>> {
        if self.has_accounts()? {
            return Ok(None);
        }
        let found: Option<String> = self.with(|c| {
            Ok(c.query_row("SELECT code FROM code WHERE purpose = 'owner' AND used IS NULL", [], |r| r.get(0))
                .optional()?)
        })?;
        match found {
            Some(code) => Ok(Some(code)),
            None => self.new_code(Purpose::Owner, None, None).map(Some),
        }
    }

    /// An account made before its person registers (the cloud's account page), with a link code.
    pub fn new_account(&self, quota: u64) -> Result<(i64, String)> {
        let id = self.with(|c| {
            c.execute("INSERT INTO account (quota, created) VALUES (?1, ?2)", params![quota as i64, now()])?;
            Ok(c.last_insert_rowid())
        })?;
        let code = self.new_code(Purpose::Link, Some(id), Some(Duration::from_secs(24 * 3600)))?;
        Ok((id, code))
    }

    pub fn account_of(&self, user: &str) -> Result<Option<Account>> {
        self.with(|c| {
            Ok(c.query_row("SELECT id, quota FROM account WHERE user = ?1", [user], |r| {
                Ok(Account { id: r.get(0)?, quota: r.get::<_, i64>(1)? as u64 })
            })
            .optional()?)
        })
    }

    /// Registers `user` with `code` (unused and not expired): a new account (owner, invitation;
    /// `quota` its limit) or the account the code was made for (link).
    pub fn register(&self, user: &str, code: &str, quota: u64) -> Result<Account> {
        let code = code.trim().to_ascii_uppercase();
        let now = now();
        self.with(|c| {
            let tx = c.transaction()?;
            if let Some(account) = tx
                .query_row("SELECT id, quota FROM account WHERE user = ?1", [user], |r| {
                    Ok(Account { id: r.get(0)?, quota: r.get::<_, i64>(1)? as u64 })
                })
                .optional()?
            {
                return Ok(account);
            }
            let found: Option<(String, Option<i64>)> = tx
                .query_row(
                    "SELECT purpose, account FROM code WHERE code = ?1 AND used IS NULL AND (expires IS NULL OR expires > ?2)",
                    params![code, now],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let Some((purpose, account)) = found else {
                return Err(HubError::refused(ErrorCode::BadCode, "this code is wrong, used or expired"));
            };
            let account = match (purpose.as_str(), account) {
                ("link", Some(id)) => {
                    let taken: Option<String> = tx.query_row("SELECT user FROM account WHERE id = ?1", [id], |r| r.get(0))?;
                    if taken.is_some() {
                        return Err(HubError::refused(ErrorCode::BadCode, "this account is already linked"));
                    }
                    tx.execute("UPDATE account SET user = ?1 WHERE id = ?2", params![user, id])?;
                    id
                }
                _ => {
                    tx.execute(
                        "INSERT INTO account (user, quota, created) VALUES (?1, ?2, ?3)",
                        params![user, quota as i64, now],
                    )?;
                    tx.last_insert_rowid()
                }
            };
            tx.execute("UPDATE code SET used = ?1 WHERE code = ?2", params![now, code])?;
            let quota: i64 = tx.query_row("SELECT quota FROM account WHERE id = ?1", [account], |r| r.get(0))?;
            tx.commit()?;
            Ok(Account { id: account, quota: quota as u64 })
        })
    }

    /// Notes that `device` (a network ID) of `account` is connecting; refused if it was revoked or
    /// belongs to another account.
    /// Notes `device` of `account` as seen. A device the hub does not know yet is let in only if
    /// it brought a code in this session (`admitted`) or one of the person's devices introduced it
    /// ([`Store::introduce`]); an introduction also lets an unlinked device back in.
    pub fn see_device(&self, account: i64, device: &str, admitted: bool) -> Result<()> {
        let now = now();
        self.with(|c| {
            let tx = c.transaction()?;
            let found: Option<(i64, bool)> = tx
                .query_row("SELECT account, revoked FROM device WHERE device = ?1", [device], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .optional()?;
            let introduced = tx.execute(
                "DELETE FROM expected_device WHERE device = ?1 AND account = ?2 AND expires >= ?3",
                params![device, account, now],
            )? > 0;
            match found {
                Some((owner, _)) if owner != account => {
                    return Err(HubError::refused(ErrorCode::Revoked, "this device belongs to another account"));
                }
                Some((_, true)) if !introduced => {
                    return Err(HubError::refused(ErrorCode::Revoked, "this device was removed from your account"));
                }
                Some(_) => {
                    tx.execute(
                        "UPDATE device SET last_seen = ?1, revoked = 0 WHERE device = ?2",
                        params![now, device],
                    )?;
                }
                None if admitted || introduced => {
                    tx.execute(
                        "INSERT INTO device (device, account, first_seen, last_seen) VALUES (?1, ?2, ?3, ?3)",
                        params![device, account, now],
                    )?;
                }
                None => {
                    return Err(HubError::refused(
                        ErrorCode::NotIntroduced,
                        "this device is new to your hub: link it from one of your devices (Link a Device)",
                    ));
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    /// Lets `device` in for a while as one of `account`'s (one of its devices is linking it).
    pub fn introduce(&self, account: i64, device: &str) -> Result<()> {
        let expires = now() + i64::try_from(INTRODUCTION.as_millis()).unwrap_or(i64::MAX);
        self.with(|c| {
            let owner: Option<i64> =
                c.query_row("SELECT account FROM device WHERE device = ?1", [device], |r| r.get(0)).optional()?;
            if owner.is_some_and(|o| o != account) {
                return Err(HubError::refused(ErrorCode::Protocol, "that device belongs to another account"));
            }
            c.execute(
                "INSERT INTO expected_device (account, device, expires) VALUES (?1, ?2, ?3)
                 ON CONFLICT (device) DO UPDATE SET account = excluded.account, expires = excluded.expires",
                params![account, device, expires],
            )?;
            Ok(())
        })
    }

    /// Notes that `device` syncs the library copy `author` (the device list then says whose changes
    /// are whose).
    pub fn note_author(&self, account: i64, device: &str, author: &str) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT OR IGNORE INTO device_author (account, device, author) VALUES (?1, ?2, ?3)",
                params![account, device, author],
            )?;
            Ok(())
        })
    }

    /// Keeps `device`'s box key (its signature by the device key was checked).
    pub fn set_box_key(&self, account: i64, device: &str, key: &[u8], signature: &[u8]) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT INTO device_box (account, device, box_key, box_signature) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (device) DO UPDATE SET box_key = excluded.box_key, box_signature = excluded.box_signature",
                params![account, device, key, signature],
            )?;
            Ok(())
        })
    }

    /// The library's newest key epoch (0: never rotated).
    pub fn key_epoch(&self, account: i64, library: &str) -> Result<u32> {
        self.with(|c| {
            let epoch: i64 = c.query_row(
                "SELECT COALESCE(MAX(epoch), 0) FROM key_envelope WHERE account = ?1 AND library = ?2",
                params![account, library],
                |r| r.get(0),
            )?;
            Ok(u32::try_from(epoch).unwrap_or(0))
        })
    }

    /// Stores the envelopes of the library's next epoch; refused unless `epoch` is the newest plus
    /// one (another device rotated first: it takes those keys and tries again), or for a device that
    /// is not one of `account`'s, or revoked.
    pub fn put_keys(&self, account: i64, library: &str, epoch: u32, envelopes: &[KeyEnvelope]) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            let newest: i64 = tx.query_row(
                "SELECT COALESCE(MAX(epoch), 0) FROM key_envelope WHERE account = ?1 AND library = ?2",
                params![account, library],
                |r| r.get(0),
            )?;
            if i64::from(epoch) != newest + 1 {
                return Err(HubError::refused(ErrorCode::StaleEpoch, "another device rotated the key first"));
            }
            for envelope in envelopes {
                let ours: bool = tx.query_row(
                    "SELECT EXISTS (SELECT 1 FROM device WHERE device = ?1 AND account = ?2 AND revoked = 0)",
                    params![envelope.device, account],
                    |r| r.get(0),
                )?;
                if !ours || envelope.sealed.len() > MAX_ENVELOPE {
                    return Err(HubError::refused(ErrorCode::Protocol, "keys for a device that is not one of yours"));
                }
                tx.execute(
                    "INSERT INTO key_envelope (account, library, epoch, device, sealed) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![account, library, epoch, envelope.device, envelope.sealed],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    /// `device`'s envelope of the library's newest epoch (empty if there is none for it).
    pub fn keys_for(&self, account: i64, library: &str, device: &str) -> Result<Keys> {
        let epoch = self.key_epoch(account, library)?;
        self.with(|c| {
            let sealed: Option<Vec<u8>> = c
                .query_row(
                    "SELECT sealed FROM key_envelope WHERE account = ?1 AND library = ?2 AND epoch = ?3 AND device = ?4",
                    params![account, library, epoch, device],
                    |r| r.get(0),
                )
                .optional()?;
            Ok(Keys { epoch, sealed: sealed.unwrap_or_default() })
        })
    }

    /// Records a finished session of `account`'s `device`: what it moved and, if it sent one, its
    /// sealed name (kept from earlier sessions otherwise). Counts and sizes only, never content.
    pub fn record_session(&self, account: i64, device: &str, sealed_name: &[u8], moved: [usize; 4]) -> Result<()> {
        let [batches_in, batches_out, files_in, files_out] = moved.map(|n| i64::try_from(n).unwrap_or(i64::MAX));
        let name = (!sealed_name.is_empty() && sealed_name.len() <= MAX_SEALED_NAME).then_some(sealed_name);
        self.with(|c| {
            // Only a device of this account (one that a session just saw).
            let known: bool = c.query_row(
                "SELECT EXISTS (SELECT 1 FROM device WHERE device = ?1 AND account = ?2)",
                params![device, account],
                |r| r.get(0),
            )?;
            if known {
                c.execute(
                    "INSERT INTO device_activity (device, sealed_name, last_session, batches_in, batches_out, files_in, files_out)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT (device) DO UPDATE SET sealed_name = COALESCE(excluded.sealed_name, sealed_name),
                        last_session = excluded.last_session, batches_in = excluded.batches_in,
                        batches_out = excluded.batches_out, files_in = excluded.files_in, files_out = excluded.files_out",
                    params![device, name, now(), batches_in, batches_out, files_in, files_out],
                )?;
            }
            Ok(())
        })
    }

    pub fn devices(&self, account: i64) -> Result<Vec<DeviceInfo>> {
        self.with(|c| {
            let mut statement = c.prepare(
                "SELECT d.device, d.first_seen, d.last_seen, d.revoked, a.sealed_name, a.last_session,
                        a.batches_in, a.batches_out, a.files_in, a.files_out, b.box_key, b.box_signature
                 FROM device d LEFT JOIN device_activity a ON a.device = d.device
                      LEFT JOIN device_box b ON b.device = d.device
                 WHERE d.account = ?1 ORDER BY d.first_seen",
            )?;
            let count = |r: &rusqlite::Row<'_>, i: usize| -> rusqlite::Result<u32> {
                Ok(u32::try_from(r.get::<_, Option<i64>>(i)?.unwrap_or(0)).unwrap_or(u32::MAX))
            };
            let rows = statement.query_map([account], |r| {
                Ok(DeviceInfo {
                    device: r.get(0)?,
                    first_seen: r.get(1)?,
                    last_seen: r.get(2)?,
                    revoked: r.get(3)?,
                    sealed_name: r.get::<_, Option<Vec<u8>>>(4)?.unwrap_or_default(),
                    last_session: r.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    batches_in: count(r, 6)?,
                    batches_out: count(r, 7)?,
                    files_in: count(r, 8)?,
                    files_out: count(r, 9)?,
                    box_key: r.get::<_, Option<Vec<u8>>>(10)?.unwrap_or_default(),
                    box_signature: r.get::<_, Option<Vec<u8>>>(11)?.unwrap_or_default(),
                    authors: Vec::new(),
                })
            })?;
            let mut devices: Vec<DeviceInfo> = rows.collect::<rusqlite::Result<_>>()?;
            let mut authors = c.prepare("SELECT author FROM device_author WHERE device = ?1 ORDER BY author")?;
            for device in &mut devices {
                device.authors = authors.query_map([&device.device], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
            }
            Ok(devices)
        })
    }

    /// Refuses `device` from now on; `false` if it is not one of `account`'s.
    pub fn revoke(&self, account: i64, device: &str) -> Result<bool> {
        self.with(|c| {
            Ok(c.execute("UPDATE device SET revoked = 1 WHERE device = ?1 AND account = ?2", params![device, account])?
                > 0)
        })
    }

    /// The last batch number from `author` in `library`, and the library's latest hub position.
    pub fn positions(&self, account: i64, library: &str, author: &str) -> Result<(u64, u64)> {
        self.with(|c| {
            let last: i64 = c
                .query_row(
                    "SELECT last_number FROM author WHERE account = ?1 AND library = ?2 AND author = ?3",
                    params![account, library, author],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(0);
            let seq: i64 = c
                .query_row(
                    "SELECT hub_seq FROM library WHERE account = ?1 AND library = ?2",
                    params![account, library],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(0);
            Ok((last as u64, seq as u64))
        })
    }

    /// The libraries `account` keeps here.
    pub fn libraries(&self, account: i64) -> Result<Vec<String>> {
        self.with(|c| {
            let mut statement = c.prepare("SELECT library FROM library WHERE account = ?1 ORDER BY library")?;
            let rows = statement.query_map([account], |r| r.get(0))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Bytes stored for `account`: its batches, snapshots and files.
    pub fn used(&self, account: i64) -> Result<u64> {
        self.with(|c| {
            let used: i64 = c.query_row(
                "SELECT (SELECT COALESCE(SUM(length(sealed)), 0) FROM batch WHERE account = ?1)
                      + (SELECT COALESCE(SUM(length(sealed)), 0) FROM snapshot WHERE account = ?1)
                      + (SELECT COALESCE(SUM(size), 0) FROM file WHERE account = ?1)",
                [account],
                |r| r.get(0),
            )?;
            Ok(used as u64)
        })
    }

    /// Stores a batch of `author` (already checked to be the connecting copy's); returns the
    /// library's hub position after it. A change batch whose number is not past the author's last
    /// is a repeat (a retried push) and is skipped. A snapshot replaces the previous one and drops
    /// the batches it covers.
    pub fn push(&self, account: i64, library: &str, author: &str, batch: &Batch) -> Result<u64> {
        let kind = BatchKind::try_from(batch.kind)
            .map_err(|_| HubError::refused(ErrorCode::Protocol, "unknown batch kind"))?;
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("INSERT OR IGNORE INTO library (account, library) VALUES (?1, ?2)", params![account, library])?;
            let seq: i64 =
                tx.query_row("SELECT hub_seq FROM library WHERE account = ?1 AND library = ?2", params![account, library], |r| {
                    r.get(0)
                })?;
            match kind {
                BatchKind::Changes => {
                    let last: i64 = tx
                        .query_row(
                            "SELECT last_number FROM author WHERE account = ?1 AND library = ?2 AND author = ?3",
                            params![account, library, author],
                            |r| r.get(0),
                        )
                        .optional()?
                        .unwrap_or(0);
                    let number = i64::try_from(batch.number).unwrap_or(i64::MAX);
                    if number <= last {
                        return Ok(seq as u64);
                    }
                    let next = seq + 1;
                    tx.execute(
                        "INSERT INTO batch (account, library, seq, author, number, sealed, epoch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![account, library, next, author, number, batch.sealed, batch.epoch],
                    )?;
                    tx.execute(
                        "INSERT INTO author (account, library, author, last_number) VALUES (?1, ?2, ?3, ?4)
                         ON CONFLICT (account, library, author) DO UPDATE SET last_number = excluded.last_number",
                        params![account, library, author, number],
                    )?;
                    tx.execute("UPDATE library SET hub_seq = ?1 WHERE account = ?2 AND library = ?3", params![next, account, library])?;
                    tx.commit()?;
                    Ok(next as u64)
                }
                BatchKind::Snapshot => {
                    let covers = i64::try_from(batch.covers).unwrap_or(i64::MAX);
                    if covers > seq {
                        return Err(HubError::refused(ErrorCode::Protocol, "a snapshot cannot cover batches not stored yet"));
                    }
                    let older: Option<i64> = tx
                        .query_row(
                            "SELECT covers FROM snapshot WHERE account = ?1 AND library = ?2",
                            params![account, library],
                            |r| r.get(0),
                        )
                        .optional()?;
                    if older.is_some_and(|o| o >= covers) {
                        return Ok(seq as u64);
                    }
                    tx.execute(
                        "INSERT OR REPLACE INTO snapshot (account, library, covers, author, number, sealed, epoch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![account, library, covers, author, batch.number as i64, batch.sealed, batch.epoch],
                    )?;
                    tx.execute("DELETE FROM batch WHERE account = ?1 AND library = ?2 AND seq <= ?3", params![account, library, covers])?;
                    tx.commit()?;
                    Ok(seq as u64)
                }
            }
        })
    }

    /// The batches after `after` for the copy `author` (its own skipped), at most `limit`; the
    /// snapshot first if `after` is behind it.
    pub fn pull(&self, account: i64, library: &str, author: &str, after: u64, limit: u32) -> Result<Batches> {
        self.with(|c| {
            let mut after = i64::try_from(after).unwrap_or(i64::MAX);
            let mut batches = Vec::new();
            let snapshot: Option<Batch> = c
                .query_row(
                    "SELECT covers, author, number, sealed, epoch FROM snapshot WHERE account = ?1 AND library = ?2",
                    params![account, library],
                    |r| {
                        let covers: i64 = r.get(0)?;
                        Ok(Batch {
                            author: r.get(1)?,
                            number: r.get::<_, i64>(2)? as u64,
                            kind: BatchKind::Snapshot as i32,
                            sealed: r.get(3)?,
                            covers: covers as u64,
                            seq: covers as u64,
                            epoch: r.get(4)?,
                        })
                    },
                )
                .optional()?;
            if let Some(snapshot) = snapshot.filter(|s| (s.covers as i64) > after) {
                after = snapshot.covers as i64;
                batches.push(snapshot);
            }
            let limit = i64::from(limit.clamp(1, 1000));
            let mut statement = c.prepare(
                "SELECT seq, author, number, sealed, epoch FROM batch WHERE account = ?1 AND library = ?2 AND seq > ?3 ORDER BY seq LIMIT ?4",
            )?;
            let rows: Vec<Batch> = statement
                .query_map(params![account, library, after, limit], |r| {
                    Ok(Batch {
                        seq: r.get::<_, i64>(0)? as u64,
                        author: r.get(1)?,
                        number: r.get::<_, i64>(2)? as u64,
                        kind: BatchKind::Changes as i32,
                        sealed: r.get(3)?,
                        covers: 0,
                        epoch: r.get(4)?,
                    })
                })?
                .collect::<rusqlite::Result<_>>()?;
            let up_to = rows.last().map_or(after, |b| b.seq as i64);
            let more = rows.len() as i64 == limit;
            batches.extend(rows.into_iter().filter(|b| b.author != author));
            let since_snapshot: i64 =
                c.query_row("SELECT COUNT(*) FROM batch WHERE account = ?1 AND library = ?2", params![account, library], |r| r.get(0))?;
            Ok(Batches { batches, up_to: up_to as u64, more, since_snapshot: since_snapshot as u64 })
        })
    }

    pub fn has_files(&self, account: i64, ids: &[String]) -> Result<Vec<String>> {
        self.with(|c| {
            let mut statement = c.prepare("SELECT 1 FROM file WHERE account = ?1 AND id = ?2")?;
            let mut present = Vec::new();
            for id in ids {
                if statement.exists(params![account, id])? {
                    present.push(id.clone());
                }
            }
            Ok(present)
        })
    }

    fn file_path(&self, account: i64, id: &str) -> Result<PathBuf> {
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(HubError::refused(ErrorCode::Protocol, "not a file ID"));
        }
        Ok(self.files.join(account.to_string()).join(&id[..2]).join(id))
    }

    /// Where to write the sealed file `id` of `account` as it arrives (a name of its own, so two
    /// uploads of one file do not mix); [`Store::commit_file`] makes it the file. Files are written
    /// to disk as they come, never held in memory whole.
    pub fn incoming_file(&self, account: i64, id: &str) -> Result<PathBuf> {
        let path = self.file_path(account, id)?;
        std::fs::create_dir_all(path.parent().expect("files/<account>/<xx>"))?;
        let mut tag = [0u8; 8];
        getrandom::fill(&mut tag).map_err(|e| std::io::Error::other(format!("no randomness: {e}")))?;
        let tag: String = tag.iter().map(|b| format!("{b:02x}")).collect();
        Ok(path.with_extension(format!("{tag}.partial")))
    }

    /// Makes the fully written `partial` (from [`Store::incoming_file`]) the file `id` of `account`
    /// (renamed: never half a file).
    pub fn commit_file(&self, account: i64, id: &str, partial: &Path) -> Result<()> {
        let path = self.file_path(account, id)?;
        let size = std::fs::metadata(partial)?.len();
        std::fs::rename(partial, &path)?;
        self.with(|c| {
            c.execute(
                "INSERT OR REPLACE INTO file (account, id, size, created) VALUES (?1, ?2, ?3, ?4)",
                params![account, id, size as i64, now()],
            )?;
            Ok(())
        })
    }

    /// The sealed file `id` of `account` and its size, if stored.
    pub fn stored_file(&self, account: i64, id: &str) -> Result<Option<(PathBuf, u64)>> {
        let path = self.file_path(account, id)?;
        if self.has_files(account, &[id.to_string()])?.is_empty() {
            return Ok(None);
        }
        let size = std::fs::metadata(&path)?.len();
        Ok(Some((path, size)))
    }

    /// Stores a sealed file held in memory (tests, small files).
    pub fn put_file(&self, account: i64, id: &str, sealed: &[u8]) -> Result<()> {
        let partial = self.incoming_file(account, id)?;
        std::fs::write(&partial, sealed)?;
        self.commit_file(account, id, &partial)
    }

    pub fn get_file(&self, account: i64, id: &str) -> Result<Option<Vec<u8>>> {
        self.stored_file(account, id)?.map(|(path, _)| Ok(std::fs::read(path)?)).transpose()
    }

    /// Deletes an account and everything it stored (its person's request; GDPR erasure).
    pub fn delete_account(&self, account: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute(
                "DELETE FROM device_activity WHERE device IN (SELECT device FROM device WHERE account = ?1)",
                [account],
            )?;
            for table in [
                "batch",
                "snapshot",
                "author",
                "library",
                "file",
                "device_box",
                "device_author",
                "expected_device",
                "key_envelope",
                "device",
                "code",
            ] {
                tx.execute(&format!("DELETE FROM {table} WHERE account = ?1"), [account])?;
            }
            tx.execute("DELETE FROM account WHERE id = ?1", [account])?;
            tx.commit()?;
            Ok(())
        })?;
        let dir = self.files.join(account.to_string());
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    /// Whether the database answers (the container's health check).
    pub fn healthy(&self) -> bool {
        self.with(|c| Ok(c.query_row("SELECT 1", [], |r| r.get::<_, i64>(0))?)).is_ok()
    }
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// `XXXX-XXXX-XXXX` from [`CODE_ALPHABET`] (about 59 bits): typed once, used once.
fn new_code() -> Result<String> {
    // Bytes at or past the largest multiple of the alphabet's size are drawn again (no bias).
    let limit = 256 - 256 % CODE_ALPHABET.len();
    let mut letters = String::with_capacity(12);
    while letters.len() < 12 {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|e| std::io::Error::other(format!("no randomness: {e}")))?;
        for b in bytes.iter().map(|b| usize::from(*b)).filter(|b| *b < limit) {
            if letters.len() < 12 {
                letters.push(CODE_ALPHABET[b % CODE_ALPHABET.len()] as char);
            }
        }
    }
    Ok(format!("{}-{}-{}", &letters[..4], &letters[4..8], &letters[8..]))
}

#[cfg(test)]
mod tests;
