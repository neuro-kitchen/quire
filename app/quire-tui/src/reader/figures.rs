//! Figure images on the Figures tab: which image the tab shows (that of the figure at the top of
//! the screen) and whether its file is here. Drawing happens in `view::reader` with the
//! terminal's graphics protocol; without one (no kitty, sixel or iTerm2), no image is asked for.

use std::path::PathBuf;

use super::{Reader, Tab};

/// A figure image asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Figure {
    Loading,
    Ready(PathBuf),
    Failed(String),
}

impl Reader {
    /// The URL of the image the open tab shows: on the Figures tab, the figure at the top.
    pub fn shown_figure(&self) -> Option<&str> {
        let page = self.page();
        if page.tab != Tab::Figures {
            return None;
        }
        let block = page.current_block();
        page.images.iter().rev().find(|(start, _)| *start <= block).or(page.images.first()).map(|(_, url)| url.as_str())
    }

    /// The image to download now: the shown one, if not asked for yet (then marked loading).
    pub fn wanted_figure(&mut self) -> Option<String> {
        let url = self.shown_figure()?.to_string();
        if self.figures.contains_key(&url) {
            return None;
        }
        self.figures.insert(url.clone(), Figure::Loading);
        Some(url)
    }

    /// The image of `url` arrived (or failed).
    pub fn figure_loaded(&mut self, url: String, result: Result<PathBuf, String>) {
        self.figures.insert(url, result.map_or_else(Figure::Failed, Figure::Ready));
    }
}
