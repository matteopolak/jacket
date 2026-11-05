//! The written declaration of a struct, union or enum, and the private types a public item
//! leads a reader to.
use super::super::refs::TypeRef;
use super::{EVERY_OS, Item, Module, Reader, TopIndex, class_kind, os_names, struct_params};
use crate::ast::{Decl, EnumItem, Expr, ExprKind, Stmt, StmtKind};
use crate::docs;
use crate::lexer::lex_with_trivia;
use crate::parser::ide_facts::Visibility;
use crate::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the names a declaration introduces are written: its own, its parameters', its fields',
/// the members of enums and of the types nested in it. They are not references to anything.
fn declared_names(decl: &Decl) -> Vec<(usize, usize)> {
    use crate::ast::walk::{self, Node};
    fn name(name: &crate::ast::Ident, out: &mut Vec<(usize, usize)>) {
        out.push((name.span.start as usize, name.span.end as usize));
    }
    fn statements(stmts: &[Stmt], out: &mut Vec<(usize, usize)>) {
        for stmt in stmts {
            visit(Node::Stmt(stmt), out);
        }
    }
    fn members(items: &[EnumItem], out: &mut Vec<(usize, usize)>) {
        for item in items {
            match item {
                EnumItem::Member(member) => {
                    name(&member.name, out);
                    if let Some(value) = &member.value {
                        visit(Node::Expr(value), out);
                    }
                }
                EnumItem::If {
                    then_items,
                    else_items,
                    ..
                } => {
                    members(then_items, out);
                    members(else_items, out);
                }
                _ => {}
            }
        }
    }
    fn declaration(decl: &Decl, out: &mut Vec<(usize, usize)>) {
        for declared in &decl.names {
            name(declared, out);
        }
        if let Some(ty) = &decl.ty {
            visit(Node::Expr(ty), out);
        }
        if let Some(value) = &decl.value {
            visit(Node::Expr(value), out);
        }
        for value in decl.extra_values() {
            visit(Node::Expr(value), out);
        }
    }
    fn visit(node: Node<'_>, out: &mut Vec<(usize, usize)>) {
        match node {
            Node::Stmt(Stmt {
                kind: StmtKind::Decl(decl),
                ..
            }) => {
                declaration(decl, out);
                return;
            }
            Node::Expr(Expr {
                kind: ExprKind::Struct(lit),
                ..
            }) => {
                for param in &lit.params {
                    if let Some(declared) = &param.name {
                        name(declared, out);
                    }
                    if let Some(ty) = &param.ty {
                        visit(Node::Expr(ty), out);
                    }
                    if let Some(value) = &param.default {
                        visit(Node::Expr(value), out);
                    }
                }
                statements(&lit.body, out);
                if let Some(block) = &lit.modify {
                    statements(&block.stmts, out);
                }
            }
            Node::Expr(Expr {
                kind: ExprKind::Enum(lit),
                ..
            }) => members(&lit.items, out),
            Node::Expr(Expr {
                kind: ExprKind::Proc(lit),
                ..
            }) => {
                for param in &lit.header.params {
                    if let Some(declared) = &param.name {
                        name(declared, out);
                    }
                }
                for result in &lit.header.returns {
                    if let Some(declared) = &result.name {
                        name(declared, out);
                    }
                }
                if let Some(block) = &lit.body {
                    statements(&block.stmts, out);
                }
            }
            _ => {}
        }
        // The ordinary walker handles branches, blocks and expressions; scoped bodies above
        // are entered explicitly because they are part of this complete declaration snippet.
        for child in walk::children(node) {
            visit(child, out);
        }
    }
    let mut out = Vec::new();
    declaration(decl, &mut out);
    out
}

/// A leading comment belongs to a new declaration line, never a later token on that line.
fn leading_docs(text: &str, at: usize) -> Option<String> {
    let start = text[..at].rfind('\n').map_or(0, |offset| offset + 1);
    text[start..at]
        .trim()
        .is_empty()
        .then(|| docs::above(text, at))
        .flatten()
}

/// Associated leading and trailing comments both survive; neither is a layout instruction.
fn declaration_docs(text: &str, at: usize) -> String {
    match (leading_docs(text, at), docs::trailing(text, at)) {
        (Some(above), Some(trailing)) => format!("{above}\n\n{trailing}"),
        (Some(comment), None) | (None, Some(comment)) => comment,
        (None, None) => String::new(),
    }
}

