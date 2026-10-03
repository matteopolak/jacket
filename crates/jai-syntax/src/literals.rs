//! Source literal decoding; Jai strings contain arbitrary bytes.
use super::*;
use jai_types::FloatType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DecimalLiteral(String);
/// A here string keeps its source-level modifiers distinct from quoted strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HereStringLiteral {
    pub bytes: Vec<u8>,
    pub modifiers: Vec<HereStringModifier>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HereStringModifier {
    CarriageReturn,
    FormattingEscape,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatRangeError(pub FloatType);
impl std::fmt::Display for FloatRangeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "decimal literal is outside the finite {:?} range",
            self.0
        )
    }
}

impl std::error::Error for FloatRangeError {
}
impl DecimalLiteral {
    pub fn retained_spelling_capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn spelling(&self) -> &str {
        &self.0
    }
    pub fn round_f32(&self) -> Result<f32, FloatRangeError> {
        self.0
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or(FloatRangeError(FloatType::F32))
    }
    pub fn round_f64(&self) -> Result<f64, FloatRangeError> {
        self.0
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or(FloatRangeError(FloatType::F64))
    }
    fn parse(text: &str, span: Span) -> Result<Self, Diagnostic> {
        let canonical: String = text.chars().filter(|&character| character != '_').collect();
        let bytes = canonical.as_bytes();
        let mut at = 0;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        let mut digits = at;
        if bytes.get(at) == Some(&b'.') {
            at += 1;
            let start = at;
            while bytes.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
            digits += at - start;
        }
        if digits == 0 {
            return Err(Diagnostic::new(span, "decimal literal requires digits"));
        }
        if matches!(bytes.get(at), Some(b'e' | b'E')) {
            at += 1;
            if matches!(bytes.get(at), Some(b'+' | b'-')) {
                at += 1;
            }
            let start = at;
            while bytes.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
            if at == start {
                return Err(Diagnostic::new(span, "decimal exponent requires digits"));
            }
        }
        if at != bytes.len() {
            return Err(Diagnostic::new(
                span,
                "invalid decimal floating-point literal",
            ));
        }
        Ok(Self(canonical))
    }
}

pub(super) fn string(raw: &str, span: Span) -> Result<Vec<u8>, Diagnostic> {
    let contents = raw
        .strip_prefix('"')
        .and_then(|contents| contents.strip_suffix('"'))
        .ok_or_else(|| Diagnostic::new(span, "expected a quoted string literal"))?;
    let mut bytes = Vec::new();
    let mut chars = contents.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            let mut buffer = [0; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        let escape = chars
            .next()
            .ok_or_else(|| Diagnostic::new(span, "incomplete string escape"))?;
        let byte = match escape {
            'e' => 27,
            'n' => 10,
            'r' => 13,
            't' => 9,
            '0' => 0,
            '%' => 0x1f,
            '\\' => b'\\',
            '"' => b'"',
            'x' => {
                let value = hex_digits(&mut chars, 2, span)?;
                bytes.push(value as u8);
                continue;
            }
            'd' => {
                let mut value = 0u16;
                for _ in 0..3 {
                    let digit = chars
                        .next()
                        .and_then(|character| character.to_digit(10))
                        .ok_or_else(|| {
                            Diagnostic::new(span, "decimal byte escape requires three digits")
                        })?;
                    value = value * 10 + digit as u16;
                }
                let byte = u8::try_from(value)
                    .map_err(|_| Diagnostic::new(span, "decimal byte escape exceeds 255"))?;
                bytes.push(byte);
                continue;
            }
            'u' | 'U' => {
                let value = hex_digits(
                    &mut chars,
                    if escape == 'u' {
                        4
                    } else {
                        8
                    },
                    span,
                )?;
                let character = char::from_u32(value)
                    .ok_or_else(|| Diagnostic::new(span, "invalid Unicode string escape"))?;
                let mut buffer = [0; 4];
                bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                continue;
            }
            _ => return Err(Diagnostic::new(span, "unsupported string escape")),
        };
        bytes.push(byte);
    }
    Ok(bytes)
}

