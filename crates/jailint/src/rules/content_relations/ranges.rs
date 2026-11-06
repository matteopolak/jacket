use super::domain::*;
use super::*;

pub(super) fn length(s: Scalar) -> Option<Length> {
    match s {
        Scalar::StringCount(n) => Some(Length::InputStringCount(n)),
        Scalar::Constant(k) if k >= 0 => Some(Length::Constant(k)),
        _ => None,
    }
}

pub(super) fn copy_range(site: &Site, length: Length) -> ByteRange {
    ByteRange {
        base: RangeBase::CopyDestination(site.clone()),
        offset: 0,
        length,
    }
}

fn header_range(procedure: ProcId, address: Address) -> ByteRange {
    ByteRange {
        base: RangeBase::DescriptorHeader {
            procedure,
            place: address.root,
            callers: Vec::new(),
        },
        offset: address.offset as i128,
        length: Length::Constant(jaic::layout::view::SIZE as i128),
    }
}

pub(super) fn frame_requirements(
    state: &State,
    procedure: ProcId,
    written: ByteRange,
    store: Option<Site>,
) -> Vec<Separation> {
    let mut headers: Vec<_> = state.headers.iter().copied().collect();
    headers.sort_unstable_by_key(|a| (a.root.0, a.offset));
    headers
        .into_iter()
        .map(|a| Separation {
            left: header_range(procedure, a),
            right: written.clone(),
            store: store.clone(),
        })
        .collect()
}

pub(super) fn disjoint(a: &Pointer, width: Scalar, b: &Pointer, length: Scalar) -> Option<bool> {
    if matches!(width, Scalar::Constant(k) if k < 0)
        || matches!(length, Scalar::Constant(k) if k < 0)
    {
        return None;
    }
    if width == Scalar::Constant(0) || length == Scalar::Constant(0) {
        return Some(true);
    }
    if a.base != b.base {
        return None;
    }
    // A mathematical interval proof is valid for the supported 64-bit pointer
    // circle only within one signed displacement and bounded nonnegative spans.
    i64::try_from(a.offset.checked_sub(b.offset)?).ok()?;
    if !matches!(width, Scalar::Constant(k) if (0..=i64::MAX as i128).contains(&k))
        || !matches!(
            length,
            Scalar::StringCount(_) | Scalar::Constant(0..=0x7fff_ffff_ffff_ffff)
        )
    {
        return None;
    }
    if let Scalar::Constant(width) = width
        && a.offset
            .checked_add(width)
            .is_some_and(|end| end <= b.offset)
    {
        return Some(true);
    }
    if let Scalar::Constant(length) = length
        && b.offset
            .checked_add(length)
            .is_some_and(|end| end <= a.offset)
    {
        return Some(true);
    }
    Some(false)
}

/// Resolved storage alone does not exclude an opaque destination alias.
pub(super) fn private_write(
    state: &mut State,
    address: Address,
    width: u64,
    site: &Site,
    procedure: ProcId,
) {
    if state.owned.contains(&address.root) && !state.exposed.contains(&address.root) {
        return;
    }
    let header = state.headers.iter().copied().find(|h| {
        h.root == address.root
            && h.offset <= address.offset
            && address
                .offset
                .checked_add(width)
                .zip(h.offset.checked_add(jaic::layout::view::SIZE))
                .is_some_and(|(end, header_end)| end <= header_end)
    });
    let Some(header) = header else {
        state.copies.clear();
        return;
    };
    // Caller-visible return headers are not private. Preserve only conditionally
    // on their actual storage being disjoint from the captured destination.
    for fact in &mut state.copies {
        let Some(len) = length(fact.length) else {
            continue;
        };
        let separation = Separation {
            left: header_range(procedure, header),
            right: copy_range(&fact.site, len),
            store: Some(site.clone()),
        };
        if !fact.separations.contains(&separation) {
            fact.separations.push(separation);
        }
    }
}

