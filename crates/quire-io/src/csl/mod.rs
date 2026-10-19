//! CSL-JSON: the format DOI services return (content negotiation) and Pandoc/Zotero exchange.
//!
//! - [`import`]: CSL-JSON → [`ReferenceData`](quire_model::ReferenceData), including the
//!   non-standard type names Crossref uses (`journal-article`, …).
//! - [`export`]: stored references → CSL-JSON.

mod export;
mod import;
mod types;
mod values;

pub use export::{to_csl_json, to_csl_value};
pub use import::{from_csl_json, from_csl_value};