/// `#char` denotes one decoded byte, including bytes that are not valid UTF-8.
pub(super) fn character(raw: &str, span: Span) -> Result<u8, Diagnostic> {
    let bytes = string(raw, span)?;
    match bytes.as_slice() {
        [byte] => Ok(*byte),
        _ => Err(Diagnostic::new(
            span,
            "#char requires exactly one decoded byte",
        )),
    }
}

pub(super) fn here_string(raw: &str, span: Span) -> Result<HereStringLiteral, Diagnostic> {
    let raw = raw
        .strip_prefix("#string")
        .ok_or_else(|| Diagnostic::new(span, "expected a #string literal"))?;
    let (header, contents) = raw
        .split_once('\n')
        .ok_or_else(|| Diagnostic::new(span, "unterminated here-string header"))?;
    let mut header = header.trim_start_matches([' ', '\t']);
    let mut modifiers = Vec::new();
    while let Some(remainder) = header.strip_prefix(',') {
        header = remainder.trim_start_matches([' ', '\t']);
        let end = header
            .find(|character: char| character == ',' || character.is_whitespace())
            .unwrap_or(header.len());
        let spelling = &header[..end];
        let modifier = match spelling {
            "cr" => HereStringModifier::CarriageReturn,
            "\\%" => HereStringModifier::FormattingEscape,
            "" => return Err(Diagnostic::new(span, "expected here-string modifier")),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    format!("unsupported here-string modifier `{spelling}`"),
                ));
            }
        };
        if modifiers.contains(&modifier) {
            return Err(Diagnostic::new(span, "duplicate here-string modifier"));
        }
        modifiers.push(modifier);
        header = header[end..].trim_start_matches([' ', '\t']);
    }
    let tag_end = header
        .find(|character: char| !(character.is_alphanumeric() || character == '_'))
        .unwrap_or(header.len());
    let tag = &header[..tag_end];
    if tag.is_empty()
        || !tag
            .chars()
            .next()
            .is_some_and(|character| character.is_alphabetic() || character == '_')
    {
        return Err(Diagnostic::new(span, "expected here-string terminator"));
    }
    // Match the lexer: leading spaces/tabs on the terminator line are not contents.
    let mut body_end = None;
    let mut offset = 0;
    for line in contents.split_inclusive('\n') {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if let Some(tail) = trimmed.strip_prefix(tag)
            && tail
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric() && character != '_')
        {
            if !tail.is_empty() {
                return Err(Diagnostic::new(
                    span,
                    "unexpected text after here-string terminator",
                ));
            }
            body_end = Some(offset);
            break;
        }
        offset += line.len();
    }
    let body =
        &contents[..body_end.ok_or_else(|| Diagnostic::new(span, "unterminated here-string"))?];
    let carriage_return = modifiers.contains(&HereStringModifier::CarriageReturn);
    let formatting_escape = modifiers.contains(&HereStringModifier::FormattingEscape);
    let mut bytes = Vec::with_capacity(body.len());
    let mut source = body.bytes().peekable();
    while let Some(byte) = source.next() {
        if byte == b'\r' && source.peek() == Some(&b'\n') {
            // Consume CRLF together, then emit the configured line ending.
            source.next();
            if carriage_return {
                bytes.push(b'\r');
            }
            bytes.push(b'\n');
        } else if byte == b'\n' {
            if carriage_return {
                bytes.push(b'\r');
            }
            bytes.push(b'\n');
        } else if formatting_escape && byte == b'\\' && source.peek() == Some(&b'%') {
            source.next();
            bytes.push(0x1f);
        } else {
            bytes.push(byte);
        }
    }
    Ok(HereStringLiteral {
        bytes,
        modifiers,
    })
}
fn hex_digits(
    chars: &mut impl Iterator<Item = char>,
    count: usize,
    span: Span,
) -> Result<u32, Diagnostic> {
    let mut value = 0;
    for _ in 0..count {
        let digit = chars
            .next()
            .and_then(|character| character.to_digit(16))
            .ok_or_else(|| Diagnostic::new(span, "string escape requires hexadecimal digits"))?;
        value = value * 16 + digit;
    }
    Ok(value)
}

