//! Laying a struct out: offsets, alignment and `using` clashes.
use super::model::{
    FieldDecl, FieldItem, FieldUsing, LayoutItem, OffsetNode, PathStep, StructSource, UsingFilters,
};
use crate::sema::{
    Compiler, Diagnostic, HashMap, Rc, Result, ScopeId, Span, Sym, TypeId, ast, err, scope,
};
use crate::types::{Field, LayoutState, MAX_SIZE, StructId, TypeKind};

fn solve_offsets(nodes: &[OffsetNode]) -> Result<Vec<u64>> {
    let mut state = vec![0u8; nodes.len()];
    let mut values = vec![0; nodes.len()];
    let mut stack = Vec::new();
    for root in 0..nodes.len() {
        if state[root] == 2 {
            continue;
        }
        stack.push(root);
        while let Some(&at) = stack.last() {
            state[at] = 1;
            let node = nodes[at];
            let base = if let Some(dep) = node.dependency {
                match state[dep] {
                    0 => {
                        stack.push(dep);
                        continue;
                    }
                    1 => {
                        return Err(Box::new(
                            Diagnostic::error(node.span, "cycle in struct field layout").with_note(
                                nodes[dep].span,
                                "this storage offset depends on itself",
                            ),
                        ));
                    }
                    _ => values[dep],
                }
            } else {
                0u64
            };
            values[at] = base
                .checked_add(node.addend)
                .and_then(|n| n.checked_next_multiple_of(node.align))
                .filter(|&n| n <= MAX_SIZE)
                .ok_or_else(|| {
                    Box::new(Diagnostic::error(
                        node.span,
                        format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
                    ))
                })?;
            state[at] = 2;
            stack.pop();
        }
    }
    Ok(values)
}

impl Compiler {
    pub fn layout_struct(&mut self, s: StructId, span: Span) -> Result<()> {
        self.begin_runtime_storage_dependencies();
        let result = self.layout_struct_with_runtime_dependencies(s, span);
        let values = self.end_runtime_storage_dependencies()?;
        if result.is_ok() {
            self.finish_runtime_storage_layout(s, &values);
        }
        result
    }

    fn layout_struct_with_runtime_dependencies(&mut self, s: StructId, span: Span) -> Result<()> {
        match self.ty.types.struct_info(s).layout {
            LayoutState::Done => return Ok(()),
            LayoutState::InProgress => {
                self.epochs.in_progress_misses += 1;
                let name = self.ty.types.struct_info(s).name;
                return err(
                    span,
                    format!("struct `{name}` contains itself (use a pointer)"),
                );
            }
            LayoutState::Pending => {}
        }
        crate::sema::body_probe::bump(crate::sema::body_probe::C::Layout);
        self.ty.types.struct_info_mut(s).layout = LayoutState::InProgress;
        let result = self.layout_struct_inner(s);
        if result.is_err() {
            self.ty.types.struct_info_mut(s).layout = LayoutState::Pending;
        }
        result
    }

