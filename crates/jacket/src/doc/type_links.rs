//! Validate type positions with the same declaration lookup used by HTML and search links.
//! Values are parsed too: casts, literals and procedure types can contain type positions.
use super::model::{Kind, TypeRef};
use super::site::{BUILTIN_TYPES, Site};
use jaic::ast::walk::{self, Node};
use jaic::ast::{Expr, ExprKind, StmtKind};
use jaic::source::FileId;
use std::collections::BTreeSet;

impl Site {
    pub fn unlinked_types(&self) -> Vec<String> {
        let mut reports = BTreeSet::new();
        for (mi, module) in self.modules.iter().enumerate() {
            let module_scope: Vec<_> = module.parameters.iter().map(|p| p.name.clone()).collect();
            for p in &module.parameters {
                let mut scope = module_scope.clone();
                scope.extend(poly_names(&p.ty));
                for (text, is_type) in [(&p.ty, true), (&p.default, false)] {
                    for name in self.missing_names(text, &p.refs, is_type, &scope, mi) {
                        reports.insert(format!(
                            "{} (#module_parameters {}): `{name}`",
                            Site::module_path(&module.id),
                            p.name
                        ));
                    }
                }
            }
            for item in module.items.iter().chain(&module.private) {
                let mut scope = module_scope.clone();
                scope.extend(item.scope.iter().cloned());
                let mut check = |text: &str, refs: &[TypeRef], is_type: bool, locals: &[String]| {
                    for name in self.missing_names(text, refs, is_type, locals, mi) {
                        reports.insert(format!(
                            "{} ({}): `{name}`",
                            self.item_path(&module.id, &item.name),
                            item.name
                        ));
                    }
                };
                for overload in &item.overloads {
                    let mut locals = scope.clone();
                    // Procedure parameters are local bindings, including baked value parameters.
                    locals.extend(
                        overload
                            .params
                            .iter()
                            .map(|p| p.name.trim_start_matches('$').to_owned()),
                    );
                    for p in overload.params.iter().chain(&overload.returns) {
                        locals.extend(poly_names(&p.ty));
                    }
                    for p in overload.params.iter().chain(&overload.returns) {
                        check(&p.ty, &p.refs, true, &locals);
                        check(&p.default, &p.refs, false, &locals);
                    }
                }
                for member in &item.members {
                    let is_type = matches!(item.kind, Kind::Struct | Kind::Union)
                        && !member.ty.starts_with(":=");
                    check(
                        member.ty.trim_start_matches(":="),
                        &member.refs,
                        is_type,
                        &scope,
                    );
                }
                check(&item.value, &item.refs, item.kind == Kind::Type, &scope);
            }
        }
        reports.into_iter().collect()
    }

    fn missing_names(
        &self,
        text: &str,
        refs: &[TypeRef],
        is_type: bool,
        locals: &[String],
        mi: usize,
    ) -> Vec<String> {
        let mut required = names(text, is_type, locals);
        // Resolved names in value expressions (including enum values and arrays
        // of types) must use their links too. Unresolved ordinary value operands
        // are not type positions; casts and typed literals are checked by the AST.
        required.extend(
            refs.iter()
                .filter(|r| {
                    !locals
                        .iter()
                        .any(|n| r.path.split('.').next() == Some(n.as_str()))
                })
                .map(|r| r.path.clone()),
        );
        required
            .into_iter()
            .filter(|name| {
                // Only declarations proven private, internal, or supplied by the prelude can
                // lack a page. A documented alias hop still links through the normal resolver.
                let unpublished = refs.iter().any(|r| {
                    (r.path == *name || name.starts_with(&format!("{}.", r.path))) && r.private
                });
                !self.type_is_linked(name, refs, mi) && !unpublished
            })
            .collect()
    }

    fn type_is_linked(&self, path: &str, refs: &[TypeRef], here: usize) -> bool {
        // Validate the same semantic spans that both API pages and search previews consume.
        super::export::build::text(self, here, path, refs, None)
            .is_ok_and(|text| text.spans.iter().any(|span| span.target.is_some()))
    }
}

fn parsed(text: &str) -> jaic::parser::IdeParse {
    jaic::parser::parse_file_ide(FileId(0), text)
}

fn poly_names(text: &str) -> Vec<String> {
    let parse = parsed(&format!("check :: {};", text.trim_start_matches("..")));
    let mut names = Vec::new();
    if let Some(jaic::ast::Stmt {
        kind: StmtKind::Decl(decl),
        ..
    }) = parse.file.stmts.first()
        && let Some(header) = jaic::parser::ide_facts::decl_header(decl)
    {
        names.extend(
            jaic::parser::ide_facts::type_parameters(header)
                .iter()
                .map(|n| n.name.as_str().to_owned()),
        );
    }
    walk::walk(&parse.file.stmts, &mut |node, _| {
        if let Some(Expr {
            kind:
                ExprKind::PolyVar {
                    name, ..
                }
                | ExprKind::PolyRestricted {
                    name, ..
                },
            ..
        }) = node.expr()
        {
            names.push(name.as_str().to_owned());
        }
        true
    });
    names
}

