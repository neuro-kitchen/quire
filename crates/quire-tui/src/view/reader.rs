//! Reader mode on the whole screen: a header (key, title, progress), the text in a centred column
//! with the chosen theme, search matches highlighted, the contents or search box on top, and the
//! keys at the bottom.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph};

use super::Graphics;
use crate::keys::{READER_HELP, SELECT_HELP};
use crate::reader::annotate::Marked;
use crate::reader::document::Kind;
use crate::reader::figures::Figure;
use crate::reader::layout::{self, Piece};
use crate::reader::{Overlay, Reader, Tab, Text, Theme};

/// Columns an image needs beside the text column; narrower screens show it above the text.
const MIN_IMAGE_WIDTH: u16 = 40;

/// Text and background of a theme; `None`: the terminal's own.
struct Colors {
    fg: Option<Color>,
    bg: Option<Color>,
    heading: Color,
    dim: Color,
    found: Style,
    /// Highlighted passages (a soft background, distinct from search matches).
    marked: Color,
}

fn colors(theme: Theme) -> Colors {
    let found = |bg| Style::new().bg(bg).fg(Color::Black);
    match theme {
        Theme::Dark => Colors {
            fg: None,
            bg: None,
            heading: Color::Cyan,
            dim: Color::DarkGray,
            found: found(Color::Yellow),
            marked: Color::Rgb(70, 60, 25),
        },
        Theme::Light => Colors {
            fg: Some(Color::Rgb(30, 30, 30)),
            bg: Some(Color::Rgb(250, 250, 247)),
            heading: Color::Rgb(20, 70, 140),
            dim: Color::Rgb(130, 130, 130),
            found: found(Color::Rgb(255, 225, 120)),
            marked: Color::Rgb(255, 243, 190),
        },
        Theme::Sepia => Colors {
            fg: Some(Color::Rgb(91, 70, 54)),
            bg: Some(Color::Rgb(244, 236, 216)),
            heading: Color::Rgb(140, 70, 30),
            dim: Color::Rgb(160, 140, 120),
            found: found(Color::Rgb(240, 200, 110)),
            marked: Color::Rgb(236, 222, 170),
        },
        Theme::Contrast => Colors {
            fg: Some(Color::White),
            bg: Some(Color::Black),
            heading: Color::LightYellow,
            dim: Color::Gray,
            found: found(Color::LightYellow),
            marked: Color::Rgb(60, 60, 0),
        },
    }
}

