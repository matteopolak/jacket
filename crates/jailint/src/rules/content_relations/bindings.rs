use super::domain::*;
use super::*;
use jaic::ide::lifetimes::{LifetimeArgument, LifetimeOperand};

pub(super) struct Bindings<'a> {
    pub c: &'a Compiler,
    pub args: &'a [LifetimeArgument],
    pub state: &'a State,
    pub site: EventSite,
    pub layout: Layout,
}

impl Bindings<'_> {
    fn arg(&self, formal: usize) -> Option<&LifetimeArgument> {
        self.args.iter().find(|a| a.formal == formal && !a.spread)
    }

    pub fn argument(&self, formal: usize) -> Value {
        let Some(arg) = self.arg(formal) else {
            return Value::Unknown;
        };
        if let Some(k) = arg.constant {
            return literal(self.c, arg.ty, k);
        }
        match arg.operand {
            LifetimeOperand::Value(v) => self.state.value(v),
            _ => Value::Unknown,
        }
    }

    fn descriptor(&self, formal: usize) -> Descriptor {
        let Some(arg) = self.arg(formal) else {
            return Descriptor::default();
        };
        if arg.ty != TypeId::STRING || self.args.len() > 1 {
            return Descriptor::default();
        }
        match arg.operand {
            LifetimeOperand::Place(v) | LifetimeOperand::Value(v) => {
                self.state.descriptor(v, self.layout)
            }
            LifetimeOperand::Other => Descriptor::default(),
        }
    }

    pub fn scalar(&self, value: Scalar) -> Scalar {
        match value {
            Scalar::Formal(n) => {
                if let Value::Scalar(s) = self.argument(n) {
                    s
                } else {
                    Scalar::Unknown
                }
            }
            Scalar::StringCount(n) => self.descriptor(n).count,
            other => other,
        }
    }

    pub fn pointer(&self, pointer: &Pointer) -> Option<Pointer> {
        let mut base = match &pointer.base {
            Base::Parameter(n) => {
                if let Value::Pointer(p) = self.argument(*n) {
                    p
                } else {
                    return None;
                }
            }
            Base::InputData(n) => self.descriptor(*n).data?,
            Base::Call(site, index) => Pointer {
                base: Base::Call(prefix(site, self.site)?, *index),
                offset: 0,
            },
            Base::Null => Pointer {
                base: Base::Null,
                offset: 0,
            },
            Base::Slot(_) => return None,
        };
        base.offset = base.offset.checked_add(pointer.offset)?;
        i64::try_from(base.offset).ok()?;
        Some(base)
    }

    pub fn output(&self, descriptor: &Descriptor) -> Descriptor {
        Descriptor {
            count: self.scalar(descriptor.count),
            data: descriptor.data.as_ref().and_then(|p| self.pointer(p)),
        }
    }

    fn length(&self, length: Length) -> Option<Length> {
        match length {
            Length::InputStringCount(n) => {
                super::ranges::length(self.scalar(Scalar::StringCount(n)))
            }
            other @ Length::Constant(_) => Some(other),
        }
    }

    fn range(&self, range: &ByteRange) -> Option<ByteRange> {
        let mut offset = range.offset;
        let base = match &range.base {
            RangeBase::InputStringData(n) => {
                let p = self.descriptor(*n).data?;
                let Base::InputData(n) = p.base else {
                    return None;
                };
                offset = offset.checked_add(p.offset)?;
                RangeBase::InputStringData(n)
            }
            RangeBase::CopyDestination(site) => {
                RangeBase::CopyDestination(prefix(site, self.site)?)
            }
            RangeBase::DescriptorHeader {
                procedure,
                place,
                callers,
            } => {
                if callers.len() >= MAX_DEPTH {
                    return None;
                }
                let mut callers = callers.clone();
                callers.insert(0, self.site);
                RangeBase::DescriptorHeader {
                    procedure: *procedure,
                    place: *place,
                    callers,
                }
            }
        };
        Some(ByteRange {
            base,
            offset,
            length: self.length(range.length)?,
        })
    }

    pub fn fact(&self, fact: &Fact) -> Option<Fact> {
        let mut separations = Vec::new();
        for separation in &fact.separations {
            separations.push(Separation {
                left: self.range(&separation.left)?,
                right: self.range(&separation.right)?,
                store: if let Some(site) = &separation.store {
                    Some(prefix(site, self.site)?)
                } else {
                    None
                },
            });
        }
        Some(Fact {
            destination: self.pointer(&fact.destination)?,
            source: self.pointer(&fact.source)?,
            length: self.scalar(fact.length),
            site: prefix(&fact.site, self.site)?,
            requires_disjoint: fact.requires_disjoint,
            separations,
        })
    }
}

pub(super) fn prefix(site: &Site, caller: EventSite) -> Option<Site> {
    if site.chain.len() >= MAX_DEPTH {
        return None;
    }
    let mut chain = site.chain.clone();
    chain.insert(0, caller);
    Some(Site {
        chain,
    })
}

pub(super) fn literal(c: &Compiler, ty: TypeId, k: u64) -> Value {
    if c.ty.types.pointee(ty).is_some() {
        return if k == 0 {
            Value::Pointer(Pointer {
                base: Base::Null,
                offset: 0,
            })
        } else {
            Value::Unknown
        };
    }
    if ty == TypeId::BOOL {
        return if k <= 1 {
            Value::Scalar(Scalar::Constant(k as i128))
        } else {
            Value::Unknown
        };
    }
    c.ty.types
        .int_info(ty)
        .and_then(|(width, signed)| bits(k, width, signed))
        .map_or(Value::Unknown, |n| Value::Scalar(Scalar::Constant(n)))
}

pub(super) fn bits(k: u64, width: u8, signed: bool) -> Option<i128> {
    if width == 0 || width > 64 {
        return None;
    }
    let raw = k & if width == 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    };
    Some(if signed && raw & (1_u64 << (width - 1)) != 0 {
        raw as i128 - (1_i128 << width)
    } else {
        raw as i128
    })
}
