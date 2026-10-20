//! Reading a paper: its stored Markdown as blocks of styled words ([`document`], with LaTeX shown
//! as Unicode by [`math`]), and those blocks split into what a reader shows on separate tabs —
//! running text, figures, tables, equations, references ([`paper`]). Shared by the terminal reader
//! and the apps (`quire-ffi`), so both read a paper the same way.

pub mod document;
pub mod math;
pub mod paper;