pub fn draw(frame: &mut Frame, reader: &mut Reader, graphics: &mut Graphics) {
    let c = colors(reader.settings.theme);
    let mut base = Style::new();
    if let Some(fg) = c.fg {
        base = base.fg(fg);
    }
    if let Some(bg) = c.bg {
        base = base.bg(bg);
    }
    let area = frame.area();
    frame.render_widget(Block::new().style(base), area);
    let [header, body, footer] =
        Layout::vertical([Constraint::Length(2), Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    let width = (reader.settings.width + 4).min(body.width);
    // The Figures tab with an image to show: beside the text when there is room, else above it.
    let figure = reader.shown_figure().and_then(|url| reader.figures.get(url)).cloned();
    let (text_area, image_area) = match &figure {
        Some(_) if graphics.available() && body.width >= width + MIN_IMAGE_WIDTH => {
            let [text, image] = Layout::horizontal([Constraint::Length(width), Constraint::Fill(1)]).areas(body);
            (text, Some(image))
        }
        Some(_) if graphics.available() => {
            let [image, text] = Layout::vertical([Constraint::Percentage(55), Constraint::Fill(1)]).areas(body);
            (text, Some(image))
        }
        _ => (body, None),
    };
    reader.height = text_area.height as usize;
    let [column] = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center).areas(text_area);
    let column = Rect { x: column.x + 2, width: column.width.saturating_sub(4), ..column };
    let figure_note = match (&figure, image_area) {
        (Some(Figure::Ready(path)), Some(area)) => {
            (!graphics.draw(frame, area, path)).then_some(" (this figure's image cannot be shown)")
        }
        (Some(Figure::Loading), Some(_)) => Some(" (getting the figure…)"),
        (Some(Figure::Failed(_)), Some(_)) => Some(" (the figure's image could not be downloaded)"),
        _ => None,
    };

    let page = reader.page();
    let total = page.laid.lines.len();
    let progress = if total <= reader.height { 100 } else { (page.top + reader.height).min(total) * 100 / total };
    let dim = base.fg(c.dim);
    let mut title = vec![
        Span::styled(format!(" {} ", reader.cite_key), base.fg(c.heading).add_modifier(Modifier::BOLD)),
        Span::styled(reader.title.clone(), base),
    ];
    let right = format!(" {progress}% ");
    let room = (header.width as usize).saturating_sub(right.len() + reader.cite_key.len() + 2);
    if reader.title.chars().count() > room {
        title[1] = Span::styled(reader.title.chars().take(room.saturating_sub(1)).collect::<String>() + "…", base);
    }
    let [first, tabs] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(header);
    let [left, percent] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(right.len() as u16)]).areas(first);
    frame.render_widget(Paragraph::new(Line::from(title)), left);
    frame.render_widget(Paragraph::new(Span::styled(right, dim)), percent);
    frame.render_widget(Paragraph::new(tab_bar(reader, base, &c)), tabs);

    let text: Vec<Line> = match (&reader.text, page.tab) {
        (Text::Failed(error), Tab::Paper) => {
            vec![Line::styled(format!("Could not read the paper: {error}"), base.fg(Color::Red))]
        }
        (Text::Loading, Tab::Paper) => vec![Line::styled("Getting the text (arXiv, Europe PMC or the PDF)…", dim)],
        _ => {
            let query = reader.query.to_lowercase();
            // Highlights and the selection are shown on the paper only.
            let on_paper = page.tab == Tab::Paper;
            let marks: Vec<&Marked> =
                if on_paper { reader.annotations.marks.iter().map(|(_, m)| m).collect() } else { Vec::new() };
            let selected = if on_paper { reader.selected() } else { None };
            let words = Words { marks: &marks, selected: selected.as_ref() };
            page.laid.lines[page.top.min(total)..(page.top + reader.height).min(total)]
                .iter()
                .map(|line| styled(line, base, &c, &query, &words))
                .collect()
        }
    };
    frame.render_widget(Paragraph::new(text), column);

    let status = match (&reader.overlay, reader.matches.len()) {
        _ if reader.selection.is_some() => format!(" {SELECT_HELP}"),
        (Overlay::None, n) if n > 0 => {
            format!(" “{}”: {} of {n} · {READER_HELP}", reader.query, reader.current_match + 1)
        }
        _ => format!(" {READER_HELP}"),
    };
    let status = format!("{status}{}", figure_note.unwrap_or_default());
    frame.render_widget(Paragraph::new(Span::styled(status, dim)), footer);

    match &reader.overlay {
        Overlay::None => {}
        Overlay::Search(typed) => input_box(frame, body, width, base, " Search ", typed),
        Overlay::Comment(typed) => {
            input_box(frame, body, width, base, " Comment on the highlight (Enter saves) ", typed)
        }
        Overlay::Note(typed) => input_box(frame, body, width, base, " Note on the paper (Enter saves) ", typed),
        Overlay::Contents(selected) => {
            let items: Vec<ListItem> = reader
                .page()
                .laid
                .headings
                .iter()
                .map(|(_, level, text)| {
                    let indent = "  ".repeat(level.saturating_sub(1).min(3) as usize);
                    ListItem::new(format!("{indent}{text}"))
                })
                .collect();
            let height = (items.len() as u16 + 2).min(body.height);
            let [row] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(body);
            let [popup] = Layout::horizontal([Constraint::Length(width.min(70))]).flex(Flex::Center).areas(row);
            frame.render_widget(Clear, popup);
            let list = List::new(items)
                .style(base)
                .highlight_style(base.fg(c.heading).add_modifier(Modifier::BOLD | Modifier::REVERSED))
                .block(Block::bordered().title(" Contents (Enter jumps, Esc closes) "));
            frame.render_stateful_widget(list, popup, &mut ListState::default().with_selected(Some(*selected)));
        }
    }
}

