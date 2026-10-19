//! Workspace lifecycle, build options and import control.

use super::{
    Call, Interp, MetaOp, Path, PathBuf, ProvidedImport, ProvidedKind, Rets, Trap, Workspace,
    WorkspaceFlag, WorkspaceList, WorkspaceNumber, WorkspaceText, option_snapshot,
};

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let trap = Trap::new;
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let text = |interp: &Interp, i: usize| cx.text(interp, i);
    match op {
        MetaOp::AddLibraryDir => {
            let dir = text(interp, 0)?;
            crate::interp::add_library_dir(PathBuf::from(dir));
            Ok(Rets::default())
        }
        MetaOp::RemapImport => {
            let id = arg(0) as i64;
            let rule = (text(interp, 1)?, text(interp, 2)?, text(interp, 3)?);
            shared.borrow_mut().ws(id).map_err(trap)?.remaps.push(rule);
            Ok(Rets::default())
        }
        MetaOp::ProvideImport => {
            let id = arg(0) as i64;
            let kind = match arg(3) {
                0 => ProvidedKind::ShortName,
                1 => ProvidedKind::PathToFile,
                2 => ProvidedKind::PathToDirectory,
                _ => ProvidedKind::FullText,
            };
            let offer = ProvidedImport {
                host: text(interp, 1)?,
                target: text(interp, 2)?,
                kind,
                value: text(interp, 4)?,
            };
            // An answer already given adds nothing: the load would fail the same way again, and a
            // metaprogram that answers every FAILED_IMPORT with it (a file the host cannot
            // see, say) would loop forever, each retry counting as a new offer.
            let mut reg = shared.borrow_mut();
            let provided = &mut reg.ws(id).map_err(trap)?.provided;
            if !provided.contains(&offer) {
                provided.push(offer);
            }
            Ok(Rets::default())
        }
        MetaOp::MakeLive => {
            let id = arg(0) as i64;
            shared
                .borrow_mut()
                .ws(id)
                .map_err(trap)?
                .live
                .push(arg(1) as i64);
            Ok(Rets::default())
        }
        MetaOp::WorkspaceCreate => {
            let name = text(interp, 0)?;
            let mut reg = shared.borrow_mut();
            let ws = Workspace::new(name);
            reg.list.push(ws);
            Ok(Rets::one(reg.list.len() as u64 - 1))
        }
        MetaOp::CurrentWorkspace => Ok(Rets::one(shared.borrow().current_id() as u64)),
        MetaOp::ValidateWorkspace => {
            shared.borrow_mut().ws(arg(0) as i64).map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::DestroyWorkspace => {
            shared
                .borrow_mut()
                .destroy_workspace(arg(0) as i64)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::SetText => {
            let option = WorkspaceText::from_raw(arg(1))
                .ok_or_else(|| trap("internal: unknown text option".into()))?;
            let value = text(interp, 2)?;
            shared
                .borrow_mut()
                .set_text(arg(0) as i64, option, &value)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::SetFlag => {
            let option = WorkspaceFlag::from_raw(arg(1))
                .ok_or_else(|| trap("internal: unknown flag option".into()))?;
            shared
                .borrow_mut()
                .set_flag(arg(0) as i64, option, arg(2) & 1 != 0)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::SetNumber => {
            let option = WorkspaceNumber::from_raw(arg(1))
                .ok_or_else(|| trap("internal: unknown number option".into()))?;
            shared
                .borrow_mut()
                .set_number(arg(0) as i64, option, arg(2))
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::ClearList => {
            let option = WorkspaceList::from_raw(arg(1))
                .ok_or_else(|| trap("internal: unknown list option".into()))?;
            shared
                .borrow_mut()
                .clear_list(arg(0) as i64, option)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::AppendList => {
            let option = WorkspaceList::from_raw(arg(1))
                .ok_or_else(|| trap("internal: unknown list option".into()))?;
            let mut value = text(interp, 2)?;
            // A module folder relative to a sandbox's own working directory: the process's is
            // somewhere else.
            if option == WorkspaceList::ImportPath
                && Path::new(&value).is_relative()
                && let Some(cwd) = interp.host.working_directory()
            {
                value = Path::new(&cwd).join(&value).to_string_lossy().into_owned();
            }
            shared
                .borrow_mut()
                .append_list(arg(0) as i64, option, &value)
                .map_err(trap)?;
            Ok(Rets::default())
        }
        MetaOp::OptionsSnapshot => {
            let id = arg(0) as i64;
            let size = usize::try_from(arg(2)).map_err(|_| trap("invalid option size".into()))?;
            if arg(5) != 0 {
                let snapshot =
                    option_snapshot::Snapshot::capture(interp, arg(1), size, arg(3), arg(4))?;
                shared.borrow_mut().ws(id).map_err(trap)?.options = Some(snapshot);
                Ok(Rets::one(1))
            } else {
                let snapshot = shared.borrow_mut().ws(id).map_err(trap)?.options.clone();
                if let Some(snapshot) = snapshot {
                    snapshot.restore(interp, arg(1), size)?;
                    Ok(Rets::one(1))
                } else {
                    Ok(Rets::one(0))
                }
            }
        }
        MetaOp::WorkspaceStatus => {
            let mut reg = shared.borrow_mut();
            let ws = reg.ws(arg(0) as i64).map_err(trap)?;
            match arg(1) {
                0 if ws.failed => {
                    return Err(trap("a failed workspace cannot be reset to OK".into()));
                }
                0 => {}
                1 if ws.failed => {}
                1 => {
                    ws.failed = true;
                    // Nothing else says why the build fails, so say it here.
                    (reg.env.report)(
                        super::settings::ReportSeverity::Error,
                        "error: the metaprogram marked the workspace as failed",
                    );
                }
                status => return Err(trap(format!("invalid workspace status {status}"))),
            }
            Ok(Rets::default())
        }
        MetaOp::CustomLinkComplete => {
            let (id, exit_code) = (arg(0) as i64, arg(1) as i64);
            let mut reg = shared.borrow_mut();
            match reg.ws(id).map_err(trap)?.link.as_mut() {
                Some(link) => {
                    link.exit_code = Some(exit_code);
                    Ok(Rets::default())
                }
                None => Err(trap(format!(
                    "compiler_custom_link_command_is_complete: workspace {id} is not waiting \
                     for a custom link (it gets READY_FOR_CUSTOM_LINK_COMMAND first, with \
                     `use_custom_link_command` set)"
                ))),
            }
        }
        _ => unreachable!("not a workspace primitive"),
    }
}
