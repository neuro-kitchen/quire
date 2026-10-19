//! Metadata lookup over the network (`.knowledge/metadata-sources.md`).
//!
//! - [`fetcher`]: [`Fetcher`], the entry point: what the user typed → [`Fetched`], with fallbacks.
//! - [`doi`]: DOI content negotiation (Crossref, DataCite incl. arXiv, mEDRA).
//! - [`crossref`]: Crossref's REST API (fallback for DOIs).
//! - [`arxiv`]: the arXiv API (fallback for arXiv IDs).
//! - [`europepmc`]: abstracts by DOI or PMID.
//! - [`fulltext`]: a paper's structured full text for reader mode (arXiv HTML, Europe PMC JATS).
//! - [`pubmed`]: PubMed / PubMed Central via NCBI's citation exporter.
//! - [`pdf`]: finding and downloading a PDF (page link, arXiv, Unpaywall, landing page).
//! - [`web`]: metadata tags and JSON-LD of a web page.
//! - [`http`]: the rate-limited HTTP client; [`challenge`]: recognising "are you human" pages.
//! - [`names`]: person names given as one string.
//! - [`error`]: the error type, [`FetchError`].

pub mod arxiv;
pub mod challenge;
pub mod crossref;
pub mod doi;
pub mod error;
pub mod europepmc;
pub mod fetcher;
pub mod fulltext;
pub mod http;
pub mod names;
pub mod pdf;
pub mod pubmed;
pub mod web;

pub use error::{FetchError, Result};
pub use fetcher::{Endpoints, Fetched, Fetcher, Lookup};
pub use fulltext::{FullText, FullTextSource};
pub use pdf::FoundPdf;
