//! What the bridge's tests share: a library in a temporary folder with three references.

#![allow(dead_code)] // each test file uses part of it

use std::sync::Arc;

use quire_ffi::{QuireLibrary, open_library_at};

pub const BIB: &str = r#"
@article{hodgkin1952,
  title = {A quantitative description of membrane current and its application to conduction and excitation in nerve},
  author = {Hodgkin, A. L. and Huxley, A. F.},
  journal = {The Journal of Physiology},
  year = {1952}, volume = {117}, number = {4}, pages = {500--544},
  doi = {10.1113/jphysiol.1952.sp004764},
}
@article{okeefe1971,
  title = {The hippocampus as a spatial map},
  author = {O'Keefe, J. and Dostrovsky, J.},
  journal = {Brain Research},
  year = {1971},
  doi = {10.1016/0006-8993(71)90358-1},
}
@book{buzsaki2006,
  title = {Rhythms of the Brain},
  author = {Buzsáki, György},
  publisher = {Oxford University Press},
  year = {2006},
}
"#;

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub library: Arc<QuireLibrary>,
}

pub async fn fixture() -> Fixture {
    // SAFETY: tests in this file set the same value; no other thread reads the environment yet.
    unsafe { std::env::set_var("QUIRE_NO_KEYRING", "1") };
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let library = open_library_at(dir.path().join("lib").display().to_string(), Some(config.display().to_string()))
        .await
        .unwrap();
    let bib = dir.path().join("refs.bib");
    std::fs::write(&bib, BIB).unwrap();
    assert_eq!(library.import_bib(bib.display().to_string(), None).await.unwrap(), 3);
    Fixture { dir, library }
}