pub(super) fn number(text: &str, span: Span) -> Result<ExpressionKind, Diagnostic> {
    if let Some(bits) = text.strip_prefix("0h") {
        let cleaned: String = bits.chars().filter(|&character| character != '_').collect();
        return match cleaned.len() {
            8 => u32::from_str_radix(&cleaned, 16)
                .map(|value| ExpressionKind::Float(FloatLiteral::Bits32(value))),
            16 => u64::from_str_radix(&cleaned, 16)
                .map(|value| ExpressionKind::Float(FloatLiteral::Bits64(value))),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "bit float literals require 8 or 16 hexadecimal digits",
                ));
            }
        }
        .map_err(|_| Diagnostic::new(span, "invalid bit float literal"));
    }
    let hexadecimal = text.starts_with("0x");
    let float = if hexadecimal {
        text.contains(['.', 'p', 'P'])
    } else {
        !text.starts_with("0b") && text.contains(['.', 'e', 'E'])
    };
    if !float {
        return numeric_literals::integer(text)
            .map(|value| ExpressionKind::Integer(i128::from(value)))
            .map_err(|error| Diagnostic::new(span, error.to_string()));
    }
    if hexadecimal {
        return Err(Diagnostic::new(
            span,
            "hexadecimal fractional floats are not implemented",
        ));
    }
    Ok(ExpressionKind::Float(FloatLiteral::Decimal(
        DecimalLiteral::parse(text, span)?,
    )))
}

