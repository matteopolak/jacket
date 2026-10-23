//! Directives: `#run`, `#insert`, `#import`, generic directives, using filters and `#asm`.
use super::{Convert, R, STATEMENT_KINDS, expr::mk};
use crate::{
    ast::{
        self, AsmBlock, AsmDecl, AsmEvex, AsmInst, AsmItem, AsmMem, AsmMemTerm, AsmOperand, AsmPin,
        AsmSize, DirectiveKind, DirectiveStmt, ExprKind, Ident, Import, ImportSource, InsertStmt,
        RunBody, StmtKind, UsingFilter, UsingFilterKind,
    },
    builtin_keys::SyntaxKey,
    intern::Sym,
    interp::Trap,
    parser::GENERIC_DIRECTIVES,
    types::TypeId,
};

impl Convert<'_, '_> {
    /// A `#run` node's body and `,flags`.
    pub(super) fn run(&mut self, at: u64, ty: TypeId) -> R<(RunBody, Vec<Ident>)> {
        let flags = self.flag_names(at, SyntaxKey::Flags)?;
        let body = self.extra(at, SyntaxKey::RunBody)?;
        if body == 0 {
            let start = self.mark();
            let callee = self.child(at, ty, "procedure")?;
            self.word("()");
            let call = mk(
                ExprKind::Call {
                    callee: Box::new(callee),
                    args: Vec::new(),
                    hint: ast::CallHint::None,
                },
                self.since(start),
            );
            return Ok((RunBody::Expr(call), flags));
        }
        let (name, _) = self.node_type(body)?;
        let body = if name == "BLOCK" {
            RunBody::Block(self.block(body)?)
        } else if STATEMENT_KINDS.contains(&name.as_str()) {
            let stmt = self.stmt(body)?;
            RunBody::Block(ast::Block {
                span: stmt.span,
                stmts: vec![stmt],
                checks_off: ast::CheckBits::NONE,
            })
        } else {
            RunBody::Expr(self.expr(body)?)
        };
        Ok((body, flags))
    }

    pub(super) fn insert(&mut self, at: u64, ty: TypeId) -> R<InsertStmt> {
        self.word("#insert");
        let flags = self.flag_names(at, SyntaxKey::Flags)?;
        let scope = self.opt_expr(at, ty, "scope_redirection")?;
        let mut replacements = Vec::new();
        for (keyword, field) in [
            ("break", "break_replacement"),
            ("continue", "continue_replacement"),
            ("remove", "remove_replacement"),
        ] {
            let value = self.ptr(at, ty, field)?;
            if value != 0 {
                let name = self.name(keyword);
                self.word("=");
                replacements.push(ast::Arg {
                    name: Some(name),
                    target: None,
                    context: false,
                    spread: false,
                    value: self.expr(value)?,
                });
            }
        }
        Ok(InsertStmt {
            value: self.child(at, ty, "expression")?,
            flags,
            scope,
            replacements,
        })
    }

    pub(super) fn import(&mut self, at: u64, ty: TypeId) -> R<Import> {
        let start = self.mark();
        let using = if self.flag(at, SyntaxKey::Using)? {
            Some(self.private_using(at)?)
        } else {
            None
        };
        let alias = self.attribute(at, SyntaxKey::Alias)?;
        let name = (!alias.is_empty()).then(|| self.name(&alias));
        self.word("#import");
        let kind = self.int_attribute(at, SyntaxKey::ImportKind)?;
        let flags = self.flag_names(at, SyntaxKey::Flags)?;
        let path: std::rc::Rc<str> = self.text(at, ty, "name")?.into();
        self.word(&super::quote_bytes(path.as_bytes()));
        let source = match kind {
            0 => ImportSource::Module(path),
            1 => ImportSource::File(path),
            2 => ImportSource::Dir(path),
            3 => ImportSource::String(path),
            _ => return Err(Trap::new("unknown import kind")),
        };
        let call = self.ty("Code_Procedure_Call")?;
        let module = self.ptr(at, ty, "module_parameters_call")?;
        let program = self.ptr(at, ty, "program_parameters_call")?;
        let mut params = Vec::new();
        if module != 0 {
            self.word("(");
            params = self.arguments(module, call, "arguments_unsorted")?;
            self.word(")");
        }
        let program_params_start = if program == 0 {
            None
        } else {
            let start = params.len();
            self.word("(");
            params.extend(self.arguments(program, call, "arguments_unsorted")?);
            self.word(")");
            Some(start)
        };
        Ok(Import {
            source,
            params,
            module_params_present: module != 0 || program != 0,
            program_params_start,
            name,
            flags,
            using,
            span: self.since(start),
        })
    }

