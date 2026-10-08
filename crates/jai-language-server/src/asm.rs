//! Editor help inside `#asm { ... }` blocks: instruction, operand, register-class and
//! feature-modifier completion, hover on mnemonics and declared registers, and operand
//! signature help.
//!
//! The instruction data is the compiler's own table (`jaic::sema::asm_catalog`), so what is
//! offered is exactly what compiles. A block is found by a tolerant scan of the text rather
//! than the parser: while an instruction is being typed the block usually does not parse,
//! and an unterminated `#asm {` still gets help (its body runs to the brace that closes it,
//! or to the end of the file).
use crate::hover::{Block, HoverText, Inline};
use crate::session::{Session, contains, repair};
use crate::{
    CompletionItem, CompletionKind, CompletionList, DocumentUri, SignatureHelp,
    SignatureInformation, SymbolKind, semantic,
};
use jaic::sema::asm_catalog::{self as catalog, AsmInstruction};
use jaic::sema::ide::{IdeKind, IdeName};
use std::collections::BTreeMap;

fn ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `text` with comments and the contents of strings replaced by spaces, offsets kept.
fn code_only(text: &str) -> Vec<u8> {
    let b = text.as_bytes();
    let mut out = b.to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            // Block comments nest.
            let mut depth = 0usize;
            while i < b.len() {
                let pair = (b[i], b.get(i + 1).copied());
                if pair == (b'/', Some(b'*')) || pair == (b'*', Some(b'/')) {
                    out[i] = b' ';
                    out[i + 1] = b' ';
                    i += 2;
                    if pair.0 == b'/' {
                        depth += 1;
                    } else {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    continue;
                }
                if b[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
        } else if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' && b[i] != b'\n' {
                if b[i] == b'\\' && i + 1 < b.len() && b[i + 1] != b'\n' {
                    out[i] = b' ';
                    i += 1;
                }
                out[i] = b' ';
                i += 1;
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    out
}

fn str_of(code: &[u8], start: usize, end: usize) -> &str {
    std::str::from_utf8(&code[start..end]).unwrap_or("")
}

/// The identifier characters just before `byte`.
fn word_before(code: &[u8], byte: usize) -> usize {
    let mut start = byte;
    while start > 0 && ident(code[start - 1]) {
        start -= 1;
    }
    start
}

fn skip_ws(code: &[u8], mut at: usize, end: usize) -> usize {
    while at < end && code[at].is_ascii_whitespace() {
        at += 1;
    }
    at
}

fn ident_end(code: &[u8], mut at: usize, end: usize) -> usize {
    while at < end && ident(code[at]) {
        at += 1;
    }
    at
}

/// One `#asm` block found by the scan.
struct AsmBlock {
    /// The `#` of `#asm`.
    directive: usize,
    /// The `{`.
    open: usize,
    /// The closing `}`, or the end of the text when the block is unterminated.
    end: usize,
    closed: bool,
    /// Default vector width in bytes from the feature modifiers.
    width: u64,
}

enum Place {
    /// Between `#asm` and `{`: a feature modifier is being typed (its first byte).
    Features(usize),
    Body(AsmBlock),
}

/// Where `byte` is relative to the `#asm` blocks of `code`.
fn locate(code: &[u8], byte: usize) -> Option<Place> {
    let mut from = 0;
    while let Some(found) = code[from..].windows(4).position(|w| w == b"#asm") {
        let at = from + found;
        if at >= byte {
            return None;
        }
        let after = at + 4;
        from = after;
        if code.get(after).is_some_and(|&c| ident(c)) {
            continue;
        }
        // Header: feature names separated by commas, then `{`.
        let mut j = after;
        let mut width = 16;
        loop {
            while j < code.len() && (code[j].is_ascii_whitespace() || code[j] == b',') {
                j += 1;
            }
            let end = ident_end(code, j, code.len());
            if end == j {
                break;
            }
            let name = str_of(code, j, end);
            if name.starts_with("AVX512") {
                width = 64;
            } else if name.starts_with("AVX") && width < 32 {
                width = 32;
            }
            j = end;
        }
        if byte > after && byte <= j {
            return Some(Place::Features(word_before(code, byte)));
        }
        if code.get(j) != Some(&b'{') {
            continue;
        }
        let mut depth = 1;
        let mut k = j + 1;
        while k < code.len() {
            match code[k] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        if byte > j && byte <= k {
            return Some(Place::Body(AsmBlock {
                directive: at,
                open: j,
                end: k.min(code.len()),
                closed: k < code.len(),
                width,
            }));
        }
        from = k.min(code.len());
    }
    None
}

/// A parsed (possibly partial) `#asm` statement; ranges are byte offsets into the text.
#[derive(Debug)]
enum Stmt {
    Empty,
    /// `name: class` or `name:`, optionally pinned.
    Decl {
        name: (usize, usize),
        class: Option<(usize, usize)>,
        /// Start of the text after `===`, if any.
        pin: Option<usize>,
    },
    /// `name === reg`: pins an existing variable.
    Pin,
    Inst {
        mnemonic: (usize, usize),
        suffix: Option<(usize, usize)>,
        /// Start of the operand list (after the whitespace following the mnemonic).
        operands: Option<usize>,
    },
    Other,
}

fn pin_after(code: &[u8], at: usize, end: usize) -> Option<usize> {
    code[at..end]
        .windows(3)
        .position(|w| w == b"===")
        .map(|p| at + p + 3)
}

fn parse_stmt(code: &[u8], start: usize, end: usize) -> Stmt {
    let p = skip_ws(code, start, end);
    if p == end {
        return Stmt::Empty;
    }
    let q = ident_end(code, p, end);
    if q == p {
        return Stmt::Other;
    }
    let r = skip_ws(code, q, end);
    if r < end && code[r] == b':' && code.get(r + 1) != Some(&b':') {
        let c = skip_ws(code, r + 1, end);
        let ce = ident_end(code, c, end);
        return Stmt::Decl {
            name: (p, q),
            class: (ce > c).then_some((c, ce)),
            pin: pin_after(code, r + 1, end),
        };
    }
    if code[r..end].starts_with(b"===") {
        return Stmt::Pin;
    }
    let mut s = q;
    let mut suffix = None;
    if s < end && code[s] == b'.' {
        let e = ident_end(code, s + 1, end);
        suffix = Some((s + 1, e));
        s = e;
    } else if s < end && code[s] == b'?' {
        while s < end && !code[s].is_ascii_whitespace() {
            s += 1;
        }
    }
    let operands = (s < end && code[s].is_ascii_whitespace()).then(|| skip_ws(code, s, end));
    if operands.is_none() && s < end {
        return Stmt::Other;
    }
    Stmt::Inst {
        mnemonic: (p, q),
        suffix,
        operands,
    }
}

/// The operands of `[start, end)`, split at commas outside `[...]` (untrimmed ranges).
fn split_operands(code: &[u8], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut from = start;
    for (i, &c) in code.iter().enumerate().take(end).skip(start) {
        match c {
            b'[' | b'(' => depth += 1,
            b']' | b')' => depth -= 1,
            b',' if depth <= 0 => {
                out.push((from, i));
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push((from, end));
    out
}

/// `name:` (not `::`) at the start of an operand: the name and what follows the colon.
fn inline_decl(code: &[u8], start: usize, end: usize) -> Option<((usize, usize), usize)> {
    let p = skip_ws(code, start, end);
    let q = ident_end(code, p, end);
    if q == p || code[p].is_ascii_digit() {
        return None;
    }
    let r = skip_ws(code, q, end);
    (r < end && code[r] == b':' && code.get(r + 1) != Some(&b':')).then_some(((p, q), r + 1))
}

/// A register declared in a block.
struct Reg {
    name: String,
    class: &'static str,
    span: (usize, usize),
}

fn class_name(name: &str) -> &'static str {
    catalog::register_classes()
        .into_iter()
        .find(|c| c.name == name)
        .map_or("gpr", |c| c.name)
}

/// Statement ranges of a block body.
fn statements(code: &[u8], block: &AsmBlock) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = block.open + 1;
    for (i, &c) in code.iter().enumerate().take(block.end).skip(block.open + 1) {
        if c == b';' {
            out.push((from, i));
            from = i + 1;
        }
    }
    out.push((from, block.end));
    out
}

fn width_of(code: &[u8], suffix: Option<(usize, usize)>, default: u64) -> u64 {
    match suffix.map(|(s, e)| str_of(code, s, e)) {
        Some("x") => 16,
        Some("y") => 32,
        Some("z") => 64,
        _ => default,
    }
}

/// Registers the block declares: `x: class;` statements and inline `x:` operands.
fn registers(code: &[u8], block: &AsmBlock) -> Vec<Reg> {
    let mut out = Vec::new();
    for (start, end) in statements(code, block) {
        match parse_stmt(code, start, end) {
            Stmt::Decl {
                name,
                class,
                ..
            } => out.push(Reg {
                name: str_of(code, name.0, name.1).into(),
                class: class.map_or("gpr", |(s, e)| class_name(str_of(code, s, e))),
                span: name,
            }),
            Stmt::Inst {
                mnemonic,
                suffix,
                operands: Some(ops),
            } => {
                let mnemonic = str_of(code, mnemonic.0, mnemonic.1);
                let width = width_of(code, suffix, block.width);
                for (i, (s, e)) in split_operands(code, ops, end).into_iter().enumerate() {
                    let Some((name, after)) = inline_decl(code, s, e) else {
                        continue;
                    };
                    let c = skip_ws(code, after, e);
                    let ce = ident_end(code, c, e);
                    let class = if ce > c {
                        class_name(str_of(code, c, ce))
                    } else {
                        catalog::inline_register_class(mnemonic, width, i).unwrap_or("gpr")
                    };
                    out.push(Reg {
                        name: str_of(code, name.0, name.1).into(),
                        class,
                        span: name,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

/// What the cursor is completing inside a block.
enum Slot {
    /// The start of a statement: a mnemonic or a declaration.
    Mnemonic,
    /// After `mnemonic.`.
    Suffix,
    /// After `name:`.
    Class,
    /// After `===`.
    Pin,
    Operand {
        mnemonic: String,
        index: usize,
    },
}

fn slot(code: &[u8], start: usize, byte: usize) -> Option<Slot> {
    Some(match parse_stmt(code, start, byte) {
        Stmt::Empty => Slot::Mnemonic,
        Stmt::Decl {
            pin: Some(_), ..
        }
        | Stmt::Pin => Slot::Pin,
        Stmt::Decl {
            class, ..
        } => {
            // Only while the class itself is typed: nothing may follow it.
            let at = class.map_or(byte, |(_, e)| e);
            if at != byte {
                return None;
            }
            Slot::Class
        }
        Stmt::Inst {
            mnemonic,
            suffix,
            operands,
        } => match operands {
            None if suffix.is_some_and(|(_, e)| e == byte) => Slot::Suffix,
            None if mnemonic.1 == byte => Slot::Mnemonic,
            None => return None,
            Some(ops) => {
                let parts = split_operands(code, ops, byte);
                let (s, e) = *parts.last()?;
                if pin_after(code, s, e).is_some() {
                    Slot::Pin
                } else if let Some((_, after)) = inline_decl(code, s, e) {
                    // `tmp: cl|` names the class of an inline declaration.
                    let c = skip_ws(code, after, e);
                    if ident_end(code, c, e) != e {
                        return None;
                    }
                    Slot::Class
                } else {
                    Slot::Operand {
                        mnemonic: str_of(code, mnemonic.0, mnemonic.1).into(),
                        index: parts.len() - 1,
                    }
                }
            }
        },
        Stmt::Other => return None,
    })
}

/// The start of the statement holding `byte` in `block`.
fn statement_start(code: &[u8], block: &AsmBlock, byte: usize) -> usize {
    code[block.open + 1..byte]
        .iter()
        .rposition(|&c| c == b';')
        .map_or(block.open + 1, |p| block.open + 1 + p + 1)
}

fn statement_end(code: &[u8], block: &AsmBlock, byte: usize) -> usize {
    code[byte..block.end]
        .iter()
        .position(|&c| c == b';')
        .map_or(block.end, |p| byte + p)
}

/// The Markdown documentation of an instruction.
fn instruction_doc(i: &AsmInstruction) -> String {
    format!(
        "{}\n\n```jai\n{}\n```\n\nRequires `{}`.",
        i.description,
        i.forms.join("\n"),
        i.feature
    )
}

fn instruction_item(i: &AsmInstruction) -> CompletionItem {
    CompletionItem {
        label: i.mnemonic.clone(),
        kind: CompletionKind::Instruction,
        detail: format!("{} ({})", i.forms[0], i.feature),
        documentation: Some(instruction_doc(i)),
        insert_text: None,
        ..CompletionItem::default()
    }
}

fn simple(label: &str, kind: CompletionKind, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.into(),
        kind,
        detail: detail.into(),
        ..CompletionItem::default()
    }
}

/// Operand size and vector width suffixes after `mnemonic.`.
const SUFFIXES: &[(&str, &str)] = &[
    ("b", "8-bit operation"),
    ("w", "16-bit operation"),
    ("d", "32-bit operation"),
    ("q", "64-bit operation (or the 8-byte MMX form)"),
    ("x", "128-bit vector (xmm)"),
    ("y", "256-bit vector (ymm)"),
    ("z", "512-bit vector (zmm)"),
];

impl Session {
    /// Completion inside an `#asm` block or its feature list; `None` elsewhere.
    pub(crate) fn asm_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<CompletionList> {
        let code = code_only(text);
        let block = match locate(&code, byte)? {
            Place::Features(start) => {
                let typed = str_of(&code, start, byte).to_ascii_uppercase();
                return Some(CompletionList {
                    is_incomplete: false,
                    items: catalog::feature_modifiers()
                        .iter()
                        .filter(|f| f.starts_with(&typed))
                        .map(|f| simple(f, CompletionKind::Constant, "#asm feature modifier"))
                        .collect(),
                });
            }
            Place::Body(block) => block,
        };
        let start = statement_start(&code, &block, byte);
        let word = word_before(&code, byte);
        let prefix = str_of(&code, word, byte);
        let lower = prefix.to_ascii_lowercase();
        let mut incomplete = false;
        let items: Vec<CompletionItem> = match slot(&code, start, byte)? {
            Slot::Mnemonic => {
                let mut items: Vec<CompletionItem> = Vec::new();
                for i in catalog::instructions() {
                    if !i.mnemonic.starts_with(&lower) {
                        continue;
                    }
                    if items.len() >= self.limits.symbols {
                        incomplete = true;
                        break;
                    }
                    items.push(instruction_item(i));
                }
                if prefix.is_empty() {
                    for class in catalog::register_classes() {
                        items.push(CompletionItem {
                            label: format!("name: {}", class.name),
                            kind: CompletionKind::Snippet,
                            detail: format!("declare a {} register", class.name),
                            documentation: Some(class.description.into()),
                            insert_text: Some(format!("${{1:name}}: {};", class.name)),
                            ..CompletionItem::default()
                        });
                    }
                }
                items
            }
            Slot::Suffix => SUFFIXES
                .iter()
                .filter(|(s, _)| s.starts_with(&lower))
                .map(|(s, d)| simple(s, CompletionKind::Keyword, d))
                .collect(),
            Slot::Class => catalog::register_classes()
                .into_iter()
                .filter(|c| c.name.starts_with(&lower))
                .map(|c| simple(c.name, CompletionKind::TypeAlias, c.description))
                .collect(),
            Slot::Pin => catalog::pin_names()
                .into_iter()
                .filter(|n| n.starts_with(&lower))
                .map(|n| {
                    simple(
                        &n,
                        CompletionKind::Constant,
                        "register pin (accepted and ignored)",
                    )
                })
                .collect(),
            Slot::Operand {
                ..
            } => {
                if prefix.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
                    return Some(CompletionList::default());
                }
                let mut items = BTreeMap::new();
                for reg in registers(&code, &block) {
                    if reg.span.1 <= word && reg.name.starts_with(prefix) {
                        items.entry(reg.name.clone()).or_insert(CompletionItem {
                            label: reg.name,
                            kind: CompletionKind::Variable,
                            detail: format!("{} register (#asm)", reg.class),
                            ..CompletionItem::default()
                        });
                    }
                }
                for name in self.asm_scope_names(uri, text, &code, &block) {
                    if !name.name.starts_with(prefix) || items.len() >= self.limits.symbols {
                        continue;
                    }
                    let kind = if name.kind == IdeKind::Constant {
                        CompletionKind::Constant
                    } else {
                        CompletionKind::Variable
                    };
                    items.entry(name.name.clone()).or_insert(CompletionItem {
                        label: name.name,
                        kind,
                        detail: name.detail,
                        ..CompletionItem::default()
                    });
                }
                items.into_values().collect()
            }
        };
        Some(CompletionList {
            is_incomplete: incomplete,
            items,
        })
    }

    /// Jai variables and constants visible where the block starts: from the type checker with
    /// the block blanked out (so an unfinished block does not stop it), else from the syntax.
    fn asm_scope_names(
        &self,
        uri: &DocumentUri,
        text: &str,
        code: &[u8],
        block: &AsmBlock,
    ) -> Vec<IdeName> {
        let at = block.directive;
        let blank = |from: usize, to: usize| -> String {
            let mut bytes = text.as_bytes().to_vec();
            for b in &mut bytes[from..to] {
                if *b != b'\n' {
                    *b = b' ';
                }
            }
            String::from_utf8(bytes).unwrap_or_default()
        };
        if self.environment.is_some() {
            // Without the block; braces it left open (an unterminated block took the
            // procedure's `}`) are closed at the end, which keeps every offset.
            let mut probe = blank(at, (block.end + usize::from(block.closed)).min(code.len()));
            let open = code_only(&probe).iter().fold(0i64, |depth, &c| match c {
                b'{' => depth + 1,
                b'}' => depth - 1,
                _ => depth,
            });
            if open > 0 {
                probe.push('\n');
                probe.push_str(&"}".repeat(open as usize));
            }
            let probe = if semantic::parse_error(&probe).is_none() {
                Some(probe)
            } else {
                repair(&probe, None)
            };
            if let Some(probe) = probe
                && let Some(names) =
                    self.with_semantic(uri, &probe, |a, path| a.complete(path, at, &[]))
            {
                return names
                    .into_iter()
                    .filter(|n| matches!(n.kind, IdeKind::Variable | IdeKind::Constant))
                    .collect();
            }
        }
        let analysis = match self.analyses.get(uri) {
            Some(a) if a.complete => a,
            _ => match self.parsed.get(uri).or_else(|| self.analyses.get(uri)) {
                Some(a) => a,
                None => return Vec::new(),
            },
        };
        analysis
            .rows
            .iter()
            .filter(|row| matches!(row.kind, SymbolKind::Variable | SymbolKind::Constant))
            .filter(|row| {
                if row.local {
                    contains(row.scope, at) && row.selection.start <= at
                } else {
                    row.parent.is_none()
                }
            })
            .map(|row| IdeName {
                name: row.name.as_str().into(),
                kind: if row.kind == SymbolKind::Constant {
                    IdeKind::Constant
                } else {
                    IdeKind::Variable
                },
                detail: self.source_detail(uri, row).into(),
            })
            .collect()
    }

    /// Hover on a mnemonic or a declared register inside an `#asm` block.
    pub(crate) fn asm_hover(&self, text: &str, byte: usize) -> Option<(usize, usize, HoverText)> {
        let code = code_only(text);
        let Place::Body(block) = locate(&code, byte)? else {
            return None;
        };
        let start = statement_start(&code, &block, byte);
        let end = statement_end(&code, &block, byte);
        if let Stmt::Inst {
            mnemonic: (ms, me),
            ..
        } = parse_stmt(&code, start, end)
            && ms <= byte
            && byte <= me
        {
            let i = catalog::instruction(str_of(&code, ms, me))?;
            let text = HoverText::new(vec![
                Block::Code(i.forms.join("\n")),
                Block::Para(vec![Inline::Text(i.description.clone())]),
                Block::Para(vec![
                    Inline::Text("Requires ".into()),
                    Inline::Code(i.feature.into()),
                ]),
            ]);
            return Some((ms, me, text));
        }
        let ws = word_before(&code, byte);
        let we = ident_end(&code, byte, code.len());
        let word = str_of(&code, ws, we);
        let reg = registers(&code, &block)
            .into_iter()
            .find(|r| r.name == word && !word.is_empty())?;
        let description = catalog::register_classes()
            .into_iter()
            .find(|c| c.name == reg.class)
            .map_or("", |c| c.description);
        Some((
            ws,
            we,
            HoverText::new(vec![
                Block::Code(format!("{}: {}", reg.name, reg.class)),
                Block::Para(vec![Inline::Text(format!("#asm {description}"))]),
            ]),
        ))
    }

    /// The operand forms of the instruction whose operands are being typed.
    pub(crate) fn asm_signature_help(&self, text: &str, byte: usize) -> Option<SignatureHelp> {
        let code = code_only(text);
        let Place::Body(block) = locate(&code, byte)? else {
            return None;
        };
        let start = statement_start(&code, &block, byte);
        let Slot::Operand {
            mnemonic,
            index,
        } = slot(&code, start, byte)?
        else {
            return None;
        };
        let i = catalog::instruction(&mnemonic)?;
        let signatures: Vec<SignatureInformation> = i
            .forms
            .iter()
            .map(|form| SignatureInformation {
                label: form.clone(),
                parameters: form
                    .split_once(' ')
                    .map(|(_, ops)| ops.split(", ").map(String::from).collect())
                    .unwrap_or_default(),
            })
            .collect();
        let active = signatures
            .iter()
            .position(|s| s.parameters.len() > index)
            .unwrap_or(0);
        Some(SignatureHelp {
            signatures,
            active_signature: active,
            active_parameter: index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(text: &str, marker: &str) -> Option<Place> {
        let byte = text.find(marker).unwrap() + marker.len();
        locate(&code_only(text), byte)
    }

    #[test]
    fn blocks_are_found_without_parsing() {
        let text = "f :: () {\n    #asm AVX2 {\n        vpad|\n}\n";
        assert!(matches!(place(text, "vpad"), Some(Place::Body(b)) if b.width == 32));
        assert!(matches!(place(text, "#asm AV"), Some(Place::Features(_))));
        assert!(place(text, "f ::").is_none());
        let commented = "// #asm { add\nx := 1;";
        assert!(place(commented, "x :=").is_none());
        let after = "#asm { add x, 1; }\ny := 2;";
        assert!(place(after, "y :=").is_none());
    }

    #[test]
    fn slots_follow_the_statement_shape() {
        let code = code_only("#asm { mov.q x, ab; t: gp");
        let at = |m: &str| {
            let s = std::str::from_utf8(&code).unwrap();
            s.find(m).unwrap() + m.len()
        };
        let Place::Body(block) = locate(&code, at("ab")).unwrap() else {
            panic!()
        };
        let start = statement_start(&code, &block, at("ab"));
        assert!(matches!(
            slot(&code, start, at("ab")),
            Some(Slot::Operand {
                index: 1,
                ..
            })
        ));
        assert!(matches!(
            slot(&code, start, at("mov")),
            Some(Slot::Mnemonic)
        ));
        assert!(matches!(
            slot(&code, start, at("mov.q")),
            Some(Slot::Suffix)
        ));
        let start = statement_start(&code, &block, at("gp"));
        assert!(matches!(slot(&code, start, at("gp")), Some(Slot::Class)));
    }
}