/// `written` with everything that cannot name a type blanked to spaces (same length, so
/// offsets hold): comments, notes (`@json(default)`) and the names the declaration introduces.
fn blanked(written: &str, start: usize, decl: &Decl) -> String {
    let (_, comments, _) = lex_with_trivia(FileId(0), written);
    let mut bytes = written.as_bytes().to_vec();
    let mut blank = |from: usize, to: usize| {
        for byte in bytes.iter_mut().take(to.min(written.len())).skip(from) {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    };
    for c in &comments {
        blank(c.span.start as usize, c.span.end as usize);
    }
    for (from, to) in declared_names(decl) {
        if from >= start {
            blank(from - start, to - start);
        }
    }
    let mut out = String::from_utf8_lossy(&bytes).into_owned();
    // A note is `@` and a name; its arguments are values, not types.
    let chars: Vec<char> = out.chars().collect();
    let mut cleaned = String::with_capacity(out.len());
    let (mut i, mut in_string) = (0, false);
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            cleaned.push(c);
            if c == '\\' && i + 1 < chars.len() {
                cleaned.push(chars[i + 1]);
                i += 1;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            cleaned.push(c);
        } else if c == '@' {
            cleaned.push(' ');
            while chars
                .get(i + 1)
                .is_some_and(|n| n.is_alphanumeric() || *n == '_')
            {
                cleaned.push(' ');
                i += 1;
            }
        } else {
            cleaned.push(c);
        }
        i += 1;
    }
    out = cleaned;
    out
}

