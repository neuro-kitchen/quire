//! LaTeX math → Unicode text a terminal can show: `\frac{QK^T}{\sqrt{d_k}}` → `(QKᵀ)/√dₖ`. Greek
//! letters and operators become their characters, scripts become superscript/subscript
//! characters where Unicode has them all (else `^(…)` / `_(…)`), fractions `a/b`, roots `√x`,
//! accents combining marks; styling (`\mathrm`, `\displaystyle`) is dropped. `\\` (a new row)
//! becomes a line break.

/// The Unicode text of `tex`.
pub fn to_unicode(tex: &str) -> String {
    let mut parser = Parser { chars: tex.chars().collect(), pos: 0 };
    let text = parser.sequence(None);
    text.split('\n').map(tidy).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    /// Text up to `end` (consumed) or the end.
    fn sequence(&mut self, end: Option<char>) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if Some(c) == end {
                self.pos += 1;
                break;
            }
            match c {
                '{' => {
                    self.pos += 1;
                    out.push_str(&self.sequence(Some('}')));
                }
                '}' => self.pos += 1,
                '^' | '_' => {
                    self.pos += 1;
                    let argument = self.argument();
                    out.push_str(&script(&argument, c == '^'));
                }
                '\\' => out.push_str(&self.command()),
                '&' | '~' => {
                    self.pos += 1;
                    out.push(' ');
                }
                '\'' => {
                    self.pos += 1;
                    out.push('′');
                }
                _ => {
                    self.pos += 1;
                    out.push(c);
                }
            }
        }
        out
    }

    /// One argument: a `{group}`, a command or a character.
    fn argument(&mut self) -> String {
        self.skip_spaces();
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                self.sequence(Some('}'))
            }
            Some('\\') => self.command(),
            Some(c) => {
                self.pos += 1;
                c.to_string()
            }
            None => String::new(),
        }
    }

    /// `[…]` if present.
    fn optional(&mut self) -> Option<String> {
        self.skip_spaces();
        if self.peek() != Some('[') {
            return None;
        }
        self.pos += 1;
        Some(self.sequence(Some(']')))
    }

    /// The name after `\`: letters, or one other character.
    fn name(&mut self) -> String {
        self.pos += 1;
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        if self.pos == start && self.peek().is_some() {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn command(&mut self) -> String {
        let name = self.name();
        match name.as_str() {
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let (top, bottom) = (self.argument(), self.argument());
                format!("{}/{}", atom(&top), atom(&bottom))
            }
            "sqrt" => {
                let index = self.optional();
                let root = match index.as_deref().map(str::trim) {
                    Some("3") => "∛",
                    Some("4") => "∜",
                    _ => "√",
                };
                format!("{root}{}", atom(&self.argument()))
            }
            "mathbb" => self.argument().chars().map(double_struck).collect(),
            "mathrm" | "text" | "textrm" | "textit" | "textbf" | "mathit" | "mathbf" | "boldsymbol" | "bm"
            | "mathsf" | "mathtt" | "mathcal" | "mathscr" | "mathfrak" | "mathnormal" | "operatorname" | "mbox"
            | "emph" | "hbox" => self.argument(),
            "hat" | "widehat" => accent(&self.argument(), '\u{302}'),
            "bar" | "overline" => accent(&self.argument(), '\u{304}'),
            "tilde" | "widetilde" => accent(&self.argument(), '\u{303}'),
            "dot" => accent(&self.argument(), '\u{307}'),
            "ddot" => accent(&self.argument(), '\u{308}'),
            "vec" | "overrightarrow" => accent(&self.argument(), '\u{20d7}'),
            "left" | "right" | "big" | "Big" | "bigg" | "Bigg" | "bigl" | "bigr" | "Bigl" | "Bigr" | "middle" => {
                self.skip_spaces();
                // `\left.`: an invisible delimiter.
                if self.peek() == Some('.') {
                    self.pos += 1;
                }
                String::new()
            }
            "begin" | "end" => {
                let environment = self.argument();
                if name == "begin" && matches!(environment.as_str(), "array" | "tabular") {
                    self.argument();
                }
                if environment == "cases" && name == "begin" { "{ ".into() } else { String::new() }
            }
            "label" | "tag" | "hspace" | "vspace" | "phantom" => {
                self.argument();
                String::new()
            }
            "\\" | "newline" | "cr" => "\n".into(),
            "quad" => "  ".into(),
            "qquad" => "    ".into(),
            "," | ";" | ":" | " " | ">" => " ".into(),
            "!" | "displaystyle" | "textstyle" | "scriptstyle" | "limits" | "nolimits" | "nonumber" | "notag"
            | "mathstrut" | "relax" => String::new(),
            "{" | "}" | "%" | "$" | "#" | "_" | "&" => name,
            "|" => "‖".into(),
            other => symbol(other).map_or_else(|| other.to_string(), str::to_string),
        }
    }
}

