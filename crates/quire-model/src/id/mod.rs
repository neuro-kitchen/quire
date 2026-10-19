//! Record IDs: a UUIDv7 per record, typed by what it identifies, shown as a TypeID
//! (`ref_01h455vb4pex5vsknk084sn02q`). See `.knowledge/data-foundations.md` (1, 1b, 1c).

pub mod kind;
mod typed;

pub use kind::IdKind;
pub use typed::Id;

pub type ReferenceId = Id<kind::Reference>;
pub type ProjectId = Id<kind::Project>;
pub type TagId = Id<kind::Tag>;
pub type NoteId = Id<kind::Note>;
pub type AttachmentId = Id<kind::Attachment>;
pub type DeviceId = Id<kind::Device>;
