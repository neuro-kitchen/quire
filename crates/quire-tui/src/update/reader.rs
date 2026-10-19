//! Opening the reader and passing its keys and results through ([`crate::reader::Reader`] does
//! the work).

use super::Effect;
use crate::reader::{Reader, ReaderAction, ReaderEffect, paper};
use crate::state::{Row, State};

pub(super) fn has_pdf(row: &Row) -> bool {
    matches!(row.pdf, Some(quire_engine::PdfStatus::Available { .. }))
}

/// Opens the selected reference in the reader, on its abstract; its text (arXiv, Europe PMC or
/// the PDF) and its highlights and notes load in the background.
pub(super) fn open(state: &mut State) -> Vec<Effect> {
    let Some(row) = state.current() else { return Vec::new() };
    let has_text = has_pdf(row) || quire_engine::online_candidate(&row.reference);
    let reference = &row.reference;
    let id = reference.id;
    let title = reference.data.title().unwrap_or("(no title)").to_string();
    let front = paper::front_matter(reference);
    let resume = state.positions.get(&id).copied();
    let key = reference.cite_key.to_string();
    state.reader = Some(Reader::new(id, key, title, front, has_text, state.reader_settings, resume));
    let mut effects = vec![Effect::LoadAnnotations(id)];
    if has_text {
        effects.push(Effect::LoadText(id));
    }
    effects
}

/// The text arrived (or failed) for the paper `id`.
pub(super) fn text_loaded(
    state: &mut State,
    id: quire_model::ReferenceId,
    text: Result<quire_engine::PaperText, String>,
) {
    let has_pdf = state.rows.iter().any(|r| r.reference.id == id && has_pdf(r));
    let Some(reader) = state.reader.as_mut().filter(|r| r.id == id) else { return };
    match text {
        Ok(text) => {
            reader.loaded(&text.markdown, text.source.has_math());
            reader.source = Some(text.source.label());
        }
        // Without a PDF, "no text" is not an error: the abstract is all there is.
        Err(_) if !has_pdf => reader.no_text(),
        Err(error) => reader.failed(error),
    }
}

pub(super) fn action(state: &mut State, action: ReaderAction) -> Vec<Effect> {
    let Some(reader) = state.reader.as_mut() else { return Vec::new() };
    match reader.apply(action) {
        ReaderEffect::None => Vec::new(),
        ReaderEffect::Refresh => vec![Effect::RefreshText(reader.id)],
        ReaderEffect::SaveHighlight { quote, prefix, comment } => {
            vec![Effect::AddHighlight { reference: reader.id, quote, prefix, comment }]
        }
        ReaderEffect::RemoveHighlight(highlight) => vec![Effect::RemoveHighlight { reference: reader.id, highlight }],
        ReaderEffect::AddNote(body) => vec![Effect::AddNote { reference: reader.id, body }],
        ReaderEffect::SaveSettings => {
            state.reader_settings = reader.settings;
            vec![Effect::SaveReaderSettings(reader.settings)]
        }
        ReaderEffect::Close => {
            let (id, block) = (reader.id, reader.paper_block());
            state.reader = None;
            let Some(block) = block else { return Vec::new() };
            state.positions.insert(id, block);
            vec![Effect::SavePositions(state.positions.iter().map(|(k, v)| (*k, *v)).collect())]
        }
    }
}