/// `x` stays, `a+b` becomes `(a+b)` (for fractions and roots).
fn atom(text: &str) -> String {
    let text = text.trim();
    let simple = text.chars().count() <= 1
        || text.chars().all(|c| {
            c.is_alphanumeric()
                || matches!(c, '√' | '∛' | '∜' | '′' | '\u{300}'..='\u{36f}' | '\u{20d7}')
                || SUPERSCRIPTS.iter().chain(SUBSCRIPTS).any(|(_, s)| *s == c)
        })
        || (text.starts_with('(') && text.ends_with(')') && balanced(&text[1..text.len() - 1]));
    if simple { text.to_string() } else { format!("({text})") }
}

fn balanced(text: &str) -> bool {
    let mut depth = 0i32;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}

/// `T` as `ᵀ` (or `_`/`^(…)` when a character has no script form).
fn script(text: &str, superscript: bool) -> String {
    let text = text.trim();
    let table = if superscript { SUPERSCRIPTS } else { SUBSCRIPTS };
    let mapped: Option<String> =
        text.chars().map(|c| table.iter().find(|(from, _)| *from == c).map(|(_, to)| *to)).collect();
    match mapped {
        Some(mapped) if !mapped.is_empty() => mapped,
        _ => format!("{}{}", if superscript { '^' } else { '_' }, atom(text)),
    }
}

fn accent(text: &str, mark: char) -> String {
    let text = text.trim();
    if text.chars().count() == 1 { format!("{text}{mark}") } else { format!("{}{mark}", atom(text)) }
}

fn double_struck(c: char) -> char {
    match c {
        'R' => 'ℝ',
        'N' => 'ℕ',
        'Z' => 'ℤ',
        'Q' => 'ℚ',
        'C' => 'ℂ',
        'E' => '𝔼',
        'P' => 'ℙ',
        '1' => '𝟙',
        other => other,
    }
}

/// Spaces collapsed, relations spaced, none inside brackets or before punctuation.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if RELATIONS.contains(&c) {
            let trimmed = out.trim_end().len();
            out.truncate(trimmed);
            out.push(' ');
            out.push(c);
            out.push(' ');
        } else if c.is_whitespace() {
            if !out.ends_with([' ', '(', '[']) && !out.is_empty() {
                out.push(' ');
            }
        } else {
            if matches!(c, ')' | ']' | ',' | ';' | '.') && out.ends_with(' ') && !out.ends_with(" = ") {
                out.pop();
            }
            out.push(c);
        }
    }
    out.trim().to_string()
}

const RELATIONS: &[char] =
    &['=', '≠', '≤', '≥', '≈', '≡', '∼', '≃', '≅', '∝', '←', '→', '⇒', '⇔', '↦', '∈', '∉', '⊂', '⊆'];

