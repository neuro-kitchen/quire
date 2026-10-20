//! Images in the terminal (figures in the reader), when it has a graphics protocol: kitty, sixel
//! or iTerm2, found by asking the terminal the first time a figure is to be shown (not at start:
//! a terminal that does not answer would delay every start). Kept out of
//! [`crate::state::State`] (the protocol state is neither `Clone` nor comparable); the event loop
//! owns it.

use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::cap_parser::QueryStdioOptions;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};

/// How long the terminal has to answer each step of the query (local terminals answer at once).
const QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(800);

#[derive(Default)]
pub struct Graphics {
    picker: Option<Picker>,
    /// The image drawn last, ready to draw again (decoding is the slow part).
    shown: Option<(PathBuf, StatefulProtocol)>,
}

impl Graphics {
    /// Asks the terminal what it can show; call in the alternate screen while nothing else reads
    /// the keyboard (the answer comes on stdin). Half blocks (any terminal) are not used: too
    /// coarse for figures.
    pub fn detect() -> Self {
        let options = QueryStdioOptions { timeout: QUERY_TIMEOUT, ..QueryStdioOptions::default() };
        let picker = Picker::from_query_stdio_with_options(options)
            .ok()
            .filter(|p| p.protocol_type() != ProtocolType::Halfblocks);
        Self { picker, shown: None }
    }

    pub fn available(&self) -> bool {
        self.picker.is_some()
    }

    /// Draws the image file at `path` fitted into `area`; `false` if it cannot be shown.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, path: &Path) -> bool {
        let Some(picker) = &self.picker else { return false };
        if self.shown.as_ref().is_none_or(|(shown, _)| shown != path) {
            let Ok(image) = image::ImageReader::open(path).and_then(|r| r.with_guessed_format()).map(|r| r.decode())
            else {
                return false;
            };
            let Ok(image) = image else { return false };
            self.shown = Some((path.to_path_buf(), picker.new_resize_protocol(image)));
        }
        let Some((_, protocol)) = self.shown.as_mut() else { return false };
        frame.render_stateful_widget(StatefulImage::default().resize(Resize::Fit(None)), area, protocol);
        true
    }
}
