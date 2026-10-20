//! `tag add|rm|list|rename|delete|color`: labels on references, as in Zotero. Tags are created
//! on first use and are unique ignoring case; renaming onto an existing tag merges the two. Up to
//! nine tags can be colored (their position 1–9 is their key in the TUI).

use clap::{Args, Subcommand};
use quire_engine::{Engine, Tag};

use crate::output::{self, Tone, style};

#[derive(Args, Debug)]
pub struct TagArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Put tags on a reference
    Add {
        /// Cite key, DOI / arXiv ID / PMID, or reference ID
        reference: String,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Take tags off a reference
    Rm {
        reference: String,
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// List tags with how many references carry each
    List {
        /// Print JSON
        #[arg(long)]
        json: bool,
    },
    /// Rename a tag (onto an existing name: merge the two)
    Rename { old: String, new: String },
    /// Delete a tag from every reference
    Delete { name: String },
    /// Color a tag (`none` removes the color): red, orange, yellow, green, blue, purple, pink, gray or #rrggbb
    Color {
        name: String,
        color: String,
        /// Its position and key, 1–9 (default: the first free one)
        #[arg(long)]
        key: Option<u8>,
    },
}

pub async fn run(engine: &Engine, args: &TagArgs) -> anyhow::Result<()> {
    match &args.action {
        Action::Add { reference, tags } => {
            let (reference, outcome) = engine.tag(reference, tags).await?;
            for (name, added) in outcome {
                let (tone, label) = if added { (Tone::Ok, "tagged") } else { (Tone::Warn, "unchanged") };
                output::report(tone, label, format!("{}  {}", style::paint(style::KEY, &reference.cite_key), name));
            }
        }
        Action::Rm { reference, tags } => {
            let (reference, outcome) = engine.untag(reference, tags).await?;
            for (name, removed) in outcome {
                let (tone, label) = if removed { (Tone::Ok, "untagged") } else { (Tone::Warn, "unchanged") };
                output::report(tone, label, format!("{}  {}", style::paint(style::KEY, &reference.cite_key), name));
            }
        }
        Action::List { json } => {
            let tags = engine.tags().await?;
            if *json {
                let list: Vec<_> = tags
                    .iter()
                    .map(|(t, n)| serde_json::json!({ "name": t.name, "color": t.color, "key": t.position, "references": n }))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else if tags.is_empty() {
                output::hint("No tags yet: `quire tag add <reference> <tag>`.");
            } else {
                let width = tags.iter().map(|(t, _)| t.name.chars().count()).max().unwrap_or(0);
                for (tag, count) in &tags {
                    let key = tag.position.map(|p| format!("[{p}] ")).unwrap_or_else(|| "    ".into());
                    let name = format!("{:<width$}", tag.name);
                    anstream::println!(
                        "{}{}  {}",
                        style::paint(style::DIM, key),
                        paint_tag(tag, &name),
                        style::paint(style::DIM, count)
                    );
                }
            }
        }
        Action::Rename { old, new } => {
            let (tag, merged) = engine.rename_tag(old, new).await?;
            let label = if merged { "merged" } else { "renamed" };
            output::report(Tone::Ok, label, format!("{old} → {}", paint_tag(&tag, &tag.name)));
        }
        Action::Delete { name } => {
            let count = engine.delete_tag(name).await?;
            output::report(Tone::Info, "deleted", format!("{name} (was on {count} references)"));
        }
        Action::Color { name, color, key } => {
            let color = (!color.eq_ignore_ascii_case("none")).then_some(color.as_str());
            let tag = engine.color_tag(name, color, *key).await?;
            match tag.position {
                Some(position) => {
                    output::report(Tone::Ok, "colored", format!("{} (key {position})", paint_tag(&tag, &tag.name)))
                }
                None => output::report(Tone::Ok, "uncolored", &tag.name),
            }
        }
    }
    Ok(())
}

/// `text` in the tag's color (24-bit), plain for an uncolored tag.
pub fn paint_tag(tag: &Tag, text: &str) -> String {
    let Some(rgb) = tag.color.as_deref().and_then(parse_hex) else { return text.to_string() };
    let color = anstyle::RgbColor(rgb.0, rgb.1, rgb.2);
    style::paint(anstyle::Style::new().fg_color(Some(color.into())).bold(), text)
}

fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    Some((byte(0)?, byte(2)?, byte(4)?))
}
