//! Exporting type instantiations, lists, filters and literals.
use super::Exporter;
use super::constants::{inst_flags, node};
use crate::builtin_keys::SyntaxKey;
use crate::records::{Field, Item, Record, RecordTag};
use crate::sema::{Span, ast, procs};

impl Exporter<'_> {
    /// A `Code_Type_Instantiation` for a type expression, with its resolved type when it has one.
    pub(super) fn type_inst(&mut self, expr: &ast::Expr) -> i64 {
        let mut rec = self.node(
            RecordTag::CodeTypeInstantiation,
            node::TYPE_INSTANTIATION,
            expr.span,
        );
        if !procs::has_poly(expr)
            && let Some(t) = self.eval_type(expr)
        {
            let t = self.ty(t);
            rec.ptr("result", t);
            rec.ptr("type", t);
        }
        match &expr.kind {
            ast::ExprKind::ArrayType {
                size,
                elem,
            } => {
                let elem = self.type_inst(elem);
                rec.ptr("array_element_type", elem);
                match size {
                    ast::ArraySize::Fixed(size) => {
                        let size = self.expr(size);
                        rec.ptr("array_dimension", size);
                    }
                    ast::ArraySize::View => {
                        rec.int("inst_flags", inst_flags::ARRAY_VIEW);
                    }
                    ast::ArraySize::Resizable => {
                        rec.int("inst_flags", inst_flags::RESIZABLE);
                    }
                }
            }
            ast::ExprKind::Unary(ast::UnOp::Star, pointee) => {
                let pointee = self.type_inst(pointee);
                rec.ptr("pointer_to", pointee);
            }
            ast::ExprKind::ProcType(header) => {
                let lit = ast::ProcLit {
                    header: header.clone(),
                    body: None,
                };
                let value = self.proc(&lit, &[], None, expr.span);
                rec.ptr("type_valued_expression", value)
                    .int("inst_flags", inst_flags::TYPE_OF_PROCEDURE);
            }
            ast::ExprKind::TypeDirective {
                modifier,
                ty,
            } => {
                let value = self.type_inst(ty);
                rec.ptr("type_directive_target", value).int(
                    "inst_flags",
                    match modifier {
                        ast::TypeModifier::Plain => inst_flags::TYPE_DIRECTIVE,
                        ast::TypeModifier::Distinct => inst_flags::TYPE_DIRECTIVE_DISTINCT,
                        ast::TypeModifier::Isa => inst_flags::TYPE_DIRECTIVE_ISA,
                    },
                );
            }
            _ => {
                let value = self.expr(expr);
                rec.ptr("type_valued_expression", value);
            }
        }
        self.add(rec)
    }

    pub(super) fn comma_values(&mut self, values: &[ast::Expr], span: Span) -> i64 {
        let nodes = values
            .iter()
            .map(|value| self.expr(value))
            .collect::<Vec<_>>();
        self.node_list(&nodes, span)
    }

    pub(super) fn node_list(&mut self, nodes: &[i64], span: Span) -> i64 {
        if nodes.len() <= 1 {
            return nodes.first().copied().unwrap_or(0);
        }
        let arguments: Vec<_> = nodes
            .iter()
            .map(|&node| {
                let mut arg = Record::new(RecordTag::CodeCommaSeparatedArgument);
                arg.ptr("node", node).int("modifier", 0);
                self.r.add(arg)
            })
            .collect();
        let mut rec = self.node(
            RecordTag::CodeCommaSeparatedArguments,
            node::COMMA_SEPARATED_ARGUMENTS,
            span,
        );
        rec.refs("arguments", arguments);
        self.add(rec)
    }

    pub(super) fn syntax_block(&mut self, block: &ast::Block) -> i64 {
        let id = self.block(&block.stmts, 1, block.span);
        self.r
            .get_mut(id)
            .unwrap()
            .int("block_flags", i64::from(block.checks_off.bits()));
        id
    }

    pub(super) fn modify_syntax(&mut self, block: Option<&ast::Block>) -> Vec<i64> {
        let Some(block) = block else {
            return Vec::new();
        };
        let body = self.syntax_block(block);
        let mut rec = self.node(
            RecordTag::CodeDirectiveModify,
            node::DIRECTIVE_MODIFY,
            block.span,
        );
        rec.ptr("block", body);
        vec![self.add(rec)]
    }

    pub(super) fn insert_syntax(
        &mut self,
        value: &ast::Expr,
        scope: Option<&ast::Expr>,
        replacements: &[ast::Arg],
        flags: &[ast::Ident],
        span: Span,
    ) -> Record {
        let value = self.expr(value);
        let scope = self.opt_expr(scope);
        let mut rec = self.node(RecordTag::CodeDirectiveInsert, node::DIRECTIVE_INSERT, span);
        rec.ptr("expression", value)
            .ptr("scope_redirection", scope)
            .str(
                SyntaxKey::Flags,
                flags
                    .iter()
                    .map(|flag| format!(",{}", flag.name))
                    .collect::<String>()
                    .as_bytes(),
            );
        for replacement in replacements {
            let Some(name) = replacement.name else {
                rec.int(SyntaxKey::Incomplete, 1);
                continue;
            };
            let field = match name.name.as_str() {
                "break" => "break_replacement",
                "continue" => "continue_replacement",
                "remove" => "remove_replacement",
                _ => {
                    rec.int(SyntaxKey::Incomplete, 1);
                    continue;
                }
            };
            let value = self.expr(&replacement.value);
            rec.ptr(field, value);
        }
        rec
    }

    pub(super) fn opt_expr(&mut self, e: Option<&ast::Expr>) -> i64 {
        e.map_or(0, |e| self.expr(e))
    }

    pub(super) fn literal(&mut self, span: Span, value_type: i64) -> Record {
        let mut rec = self.node(RecordTag::CodeLiteral, node::LITERAL, span);
        rec.int("value_type", value_type);
        rec
    }

    pub(super) fn using_filter(&mut self, filter: &ast::UsingFilter) -> (i64, i64) {
        use ast::{UsingFilter, UsingFilterKind};
        match filter {
            UsingFilter::None => (0, 0),
            // Static name-list/map expression shapes are not established by the
            // reviewed public record contract. Preserve their category without
            // fabricating a replacement expression that looks reference-derived.
            UsingFilter::Only(_) => (1, 0),
            UsingFilter::Except(_) => (2, 0),
            UsingFilter::Map(_) => (3, 0),
            UsingFilter::Computed {
                kind,
                expr,
            } => {
                let kind = match kind {
                    UsingFilterKind::Only => 1,
                    UsingFilterKind::Except => 2,
                    UsingFilterKind::Map => 3,
                };
                (kind, self.expr(expr))
            }
        }
    }

    pub(super) fn filter_syntax(filter: &ast::UsingFilter) -> String {
        let kind = match filter {
            ast::UsingFilter::Only(_) => "only",
            ast::UsingFilter::Except(_) => "except",
            ast::UsingFilter::Map(_) => "map",
            _ => return String::new(),
        };
        format!(",{kind}({})", Self::filter_arguments(filter))
    }

    /// Canonical static arguments, separate from the editable public filter category.
    pub(super) fn filter_arguments(filter: &ast::UsingFilter) -> String {
        match filter {
            ast::UsingFilter::Only(names) | ast::UsingFilter::Except(names) => names
                .iter()
                .map(|name| name.name.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            ast::UsingFilter::Map(pairs) => pairs
                .iter()
                .map(|(a, b)| format!("{}={}", a.name, b.name))
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        }
    }

    pub(super) fn literal_number_flags(
        &self,
        span: Span,
        float: Option<f64>,
        negative: bool,
    ) -> i64 {
        let text = self.c.as_ref().map_or("", |c| c.sources.snippet(span));
        let spelling = text.trim_start_matches('-').trim_start();
        let mut flags = 0x1
            | if negative {
                0x40
            } else {
                0
            };
        if spelling.starts_with("0x")
            || spelling.starts_with("0X")
            || spelling.starts_with("0h")
            || spelling.starts_with("0H")
        {
            flags |= 0x2;
        }
        if spelling.starts_with("0b") || spelling.starts_with("0B") {
            flags |= 0x20;
        }
        if let Some(value) = float {
            flags |= 0x4;
            let precision = crate::lexer::float_literal_precision(spelling, value);
            if precision.defaults_to_float64 {
                flags |= 0x100;
            }
            if precision.requires_float64 {
                flags |= 0x200;
            }
        }
        flags
    }

    pub(super) fn expr(&mut self, e: &ast::Expr) -> i64 {
        let id = self.expr_inner(e);
        // Use the result of actual checking in the correct instance/scope, not
        // another check in the global scope (which loses locals and overloads).
        let checked = self.procedure.and_then(|procedure| {
            self.c
                .as_ref()
                .and_then(|c| c.export.expression_types.get(&procedure))
                .and_then(|types| types.get(e.span))
        });
        if let Some(checked) = checked {
            // An ambiguous expansion must not inherit a guessed identifier/call type.
            let ty = checked.map_or(0, |ty| self.ty(ty));
            self.r
                .get_mut(id)
                .unwrap()
                .set("type", Field::Item(Item::Ref(ty)));
        }
        id
    }
}
