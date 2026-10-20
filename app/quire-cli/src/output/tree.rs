//! Projects drawn as a tree:
//!
//! ```text
//! thesis  Thesis  3
//! ├── chapter-1  Chapter 1  2
//! │   └── methods  Methods  0
//! └── chapter-2  Chapter 2  1
//! ```

use quire_engine::ProjectSummary;

use super::style::{self, paint};

/// The tree for `projects`, which must be in depth-first order (as the engine returns them).
pub fn projects(projects: &[ProjectSummary]) -> String {
    let depths: Vec<usize> = projects.iter().map(|p| p.path.segments().len() - 1).collect();
    let mut lines = Vec::with_capacity(projects.len());
    for (index, summary) in projects.iter().enumerate() {
        let depth = depths[index];
        let mut prefix = String::new();
        for level in 1..=depth {
            let last = is_last(&depths, index, level);
            prefix.push_str(match (level == depth, last) {
                (true, true) => "└── ",
                (true, false) => "├── ",
                (false, true) => "    ",
                (false, false) => "│   ",
            });
        }
        let slug = summary.path.leaf().to_string();
        let name = if summary.project.name == slug { String::new() } else { format!("  {}", summary.project.name) };
        let count = paint(style::DIM, format!("  {}", summary.references));
        lines.push(format!("{}{}{name}{count}", paint(style::DIM, prefix), paint(style::KEY, slug)));
    }
    lines.join("\n")
}

/// Whether the ancestor of item `index` at `level` (or the item itself, at its own depth) is the
/// last among its siblings: no later item at that level before one at a shallower level.
fn is_last(depths: &[usize], index: usize, level: usize) -> bool {
    let mut current = index;
    while depths[current] > level {
        current -= 1;
    }
    depths[current + 1..].iter().take_while(|d| **d >= level).all(|d| *d != level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_siblings() {
        // a, a/b, a/b/c, a/d
        let depths = [0, 1, 2, 1];
        assert!(!is_last(&depths, 1, 1));
        assert!(is_last(&depths, 2, 2));
        assert!(!is_last(&depths, 2, 1));
        assert!(is_last(&depths, 3, 1));
    }
}
