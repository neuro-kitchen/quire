//! Text from the library on a terminal. Titles, notes and errors come from outside (web pages,
//! `.bib` files, collaborators' devices); an escape sequence in one would drive the terminal: set
//! its title, write the clipboard (OSC 52), move the cursor, clear the screen. Only colour and style
//! (SGR, `ESC [ … m`, what quire's own styles are) pass; every other escape sequence and control
//! character but newline and tab is dropped.
//!
//! [`out!`](crate::out) and [`err!`](crate::err) print lines through it; [`data!`](crate::data)
//! prints raw data (an export, a paper's text) through it only when stdout is a terminal, so a
//! redirect to a file stays byte for byte.

use std::borrow::Cow;

/// `text` with every escape sequence but SGR, and every control character but `\n` and `\t`, left out.
pub fn for_terminal(text: &str) -> Cow<'_, str> {
    if !text.chars().any(unsafe_char) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            if !unsafe_char(c) {
                out.push(c);
            }
            continue;
        }
        match chars.next() {
            // CSI: kept only as SGR (`ESC [` digits and separators, then `m`).
            Some('[') => {
                let mut params = String::new();
                while let Some(&p) = chars.peek().filter(|p| p.is_ascii_digit() || matches!(p, ';' | ':')) {
                    params.push(p);
                    chars.next();
                }
                match chars.next() {
                    Some('m') => {
                        out.push_str("\x1b[");
                        out.push_str(&params);
                        out.push('m');
                    }
                    // Another CSI: skip to its final byte.
                    Some(f) if !('@'..='~').contains(&f) => {
                        for f in chars.by_ref() {
                            if ('@'..='~').contains(&f) {
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            // OSC, DCS, SOS, PM, APC: strings ended by BEL or `ESC \`.
            Some(']' | 'P' | 'X' | '^' | '_') => {
                while let Some(s) = chars.next() {
                    if s == '\x07' || (s == '\x1b' && chars.next_if_eq(&'\\').is_some()) {
                        break;
                    }
                }
            }
            // Any other escape: it and the next character.
            _ => {}
        }
    }
    Cow::Owned(out)
}

/// C0 controls (newline and tab are fine), DEL and C1 controls (`U+009B` is an 8-bit CSI).
fn unsafe_char(c: char) -> bool {
    (c.is_control() && c != '\n' && c != '\t') || ('\u{80}'..='\u{9f}').contains(&c)
}

/// `println!` of library text, safe on a terminal (see the module).
#[macro_export]
macro_rules! out {
    () => { anstream::println!() };
    ($($arg:tt)*) => { anstream::println!("{}", $crate::output::safe::for_terminal(&format!($($arg)*))) };
}

/// `eprintln!` of library text, safe on a terminal.
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => { anstream::eprintln!("{}", $crate::output::safe::for_terminal(&format!($($arg)*))) };
}

/// `print!` of raw data (an export, a paper's text): unchanged into a file or pipe, made safe on a
/// terminal.
#[macro_export]
macro_rules! data {
    ($($arg:tt)*) => {{
        let text = format!($($arg)*);
        if $crate::output::terminal::stdout_is_terminal() {
            print!("{}", $crate::output::safe::for_terminal(&text));
        } else {
            print!("{text}");
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A title from a crafted `.bib` file or a collaborator's device set the terminal's title and
    /// wrote its clipboard when `quire list` or `ref show` printed it.
    #[test]
    fn only_colour_passes() {
        let ours = "\x1b[1;36mkey\x1b[0m";
        assert_eq!(for_terminal(ours), ours);
        assert_eq!(for_terminal("Evil \x1b]0;PWNED\x07 title"), "Evil  title");
        assert_eq!(for_terminal("a\x1b]52;c;cGF5bG9hZA==\x1b\\b"), "ab");
        assert_eq!(for_terminal("clear\x1b[2Jme\x1b[10;20H"), "clearme");
        assert_eq!(for_terminal("over\rwrite\x07 \u{9b}31m"), "overwrite 31m");
        assert_eq!(for_terminal("lines\nand\ttabs"), "lines\nand\ttabs");
        assert!(matches!(for_terminal("plain"), Cow::Borrowed(_)));
    }
}