fn symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ϵ",
        "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "ϑ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "varpi" => "ϖ",
        "rho" => "ρ",
        "varrho" => "ϱ",
        "sigma" => "σ",
        "varsigma" => "ς",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "ϕ",
        "varphi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        "cdot" => "·",
        "times" => "×",
        "div" => "÷",
        "pm" => "±",
        "mp" => "∓",
        "ast" => "∗",
        "star" => "⋆",
        "circ" => "∘",
        "bullet" => "•",
        "oplus" => "⊕",
        "otimes" => "⊗",
        "odot" => "⊙",
        "leq" | "le" => "≤",
        "geq" | "ge" => "≥",
        "neq" | "ne" => "≠",
        "approx" => "≈",
        "equiv" => "≡",
        "sim" => "∼",
        "simeq" => "≃",
        "cong" => "≅",
        "propto" => "∝",
        "ll" => "≪",
        "gg" => "≫",
        "in" => "∈",
        "notin" => "∉",
        "ni" => "∋",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "cup" => "∪",
        "cap" => "∩",
        "emptyset" | "varnothing" => "∅",
        "forall" => "∀",
        "exists" => "∃",
        "neg" | "lnot" => "¬",
        "land" | "wedge" => "∧",
        "lor" | "vee" => "∨",
        "to" | "rightarrow" => "→",
        "leftarrow" | "gets" => "←",
        "Rightarrow" | "implies" => "⇒",
        "Leftarrow" => "⇐",
        "Leftrightarrow" | "iff" => "⇔",
        "leftrightarrow" => "↔",
        "mapsto" => "↦",
        "uparrow" => "↑",
        "downarrow" => "↓",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "iint" => "∬",
        "oint" => "∮",
        "ldots" | "dots" | "dotsc" | "dotsb" => "…",
        "cdots" => "⋯",
        "vdots" => "⋮",
        "ddots" => "⋱",
        "prime" => "′",
        "ell" => "ℓ",
        "hbar" => "ℏ",
        "Re" => "ℜ",
        "Im" => "ℑ",
        "angle" => "∠",
        "perp" => "⊥",
        "parallel" => "∥",
        "mid" => "|",
        "langle" => "⟨",
        "rangle" => "⟩",
        "lceil" => "⌈",
        "rceil" => "⌉",
        "lfloor" => "⌊",
        "rfloor" => "⌋",
        "lvert" | "rvert" | "vert" => "|",
        "lVert" | "rVert" | "Vert" => "‖",
        "degree" => "°",
        "top" => "ᵀ",
        "sin" | "cos" | "tan" | "log" | "ln" | "exp" | "max" | "min" | "sup" | "inf" | "lim" | "det" | "arg"
        | "tanh" | "sinh" | "cosh" | "arcsin" | "arccos" | "arctan" | "mod" | "gcd" | "Pr" | "deg" | "dim" | "ker"
        | "liminf" | "limsup" | "argmax" | "argmin" => return Some(function(name)),
        _ => return None,
    })
}

/// Function names keep their letters (`\max` → `max`).
fn function(name: &str) -> &'static str {
    [
        "sin", "cos", "tan", "log", "ln", "exp", "max", "min", "sup", "inf", "lim", "det", "arg", "tanh", "sinh",
        "cosh", "arcsin", "arccos", "arctan", "mod", "gcd", "Pr", "deg", "dim", "ker", "liminf", "limsup", "argmax",
        "argmin",
    ]
    .into_iter()
    .find(|f| *f == name)
    .unwrap_or("")
}

/// Superscript forms Unicode has (all digits, most letters, a few signs).
pub(crate) const SUPERSCRIPTS: &[(char, char)] = &[
    ('0', '⁰'),
    ('1', '¹'),
    ('2', '²'),
    ('3', '³'),
    ('4', '⁴'),
    ('5', '⁵'),
    ('6', '⁶'),
    ('7', '⁷'),
    ('8', '⁸'),
    ('9', '⁹'),
    ('+', '⁺'),
    ('-', '⁻'),
    ('−', '⁻'),
    ('=', '⁼'),
    ('(', '⁽'),
    (')', '⁾'),
    ('a', 'ᵃ'),
    ('b', 'ᵇ'),
    ('c', 'ᶜ'),
    ('d', 'ᵈ'),
    ('e', 'ᵉ'),
    ('f', 'ᶠ'),
    ('g', 'ᵍ'),
    ('h', 'ʰ'),
    ('i', 'ⁱ'),
    ('j', 'ʲ'),
    ('k', 'ᵏ'),
    ('l', 'ˡ'),
    ('m', 'ᵐ'),
    ('n', 'ⁿ'),
    ('o', 'ᵒ'),
    ('p', 'ᵖ'),
    ('r', 'ʳ'),
    ('s', 'ˢ'),
    ('t', 'ᵗ'),
    ('u', 'ᵘ'),
    ('v', 'ᵛ'),
    ('w', 'ʷ'),
    ('x', 'ˣ'),
    ('y', 'ʸ'),
    ('z', 'ᶻ'),
    ('A', 'ᴬ'),
    ('B', 'ᴮ'),
    ('D', 'ᴰ'),
    ('E', 'ᴱ'),
    ('G', 'ᴳ'),
    ('H', 'ᴴ'),
    ('I', 'ᴵ'),
    ('J', 'ᴶ'),
    ('K', 'ᴷ'),
    ('L', 'ᴸ'),
    ('M', 'ᴹ'),
    ('N', 'ᴺ'),
    ('O', 'ᴼ'),
    ('P', 'ᴾ'),
    ('R', 'ᴿ'),
    ('T', 'ᵀ'),
    ('U', 'ᵁ'),
    ('V', 'ⱽ'),
    ('W', 'ᵂ'),
    ('′', '′'),
    ('∗', '∗'),
    ('*', '∗'),
    ('†', '†'),
    ('‡', '‡'),
    (',', ','),
    ('α', 'ᵅ'),
    ('β', 'ᵝ'),
    ('γ', 'ᵞ'),
    ('δ', 'ᵟ'),
    ('θ', 'ᶿ'),
    ('ϕ', 'ᵠ'),
    ('φ', 'ᵠ'),
    ('χ', 'ᵡ'),
];