    /// A `PLACEHOLDER` statement: `#placeholder`, a generic directive, or an expression.
    pub(super) fn directive_stmt(&mut self, at: u64, name: &str, ty: TypeId) -> R<StmtKind> {
        let directive = self.attribute(at, SyntaxKey::DirectiveName)?;
        if directive == "placeholder" {
            self.word("#placeholder");
            let names = self.extra_list(at, SyntaxKey::DirectiveArgs)?;
            return Ok(StmtKind::Placeholder(
                names
                    .into_iter()
                    .map(|name| self.ident_name(name))
                    .collect::<R<_>>()?,
            ));
        }
        if GENERIC_DIRECTIVES.contains(&directive.as_str()) {
            let span = self.word(&format!("#{directive}"));
            let flags = self.flag_names(at, SyntaxKey::Flags)?;
            let args = self
                .extra_list(at, SyntaxKey::DirectiveArgs)?
                .into_iter()
                .map(|arg| self.expr(arg))
                .collect::<R<_>>()?;
            return Ok(StmtKind::Directive(Box::new(DirectiveStmt {
                name: Ident {
                    name: Sym::intern(&directive),
                    span,
                },
                kind: DirectiveKind::of(&directive),
                flags,
                args,
            })));
        }
        Ok(StmtKind::Expr(self.expr_node(at, name, ty)?))
    }

    /// The filter of a `Code_Using` (a statement or a parameter's).
    pub(super) fn using_filter(&mut self, at: u64, ty: TypeId) -> R<UsingFilter> {
        if self.num(at, ty, "no_parameters")? != 0 {
            return Err(Trap::new("cannot serialize using without parameters"));
        }
        self.word("using");
        if self.num(at, ty, "filter_type")? == 0 {
            return Ok(UsingFilter::None);
        }
        let kind = if self.enum_is(at, ty, "filter_type", "ONLY")? {
            UsingFilterKind::Only
        } else if self.enum_is(at, ty, "filter_type", "EXCEPT")? {
            UsingFilterKind::Except
        } else if self.enum_is(at, ty, "filter_type", "MAP")? {
            UsingFilterKind::Map
        } else {
            return Err(Trap::new("unknown using filter"));
        };
        if let Some(expr) = self.opt_expr(at, ty, "filter_expression")? {
            return Ok(UsingFilter::Computed {
                kind,
                expr,
            });
        }
        let original = self.int_attribute(at, SyntaxKey::StaticFilter)?;
        if original == 0 || (original == 3) != (kind == UsingFilterKind::Map) {
            return Err(Trap::new("using filter has no matching argument syntax"));
        }
        let arguments = self.attribute(at, SyntaxKey::FilterArguments)?;
        self.filter_names(kind, &arguments)
    }

    /// The filter of a declaration or import's `using`, kept as attributes.
    pub(super) fn private_using(&mut self, at: u64) -> R<UsingFilter> {
        self.word("using");
        let expression = self.extra(at, SyntaxKey::FilterExpression)?;
        if expression == 0 {
            let filter = self.attribute(at, SyntaxKey::Filter)?;
            if filter.is_empty() {
                return Ok(UsingFilter::None);
            }
            let parsed = filter.strip_prefix(',').and_then(|rest| {
                let (kind, rest) = rest.split_once('(')?;
                Some((kind.to_owned(), rest.strip_suffix(')')?.to_owned()))
            });
            let kind = match parsed.as_ref().map(|(kind, _)| kind.as_str()) {
                Some("only") => UsingFilterKind::Only,
                Some("except") => UsingFilterKind::Except,
                Some("map") => UsingFilterKind::Map,
                _ => return Err(Trap::new("unknown private using filter")),
            };
            let arguments = parsed.map(|(_, arguments)| arguments).unwrap_or_default();
            return self.filter_names(kind, &arguments);
        }
        let kind = match self.int_attribute(at, SyntaxKey::FilterKind)? {
            1 => UsingFilterKind::Only,
            2 => UsingFilterKind::Except,
            3 => UsingFilterKind::Map,
            _ => return Err(Trap::new("unknown private using filter")),
        };
        Ok(UsingFilter::Computed {
            kind,
            expr: self.expr(expression)?,
        })
    }

    /// `a, b` or `a=b, c=d`.
    fn filter_names(&mut self, kind: UsingFilterKind, arguments: &str) -> R<UsingFilter> {
        let parts = arguments
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        if kind == UsingFilterKind::Map {
            let mut pairs = Vec::new();
            for part in parts {
                let Some((from, to)) = part.split_once('=') else {
                    return Err(Trap::new("using filter has no matching argument syntax"));
                };
                pairs.push((self.name(from.trim()), self.name(to.trim())));
            }
            return Ok(UsingFilter::Map(pairs));
        }
        let names = parts.map(|part| self.name(part)).collect();
        Ok(match kind {
            UsingFilterKind::Only => UsingFilter::Only(names),
            _ => UsingFilter::Except(names),
        })
    }

