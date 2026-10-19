//! Metadata lookup over the network (`.knowledge/metadata-sources.md`).
//!
//! - [`fetcher`]: [`Fetcher`], the entry point: what the user typed → [`Fetched`].
//! - [`doi`]: DOI content negotiation (Crossref, DataCite incl. arXiv, mEDRA).
//! - [`pubmed`]: PubMed / PubMed Central via NCBI's citation exporter.
//! - [`web`]: metadata tags of a web page.
//! - [`http`]: the rate-limited HTTP client.
//! - [`error`]: the error type, [`FetchError`].

pub mod doi;
pub mod error;
pub mod fetcher;
pub mod http;
pub mod pubmed;
pub mod web;

pub use error::{FetchError, Result};
pub use fetcher::{Endpoints, Fetched, Fetcher, Lookup};
