//! Operators are selected by source tokens while retaining typed declaration IDs.
use super::*;
use std::collections::BTreeMap;
use syntax::OperatorKind;

impl Resolver<'_> {
    pub(super) fn using_module_members(
        &self,
        module: jai_source::ModuleId,
        span: Span,
    ) -> Result<Vec<UsingMember>, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "using a module requires a defining source scope")
        })?;
        let mut members = scope
            .module_exports(module)
            .into_iter()
            .map(|(name, binding)| {
                UsingMember::named(self.symbols, name, Binding::Imported(binding))
            })
            .collect::<Vec<_>>();
        for (name, marker) in scope.module_placeholder_exports(module, span) {
            let name = self.symbols.name(name).as_bytes().to_vec();
            if !members.iter().any(|member| member.name == name) {
                members.push(UsingMember {
                    name,
                    binding: UsingMemberBinding::Placeholder(marker),
                });
            }
        }
        let mut operators = BTreeMap::<&str, Vec<jai_source::DeclarationId>>::new();
        for (kind, declarations) in scope.using_exported_operators(module) {
            let name = operator_name(kind).ok_or_else(|| {
                Diagnostic::new(span, "using target contains an unsupported operator token")
            })?;
            operators.entry(name).or_default().extend(declarations);
        }
        for (name, mut declarations) in operators {
            declarations.sort_by_key(|id| id.index());
            declarations.dedup();
            members.push(UsingMember {
                name: name.as_bytes().to_vec(),
                binding: UsingMemberBinding::Operators(declarations),
            });
        }
        Ok(members)
    }
}

fn operator_name(kind: OperatorKind) -> Option<&'static str> {
    Some(match kind {
        OperatorKind::Index => "[]",
        OperatorKind::IndexAssign => "[]=",
        OperatorKind::IndexAddress => "*[]",
        OperatorKind::Unary(UnaryOp::Positive) => "+",
        OperatorKind::Unary(UnaryOp::Negate) => "-",
        OperatorKind::Unary(UnaryOp::LogicalNot) => "!",
        OperatorKind::Unary(UnaryOp::Complement) => "~",
        OperatorKind::Binary(op) => binary_name(op),
        OperatorKind::Compound(op) => match op {
            BinaryOp::Add => "+=",
            BinaryOp::Subtract => "-=",
            BinaryOp::Multiply => "*=",
            BinaryOp::Divide => "/=",
            BinaryOp::Remainder => "%=",
            BinaryOp::BitAnd => "&=",
            BinaryOp::BitOr => "|=",
            BinaryOp::BitXor => "^=",
            BinaryOp::ShiftLeft => "<<=",
            BinaryOp::ShiftRight => ">>=",
            BinaryOp::LogicalAnd => "&&=",
            BinaryOp::LogicalOr => "||=",
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => return None,
        },
    })
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Remainder => "%",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::LogicalAnd => "&&",
        BinaryOp::LogicalOr => "||",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
    }
}