    pub(super) fn assembly(&mut self, at: u64) -> R<AsmBlock> {
        if !self.flag(at, SyntaxKey::AsmPresent)? {
            return Err(Trap::new("assembly node has no syntax metadata"));
        }
        self.word("#asm");
        let features = self.attribute(at, SyntaxKey::Features)?;
        let features = features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let features = self.idents(features);
        let mut items = Vec::new();
        for item in self.extra_list(at, SyntaxKey::AsmItems)? {
            items.push(
                if self.attribute(item, SyntaxKey::AsmKind)? == "declaration" {
                    AsmItem::Decl(self.asm_decl(item)?)
                } else {
                    AsmItem::Inst(self.asm_inst(item)?)
                },
            );
            self.word(";");
        }
        Ok(AsmBlock {
            features,
            items,
        })
    }

    fn asm_inst(&mut self, at: u64) -> R<AsmInst> {
        let start = self.mark();
        let mnemonic = self.attribute(at, SyntaxKey::Mnemonic)?;
        let mnemonic = self.name(&mnemonic);
        let suffix = self.attribute(at, SyntaxKey::Suffix)?;
        let size = if suffix.is_empty() {
            self.extra_expr(at, SyntaxKey::Size)?
                .map(|size| AsmSize::Dynamic(Box::new(size)))
        } else {
            Some(AsmSize::Suffix(self.name(&suffix)))
        };
        let round = self.attribute(at, SyntaxKey::Round)?;
        let round = match round.strip_prefix('!') {
            None if round.is_empty() => None,
            Some(mode) => Some(mode.chars().next()),
            None => return Err(Trap::new("unknown assembly rounding modifier")),
        };
        let mut operands = Vec::new();
        let mut rounded = false;
        for operand in self.extra_list(at, SyntaxKey::Operands)? {
            let operand = self.asm_operand(operand)?;
            rounded |= !matches!(operand, AsmOperand::Mem(_));
            operands.push(operand);
        }
        if round.is_some() && !rounded {
            return Err(Trap::new(
                "assembly rounding modifier requires a register operand",
            ));
        }
        let mask = self.extra(at, SyntaxKey::Mask)?;
        let mask = if mask == 0 {
            None
        } else {
            let zeroing = self.flag(at, SyntaxKey::Zeroing)?;
            Some((Box::new(self.asm_operand(mask)?), zeroing))
        };
        Ok(AsmInst {
            mnemonic,
            size,
            operands,
            evex: AsmEvex {
                mask,
                round,
            },
            span: self.since(start),
        })
    }

    fn asm_operand(&mut self, at: u64) -> R<AsmOperand> {
        match self.attribute(at, SyntaxKey::AsmKind)?.as_str() {
            "declaration" => Ok(AsmOperand::Decl(self.asm_decl(at)?)),
            "memory" => {
                let start = self.mark();
                self.word("[");
                let mut terms = Vec::new();
                for term in self.extra_list(at, SyntaxKey::Terms)? {
                    let negate = self.flag(term, SyntaxKey::Negate)?;
                    if negate || !terms.is_empty() {
                        self.word(if negate {
                            "-"
                        } else {
                            "+"
                        });
                    }
                    let Some(value) = self.extra_expr(term, SyntaxKey::Value)? else {
                        return Err(Trap::new("a memory operand term needs a value"));
                    };
                    let scale = self.extra_expr(term, SyntaxKey::Scale)?;
                    terms.push(AsmMemTerm {
                        negate,
                        value,
                        scale,
                    });
                }
                self.word("]");
                Ok(AsmOperand::Mem(AsmMem {
                    terms,
                    broadcast: self.flag(at, SyntaxKey::Broadcast)?,
                    span: self.since(start),
                }))
            }
            _ => Ok(AsmOperand::Value(self.expr(at)?)),
        }
    }

    fn asm_decl(&mut self, at: u64) -> R<AsmDecl> {
        let name = self.attribute(at, SyntaxKey::Name)?;
        let name = self.name(&name);
        let colon = self.flag(at, SyntaxKey::Colon)?;
        let class = self.attribute(at, SyntaxKey::Class)?;
        let class = (!class.is_empty()).then(|| self.name(&class));
        let pin = self.attribute(at, SyntaxKey::Pin)?;
        let pin = if pin.is_empty() {
            None
        } else {
            Some(match pin.parse::<u128>() {
                Ok(index) => {
                    self.word(&pin);
                    AsmPin::Index(index)
                }
                Err(_) => AsmPin::Name(self.name(&pin)),
            })
        };
        Ok(AsmDecl {
            name,
            colon,
            class,
            pin,
        })
    }
}
