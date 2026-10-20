//! Projects (Zotero's collections): renaming, removing, and which references they hold. Listing
//! and creating are in the crate root.

use crate::records::ProjectRecord;
use crate::{QuireLibrary, Result, query};

#[uniffi::export]
impl QuireLibrary {
    /// Renames the project at `path`; returns it with its new path.
    pub async fn rename_project(&self, path: String, name: String) -> Result<ProjectRecord> {
        self.with(|engine| async move {
            let (path, project) = engine.rename_project(&query::slug_path(&path)?, &name).await?;
            let references = query::projects(&engine)
                .await?
                .into_iter()
                .find(|p| p.path == path.to_string())
                .map_or(0, |p| p.references);
            Ok(ProjectRecord {
                path: path.to_string(),
                name: project.name,
                parent: path.parent().map(|p| p.to_string()),
                references,
            })
        })
        .await
    }

    /// Moves the project at `path` (and its subprojects if `recursive`) to the trash; its
    /// references stay in the library. Returns how many projects were removed.
    pub async fn remove_project(&self, path: String, recursive: bool) -> Result<u32> {
        self.with(|engine| async move { Ok(engine.remove_project(&query::slug_path(&path)?, recursive).await? as u32) })
            .await
    }

    /// Puts a reference into the project at `path`; `false` if it already was.
    pub async fn link(&self, id: String, path: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.link(&id, &query::slug_path(&path)?).await?.1) }).await
    }

    /// Takes a reference out of the project at `path` (it stays in the library); `false` if it was not in it.
    pub async fn unlink(&self, id: String, path: String) -> Result<bool> {
        self.with(|engine| async move { Ok(engine.unlink(&id, &query::slug_path(&path)?).await?.1) }).await
    }
}
