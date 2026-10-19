//! One module per command (file name = command name); each defines its arguments and `run`.
//! `link` also serves `unlink`, `trash` serves `rm` and `restore`.

pub mod add;
pub mod config;
pub mod export;
pub mod import;
pub mod init;
pub mod link;
pub mod list;
pub mod project;
pub mod show;
pub mod trash;
