//! Real CSL-JSON responses (saved from doi.org content negotiation, 2026-10-18) through to `.bib`:
//! a Crossref journal article and an arXiv preprint (DataCite).

use quire_io::{BibFormat, from_csl_json, to_bib};
use quire_model::{CiteKey, DeviceId, IdentifierKind, Origin, Reference, ReferenceData};

fn load(name: &str) -> ReferenceData {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    from_csl_json(&text).unwrap().remove(0)
}

fn bib(key: &str, data: ReferenceData) -> String {
    let reference = Reference::new(CiteKey::parse(key).unwrap(), data, Origin::User, DeviceId::new());
    to_bib(&[reference], BibFormat::Biblatex).unwrap()
}

#[test]
fn crossref_journal_article() {
    let data = load("csl-crossref-nature12373.json");
    assert_eq!(data.creators.len(), 8);
    assert_eq!(data.identifiers[0].kind(), IdentifierKind::Doi);
    insta::assert_snapshot!(bib("kucsko2013", data));
}

#[test]
fn arxiv_preprint() {
    let data = load("csl-arxiv-1706.03762.json");
    assert_eq!(data.identifiers[0].kind(), IdentifierKind::Arxiv);
    insta::assert_snapshot!(bib("vaswani2017", data));
}
