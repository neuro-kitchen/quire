//! `search`: full-text search of the library (titles, authors, abstracts, journals, identifiers,
//! tags, notes, highlights, and the text of papers opened in the reader or extracted with
//! `quire pdf extract`). Best matches first, each with the words found in context. Filters
//! (`status:`, `year:`, `pdf:`, `type:`, `in:`) narrow it; `--save NAME` keeps it (`quire saved`).

use clap::Args;
use quire_engine::{Engine, Reference, SEARCH_LIMIT, SearchHit};

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Words (all must match, as word prefixes); "a phrase"; field:word for title, author,
    /// abstract, journal, tag, note, key or doi; filters status:unread|reading|read,
    /// year:2019 or year:2019..2021, pdf:yes|no, type:<CSL type>, in:<project>
    #[arg(required_unless_present = "reindex")]
    query: Vec<String>,
    /// Also keep this search under a name (run it again with `quire saved run`)
    #[arg(long, value_name = "NAME")]
    save: Option<String>,
    /// Results shown at most
    #[arg(short = 'n', long, default_value_t = SEARCH_LIMIT)]
    limit: usize,
    /// First index the paper texts already stored (those extracted before search existed)
    #[arg(long)]
    reindex: bool,
}

pub async fn run(engine: &Engine, args: &SearchArgs) -> anyhow::Result<()> {
    if args.reindex {
        let count = engine.index_stored_texts().await?;
        output::hint(format!("{count} paper text(s) indexed"));
        if args.query.is_empty() {
            return Ok(());
        }
    }
    let query = args.query.join(" ");
    let hits = engine.search(&query, args.limit).await?;
    print_hits(&hits, &query);
    if let Some(name) = &args.save {
        let saved = engine.save_search(name, &query).await?;
        output::report(Tone::Ok, "saved", format!("{} (quire saved run {:?})", saved.name, saved.name));
    }
    Ok(())
}

/// The hits of a search, best first, with where each matched; or why nothing was found.
pub fn print_hits(hits: &[(Reference, SearchHit)], query: &str) {
    for (reference, hit) in hits {
        let year = reference.data.issued.map(|d| d.get_year().to_string()).unwrap_or_default();
        let title = reference.data.title().unwrap_or("(no title)");
        let place = if hit.in_text { "  (in the paper's text)" } else { "" };
        anstream::println!(
            "{}  {year}  {title}{}",
            style::paint(style::KEY, &reference.cite_key),
            style::paint(style::DIM, place)
        );
        if !hit.snippet.trim().is_empty() {
            anstream::println!("    {}", style::paint(style::DIM, hit.snippet.replace('\n', " ")));
        }
    }
    match hits.len() {
        0 => output::hint(format!(
            "nothing found for `{query}` (paper text is searched once extracted: `quire pdf extract`)"
        )),
        n => output::hint(format!("{n} found")),
    }
}
