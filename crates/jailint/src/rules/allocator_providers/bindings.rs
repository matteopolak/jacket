use super::domain::*;
use super::*;
use jaic::ide::lifetimes::{LifetimeArgument, LifetimeOperand};

pub(super) struct Bindings<'a> {
    pub c: &'a Compiler,
    pub args: &'a [LifetimeArgument],
    pub state: &'a mut State,
    pub call: &'a Site,
    pub summary: &'a Summary,
}

impl Bindings<'_> {
    fn arg(&self, n: usize) -> Option<&LifetimeArgument> {
        self.args.iter().find(|a| a.formal == n && !a.spread)
    }

    pub fn argument(&self, n: usize) -> Value {
        let Some(a) = self.arg(n) else {
            return Value::unknown(TypeId::VOID);
        };
        if let Some(bits) = a.constant {
            return literal(self.c, a.ty, bits);
        }
        match a.operand {
            LifetimeOperand::Value(v) => super::checked::operand(self.c, self.state, v, a.ty),
            _ => Value::unknown(a.ty),
        }
    }

    fn formal(&mut self, n: usize, slot: ByteSlot) -> Value {
        let Some(a) = self.arg(n).cloned() else {
            return Value::unknown(slot.ty);
        };
        if scalar(self.c, a.ty) {
            if slot.offset != 0 || slot.width != self.c.ty.types.size_of(a.ty) {
                return Value::unknown(slot.ty);
            }
            return self.argument(n).retyped(self.c, slot.ty);
        }
        let (LifetimeOperand::Place(v) | LifetimeOperand::Value(v)) = a.operand else {
            return Value::unknown(slot.ty);
        };
        let Some(address) = self.state.address(v) else {
            return Value::unknown(slot.ty);
        };
        // Only a checked entry spill certifies a callee argument copy. An earlier
        // caller spill retains its original field values and read/copy epochs.
        let Some(copy) = self.summary.parameter_copies.get(&n) else {
            return Value::unknown(slot.ty);
        };
        let Some(copy) = prefix(copy, self.call) else {
            return Value::unknown(slot.ty);
        };
        let epoch = ReadEpoch::ArgumentCopy {
            call: copy,
            formal: n,
        };
        self.state.read(self.c, &address, slot, epoch)
    }

    fn epoch(&self, e: &ReadEpoch) -> Option<ReadEpoch> {
        Some(match e {
            ReadEpoch::Load(s) => ReadEpoch::Load(prefix(s, self.call)?),
            ReadEpoch::AggregateCopy(s) => ReadEpoch::AggregateCopy(prefix(s, self.call)?),
            ReadEpoch::ArgumentCopy {
                call,
                formal,
            } => ReadEpoch::ArgumentCopy {
                call: prefix(call, self.call)?,
                formal: *formal,
            },
        })
    }

    fn context_selection(&self, site: &Site) -> Option<Site> {
        if self.summary.initial_context.as_ref() == Some(site) {
            self.state
                .current_context
                .clone()
                .or_else(|| prefix(site, self.call))
        } else {
            prefix(site, self.call)
        }
    }

    fn capture(&mut self, capture: &Captured) -> Captured {
        match capture {
            Captured::FormalValue {
                formal,
                slot,
            } => self.formal(*formal, *slot).capture,
            Captured::ReceiverRead {
                formal,
                receiver_ty: _,
                slot,
                epoch,
            } => {
                let value = self.argument(*formal);
                let Some(address) = value.address else {
                    return Captured::Unknown;
                };
                let Some(projected) =
                    self.state
                        .project(self.c, &address, slot.offset as i128, slot.width)
                else {
                    return Captured::Unknown;
                };
                let Some(epoch) = self.epoch(epoch) else {
                    return Captured::Unknown;
                };
                let slot = ByteSlot {
                    offset: projected.offset,
                    ..*slot
                };
                match projected.root {
                    Root::Receiver {
                        formal,
                        ty,
                    } => Captured::ReceiverRead {
                        formal,
                        receiver_ty: ty,
                        slot,
                        epoch,
                    },
                    Root::Context {
                        selection,
                        ty,
                    } => Captured::ContextRead {
                        context_ty: ty,
                        selection,
                        slot,
                        epoch,
                    },
                    Root::Private(_) => Captured::Unknown,
                }
            }
            Captured::ContextRead {
                context_ty,
                selection,
                slot,
                epoch,
            } => {
                if self.summary.initial_context.as_ref() == Some(selection)
                    && let Some(a) = self.state.current_context_address.as_ref()
                    && self
                        .state
                        .project(self.c, a, slot.offset as i128, slot.width)
                        .is_none()
                {
                    return Captured::Unknown;
                }

                let (Some(selection), Some(epoch)) =
                    (self.context_selection(selection), self.epoch(epoch))
                else {
                    return Captured::Unknown;
                };
                Captured::ContextRead {
                    context_ty: *context_ty,
                    selection,
                    slot: *slot,
                    epoch,
                }
            }
            other => other.clone(),
        }
    }

    fn source(&self, s: &ValueSource) -> ValueSource {
        match s {
            ValueSource::Callback {
                use_site,
                result,
                byte_offset,
            } => prefix(use_site, self.call).map_or(ValueSource::Unknown, |use_site| {
                ValueSource::Callback {
                    use_site,
                    result: *result,
                    byte_offset: *byte_offset,
                }
            }),
            other => other.clone(),
        }
    }

    fn address(&mut self, a: &Address) -> Option<Address> {
        match &a.root {
            Root::Private(_) => None,
            Root::Receiver {
                formal, ..
            } => {
                let base = self.argument(*formal).address?;
                self.state.project(self.c, &base, a.offset as i128, 1)
            }
            Root::Context {
                selection,
                ty,
            } => Some(Address {
                root: Root::Context {
                    selection: self.context_selection(selection)?,
                    ty: *ty,
                },
                offset: a.offset,
            }),
        }
    }

    pub fn value(&mut self, v: &Value) -> Value {
        if let Some((formal, offset)) = v.parameter_pointer {
            return displacement(self.c, self.argument(formal), offset).retyped(self.c, v.ty);
        }
        let mut value = if let Captured::FormalValue {
            formal,
            slot,
        } = v.capture
        {
            self.formal(formal, slot)
        } else {
            Value::captured(v.ty, self.capture(&v.capture))
        };
        value.ty = v.ty;
        if !matches!(v.capture, Captured::FormalValue { .. }) {
            value.source = self.source(&v.source);
            value.address = v.address.as_ref().and_then(|a| self.address(a));
        }
        if let Some(p) = &v.predicate {
            value.predicate = Some(self.predicate(p));
        }
        value
    }

    pub fn predicate(&mut self, p: &Predicate) -> Predicate {
        match p {
            Predicate::Compare {
                value,
                op,
                constant,
                ty,
            } => {
                let value = match value {
                    GuardValue::Captured(Captured::FormalValue {
                        formal,
                        slot,
                    }) => self.formal(*formal, *slot),
                    GuardValue::Captured(c) => {
                        let c = self.capture(c);
                        let ty = match &c {
                            Captured::FormalValue {
                                slot, ..
                            }
                            | Captured::ReceiverRead {
                                slot, ..
                            }
                            | Captured::ContextRead {
                                slot, ..
                            } => slot.ty,
                            Captured::Constant {
                                ty, ..
                            } => *ty,
                            _ => TypeId::VOID_PTR,
                        };
                        Value::captured(ty, c)
                    }
                    GuardValue::Source(s) => Value {
                        source: self.source(s),
                        ..Value::unknown(TypeId::VOID_PTR)
                    },
                    GuardValue::Unknown => Value::unknown(TypeId::VOID),
                };
                compare(self.c, &value, *op, *constant, *ty)
            }
            Predicate::Not(a) => {
                let a = self.predicate(a);
                Predicate::not(&a)
            }
            Predicate::And(a, b) => {
                let a = self.predicate(a);
                let b = self.predicate(b);
                Predicate::and(&a, &b)
            }
            Predicate::Or(a, b) => {
                let a = self.predicate(a);
                let b = self.predicate(b);
                Predicate::or(&a, &b)
            }
            Predicate::Unknown => Predicate::Unknown,
        }
    }

    pub fn guards(&mut self, g: &[Guard]) -> Vec<Guard> {
        g.iter()
            .map(|g| Guard {
                predicate: self.predicate(&g.predicate),
                truth: g.truth,
            })
            .collect()
    }

    pub fn use_(&mut self, u: &Use) -> Option<Use> {
        let site = prefix(&u.row.site, self.call)?;
        let operation_value = self.value(&u.operation);
        let procedure = self.value(&u.procedure);
        let data = self.value(&u.data);
        let mut guards = self.state.guards.clone();
        guards.extend(self.guards(&u.row.guards));
        if guards.len() > 32
            || guards.iter().any(|g| {
                self.state
                    .simplified(&g.predicate)
                    .known()
                    .is_some_and(|k| k != g.truth)
            })
        {
            return None;
        }
        Some(Use {
            row: ProviderUse {
                site,
                operation: operation(self.c, &operation_value, u.protocol.as_ref()),
                provider: Pair {
                    procedure_value: procedure.capture.clone(),
                    data_value: data.capture.clone(),
                },
                guards,
            },
            operation: operation_value,
            procedure,
            data,
            protocol: u.protocol.clone(),
        })
    }

    pub fn store(&mut self, s: &StoreSource) -> Option<StoreSource> {
        let address = self.argument(s.receiver_formal).address?;
        let address = self
            .state
            .project(self.c, &address, s.slot.offset as i128, s.slot.width)?;
        let Root::Receiver {
            formal,
            ty,
        } = address.root
        else {
            return None;
        };
        Some(StoreSource {
            site: prefix(&s.site, self.call)?,
            receiver_formal: formal,
            receiver_ty: ty,
            slot: ByteSlot {
                offset: address.offset,
                ..s.slot
            },
            source: self.source(&s.source),
            guards: {
                let mut guards = self.state.guards.clone();
                guards.extend(self.guards(&s.guards));
                if guards.len() > 32
                    || guards.iter().any(|g| {
                        self.state
                            .simplified(&g.predicate)
                            .known()
                            .is_some_and(|k| k != g.truth)
                    })
                {
                    return None;
                }
                guards
            },
        })
    }
}
