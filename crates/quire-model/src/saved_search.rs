//! [`SavedSearch`]: a search of the library kept under a name, run again whenever it is opened
//! (a "smart project": new matching references appear in it by themselves). [`Alert`]: a search
//! of arXiv, PubMed or Crossref, checked for papers published since the last check. Personal,
//! like reading status.

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};
use crate::id::{AlertId, DeviceId, SavedSearchId};
use crate::record::{RecordMeta, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: SavedSearchId,
    /// As typed; unique ignoring case ([`key`](Self::key)).
    pub name: String,
    /// What `quire search` takes (words, `field:` terms, filters).
    pub query: String,
    pub meta: RecordMeta,
}

impl SavedSearch {
    pub fn new(name: &str, query: &str, device: DeviceId) -> Result<Self> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ModelError::InvalidSearchName(name.into()));
        }
        Ok(Self {
            id: SavedSearchId::new(),
            name: name.into(),
            query: query.trim().into(),
            meta: RecordMeta::new(device),
        })
    }

    /// The name as compared: lowercase.
    pub fn key(&self) -> String {
        name_key(&self.name)
    }
}

/// How a saved search's name is compared.
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

name_enum! {
    /// Where an alert searches.
    pub enum AlertSource (unknown: ModelError::UnknownAlertSource) {
        Arxiv => "arxiv",
        Pubmed => "pubmed",
        Crossref => "crossref",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alert {
    pub id: AlertId,
    /// Unique ignoring case, like a saved search's.
    pub name: String,
    pub source: AlertSource,
    /// In the source's own syntax (arXiv `au:`, PubMed `[au]`, Crossref `author:`), or plain words.
    pub query: String,
    /// The last check; the next one looks for papers since then.
    pub checked_at: Option<Timestamp>,
    pub meta: RecordMeta,
}

impl Alert {
    pub fn new(name: &str, source: AlertSource, query: &str, device: DeviceId) -> Result<Self> {
        let name = name.trim();
        if name.is_empty() || query.trim().is_empty() {
            return Err(ModelError::InvalidSearchName(name.into()));
        }
        Ok(Self {
            id: AlertId::new(),
            name: name.into(),
            source,
            query: query.trim().into(),
            checked_at: None,
            meta: RecordMeta::new(device),
        })
    }

    pub fn key(&self) -> String {
        name_key(&self.name)
    }
}