impl Parser<'_> {
    pub(super) fn initializer(&mut self) -> Result<Expression, Diagnostic> {
        if self.is(Punct::Uninitialized) {
            let span = self.token().span;
            self.at += 1;
            return Ok(Expression {
                kind: ExpressionKind::Uninitialized,
                span,
            });
        }
        self.expression(0)
    }
    pub(super) fn array_literal(
        &mut self,
        element_type: Option<TypeSyntax>,
        start: usize,
    ) -> Result<Expression, Diagnostic> {
        self.need(Punct::ArrayLiteral)?;
        let mut elements = Vec::new();
        if !self.take(Punct::CloseBracket) {
            loop {
                elements.push(self.expression(0)?);
                if self.take(Punct::CloseBracket) {
                    break;
                }
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseBracket) {
                    break;
                }
            }
        }
        Ok(Expression {
            kind: ExpressionKind::ArrayLiteral(ArrayLiteral {
                element_type,
                elements,
            }),
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_rounding_uses_the_context_directly() {
        let decimal = DecimalLiteral::parse("1.0000000596046448", Span::default()).unwrap();
        assert_eq!(decimal.round_f32().unwrap().to_bits(), 1.0f32.to_bits() + 1);
        assert_eq!(
            (decimal.round_f64().unwrap() as f32).to_bits(),
            1.0f32.to_bits()
        );
        assert_eq!(decimal.spelling(), "1.0000000596046448");
        let huge = DecimalLiteral::parse("1e1000", Span::default()).unwrap();
        assert_eq!(huge.round_f32(), Err(FloatRangeError(FloatType::F32)));
        assert_eq!(huge.round_f64(), Err(FloatRangeError(FloatType::F64)));
        assert_eq!(
            DecimalLiteral::parse("1_2.5e-1", Span::default())
                .unwrap()
                .round_f64(),
            Ok(1.25)
        );
    }
    #[test]
    fn documented_string_escapes_preserve_arbitrary_bytes() {
        let value = string(r#""A\xFF\d255\0\u00E9\U0001F600\n\e""#, Span::default()).unwrap();
        assert_eq!(&value[..4], &[b'A', 255, 255, 0]);
        assert_eq!(&value[4..], "é😀\n\u{1b}".as_bytes());
        for raw in [
            r#""\q""#,
            r#""\xG0""#,
            r#""\d256""#,
            r#""\d12""#,
            r#""\uD800""#,
            r#""\U00110000""#,
        ] {
            assert!(string(raw, Span::default()).is_err(), "{raw}");
        }
        assert_eq!(string(r#""\%\x1f""#, Span::default()).unwrap(), [31, 31]);
        for raw in ["", "\"", "unquoted"] {
            assert!(string(raw, Span::default()).is_err(), "{raw}");
        }
    }
    #[test]
    fn character_literals_require_one_decoded_byte() {
        for (raw, expected) in [
            (r#""A""#, b'A'),
            (r#""\"""#, b'"'),
            (r#""\\""#, b'\\'),
            (r#""\xFF""#, 255),
            (r#""\d255""#, 255),
            (r#""\u0041""#, b'A'),
            (r#""\0""#, 0),
        ] {
            assert_eq!(character(raw, Span::default()).unwrap(), expected);
        }
        for raw in [
            r#""""#,
            r#""AB""#,
            r#""é""#,
            r#""\u00E9""#,
            r#""\U0001F600""#,
        ] {
            assert!(character(raw, Span::default()).is_err(), "{raw}");
        }
    }
    #[test]
    fn here_strings_preserve_contents_and_normalize_line_endings() {
        let raw = "#string DONE\r\n  \"\\n\" /* unclosed\r\nDONE_more\n \tDONE";
        let decoded = here_string(raw, Span::default()).unwrap();
        assert_eq!(decoded.bytes, b"  \"\\n\" /* unclosed\nDONE_more\n");
        assert!(decoded.modifiers.is_empty());
        let decoded = here_string("#string,cr END\nfirst\r\nsecond\nEND", Span::default()).unwrap();
        assert_eq!(decoded.bytes, b"first\r\nsecond\r\n");
        assert_eq!(decoded.modifiers, [HereStringModifier::CarriageReturn]);
        assert!(
            here_string("#string END\nEND", Span::default())
                .unwrap()
                .bytes
                .is_empty()
        );
    }
    #[test]
    fn formatting_escape_modifier_changes_only_the_selected_escape() {
        let decoded = here_string(
            "#string,\\%, cr JAI\n  \\% \\n \\x1f %\r\nJAI",
            Span::default(),
        )
        .unwrap();
        assert_eq!(decoded.bytes, b"  \x1f \\n \\x1f %\r\n");
        assert_eq!(
            decoded.modifiers,
            [
                HereStringModifier::FormattingEscape,
                HereStringModifier::CarriageReturn
            ]
        );
        assert_eq!(
            here_string("#string JAI\n\\%\nJAI", Span::default())
                .unwrap()
                .bytes,
            b"\\%\n"
        );
    }
    #[test]
    fn malformed_and_unknown_here_string_forms_are_diagnostic() {
        for raw in [
            "#string END",
            "#string, END\nEND",
            "#string,indent END\nbody\nEND",
            "#string,\\n END\nbody\nEND",
            "#string,cr,cr END\nbody\nEND",
            "#string END\nbody\nENDING",
            "#string END\nbody\nEND trailing",
            "#string 123\nbody\n123",
        ] {
            let span = Span::new(10, 10 + raw.len());
            let error = here_string(raw, span).unwrap_err();
            assert_eq!(error.span, span, "{raw}");
        }
        let error = here_string("#string,indent END\nEND", Span::default()).unwrap_err();
        assert!(error.message.contains("unsupported here-string modifier"));
    }
    #[test]
    fn supplied_and_recent_source_here_string_tokens_decode() {
        for relative in [
            "../../../corpus/upstream/focus-editor--focus/src/config_parser.jai",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!(
                        "skipping optional source fixture absent at {}",
                        path.display()
                    );
                    continue;
                }
                Err(error) => panic!("could not read {}: {error}", path.display()),
            };
            let tokens = jai_lexer::lex(&source).unwrap();
            let mut count = 0;
            for token in tokens.iter().filter(|token| token.kind == Kind::HereString) {
                here_string(token.span.text(&source), token.span).unwrap();
                count += 1;
            }
            assert!(count > 0);
        }
    }
    #[test]
    fn decimal_syntax_and_bit_floats_are_validated_without_eager_rounding() {
        assert!(matches!(
            number("1e1000", Span::default()).unwrap(),
            ExpressionKind::Float(FloatLiteral::Decimal(_))
        ));
        assert!(matches!(
            number("0h7fbf_ffff", Span::default()).unwrap(),
            ExpressionKind::Float(FloatLiteral::Bits32(0x7fbfffff))
        ));
        assert!(matches!(
            number("0h8000_0000_0000_0000", Span::default()).unwrap(),
            ExpressionKind::Float(FloatLiteral::Bits64(0x8000000000000000))
        ));
        for text in ["1e", "1e+", "1.2.3", "0h123", "0hxyzxyzzz", "0x1.fp+3"] {
            assert!(number(text, Span::default()).is_err(), "{text}");
        }
    }
}