fn names(text: &str, is_type: bool, scope: &[String]) -> BTreeSet<String> {
    let source = format!("check :: {};", text.trim_start_matches(".."));
    let parse = parsed(&source);
    let mut names = BTreeSet::new();
    let mut scope = scope.to_vec();
    scope.extend(poly_names(text));
    let Some(jaic::ast::Stmt {
        kind: StmtKind::Decl(decl),
        ..
    }) = parse.file.stmts.first()
    else {
        return names;
    };
    if let Some(value) = &decl.value {
        collect(value, is_type, &source, &mut scope, &mut names);
    }
    names
}

fn collect(
    expr: &Expr,
    is_type: bool,
    source: &str,
    scope: &mut Vec<String>,
    out: &mut BTreeSet<String>,
) {
    match &expr.kind {
        ExprKind::Ident(_) | ExprKind::Member(..) if is_type => {
            let name = source[expr.span.start as usize..expr.span.end as usize].trim();
            let root = name.split('.').next().unwrap_or(name);
            // Language-provided types have no declaration; polymorphic and struct/module
            // parameters are local bindings with no page. Never exempt an unknown name.
            if !BUILTIN_TYPES.contains(&name) && !scope.iter().any(|n| n == root) {
                out.insert(name.to_owned());
            }
        }
        // `$T` introduces a local type rather than referring to a documented declaration.
        ExprKind::PolyVar {
            ..
        } => {}
        ExprKind::PolyRestricted {
            restriction, ..
        } => collect(restriction, true, source, scope, out),
        ExprKind::ArrayType {
            elem, ..
        } => {
            // Array lengths are value expressions, not type names.
            collect(elem, true, source, scope, out);
        }
        ExprKind::ProcType(header) => {
            scope.extend(
                header
                    .params
                    .iter()
                    .filter_map(|p| p.name.as_ref())
                    .chain(header.returns.iter().filter_map(|p| p.name.as_ref()))
                    .map(|n| n.name.as_str().to_owned()),
            );
            for (ty, default) in header
                .params
                .iter()
                .map(|p| (&p.ty, &p.default))
                .chain(header.returns.iter().map(|p| (&p.ty, &p.default)))
            {
                if let Some(ty) = ty {
                    collect(ty, true, source, scope, out);
                }
                if let Some(default) = default {
                    collect(default, false, source, scope, out);
                }
            }
        }
        ExprKind::Cast {
            ty,
            value,
            ..
        } => {
            if let Some(ty) = ty {
                collect(ty, true, source, scope, out);
            }
            collect(value, false, source, scope, out);
        }
        ExprKind::StructLit {
            ty,
            fields,
        } => {
            if let Some(ty) = ty {
                collect(ty, true, source, scope, out);
            }
            for field in fields {
                collect(&field.value, false, source, scope, out);
            }
        }
        ExprKind::ArrayLit {
            ty,
            elems,
        } => {
            if let Some(ty) = ty {
                collect(ty, true, source, scope, out);
            }
            for elem in elems {
                collect(elem, false, source, scope, out);
            }
        }
        // `type_of` is a compiler query taking a value expression, not a declared type
        // constructor. Neither the query operator nor its value operand needs a type page.
        ExprKind::Call {
            callee,
            args,
            ..
        } if matches!(&callee.kind, ExprKind::Ident(n) if n.as_str() == "type_of") => {
            for arg in args {
                collect(&arg.value, false, source, scope, out);
            }
        }
        ExprKind::Call {
            callee,
            args,
            ..
        } if is_type => {
            collect(callee, true, source, scope, out);
            for arg in args {
                collect(&arg.value, true, source, scope, out);
            }
        }
        ExprKind::Unary(_, inner)
        | ExprKind::TypeDirective {
            ty: inner, ..
        } => collect(inner, is_type, source, scope, out),
        // Literal contents, enum values, directive operands and ordinary value names are
        // not type references. Still visit expressions within them for explicit type positions.
        _ => {
            for node in walk::children(Node::Expr(expr)) {
                if let Node::Expr(child) = node {
                    collect(child, false, source, scope, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::model::{self, Item, Module, Overload, Param};

    fn site(items: Vec<Item>) -> Site {
        Site::new(
            vec![Module {
                id: "Test".into(),
                items,
                ..Module::default()
            }],
            vec![],
            "/".into(),
            String::new(),
            String::new(),
            String::new(),
            false,
        )
    }

    #[test]
    fn missing_type_names_report_page_item_and_name() {
        let site = site(vec![Item {
            name: "use".into(),
            overloads: vec![Overload {
                params: vec![Param {
                    name: "x".into(),
                    ty: "*Missing".into(),
                    ..Param::default()
                }],
                returns: vec![Param {
                    ty: "[] Other".into(),
                    ..Param::default()
                }],
                ..Overload::default()
            }],
            ..Item::default()
        }]);
        assert_eq!(
            site.unlinked_types(),
            [
                "api/Test/use.html (use): `Missing`",
                "api/Test/use.html (use): `Other`"
            ]
        );
    }

    #[test]
    fn local_parameters_builtins_and_proven_private_types_need_no_page() {
        let site = site(vec![Item {
            name: "Box".into(),
            kind: Kind::Struct,
            scope: vec!["Element".into()],
            members: vec![model::Member {
                name: "value".into(),
                ty: "Element".into(),
                ..model::Member::default()
            }],
            overloads: vec![Overload {
                params: vec![Param {
                    name: "x".into(),
                    ty: "*$T".into(),
                    ..Param::default()
                }],
                returns: vec![
                    Param {
                        ty: "T".into(),
                        ..Param::default()
                    },
                    Param {
                        ty: "int".into(),
                        ..Param::default()
                    },
                    Param {
                        ty: "Private".into(),
                        refs: vec![TypeRef {
                            path: "Private".into(),
                            private: true,
                            ..TypeRef::default()
                        }],
                        ..Param::default()
                    },
                ],
                ..Overload::default()
            }],
            ..Item::default()
        }]);
        assert!(
            site.unlinked_types().is_empty(),
            "{:?}",
            site.unlinked_types()
        );
    }

    #[test]
    fn alias_chain_selects_nearest_page_for_html_and_search_and_skips_private_hops() {
        let site = site(vec![
            Item {
                name: "Color".into(),
                kind: Kind::Type,
                file: "color.jai".into(),
                line: 2,
                ..Item::default()
            },
            Item {
                name: "Vector".into(),
                kind: Kind::Struct,
                file: "vector.jai".into(),
                line: 3,
                also: vec![("other-branch.jai".into(), 4)],
                ..Item::default()
            },
        ]);
        let refs = vec![
            TypeRef {
                path: "Alias".into(),
                file: "private.jai".into(),
                line: 1,
                private: true,
            },
            TypeRef {
                path: "Alias".into(),
                file: "color.jai".into(),
                line: 2,
                private: false,
            },
            TypeRef {
                path: "Alias".into(),
                file: "vector.jai".into(),
                line: 3,
                private: false,
            },
        ];
        assert_eq!(site.resolved("Alias", &refs, 0), Some((0, "Color")));
        assert!(site.type_is_linked("Alias", &refs, 0));
        assert_eq!(site.resolved("Alias", &refs[2..], 0), Some((0, "Vector")));
        assert!(site.type_is_linked("Alias", &refs[2..], 0));
        let branch = [TypeRef {
            path: "Vector".into(),
            file: "other-branch.jai".into(),
            line: 4,
            private: false,
        }];
        assert!(site.type_is_linked("Vector", &branch, 0));
    }

    #[test]
    fn values_and_members_are_checked_but_literal_contents_are_not_types() {
        let site = site(vec![
            Item {
                name: "C".into(),
                kind: Kind::Constant,
                value: "cast(Missing) 0".into(),
                ..Item::default()
            },
            Item {
                name: "S".into(),
                kind: Kind::Union,
                members: vec![model::Member {
                    ty: "Field_Type".into(),
                    ..model::Member::default()
                }],
                ..Item::default()
            },
            Item {
                name: "E".into(),
                kind: Kind::Enum,
                members: vec![model::Member {
                    ty: "cast(Enum_Type) 0".into(),
                    ..model::Member::default()
                }],
                ..Item::default()
            },
            Item {
                name: "Alias".into(),
                kind: Kind::Type,
                value: "#type (p: Arg) -> Result".into(),
                ..Item::default()
            },
            Item {
                name: "Text".into(),
                kind: Kind::Constant,
                value: "\"Not_A_Type\"".into(),
                ..Item::default()
            },
        ]);
        let reports = site.unlinked_types().join("\n");
        for name in ["Missing", "Field_Type", "Enum_Type", "Arg", "Result"] {
            assert!(reports.contains(name), "{reports}");
        }
        assert!(!reports.contains("Not_A_Type"));
    }
}
