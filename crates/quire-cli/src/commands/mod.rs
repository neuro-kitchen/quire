//! One module per command or command group; each defines its arguments and `run`.
//!
//! - [`reference`]: `ref add|list|show|link|unlink|rm|restore|copy` (and the `add`, `list`,
//!   `show` shortcuts).
//! - [`project`], [`library`], [`config`], [`folder`], [`pdf`], [`access`], [`tag`], [`note`], [`highlight`]: their
//!   groups.
//! - [`editor`]: the user's editor, shared.
//! - [`import`], [`export`], [`init`], [`serve`], [`completions`]: single commands.

pub mod access;
pub mod completions;
pub mod config;
pub mod editor;
pub mod export;
pub mod folder;
pub mod highlight;
pub mod import;
pub mod init;
pub mod library;
pub mod note;
pub mod pdf;
pub mod project;
pub mod reference;
pub mod serve;
pub mod tag;
