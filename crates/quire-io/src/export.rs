//! Writing references in any export format, with an export profile's settings (Better BibTeX's
//! export options, `.knowledge/roadmap.md` M6): fields to leave out, LaTeX commands for non-ASCII
//! text, month macros.

use biblatex::{Bibliography, ChunksExt};
use quire_model::Reference;

use crate::bib::{self, BibFormat, BibOptions, PreprintForm};
use crate::csl;
use crate::error::{IoError, Result};

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// BibLaTeX (Biber).
    #[default]
    Biblatex,
    /// Classic BibTeX.
    Bibtex,
    /// CSL-JSON (Pandoc, Zotero).
    CslJson,
    /// Hayagriva YAML (Typst).
    Hayagriva,
}

impl ExportFormat {
    pub const ALL: &[Self] = &[Self::Biblatex, Self::Bibtex, Self::CslJson, Self::Hayagriva];

    /// Name in config files and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Biblatex => "biblatex",
            Self::Bibtex => "bibtex",
            Self::CslJson => "csl-json",
            Self::Hayagriva => "hayagriva",
        }
    }

    /// Usual file extension.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Biblatex | Self::Bibtex => "bib",
            Self::CslJson => "json",
            Self::Hayagriva => "yml",
        }
    }

    /// The format a file name's extension asks for: `.bib` BibLaTeX, `.yml`/`.yaml` Hayagriva,
    /// `.json` CSL-JSON (any case).
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "bib" => Some(Self::Biblatex),
            "yml" | "yaml" => Some(Self::Hayagriva),
            "json" => Some(Self::CslJson),
            _ => None,
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = IoError;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL.iter().copied().find(|f| f.as_str() == s).ok_or_else(|| IoError::UnknownFormat(s.into()))
    }
}

/// How to export.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportOptions {
    pub format: ExportFormat,
    /// Fields left out: BibLaTeX names (`abstract`, `file`) for `.bib` and Hayagriva, CSL
    /// variable names for CSL-JSON.
    pub omit: Vec<String>,
    /// `.bib` only: non-ASCII as LaTeX commands.
    pub ascii: bool,
    /// BibTeX only: `month = jul`.
    pub month_macros: bool,
    /// `.bib` only: preprints as `@online` (Biber) or `@article` (Typst).
    pub preprints: PreprintForm,
}

/// `references` written as `options` says.
pub fn export(references: &[Reference], options: &ExportOptions) -> Result<String> {
    match options.format {
        ExportFormat::Biblatex | ExportFormat::Bibtex => {
            let format = if options.format == ExportFormat::Bibtex { BibFormat::Bibtex } else { BibFormat::Biblatex };
            let bib = BibOptions {
                format,
                omit: options.omit.clone(),
                ascii: options.ascii,
                month_macros: options.month_macros,
                preprints: options.preprints,
            };
            bib::to_bib_with(references, &bib)
        }
        ExportFormat::CslJson => {
            let items: Vec<serde_json::Value> = references
                .iter()
                .map(|reference| {
                    let mut item = csl::to_csl_value(reference);
                    if let Some(object) = item.as_object_mut() {
                        for field in &options.omit {
                            object.remove(field);
                        }
                    }
                    item
                })
                .collect();
            Ok(serde_json::to_string_pretty(&items)? + "\n")
        }
        ExportFormat::Hayagriva => to_hayagriva(references, &options.omit),
    }
}

/// Hayagriva YAML, converted from our BibLaTeX entries by Typst's own `hayagriva` crate, so the
/// result reads exactly as Typst reads a `.bib` file.
fn to_hayagriva(references: &[Reference], omit: &[String]) -> Result<String> {
    hayagriva::io::to_yaml_str(&hayagriva_library(references, omit)?).map_err(|e| IoError::Hayagriva(e.to_string()))
}

/// `references` as `hayagriva` entries (keyed by cite key), converted from our BibLaTeX entries.
/// Preprints use the `@article` form, which `hayagriva` renders as a preprint.
fn hayagriva_library(references: &[Reference], omit: &[String]) -> Result<hayagriva::Library> {
    let mut bibliography = Bibliography::new();
    for mut entry in bib::entries(references, omit, PreprintForm::Article)? {
        // `hayagriva` turns numeric `54--58` into a range but keeps `D480--D489` as written, which
        // Typst then prints with two dashes; one hyphen is read as a range either way.
        if let Some(pages) = entry.get("pages").map(|chunks| chunks.format_verbatim()) {
            entry.set("pages", vec![biblatex::Spanned::detached(biblatex::Chunk::Normal(pages.replace("--", "-")))]);
        }
        bibliography.insert(entry);
    }
    hayagriva::io::from_biblatex(&bibliography)
        .map_err(|errors| IoError::Hayagriva(errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_model::{
        CiteKey, Creator, CreatorRole, Date, DeviceId, EntryKind, Identifier, Origin, ReferenceData, TextField,
    };

    fn reference() -> Reference {
        let mut data = ReferenceData::new(EntryKind::ArticleJournal);
        data.creators.push(Creator::person(CreatorRole::Author, "Kucsko", Some("G.".into())));
        data.issued = Some(Date::new(2013, Some(7), Some(31)).unwrap());
        data.set(TextField::Title, "Nanometre-scale thermometry in a living cell");
        data.set(TextField::ContainerTitle, "Nature");
        data.set(TextField::Volume, "500");
        data.set(TextField::Page, "D480-D489");
        data.set(TextField::Abstract, "Long text");
        data.identifiers.push(Identifier::parse("10.1038/nature12373").unwrap());
        Reference::new(CiteKey::parse("kucsko2013").unwrap(), data, Origin::User, DeviceId::new())
    }

    #[test]
    fn format_from_a_file_name() {
        let format = |name: &str| ExportFormat::from_path(std::path::Path::new(name));
        assert_eq!(format("refs.bib"), Some(ExportFormat::Biblatex));
        assert_eq!(format("~/thesis/refs.YAML"), Some(ExportFormat::Hayagriva));
        assert_eq!(format("refs.yml"), Some(ExportFormat::Hayagriva));
        assert_eq!(format("refs.json"), Some(ExportFormat::CslJson));
        assert_eq!(format("refs.txt"), None);
        assert_eq!(format("refs"), None);
    }

    #[test]
    fn hayagriva_yaml() {
        let options = ExportOptions {
            format: ExportFormat::Hayagriva,
            omit: vec!["abstract".into()],
            ..ExportOptions::default()
        };
        let yaml = export(&[reference()], &options).unwrap();
        insta::assert_snapshot!(yaml);
    }

    #[test]
    fn csl_json_omits_variables() {
        let options =
            ExportOptions { format: ExportFormat::CslJson, omit: vec!["abstract".into()], ..ExportOptions::default() };
        let json = export(&[reference()], &options).unwrap();
        assert!(json.contains("\"container-title\": \"Nature\"") && !json.contains("abstract"), "{json}");
    }

    #[test]
    fn format_names() {
        for format in ExportFormat::ALL {
            assert_eq!(format.as_str().parse::<ExportFormat>().unwrap(), *format);
        }
        assert!("ris".parse::<ExportFormat>().is_err());
    }
}