    /// The members of a struct in declaration order: its fields, aliases and layout directives,
    /// with the field declarations of `using` members resolved.
    fn struct_field_items(&mut self, src: &StructSource) -> Result<Vec<FieldItem>> {
        let mut items = Vec::new();
        self.ty.field_types.remove(&src.scope);
        self.ty.field_using_order.remove(&src.scope);
        if let Some(ast::UnionTag {
            name: tag,
            ty,
            value,
        }) = &src.lit.tag
        {
            // `union tag: T { ... }`: the tag field, then an anonymous union of the members.
            let tag_ty = match (ty, value) {
                (Some(ty), _) => self.eval_type(src.scope, ty)?,
                (None, Some(value)) => self.eval_const(src.scope, value, None)?.ty(),
                (None, None) => return err(tag.span, "union tag needs a type"),
            };
            // `.A ,, a: int;`: each member's tag value names a member of the tag's enum.
            if self.ed.is_live() {
                for stmt in &src.lit.body {
                    if let ast::StmtKind::Decl(d) = &stmt.kind
                        && let Some(t) = &d.union_tag()
                        && let ast::ExprKind::InferredMember(m) = &t.kind
                    {
                        self.ide_note_member_use(m.span, tag_ty, m.name);
                        self.ide_note_member_ref(m.span, m.name, tag_ty);
                    }
                }
            }
            let mut members = (*src.lit).clone();
            members.id = ast::AstId::fresh();
            members.tag = None;
            members.params = Vec::new();
            members.notes = Vec::new();
            let members = self.new_struct_type(
                Sym::intern("anonymous"),
                Rc::new(members),
                src.scope,
                Vec::new(),
                None,
            );
            let tag_init = value.as_deref().cloned();
            for (name, ty, using, init) in [
                (Some(tag.name), tag_ty, false, tag_init),
                (None, members, true, None),
            ] {
                items.push(FieldItem::Field(FieldDecl {
                    name,
                    ty,
                    using,
                    as_: false,
                    notes: Vec::new(),
                    span: tag.span,
                    init,
                    scope: src.scope,
                    align: None,
                    filter: None,
                }));
            }
        } else {
            self.collect_fields(src.scope, &src.lit.body, &mut items)?;
        }
        for (decl, scope) in &src.extra {
            self.ty.collect_decl_fields(*scope, decl, &mut items)?;
        }
        let mut using_order = Vec::new();
        for item in &mut items {
            match item {
                FieldItem::Declaration(scope, decl) if decl.using => {
                    using_order.push(FieldUsing::Declaration(*scope, decl.clone()));
                }
                FieldItem::Field(d) if d.using => {
                    let filter = match d.filter.as_ref() {
                        Some(filter) => match self.filter_using_entry(
                            d.scope,
                            scope::UsingEntry::Type(d.ty),
                            filter,
                            d.span,
                        )? {
                            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
                            _ => filter.clone(),
                        },
                        None => ast::UsingFilter::None,
                    };
                    d.filter = Some(filter.clone());
                    using_order.push(FieldUsing::Inline(d.ty, filter));
                }
                _ => {}
            }
        }
        self.ty.field_using_order.insert(src.scope, using_order);
        let mut typed_items = Vec::with_capacity(items.len());
        for item in items {
            match item {
                FieldItem::Declaration(scope, decl) => {
                    self.resolve_decl_fields(scope, &decl, &mut typed_items)?;
                }
                other => typed_items.push(other),
            }
        }
        Ok(typed_items)
    }