/// Preserve lexical spellings (including literal contents), while deriving row/name boundaries
/// from parsed declarations rather than authored newlines or alignment.
fn declaration_tokens(
    text: &str,
    decl: &Decl,
    start: usize,
    end: usize,
) -> Vec<super::DeclarationToken> {
    use crate::ast::walk::{self, Node};
    use crate::lexer::{P, Tok, lex};
    use std::collections::BTreeMap;
    let mut boundaries = BTreeMap::<u32, (Vec<String>, String)>::new();
    fn enums(
        items: &[EnumItem],
        text: &str,
        boundaries: &mut BTreeMap<u32, (Vec<String>, String)>,
    ) {
        for item in items {
            match item {
                EnumItem::Member(member) => {
                    let at = member.name.span.start;
                    boundaries.insert(
                        at,
                        (
                            vec![member.name.name.to_string()],
                            declaration_docs(text, at as usize),
                        ),
                    );
                }
                EnumItem::If {
                    then_items,
                    else_items,
                    ..
                } => {
                    enums(then_items, text, boundaries);
                    enums(else_items, text, boundaries);
                }
                _ => {}
            }
        }
    }
    fn record(node: Node<'_>, text: &str, boundaries: &mut BTreeMap<u32, (Vec<String>, String)>) {
        match node {
            Node::Stmt(stmt) => {
                let (names, docs) = if let StmtKind::Decl(field) = &stmt.kind {
                    let names = field
                        .names
                        .iter()
                        .map(|name| name.name.to_string())
                        .collect();
                    (names, declaration_docs(text, stmt.span.start as usize))
                } else {
                    (Vec::new(), String::new())
                };
                boundaries.insert(stmt.span.start, (names, docs));
            }
            // The generic body walker deliberately stops at scope boundaries. A declaration
            // snippet includes those type bodies, so enter them explicitly here.
            Node::Expr(Expr {
                kind: ExprKind::Struct(lit),
                ..
            }) => {
                for param in &lit.params {
                    if let Some(name) = &param.name {
                        boundaries.insert(
                            name.span.start,
                            (Vec::new(), declaration_docs(text, name.span.start as usize)),
                        );
                    }
                }
                for stmt in &lit.body {
                    record(Node::Stmt(stmt), text, boundaries);
                }
                if let Some(block) = &lit.modify {
                    for stmt in &block.stmts {
                        record(Node::Stmt(stmt), text, boundaries);
                    }
                }
            }
            Node::Expr(Expr {
                kind: ExprKind::Enum(lit),
                ..
            }) => enums(&lit.items, text, boundaries),
            Node::Expr(Expr {
                kind: ExprKind::Proc(lit),
                ..
            }) => {
                if let Some(block) = &lit.body {
                    for stmt in &block.stmts {
                        record(Node::Stmt(stmt), text, boundaries);
                    }
                }
            }
            Node::Expr(_) => {}
        }
        for child in walk::children(node) {
            record(child, text, boundaries);
        }
    }
    if let Some(value) = &decl.value {
        record(Node::Expr(value), text, &mut boundaries);
    }
    let Ok(tokens) = lex(FileId(0), &text[start..end]) else {
        return Vec::new();
    };
    let mut member_names = BTreeMap::new();
    fn enum_names(items: &[EnumItem], out: &mut BTreeMap<u32, String>) {
        for item in items {
            match item {
                EnumItem::Member(member) => {
                    out.insert(member.name.span.start, member.name.name.to_string());
                }
                EnumItem::If {
                    then_items,
                    else_items,
                    ..
                } => {
                    enum_names(then_items, out);
                    enum_names(else_items, out);
                }
                _ => {}
            }
        }
    }
    match decl.value.as_ref().map(|value| &value.kind) {
        Some(ExprKind::Struct(lit)) => {
            for stmt in super::type_fields(&lit.body) {
                if let StmtKind::Decl(field) = &stmt.kind {
                    for name in &field.names {
                        member_names.insert(name.span.start, name.name.to_string());
                    }
                }
            }
        }
        Some(ExprKind::Enum(lit)) => enum_names(&lit.items, &mut member_names),
        _ => {}
    }
    let declared = declared_names(decl).into_iter().collect::<BTreeSet<_>>();
    let mut out = Vec::new();
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        if matches!(token.tok, Tok::Eof) {
            break;
        }
        let at = start as u32 + token.span.start;
        let reference = matches!(token.tok, Tok::Ident(_))
            && !declared.contains(&(at as usize, start + token.span.end as usize));
        let statement = boundaries.contains_key(&at);
        let (_, mut docs) = boundaries.remove(&at).unwrap_or_default();
        let names = member_names.remove(&at).into_iter().collect();
        // The public member list belongs to the outer declaration. Nested names remain lexical
        // facts, but cannot accidentally borrow an unrelated outer member's fragment identity.
        if docs.is_empty() && matches!(token.tok, Tok::Punct(P::RBrace)) {
            docs = leading_docs(text, at as usize).unwrap_or_default();
        }
        let mut token_end = start as u32 + token.span.end;
        let mut spelling =
            text[start + token.span.start as usize..start + token.span.end as usize].to_owned();
        // A resolved qualified name is one semantic path, even when the author spaces its dots.
        if matches!(token.tok, Tok::Ident(_)) {
            while tokens
                .peek()
                .is_some_and(|next| matches!(next.tok, Tok::Punct(P::Dot)))
            {
                let mut lookahead = tokens.clone();
                lookahead.next();
                if !lookahead
                    .next()
                    .is_some_and(|next| matches!(next.tok, Tok::Ident(_)))
                {
                    break;
                }
                tokens.next();
                let name = tokens.next().unwrap();
                spelling.push('.');
                spelling.push_str(
                    &text[start + name.span.start as usize..start + name.span.end as usize],
                );
                token_end = start as u32 + name.span.end;
            }
        }
        out.push(super::DeclarationToken {
            text: spelling,
            start: at,
            end: token_end,
            statement,
            reference,
            names,
            docs,
        });
    }
    out
}

