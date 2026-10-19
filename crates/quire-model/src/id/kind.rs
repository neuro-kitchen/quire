//! What an [`Id`](super::Id) identifies: one marker type per record kind, each with its TypeID prefix.
//! The markers are uninhabited; they exist only as type parameters.

/// A record kind: its TypeID prefix and a name for messages.
pub trait IdKind {
    /// TypeID prefix, lowercase ASCII letters (`ref` in `ref_01h…`).
    const PREFIX: &'static str;
    /// Name used in error messages.
    const NAME: &'static str;
}

macro_rules! id_kinds {
    ($($(#[$doc:meta])* $marker:ident => $prefix:literal, $name:literal;)+) => {
        $(
            $(#[$doc])*
            #[derive(Debug)]
            pub enum $marker {}

            impl IdKind for $marker {
                const PREFIX: &'static str = $prefix;
                const NAME: &'static str = $name;
            }
        )+

        /// Every prefix in use; no two kinds may share one.
        pub const PREFIXES: &[&str] = &[$($prefix),+];
    };
}

id_kinds! {
    /// A bibliographic reference.
    Reference => "ref", "reference";
    /// A project (a group of references; may be nested).
    Project => "prj", "project";
    Tag => "tag", "tag";
    /// The link between a reference and a stored file.
    Attachment => "att", "attachment";
    /// A device holding a copy of the library (for sync).
    Device => "dev", "device";
}

#[cfg(test)]
mod tests {
    use super::PREFIXES;

    #[test]
    fn prefixes_are_unique_lowercase_ascii() {
        for (i, p) in PREFIXES.iter().enumerate() {
            assert!(p.bytes().all(|b| b.is_ascii_lowercase()), "{p}");
            assert!(!PREFIXES[i + 1..].contains(p), "duplicate prefix {p}");
        }
    }
}