    fn layout_struct_inner(&mut self, s: StructId) -> Result<()> {
        let src = self
            .ty
            .struct_asts
            .get(&s)
            .cloned()
            .expect("struct without source");
        let items = self.struct_field_items(&src)?;
        let is_union = src.lit.kind == ast::StructKind::Union && src.lit.tag.is_none();
        let no_padding = src.lit.flags.has(ast::StructBits::NO_PADDING);
        let mut fields = Vec::new();
        let mut inits = Vec::new();
        let mut end = 0u64;
        let mut align = 1u64;
        let mut aliases = Vec::new();
        let mut filters = UsingFilters::default();
        let mut sizes = Vec::new();
        let mut layout = Vec::new();
        let mut names = HashMap::default();
        for item in items {
            match item {
                FieldItem::Declaration(..) => unreachable!("field declarations were resolved"),
                FieldItem::Alias(alias) => aliases.push(alias),
                FieldItem::Place(name, span) => layout.push(LayoutItem::Place(name, span)),
                FieldItem::Overlay(name, span) => layout.push(LayoutItem::Overlay(name, span)),
                FieldItem::Field(d) => {
                    self.ensure_storage_type(d.ty, d.span)?;
                    let size = self.size_of(d.ty, d.span)?;
                    let a = match d.align {
                        Some(n) if n > 0 => n,
                        _ => self.align_of(d.ty, d.span)?,
                    };
                    align = align.max(a);
                    let index = fields.len();
                    if let Some(name) = d.name
                        && names.insert(name, index).is_some()
                    {
                        return err(
                            d.span,
                            format!("member `{name}` is declared more than once"),
                        );
                    }
                    if d.using
                        && let Some(filter) = d.filter.clone()
                    {
                        let resolved = self.filter_using_entry(
                            d.scope,
                            scope::UsingEntry::Type(d.ty),
                            &filter,
                            d.span,
                        )?;
                        let filter = match resolved {
                            scope::UsingEntry::Filtered(_, filter) => (*filter).clone(),
                            _ => filter,
                        };
                        filters.push(index, filter);
                    }
                    fields.push(Field {
                        name: d.name,
                        ty: d.ty,
                        offset: 0,
                        align: a,
                        using: d.using,
                        as_: d.as_,
                        overlay: false,
                        notes: d.notes,
                        span: d.span,
                    });
                    self.callable_fields.remove(&(s, index));
                    if matches!(self.ty.types.kind(d.ty), TypeKind::Proc(_))
                        && let Some(declaration) = d
                            .name
                            .and_then(|name| self.ty.field_declarations.get(&d.scope)?.get(&name))
                            .cloned()
                        && let Some(parameters) = self.callable_expression_parameters(
                            d.scope,
                            declaration.ty.as_ref().or(declaration.value.as_ref()),
                        )?
                    {
                        self.callable_fields.insert((s, index), parameters);
                    }
                    sizes.push(size);
                    inits.push((d.init, d.scope));
                    layout.push(LayoutItem::Field(index));
                }
            }
        }
        // Node zero is the initial cursor, nodes 1..=fields.len() are field offsets.
        let zero = OffsetNode {
            dependency: None,
            addend: 0,
            align: 1,
            span: src.lit.span,
        };
        let mut nodes = vec![zero; fields.len() + 1];
        let mut cursor = 0;
        let mut overlay = None;
        let mut targets = HashMap::default();
        for item in layout {
            match item {
                LayoutItem::Field(index) => {
                    let field = &mut fields[index];
                    let overlaid = overlay.take();
                    field.overlay = overlaid.is_some();
                    nodes[index + 1] = OffsetNode {
                        dependency: Some(overlaid.unwrap_or(if is_union {
                            0
                        } else {
                            cursor
                        })),
                        addend: 0,
                        align: if field.overlay || no_padding || is_union {
                            1
                        } else {
                            field.align
                        },
                        span: field.span,
                    };
                    if !field.overlay && !is_union {
                        cursor = nodes.len();
                        nodes.push(OffsetNode {
                            dependency: Some(index + 1),
                            addend: sizes[index],
                            align: 1,
                            span: field.span,
                        });
                    }
                }
                LayoutItem::Place(name, span) | LayoutItem::Overlay(name, span) => {
                    let (index, relative) = if let Some(&target) = targets.get(&name) {
                        target
                    } else {
                        let target =
                            self.layout_anchor(s, &fields, &filters, &names, name, span)?;
                        targets.insert(name, target);
                        target
                    };
                    let node = if relative == 0 {
                        index + 1
                    } else {
                        let node = nodes.len();
                        nodes.push(OffsetNode {
                            dependency: Some(index + 1),
                            addend: relative,
                            align: 1,
                            span,
                        });
                        node
                    };
                    if let Some(&index) = names.get(&name) {
                        self.ide_note_named_field(span, name, fields[index].span, fields[index].ty);
                    }
                    if matches!(item, LayoutItem::Place(..)) {
                        cursor = node;
                    } else {
                        overlay = Some(node);
                    }
                }
            }
        }
        if overlay.is_some() {
            return err(src.lit.span, "#overlay needs a following stored member");
        }
        let offsets = solve_offsets(&nodes)?;
        for (index, field) in fields.iter_mut().enumerate() {
            field.offset = offsets[index + 1];
            let extent = field
                .offset
                .checked_add(sizes[index])
                .filter(|&n| n <= MAX_SIZE)
                .ok_or_else(|| {
                    Box::new(Diagnostic::error(
                        field.span,
                        format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
                    ))
                })?;
            end = end.max(extent);
        }
        if let Some(a) = &src.lit.flags.align {
            let a = self.eval_align(src.scope, a)?;
            align = align.max(a);
        }
        let size = if no_padding {
            end
        } else {
            end.next_multiple_of(align.max(1))
        };
        if size > MAX_SIZE {
            return err(
                src.lit.span,
                format!("struct is too large (the limit is {MAX_SIZE} bytes)"),
            );
        }
        self.check_using_clashes(&fields, &filters)?;
        let info = self.ty.types.struct_info_mut(s);
        info.fields = fields.into();
        if !filters.is_empty() {
            self.ty.using_filters.insert(s, filters);
        }
        if !aliases.is_empty() {
            self.ty.member_aliases.insert(s, aliases);
        }
        info.size = size;
        info.align = align;
        info.layout = LayoutState::Done;
        self.ty.struct_asts.get_mut(&s).unwrap().inits = inits;
        if self.ed.is_live() {
            self.ide_note_layout(s);
        }
        Ok(())
    }

    /// Resolve a storage anchor against all declarations, without needing outer offsets yet.
    fn layout_anchor(
        &mut self,
        owner: StructId,
        fields: &[Field],
        filters: &UsingFilters,
        names: &HashMap<Sym, usize>,
        name: Sym,
        span: Span,
    ) -> Result<(usize, u64)> {
        if let Some(&index) = names.get(&name) {
            return Ok((index, 0));
        }
        for (index, field) in fields.iter().enumerate().filter(|(_, f)| f.using) {
            let Some(target) = filters.target(index, name) else {
                continue;
            };
            if let Some((path, _, _)) =
                self.find_promoted_member(vec![(field.ty, 0, target)], span, &mut vec![owner])?
            {
                let mut relative = 0u64;
                for step in path {
                    match step {
                        PathStep::Offset(at) => {
                            relative = relative
                                .checked_add(at)
                                .filter(|&n| n <= MAX_SIZE)
                                .ok_or_else(|| {
                                    Box::new(Diagnostic::error(
                                        span,
                                        "overlay target offset is too large",
                                    ))
                                })?;
                        }
                        PathStep::Deref => {
                            return err(
                                span,
                                format!(
                                    "`{name}` is behind a using pointer and has no inline storage offset"
                                ),
                            );
                        }
                    }
                }
                return Ok((index, relative));
            }
        }
        err(span, format!("no field named `{name}` in this struct"))
    }

