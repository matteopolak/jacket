use super::{
    Address, ByteRange, CmpOp, Compiler, Expr, HandleEffect, MAX_DEPTH, Operand, Site, Slot, State,
    Ty, TypeId, TypeKind, Val,
};

pub(super) fn reverse(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::SLt => CmpOp::SGt,
        CmpOp::SLe => CmpOp::SGe,
        CmpOp::SGt => CmpOp::SLt,
        CmpOp::SGe => CmpOp::SLe,
        CmpOp::ULt => CmpOp::UGt,
        CmpOp::ULe => CmpOp::UGe,
        CmpOp::UGt => CmpOp::ULt,
        CmpOp::UGe => CmpOp::ULe,
        x => x,
    }
}

pub(super) fn scalar(c: &Compiler, ty: TypeId) -> bool {
    matches!(
        c.ty.types.kind(c.ty.types.repr(ty)),
        TypeKind::Pointer(_)
            | TypeKind::Int { .. }
            | TypeKind::Bool
            | TypeKind::Null
            | TypeKind::Enum(_)
    )
}

pub(super) fn leaves(c: &Compiler, ty: TypeId) -> Vec<Slot> {
    fn add(c: &Compiler, ty: TypeId, offset: u64, depth: usize, out: &mut Vec<Slot>) {
        if depth > MAX_DEPTH || out.len() > 64 {
            return;
        }
        if scalar(c, ty) {
            out.push(Slot {
                offset,
                width: c.ty.types.size_of(ty),
                ty,
            });
        } else if let Some(st) = c.ty.types.as_struct(ty) {
            let info = c.ty.types.struct_info(st);
            if info.is_union || info.fields.iter().any(|f| f.overlay) {
                return;
            }
            for f in info.fields.iter() {
                if let Some(offset) = offset.checked_add(f.offset) {
                    add(c, f.ty, offset, depth + 1, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    add(c, ty, 0, 0, &mut out);
    out
}

impl State {
    pub(super) fn original(&self, root: Val) -> bool {
        self.roots
            .as_ref()
            .is_some_and(|roots| roots.extent(root).is_some())
    }

    pub(super) fn extent(&self, root: Val) -> Option<u64> {
        // An unknown original bound must never acquire a borrowed-view size.
        self.roots
            .as_ref()
            .and_then(|roots| roots.extent(root))
            .unwrap_or_else(|| self.borrowed.get(&root).copied())
    }

    pub(super) fn checked(&self, a: Address, width: u64) -> bool {
        width > 0
            && ByteRange {
                offset: a.offset,
                width,
            }
            .within(self.extent(a.root))
    }

    pub(super) fn region(&mut self, v: Val, width: u64) {
        let a = self.address(v);
        if !self.original(a.root) {
            self.borrowed.entry(a.root).or_insert(width);
        }
        self.addresses.entry(v).or_insert(a);
    }

    pub(super) fn address(&self, v: Val) -> Address {
        self.addresses.get(&v).copied().unwrap_or(Address {
            root: v,
            offset: 0,
        })
    }

    pub(super) fn value(&self, v: Val) -> Expr {
        self.values.get(&v).cloned().unwrap_or(Expr::Unknown)
    }

    pub(super) fn get(&self, v: Val, slot: Slot) -> Expr {
        let mut a = self.address(v);
        let Some(offset) = a.offset.checked_add(slot.offset) else {
            return Expr::Unknown;
        };
        a.offset = offset;
        if !self.checked(a, slot.width) {
            return Expr::Unknown;
        }
        self.slots
            .get(&(
                a,
                Slot {
                    offset: 0,
                    ..slot
                },
            ))
            .cloned()
            .unwrap_or(Expr::Unknown)
    }

    pub(super) fn put(&mut self, v: Val, slot: Slot, x: Expr) {
        let mut a = self.address(v);
        let Some(offset) = a.offset.checked_add(slot.offset) else {
            return;
        };
        a.offset = offset;
        if !self.checked(a, slot.width) {
            return;
        }
        self.clear_address(a, slot.width);
        self.slots.insert(
            (
                a,
                Slot {
                    offset: 0,
                    ..slot
                },
            ),
            x,
        );
    }

    pub(super) fn clear_address(&mut self, a: Address, width: u64) {
        self.slots.retain(|(b, s), _| {
            b.root != a.root
                || ByteRange {
                    offset: b.offset,
                    width: s.width,
                }
                .separated(ByteRange {
                    offset: a.offset,
                    width,
                })
        });
    }

    pub(super) fn clear(&mut self, v: Val, width: u64) {
        self.clear_address(self.address(v), width);
    }

    pub(super) fn copy_slots(&mut self, to: Val, from: Val, width: Option<u64>) {
        let a = self.address(from);
        let supported = width
            .is_some_and(|width| self.checked(a, width) && self.checked(self.address(to), width));
        let entries: Vec<_> = self
            .slots
            .iter()
            .filter(|((b, slot), _)| {
                supported
                    && b.root == a.root
                    && b.offset >= a.offset
                    && width.is_none_or(|w| {
                        b.offset
                            .checked_add(slot.width)
                            .is_some_and(|end| end <= a.offset.saturating_add(w))
                    })
            })
            .map(|((b, slot), x)| {
                (
                    Slot {
                        offset: b.offset - a.offset,
                        ..*slot
                    },
                    x.clone(),
                )
            })
            .collect();
        if let Some(width) = width {
            self.clear(to, width);
        } else {
            let root = self.address(to).root;
            self.slots.retain(|(a, _), _| a.root != root);
        }
        for (slot, x) in entries {
            self.put(to, slot, x);
        }
    }

    pub(super) fn opaque(&mut self, args: &[jaic::ide::lifetimes::LifetimeArgument]) {
        self.interferes = true;
        for arg in args {
            let (Operand::Value(val) | Operand::Place(val)) = arg.operand else {
                continue;
            };
            let x = self.value(val);
            if let Expr::Fresh(t) = &x {
                self.invalid.insert(t.clone());
            }
            if related(&x) {
                self.uncertain.push(HandleEffect {
                    value: x,
                    kind: None,
                });
            }
            let a = self.address(val);
            if self.addresses.contains_key(&val) {
                self.exposed.insert(a.root);
            }
            let affected: Vec<_> = self
                .slots
                .iter()
                .filter(|((b, _), _)| b.root == a.root)
                .map(|(_, x)| x.clone())
                .collect();
            for x in affected {
                if let Expr::Fresh(t) = &x {
                    self.invalid.insert(t.clone());
                }
                if related(&x) {
                    self.uncertain.push(HandleEffect {
                        value: x,
                        kind: None,
                    });
                }
            }
            self.slots.retain(|(b, _), _| b.root != a.root);
        }
        // Escaped storage may be overwritten without its address being passed again.
        // Immutable SSA copies remain separate and require native liveness frames.
        self.slots
            .retain(|(a, _), _| !self.exposed.contains(&a.root));
    }
}

pub(super) fn substitute(
    c: &Compiler,
    x: &Expr,
    args: &[jaic::ide::lifetimes::LifetimeArgument],
    s: &State,
    site: &Site,
) -> Expr {
    match x {
        Expr::Input(formal, slot) => {
            args.iter()
                .find(|a| a.formal == *formal)
                .map_or(Expr::Unknown, |a| {
                    if let Some(k) = a.constant {
                        let k = if let Some((bits, signed)) = c.ty.types.int_info(a.ty) {
                            if bits == 0 || bits > 64 {
                                return Expr::Unknown;
                            }
                            let raw = k & if bits == 64 {
                                u64::MAX
                            } else {
                                (1_u64 << bits) - 1
                            };
                            if signed && raw & (1_u64 << (bits - 1)) != 0 {
                                (raw as i128 - (1_i128 << bits)) as i64 as u64
                            } else {
                                raw
                            }
                        } else {
                            k
                        };
                        return Expr::Constant(k);
                    }
                    // Multi-argument aggregate evaluation needs a distinct captured copy.
                    // The sidecar can reuse an aggregate address SSA, so a later argument
                    // may have overwritten its source before this call event.
                    if !scalar(c, a.ty) && args.len() > 1 {
                        return Expr::Unknown;
                    }
                    match a.operand {
                        Operand::Value(v) => {
                            if slot.offset == 0 && scalar_expr(&s.value(v)) {
                                s.value(v)
                            } else {
                                s.get(v, *slot)
                            }
                        }
                        Operand::Place(v) => s.get(v, *slot),
                        Operand::Other => Expr::Unknown,
                    }
                })
        }
        Expr::Fresh(token) => {
            let mut token = token.clone();
            if token.sites.len() >= MAX_DEPTH {
                return Expr::Unknown;
            }
            token.sites.insert(0, site.clone());
            Expr::Fresh(token)
        }
        Expr::Compare(a, op, k, ty) => {
            Expr::compare(Some(&substitute(c, a, args, s, site)), *op, *k, *ty)
        }
        Expr::Not(a) => Expr::not(Some(&substitute(c, a, args, s, site))),
        Expr::And(a, b) => {
            let left = substitute(c, a, args, s, site);
            let right = substitute(c, b, args, s, site);
            Expr::and(Some(&left), Some(&right))
        }
        Expr::Or(a, b) => {
            let left = substitute(c, a, args, s, site);
            let right = substitute(c, b, args, s, site);
            Expr::or(Some(&left), Some(&right))
        }
        x => x.clone(),
    }
}

fn scalar_expr(x: &Expr) -> bool {
    !matches!(x, Expr::Unknown)
}

/// Normalize both operands in the checked comparison's bit width, then extend
/// according to its signed predicate. Equality compares the same masked bits.
pub(super) fn comparison_constant(ty: Ty, op: CmpOp, k: u64) -> Option<u64> {
    let width = match ty {
        Ty::I8 => 8,
        Ty::I16 => 16,
        Ty::I32 => 32,
        Ty::I64 | Ty::Ptr => 64,
        _ => return None,
    };
    let mask = u64::MAX >> (64 - width);
    let bits = k & mask;
    let signed = matches!(op, CmpOp::SLt | CmpOp::SLe | CmpOp::SGt | CmpOp::SGe);
    Some(if signed {
        ((bits << (64 - width)) as i64 >> (64 - width)) as u64
    } else {
        bits
    })
}

pub(super) fn related(x: &Expr) -> bool {
    match x {
        Expr::Input(..) | Expr::Fresh(_) => true,
        Expr::Compare(a, ..) | Expr::Not(a) => related(a),
        Expr::And(a, b) | Expr::Or(a, b) => related(a) || related(b),
        _ => false,
    }
}
