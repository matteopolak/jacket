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

/// A `PUNCT` entry as the first four bytes of the text it matches, so a candidate is one masked
/// compare of a word instead of a call to `memcmp`.
#[derive(Clone, Copy)]
struct Candidate {
    mask: u32,
    bits: u32,
    len: usize,
    p: P,
}

/// The `PUNCT` entries that start with each byte, longest first (`PUNCT`'s order).
fn candidates() -> &'static [Vec<Candidate>; 256] {
    static BY_FIRST: std::sync::OnceLock<[Vec<Candidate>; 256]> = std::sync::OnceLock::new();
    BY_FIRST.get_or_init(|| {
        std::array::from_fn(|first| {
            PUNCT
                .iter()
                .filter(|(text, _)| text.as_bytes()[0] as usize == first)
                .map(|&(text, p)| {
                    let bytes = text.as_bytes();
                    let mut bits = [0u8; 4];
                    bits[..bytes.len()].copy_from_slice(bytes);
                    let mut mask = [0u8; 4];
                    mask[..bytes.len()].fill(0xff);
                    Candidate {
                        mask: u32::from_le_bytes(mask),
                        bits: u32::from_le_bytes(bits),
                        len: bytes.len(),
                        p,
                    }
                })
                .collect()
        })
    })
}

/// The longest punctuation `rest` starts with: its length in bytes and token.
pub(super) fn match_punct(rest: &[u8]) -> Option<(usize, P)> {
    let first = *rest.first()?;
    let mut window = [0u8; 4];
    let n = rest.len().min(4);
    window[..n].copy_from_slice(&rest[..n]);
    let word = u32::from_le_bytes(window);
    candidates()[first as usize]
        .iter()
        .find(|c| word & c.mask == c.bits)
        .map(|c| (c.len, c.p))
}

/// Every punctuation spelling, longest first within each starting character: the table a
/// formatter's lexer must match (`crates/jaic/tests/formatter_lexer.rs` compares them).
pub fn spellings() -> impl Iterator<Item = &'static str> {
    PUNCT.iter().map(|(text, _)| *text)
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