/// The tabs: the current one highlighted, each with its count (`Figures 4`); the paper's tab says
/// when its text is loading or missing.
fn tab_bar(reader: &Reader, base: Style, c: &Colors) -> Line<'static> {
    let mut spans = vec![Span::styled(" ", base)];
    for (index, page) in reader.pages.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" │ ", base.fg(c.dim)));
        }
        let mut label = page.tab.label().to_string();
        if let Some(count) = page.count {
            label = format!("{label} {count}");
        }
        if page.tab == Tab::Paper && reader.text == Text::Loading {
            label.push_str(" …");
        }
        if let (Tab::Paper, Some(source)) = (page.tab, reader.source) {
            label = format!("{label} · {source}");
        }
        let style = if index == reader.tab {
            base.fg(c.heading).add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            base.fg(c.dim)
        };
        spans.push(Span::styled(format!(" {label} "), style));
    }
    if reader.text == Text::Missing {
        spans.push(Span::styled(" │  no text: p in the library gets the PDF", base.fg(c.dim)));
    }
    Line::from(spans)
}

/// One laid-out line as styled spans: emphasis, headings, table rows, search matches.
/// A one-line text box at the bottom of the text.
fn input_box(frame: &mut Frame, body: Rect, width: u16, base: Style, title: &str, typed: &str) {
    let [row] = Layout::vertical([Constraint::Length(3)]).flex(Flex::End).areas(body);
    let [popup] = Layout::horizontal([Constraint::Length(width.min(70))]).flex(Flex::Center).areas(row);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(format!("{typed}▏")).style(base).block(Block::bordered().title(title.to_string())),
        popup,
    );
}

/// The words to mark on screen: highlighted passages and the selection.
struct Words<'a> {
    marks: &'a [&'a Marked],
    selected: Option<&'a Marked>,
}

impl Words<'_> {
    fn covers(marked: &Marked, block: Option<usize>, word: Option<usize>) -> bool {
        block == Some(marked.block) && word.is_some_and(|w| marked.words.contains(&w))
    }
}

fn styled(line: &layout::Line, base: Style, c: &Colors, query: &str, words: &Words) -> Line<'static> {
    let kind_style = match line.kind {
        Some(Kind::Heading(_)) => base.fg(c.heading).add_modifier(Modifier::BOLD),
        Some(Kind::TableRow) => base.fg(c.dim),
        Some(Kind::Equation) => base.fg(c.heading),
        _ => base,
    };
    let mut spans = Vec::new();
    for piece in &line.pieces {
        let mut style = kind_style;
        if piece.emphasis.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        if piece.emphasis.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if piece.emphasis.mention {
            style = style.fg(c.heading).add_modifier(Modifier::UNDERLINED);
        }
        if words.marks.iter().any(|m| Words::covers(m, line.block, piece.word)) {
            style = style.bg(c.marked);
        }
        if words.selected.is_some_and(|m| Words::covers(m, line.block, piece.word)) {
            style = style.add_modifier(Modifier::REVERSED);
        }
        highlight(piece, style, c.found, query, &mut spans);
    }
    Line::from(spans)
}

/// Splits `piece` around matches of `query` (already lowercase). Matches across pieces are not
/// marked; the line still counts as a match.
fn highlight(piece: &Piece, style: Style, found: Style, query: &str, spans: &mut Vec<Span<'static>>) {
    let lower = piece.text.to_lowercase();
    // Lowercasing can change byte lengths (rare letters): then no highlighting in this piece.
    if query.is_empty() || lower.len() != piece.text.len() {
        spans.push(Span::styled(piece.text.clone(), style));
        return;
    }
    let mut at = 0;
    for (start, _) in lower.match_indices(query) {
        if start < at {
            continue;
        }
        if start > at {
            spans.push(Span::styled(piece.text[at..start].to_string(), style));
        }
        spans.push(Span::styled(piece.text[start..start + query.len()].to_string(), found));
        at = start + query.len();
    }
    if at < piece.text.len() {
        spans.push(Span::styled(piece.text[at..].to_string(), style));
    }
}
