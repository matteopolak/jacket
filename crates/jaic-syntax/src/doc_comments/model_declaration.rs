//! The written declaration of a struct, union or enum, and the private types a public item
//! leads a reader to.
use super::super::refs::TypeRef;
use super::{Item, Module, Reader, TopIndex, class_kind, os_names, struct_params};
use crate::ast::{Decl, EnumItem, Expr, ExprKind, Stmt, StmtKind};
use crate::docs;
use crate::lexer::lex_with_trivia;
use crate::parser::ide_facts::Visibility;
use crate::source::FileId;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// `text` without the comments `drop` accepts (it is given each comment's start offset). Comment-only
/// lines go, a line that ends in a comment loses the trailing space, and blank lines are not
/// stacked; everything else keeps its layout. The lexer says what a comment is, so `//` in a
/// string stays.
///
/// Also for each line of `text` the line it is in the result, `None` for a line that went.
pub(super) fn stripped(text: &str, drop: &dyn Fn(usize) -> bool) -> (String, Vec<Option<usize>>) {
    let (_, comments, _) = lex_with_trivia(FileId(0), text);
    let mut dropped = vec![false; text.len()];
    for c in comments.iter().filter(|c| drop(c.span.start as usize)) {
        let (start, end) = (c.span.start as usize, (c.span.end as usize).min(text.len()));
        for (at, byte) in text.bytes().enumerate().take(end).skip(start) {
            dropped[at] = byte != b'\n';
        }
    }
    let mut lines: Vec<(String, usize)> = Vec::new();
    let mut offset = 0;
    let mut source_lines = 0;
    for (number, line) in text.split('\n').enumerate() {
        source_lines = number + 1;
        let end = offset + line.len();
        let had_comment = dropped[offset..end].iter().any(|d| *d);
        let kept: String = line
            .char_indices()
            .filter(|(i, _)| !dropped[offset + i])
            .map(|(_, c)| c)
            .collect();
        offset = end + 1;
        if !had_comment {
            lines.push((kept, number));
            continue;
        }
        let kept = kept.trim_end().to_owned();
        if !kept.trim().is_empty() {
            lines.push((kept, number));
        }
    }
    let mut out: Vec<String> = Vec::new();
    let mut place: Vec<Option<usize>> = vec![None; source_lines];
    for (line, number) in lines {
        let blank = line.trim().is_empty();
        if blank && out.last().is_none_or(|last| last.trim().is_empty()) {
            continue;
        }
        place[number] = Some(out.len());
        out.push(if blank {
            String::new()
        } else {
            line
        });
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    // A line past the end that was only blank is gone with it.
    for line in &mut place {
        if line.is_some_and(|l| l >= out.len()) {
            *line = None;
        }
    }
    (out.join("\n"), place)
}

/// Where the names a declaration introduces are written: its own, its parameters', its fields',
/// the members of enums and of the types nested in it. They are not references to anything.
fn declared_names(decl: &Decl) -> Vec<(usize, usize)> {
    fn from_stmts(stmts: &[Stmt], out: &mut Vec<(usize, usize)>) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Decl(d) => from_decl(d, out),
                StmtKind::Expr(e) => from_expr(e, out),
                StmtKind::StaticIf(branches) => {
                    // The condition is an expression about the build, not a type.
                    out.push((
                        branches.cond.span.start as usize,
                        branches.cond.span.end as usize,
                    ));
                    from_stmts(&branches.then_branch, out);
                    from_stmts(&branches.else_branch, out);
                }
                _ => {}
            }
        }
    }
    fn from_decl(decl: &Decl, out: &mut Vec<(usize, usize)>) {
        out.extend(
            decl.names
                .iter()
                .map(|n| (n.span.start as usize, n.span.end as usize)),
        );
        if let Some(value) = &decl.value {
            from_expr(value, out);
        }
    }
    fn from_members(items: &[EnumItem], out: &mut Vec<(usize, usize)>) {
        for item in items {
            match item {
                EnumItem::Member(m) => {
                    out.push((m.name.span.start as usize, m.name.span.end as usize));
                }
                EnumItem::If {
                    then_items,
                    else_items,
                    ..
                } => {
                    from_members(then_items, out);
                    from_members(else_items, out);
                }
                _ => {}
            }
        }
    }
    fn from_expr(expr: &Expr, out: &mut Vec<(usize, usize)>) {
        match &expr.kind {
            ExprKind::Struct(lit) => {
                out.extend(
                    lit.params
                        .iter()
                        .filter_map(|p| p.name.as_ref())
                        .map(|n| (n.span.start as usize, n.span.end as usize)),
                );
                from_stmts(&lit.body, out);
            }
            ExprKind::Enum(lit) => from_members(&lit.items, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    from_decl(decl, &mut out);
    out
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
        // A field's docs are its member's; any other comment stays where it was written.
        let member_docs: BTreeSet<usize> = item
            .members
            .iter()
            .flat_map(|m| docs::declaration_comments(text, m.at))
            .collect();
        let (plain, places) = stripped(written, &|at| member_docs.contains(&(start + at)));
        plain
            .trim_end_matches(';')
            .trim_end()
            .clone_into(&mut item.declaration);
        for member in &mut item.members {
            member.line = member
                .at
                .checked_sub(start)
                .and_then(|at| written.get(..at))
                .and_then(|before| places.get(before.matches('\n').count()).copied().flatten());
        }
        let mut refs = self.type_refs(file, &blanked(written, start, decl));
        refs.retain(|r| {
            let root = r.path.split('.').next().unwrap_or(&r.path);
            !item.scope.iter().any(|s| s == root)
        });
        item.decl_refs = refs;
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
                platforms: os_names(target.os(name.span.start)),
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
    let copy = |r: &TypeRef| TypeRef {
        path: r.path.clone(),
        file: r.file.clone(),
        line: r.line,
        at: r.at,
        private: r.private,
    };
    out.extend(item.refs.iter().map(copy));
    out.extend(item.decl_refs.iter().map(copy));
    for member in &item.members {
        out.extend(member.refs.iter().map(copy));
    }
    for overload in &item.overloads {
        for p in overload.params.iter().chain(&overload.returns) {
            out.extend(p.refs.iter().map(copy));
        }
    }
    out
}
