//! Queries and registrations on the program being compiled: type tables, struct locations, data segments.

use super::{Call, DataSegment, Interp, MetaOp, Rets, Trap, store, store_pair};

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let trap = Trap::new;
    let shared = cx.shared;
    let program = cx.program;
    let arg = |i: usize| cx.arg(i);
    let string = |interp: &Interp, i: usize| cx.string(interp, i);
    let return_string =
        |interp: &mut Interp, bytes: &[u8], i: usize| cx.return_string(interp, bytes, i);
    match op {
        MetaOp::SetTypeInfoFlags => {
            // (type: Type, flags): applied by the compiler after the running `#run` returns.
            let Some((global, 0)) = interp.global_at(arg(0)) else {
                return Err(trap("compiler_set_type_info_flags: not a type".into()));
            };
            interp.pending_type_flags.push((global, arg(1) as u32));
            Ok(Rets::default())
        }
        MetaOp::TypeTable => {
            // (workspace, out table): the descriptors of every type of the workspace's program
            // so far. The running workspace's table was made when its compile-time run began;
            // another workspace's compiler is between steps, so it makes one now.
            let id = arg(0) as i64;
            let current = shared.borrow().current_id();
            let (table, count) = if id < 0 || id == current {
                interp.ct_type_table
            } else {
                let taken = shared.borrow_mut().ws(id).map_err(trap)?.compiler.take();
                let Some(mut compiler) = taken else {
                    return Err(trap(
                        "get_type_table: that workspace has nothing compiled yet".into(),
                    ));
                };
                shared.borrow_mut().current.push(id);
                compiler.refresh_type_table();
                shared.borrow_mut().current.pop();
                let found = compiler.ct.interp.ct_type_table;
                shared.borrow_mut().ws(id).map_err(trap)?.compiler = Some(compiler);
                found
            };
            store_pair(interp, arg(1), count, table)?;
            Ok(Rets::default())
        }
        MetaOp::StructLocation => {
            // (info, out path, out line, out column); unknown structs leave them zero.
            let found = interp
                .global_at(arg(0))
                .and_then(|(g, offset)| (offset == 0).then_some(g))
                .and_then(|g| interp.struct_locations.get(&g).cloned());
            if let Some((path, line, col)) = found {
                return_string(interp, path.as_bytes(), 1)?;
                store(interp, arg(2), &line.to_le_bytes())?;
                store(interp, arg(3), &col.to_le_bytes())?;
            }
            Ok(Rets::default())
        }
        MetaOp::RuntimeData => {
            let id = arg(0) as i64;
            let mut reg = shared.borrow_mut();
            reg.ws(id).map_err(trap)?;
            if id != reg.current_id() {
                return Err(trap(
                    "get_runtime_info: global-data snapshots require the running workspace".into(),
                ));
            }
            Ok(Rets::one(interp.runtime_data_snapshot(program)?))
        }
        MetaOp::DataSegment => {
            let name = string(interp, 0)?;
            let characteristics = arg(1) as u32;
            let alignment = arg(2) as i32;
            let workspace = arg(3) as i64;
            let mut reg = shared.borrow_mut();
            reg.ws(workspace).map_err(trap)?;
            if workspace != reg.current_id() {
                return Err(trap(
                    "add_data_segment: registration requires the running workspace".into(),
                ));
            }
            if characteristics & !0xf != 0 || characteristics & 4 != 0 {
                return Err(trap("add_data_segment: executable or unknown segment characteristics are unavailable".into()));
            }
            if alignment <= 0 || !(alignment as u32).is_power_of_two() {
                return Err(trap(
                    "add_data_segment: alignment must be a positive power of two".into(),
                ));
            }
            let segment = DataSegment {
                // Opaque handles must survive #run freezing and relocation without
                // being copied to an unrelated address. The interpreter adopts this
                // allocation as a real compiler-owned global after the run.
                handle: interp.register_data_global(&[0; 16], 16, true, 2)?,
                _name: name,
                workspace,
                characteristics,
                alignment: alignment as u64,
            };
            let address = segment.handle;
            reg.data_segments.push(segment);
            Ok(Rets::one(address))
        }
        MetaOp::GlobalData => {
            let workspace = arg(0) as i64;
            let bytes = string(interp, 1)?;
            let segment = arg(2) as u16;
            let mut reg = shared.borrow_mut();
            reg.ws(workspace).map_err(trap)?;
            if workspace != reg.current_id() {
                return Err(trap(
                    "add_global_data: registration requires the running workspace".into(),
                ));
            }
            let (align, read_only, zeroed, tag) = match segment {
                0 => (16, false, false, 1),
                2 => (16, true, false, 2),
                3 => (16, false, true, 0),
                0x10 => {
                    let handle = arg(3);
                    let descriptor = reg
                        .data_segments
                        .iter()
                        .find(|s| s.handle == handle && s.workspace == workspace)
                        .ok_or_else(|| {
                            trap(
                                "add_global_data: invalid user segment handle for this workspace"
                                    .into(),
                            )
                        })?;
                    (
                        descriptor.alignment,
                        descriptor.characteristics & 2 == 0,
                        descriptor.characteristics & 8 != 0,
                        5,
                    )
                }
                1 => {
                    return Err(trap(
                        "add_global_data: WRITABLE_NO_RESET reset semantics are unavailable".into(),
                    ));
                }
                _ => return Err(trap("add_global_data: unknown segment index".into())),
            };
            if zeroed && bytes.iter().any(|&b| b != 0) {
                return Err(trap(
                    "add_global_data: nonzero input for BSS or ZEROED is not supported".into(),
                ));
            }
            let address = interp.register_data_global(&bytes, align, read_only, tag)?;
            store_pair(interp, arg(4), bytes.len() as u64, address)?;
            Ok(Rets::default())
        }
        _ => unreachable!("not a program_data primitive"),
    }
}
