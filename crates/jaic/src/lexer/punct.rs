//! Punctuation tokens and their spellings.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum P {
    // Multi-character punctuation, longest first in `PUNCT`.
    RotlAssign,   // <<<=
    RotrAssign,   // >>>=
    Uninit,       // ---
    AndAndAssign, // &&=
    OrOrAssign,   // ||=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
    Rotl,         // <<<
    Rotr,         // >>>
    ColonColon,   // ::
    ColonEq,      // :=
    Arrow,        // ->
    FatArrow,     // =>
    DotDot,       // ..
    DotBrace,     // .{
    DotBracket,   // .[
    DotStar,      // .*
    Shl,          // <<
    Shr,          // >>
    Le,           // <=
    Ge,           // >=
    EqEq,         // ==
    Ne,           // !=
    AndAnd,       // &&
    OrOr,         // ||
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    DollarDollar, // $$
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Semi,
    Comma,
    Colon,
    Dot,
    Dollar,
    Backtick,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Bang,
    Lt,
    Gt,
    Pipe,
    Amp,
    Caret,
    Tilde,
    Question,
}

pub(super) const INTEGER_TOO_BIG: &str =
    "integer literal is too big: integer literals must fit in 64 bits";

const PUNCT: &[(&str, P)] = &[
    ("<<<=", P::RotlAssign),
    (">>>=", P::RotrAssign),
    ("---", P::Uninit),
    ("&&=", P::AndAndAssign),
    ("||=", P::OrOrAssign),
    ("<<=", P::ShlAssign),
    (">>=", P::ShrAssign),
    ("<<<", P::Rotl),
    (">>>", P::Rotr),
    ("::", P::ColonColon),
    (":=", P::ColonEq),
    ("->", P::Arrow),
    ("=>", P::FatArrow),
    ("..", P::DotDot),
    (".{", P::DotBrace),
    (".[", P::DotBracket),
    (".*", P::DotStar),
    ("<<", P::Shl),
    (">>", P::Shr),
    ("<=", P::Le),
    (">=", P::Ge),
    ("==", P::EqEq),
    ("!=", P::Ne),
    ("&&", P::AndAnd),
    ("||", P::OrOr),
    ("+=", P::AddAssign),
    ("-=", P::SubAssign),
    ("*=", P::MulAssign),
    ("/=", P::DivAssign),
    ("%=", P::RemAssign),
    ("&=", P::AndAssign),
    ("|=", P::OrAssign),
    ("^=", P::XorAssign),
    ("$$", P::DollarDollar),
    ("{", P::LBrace),
    ("}", P::RBrace),
    ("[", P::LBracket),
    ("]", P::RBracket),
    ("(", P::LParen),
    (")", P::RParen),
    (";", P::Semi),
    (",", P::Comma),
    (":", P::Colon),
    (".", P::Dot),
    ("$", P::Dollar),
    ("`", P::Backtick),
    ("+", P::Plus),
    ("-", P::Minus),
    ("*", P::Star),
    ("/", P::Slash),
    ("%", P::Percent),
    ("=", P::Eq),
    ("!", P::Bang),
    ("<", P::Lt),
    (">", P::Gt),
    ("|", P::Pipe),
    ("&", P::Amp),
    ("^", P::Caret),
    ("~", P::Tilde),
    ("?", P::Question),
];

/// The `PUNCT` entries that start with byte `c`, longest first (`PUNCT`'s order).
pub(super) fn puncts_starting_with(c: u8) -> &'static [(&'static str, P)] {
    static BY_FIRST: std::sync::OnceLock<[Vec<(&'static str, P)>; 256]> =
        std::sync::OnceLock::new();
    &BY_FIRST.get_or_init(|| {
        std::array::from_fn(|b| {
            PUNCT
                .iter()
                .copied()
                .filter(|(t, _)| t.as_bytes()[0] as usize == b)
                .collect()
        })
    })[c as usize]
}

impl P {
    pub fn text(self) -> &'static str {
        PUNCT
            .iter()
            .find(|(_, p)| *p == self)
            .map(|(s, _)| *s)
            .unwrap()
    }
}
