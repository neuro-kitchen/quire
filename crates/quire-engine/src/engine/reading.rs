//! Reading status and saved searches (personal records) through the engine.

use std::collections::HashMap;

use quire_core::SearchHit;
use quire_core::library::{reading, saved, search};
use quire_core::prelude::*;
use quire_model::{Reading, ReadingStatus, Reference, ReferenceId, SavedSearch};

use super::Engine;
use crate::error::Result;

impl Engine {
    /// Sets the paper `query` names to `status`; `false` if it already was.
    pub async fn set_reading(&self, query: &str, status: ReadingStatus) -> Result<(Reference, Reading, bool)> {
        let query = query.to_string();
        let result = self.run(move |lib| reading::set(lib, &query, status)).await?;
        self.mark_changed();
        Ok(result)
    }

    /// Marks `reference` reading if unread (the reader opened it); returns its status.
    pub async fn opened(&self, reference: ReferenceId) -> Result<ReadingStatus> {
        let status = self.run(move |lib| reading::opened(lib, reference)).await?;
        self.mark_changed();
        Ok(status)
    }

    /// Every reference not unread, with its status.
    pub async fn reading_statuses(&self) -> Result<HashMap<ReferenceId, Reading>> {
        self.run(|lib| reading::statuses(lib)).await
    }

    /// Saves `query` under `name` (replacing that search's query).
    pub async fn save_search(&self, name: &str, query: &str) -> Result<SavedSearch> {
        let (name, query) = (name.to_string(), query.to_string());
        let search = self.run(move |lib| saved::save(lib, &name, &query)).await?;
        self.mark_changed();
        Ok(search)
    }

    pub async fn saved_searches(&self) -> Result<Vec<SavedSearch>> {
        self.run(|lib| lib.saved_searches()).await
    }

    /// Runs the saved search `name`.
    pub async fn run_saved(&self, name: &str, limit: usize) -> Result<(SavedSearch, Vec<(Reference, SearchHit)>)> {
        let name = name.to_string();
        self.run(move |lib| {
            let found = saved::find(lib, &name)?;
            let hits = search::search(lib, &found.query, limit)?;
            Ok((found, hits))
        })
        .await
    }

    pub async fn rename_saved(&self, name: &str, new: &str) -> Result<SavedSearch> {
        let (name, new) = (name.to_string(), new.to_string());
        let search = self.run(move |lib| saved::rename(lib, &name, &new)).await?;
        self.mark_changed();
        Ok(search)
    }

    pub async fn remove_saved(&self, name: &str) -> Result<SavedSearch> {
        let name = name.to_string();
        let search = self.run(move |lib| saved::remove(lib, &name)).await?;
        self.mark_changed();
        Ok(search)
    }
}