/// Subscript forms Unicode has (digits, some letters).
pub(crate) const SUBSCRIPTS: &[(char, char)] = &[
    ('0', '₀'),
    ('1', '₁'),
    ('2', '₂'),
    ('3', '₃'),
    ('4', '₄'),
    ('5', '₅'),
    ('6', '₆'),
    ('7', '₇'),
    ('8', '₈'),
    ('9', '₉'),
    ('+', '₊'),
    ('-', '₋'),
    ('−', '₋'),
    ('=', '₌'),
    ('(', '₍'),
    (')', '₎'),
    ('a', 'ₐ'),
    ('e', 'ₑ'),
    ('h', 'ₕ'),
    ('i', 'ᵢ'),
    ('j', 'ⱼ'),
    ('k', 'ₖ'),
    ('l', 'ₗ'),
    ('m', 'ₘ'),
    ('n', 'ₙ'),
    ('o', 'ₒ'),
    ('p', 'ₚ'),
    ('r', 'ᵣ'),
    ('s', 'ₛ'),
    ('t', 'ₜ'),
    ('u', 'ᵤ'),
    ('v', 'ᵥ'),
    ('x', 'ₓ'),
    (',', ','),
    ('β', 'ᵦ'),
    ('γ', 'ᵧ'),
    ('ρ', 'ᵨ'),
    ('ϕ', 'ᵩ'),
    ('φ', 'ᵩ'),
    ('χ', 'ᵪ'),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn papers_equations() {
        assert_eq!(
            to_unicode(r"\mathrm{Attention}(Q,K,V)=\mathrm{softmax}(\frac{QK^{T}}{\sqrt{d_{k}}})V"),
            "Attention(Q,K,V) = softmax(QKᵀ/√dₖ)V"
        );
        assert_eq!(to_unicode(r"\mathrm{FFN}(x)=\max(0,xW_{1}+b_{1})W_{2}+b_{2}"), "FFN(x) = max(0,xW₁+b₁)W₂+b₂");
        assert_eq!(
            to_unicode(r"\displaystyle PE_{(pos,2i)}=sin(pos/10000^{2i/d_{\text{model}}})"),
            "PE₍ₚₒₛ,₂ᵢ₎ = sin(pos/10000^(2i/d_model))"
        );
        assert_eq!(to_unicode(r"{ε}_{ma}^{*} = \sum_{i=1}^{n} \alpha_i \leq \infty"), "εₘₐ∗ = ∑ᵢ₌₁ⁿ αᵢ ≤ ∞");
        assert_eq!(to_unicode(r"\hat{x} + \bar{y} \in \mathbb{R}^{n}"), "x\u{302} + y\u{304} ∈ ℝⁿ");
        assert_eq!(to_unicode(r"\left( \frac{a+b}{2} \right)"), "((a+b)/2)");
        assert_eq!(to_unicode(r"a &= b \\ c &= d"), "a = b\nc = d");
        assert_eq!(to_unicode(r"\sqrt[3]{x}"), "∛x");
    }
}