/// Preserve a snapshot only with a proved or exported actual write/destination separation.
pub(super) fn write(
    state: &mut State,
    pointer: Option<Pointer>,
    width: u64,
    site: &Site,
    procedure: ProcId,
) -> bool {
    let Some(pointer) = pointer else {
        state.opaque(&[]);
        return true;
    };
    if let Base::Slot(root) = pointer.base {
        let Ok(offset) = u64::try_from(pointer.offset) else {
            state.opaque(&[]);
            return true;
        };
        let address = Address {
            root,
            offset,
        };
        if !state.checked(address, width) {
            state.opaque(&[]);
            return true;
        }
        private_write(state, address, width, site, procedure);
        state.clear(address, Some(width));
        return state.bounded();
    }
    let mut kept = Vec::new();
    for mut fact in state.copies.drain(..) {
        match disjoint(
            &pointer,
            Scalar::Constant(width as i128),
            &fact.destination,
            fact.length,
        ) {
            Some(true) => kept.push(fact),
            Some(false) => {}
            None => {
                let Base::InputData(formal) = pointer.base else {
                    continue;
                };
                let Some(len) = length(fact.length) else {
                    continue;
                };
                fact.separations.push(Separation {
                    left: ByteRange {
                        base: RangeBase::InputStringData(formal),
                        offset: pointer.offset,
                        length: Length::Constant(width as i128),
                    },
                    right: copy_range(&fact.site, len),
                    store: Some(site.clone()),
                });
                kept.push(fact);
            }
        }
    }
    state.copies = kept;
    if let Base::InputData(formal) = pointer.base {
        let range = ByteRange {
            base: RangeBase::InputStringData(formal),
            offset: pointer.offset,
            length: Length::Constant(width as i128),
        };
        let frames = frame_requirements(state, procedure, range, Some(site.clone()));
        for fact in &mut state.copies {
            fact.separations.extend(frames.clone());
        }
    } else {
        state
            .slots
            .retain(|(a, _), _| state.owned.contains(&a.root) && !state.exposed.contains(&a.root));
    }
    state.invalidate_ambiguous();
    state.bounded()
}

pub(super) fn record(
    state: &mut State,
    destination: Pointer,
    source: Pointer,
    count: Scalar,
    site: Site,
    requires_disjoint: bool,
    procedure: ProcId,
) -> bool {
    // A newer write may destroy an earlier snapshot, including overlap with another copy.
    let mut kept = Vec::new();
    for fact in state.copies.drain(..) {
        if disjoint(&destination, count, &fact.destination, fact.length) == Some(true) {
            kept.push(fact);
        }
    }
    state.copies = kept;
    if let Base::Slot(root) = destination.base
        && let Ok(offset) = u64::try_from(destination.offset)
    {
        state.clear(
            Address {
                root,
                offset,
            },
            if let Scalar::Constant(k) = count {
                u64::try_from(k).ok()
            } else {
                None
            },
        );
    }
    state.invalidate_ambiguous();
    let Some(len) = length(count) else {
        return true;
    };
    if !matches!(source.base, Base::InputData(_)) {
        return true;
    }
    let separations = frame_requirements(state, procedure, copy_range(&site, len), None);
    state.copies.push(Fact {
        destination,
        source,
        length: count,
        site,
        requires_disjoint,
        separations,
    });
    state.bounded()
}

pub(super) fn returned(
    summary: &Summary,
    positions: &[usize],
    layout: Layout,
) -> Option<ReturnedCopy> {
    if summary.exits.is_empty() {
        return None;
    }
    let mut result = ReturnedCopy::default();
    for position in positions {
        let mut alternatives = Vec::new();
        let mut complete = true;
        for exit in &summary.exits {
            let Some(descriptor) = exit.returns.get(position) else {
                complete = false;
                break;
            };
            let Some(data) = &descriptor.data else {
                complete = false;
                break;
            };
            let Some(fact) = exit
                .copies
                .iter()
                .find(|f| &f.destination == data && f.length == descriptor.count)
            else {
                complete = false;
                break;
            };
            let Base::InputData(formal) = fact.source.base else {
                complete = false;
                break;
            };
            if fact.source.offset != 0 {
                complete = false;
                break;
            }
            let Some(len) = length(fact.length) else {
                complete = false;
                break;
            };
            if len != Length::InputStringCount(formal) && len != Length::Constant(0) {
                complete = false;
                break;
            }
            // Returning a private callee stack buffer does not establish a usable string.
            if matches!(data.base, Base::Slot(_)) {
                complete = false;
                break;
            }
            let readable = ByteRange {
                base: RangeBase::InputStringData(formal),
                offset: fact.source.offset,
                length: len,
            };
            let writable = copy_range(&fact.site, len);
            let mut separations = fact.separations.clone();
            if fact.requires_disjoint {
                separations.push(Separation {
                    left: readable.clone(),
                    right: writable.clone(),
                    store: None,
                });
            }
            let relation = CopyRelation {
                source_formal: formal,
                result: *position,
                source_data: layout.data,
                source_count: layout.count,
                result_data: layout.data,
                result_count: layout.count,
                source_offset: fact.source.offset,
                length: len,
                read_epoch: fact.site.clone(),
                write_epoch: fact.site.clone(),
                preconditions: Preconditions {
                    nonnegative_length: true,
                    readable,
                    writable,
                    separations,
                },
            };
            if !alternatives.contains(&relation) {
                alternatives.push(relation);
            }
        }
        if complete {
            result.alternatives.extend(alternatives);
        } else {
            result.incomplete = true;
        }
    }
    (!result.alternatives.is_empty()).then_some(result)
}