    /// Two `using` members that bring in the same name are an error, as in Jai. A field of the
    /// struct itself may repeat a promoted name (it shadows it, {#using.15}). Only field names
    /// are compared; the constants and procedures declared in the members' bodies are not.
    fn check_using_clashes(&mut self, fields: &[Field], filters: &UsingFilters) -> Result<()> {
        if !fields.iter().any(|f| f.using) {
            return Ok(());
        }
        // Where each name came from: the `using` member that brought it in. A field of the
        // struct itself may share the name: the direct member shadows the promoted ones.
        let mut seen: HashMap<Sym, (Span, Option<Sym>)> = HashMap::default();
        for (i, f) in fields.iter().enumerate() {
            if !f.using {
                continue;
            }
            let filter = filters.get(i);
            let mut names = Vec::new();
            if !self.exposed_names(f.ty, filter, &mut Vec::new(), &mut names) {
                continue;
            }
            for name in names {
                match seen.get(&name) {
                    Some(&(first, via)) if first != f.span => {
                        let how = match via {
                            Some(m) => format!("through `using {m}`"),
                            None => "through an earlier `using` member".to_string(),
                        };
                        let member = f.name.map_or("this `using` member".to_string(), |n| {
                            format!("`using {n}`")
                        });
                        return Err(Box::new(
                            Diagnostic::error(
                                f.span,
                                format!("the name `{name}` is brought in twice"),
                            )
                            .with_label(format!("{member} also brings in `{name}`"))
                            .with_note(first, format!("`{name}` is already in this struct, {how}"))
                            .with_help("a `using,only(...)` or `using,except(...)` can leave one of them out"),
                        ));
                    }
                    Some(_) => {}
                    None => {
                        seen.insert(name, (f.span, f.name));
                    }
                }
            }
        }
        Ok(())
    }

    /// The field names a `using` member of type `ty` brings in (through nested `using`s too).
    /// False when they cannot be told: a filter that renames, or a type not laid out yet.
    fn exposed_names(
        &self,
        ty: TypeId,
        filter: Option<&ast::UsingFilter>,
        searched: &mut Vec<StructId>,
        out: &mut Vec<Sym>,
    ) -> bool {
        let ty = self.ty.types.repr_struct(ty);
        let ty = self.ty.types.pointee(ty).unwrap_or(ty);
        let Some(s) = self.ty.types.as_struct(self.ty.types.repr_struct(ty)) else {
            return true;
        };
        let info = self.ty.types.struct_info(s);
        if searched.contains(&s) || info.layout != LayoutState::Done {
            return !searched.is_empty();
        }
        searched.push(s);
        let own_filters = self.ty.using_filters.get(&s);
        let mut names = Vec::new();
        for (i, f) in info.fields.iter().enumerate() {
            names.extend(f.name);
            if f.using {
                let inner = own_filters.and_then(|l| l.get(i));
                if !self.exposed_names(f.ty, inner, searched, &mut names) {
                    return false;
                }
            }
        }
        match filter {
            None | Some(ast::UsingFilter::None) => {}
            Some(ast::UsingFilter::Only(keep)) => {
                names.retain(|n| keep.iter().any(|k| k.name == *n));
            }
            Some(ast::UsingFilter::Except(drop)) => {
                names.retain(|n| drop.iter().all(|k| k.name != *n));
            }
            Some(_) => return false,
        }
        out.extend(names);
        true
    }

    /// An `#align N` value. Generated bindings use odd values (`#align 9`), so only the range is
    /// checked; the bound keeps layout arithmetic from overflowing.
    pub(super) fn eval_align(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<u64> {
        const MAX_ALIGN: i128 = 1 << 30;
        let n = self.eval_int(scope, expr)?;
        if !(0..=MAX_ALIGN).contains(&n) {
            return err(
                expr.span,
                format!("alignment {n} is out of range (0 to {MAX_ALIGN})"),
            );
        }
        Ok(n as u64)
    }
}