impl Reader<'_> {
    /// Sets the declaration of a struct, union or enum as written, and the declarations its
    /// type names stand for.
    pub(super) fn item_declaration(
        &mut self,
        file: &Path,
        text: &str,
        decl: &Decl,
        item: &mut Item,
    ) {
        let Some(name) = decl.names.first() else {
            return;
        };
        let start = name.span.start as usize;
        let end = decl
            .span
            .end
            .max(decl.value.as_ref().map_or(0, |v| v.span.end)) as usize;
        let Some(written) = text.get(start..end.min(text.len())) else {
            return;
        };
        let tokens = declaration_tokens(text, decl, start, end.min(text.len()));
        let mut refs = self.type_refs(file, &blanked(written, start, decl));
        refs.retain(|r| {
            let root = r.path.split('.').next().unwrap_or(&r.path);
            !item.scope.iter().any(|s| s == root)
        });
        let parsed = self.parsed(file);
        item.definitions.push(super::Definition {
            kind: class_kind(decl),
            docs: docs::declaration(text, start).unwrap_or_default(),
            file: file.to_path_buf(),
            line: self.line_of(text, start),
            start: start as u32,
            end: end.min(text.len()) as u32,
            platforms: os_names(
                parsed
                    .as_ref()
                    .and_then(|file| file.limits.get(&name.span.start).copied())
                    .unwrap_or(EVERY_OS),
            ),
            guards: parsed
                .as_ref()
                .and_then(|file| file.guards.get(&name.span.start).cloned())
                .unwrap_or_default(),
            scope: item.scope.clone(),
            tokens,
            members: item.members.clone(),
            refs,
        });
    }

    /// Adds the private structs, unions and enums that public items name, and those they name
    /// in turn, as items marked `private`, so a reader can follow a field to what it is.
    pub(super) fn add_private_types(&mut self, out: &mut Module, names: &mut Vec<String>) {
        let public: BTreeSet<(PathBuf, usize)> = out
            .items
            .iter()
            .flat_map(|i| std::iter::once((i.file.clone(), i.line)).chain(i.also.iter().cloned()))
            .collect();
        let mut queue: Vec<TypeRef> = Vec::new();
        for item in &out.items {
            queue.extend(item_refs(item));
        }
        queue.reverse();
        let mut seen: BTreeSet<(PathBuf, usize)> = BTreeSet::new();
        while let Some(r) = queue.pop() {
            if r.at == 0 || !self.library.contains(&r.file) || !seen.insert((r.file.clone(), r.at))
            {
                continue;
            }
            let Some(target) = self.parsed(&r.file) else {
                continue;
            };
            let index = TopIndex::new(&target);
            let Some(decl) = index.at.get(&r.at).copied() else {
                continue;
            };
            let Some(name) = decl.names.first().filter(|n| n.span.start as usize == r.at) else {
                continue;
            };
            let visibility = index
                .constants(name.name.as_str())
                .iter()
                .find(|(d, _)| std::ptr::eq(*d, decl))
                .map(|(_, v)| *v);
            let kind = class_kind(decl);
            if visibility.is_none_or(|v| v == Visibility::Export)
                || !matches!(kind, "struct" | "union" | "enum" | "enum_flags")
                || names.iter().any(|n| n == name.name.as_str())
            {
                continue;
            }
            let line = self.line_of(&target.text, r.at);
            if public.contains(&(r.file.clone(), line)) {
                continue;
            }
            let mut item = Item {
                name: name.name.as_str().to_owned(),
                kind,
                file: r.file.clone(),
                line,
                docs: docs::declaration(&target.text, r.at).unwrap_or_default(),
                scope: struct_params(decl),
                platforms: os_names(
                    target
                        .limits
                        .get(&name.span.start)
                        .copied()
                        .unwrap_or(EVERY_OS),
                ),
                private: true,
                ..Item::default()
            };
            self.item_members(&r.file, &target.text, decl, &mut item);
            self.item_declaration(&r.file, &target.text, decl, &mut item);
            queue.extend(item_refs(&item).into_iter().rev());
            names.push(item.name.clone());
            out.items.push(item);
        }
    }
}

/// Every declaration an item's page names: in its signature, fields and written declaration.
fn item_refs(item: &Item) -> Vec<TypeRef> {
    let mut out: Vec<TypeRef> = Vec::new();
    out.extend(item.refs.iter().cloned());
    for member in &item.members {
        out.extend(member.refs.iter().cloned());
    }
    // Conditional variants can name private types absent from the first definition.
    // Their pages require the same dependency closure as the canonical signature.
    for definition in &item.definitions {
        out.extend(definition.refs.iter().cloned());
        for member in &definition.members {
            out.extend(member.refs.iter().cloned());
        }
    }
    for overload in &item.overloads {
        for p in overload.params.iter().chain(&overload.returns) {
            out.extend(p.refs.iter().cloned());
        }
    }
    out
}
