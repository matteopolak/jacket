//! What the public `Compiler` operations do to the workspace registry, with typed arguments.
//!
//! `public_api` reads each operation's arguments through the caller's declarations and calls
//! these; nothing here knows a Jai layout.

use super::state::{INTERCEPT_REPORT_POLYMORPHS, INTERCEPT_REPORT_RUNS};
use super::{
    DataSegment, Event, Interp, ProgramSource, ProvidedImport, ReportSeverity, SharedWorkspaces,
    Stage, TOP_LEVEL_WORKSPACE, Trap, Workspace, Workspaces, option_snapshot::Snapshot, step,
};
use crate::records::Item;
use crate::sema::value::CodeId;
use std::path::{Path, PathBuf};

fn trap(message: String) -> Trap {
    Trap::new(message)
}

impl Workspaces {
    pub(super) fn workspace(&mut self, id: i64) -> Result<&mut Workspace, Trap> {
        self.ws(id).map_err(trap)
    }

    pub(super) fn create_workspace(&mut self, name: String) -> i64 {
        self.list.push(Workspace::new(name));
        self.list.len() as i64 - 1
    }

    /// `remap_import`: (host module, imported name, replacement).
    pub(super) fn remap_import(
        &mut self,
        id: i64,
        rule: (String, String, String),
    ) -> Result<(), Trap> {
        self.workspace(id)?.remaps.push(rule);
        Ok(())
    }

    pub(super) fn provide_import(&mut self, id: i64, offer: ProvidedImport) -> Result<(), Trap> {
        // An answer already given adds nothing: the load would fail the same way again, and a
        // metaprogram that answers every FAILED_IMPORT with it (a file the host cannot see,
        // say) would loop forever, each retry counting as a new offer.
        let provided = &mut self.workspace(id)?.provided;
        if !provided.contains(&offer) {
            provided.push(offer);
        }
        Ok(())
    }

    pub(super) fn make_live(&mut self, id: i64, record: i64) -> Result<(), Trap> {
        self.workspace(id)?.live.push(record);
        Ok(())
    }

    /// `compiler_set_workspace_status`: 0 is OK, 1 failed.
    pub(super) fn set_status(&mut self, id: i64, status: u64) -> Result<(), Trap> {
        let ws = self.workspace(id)?;
        match status {
            0 if ws.failed => Err(trap("a failed workspace cannot be reset to OK".into())),
            0 => Ok(()),
            1 if ws.failed => Ok(()),
            1 => {
                ws.failed = true;
                // Nothing else says why the build fails, so say it here.
                (self.env.report)(
                    ReportSeverity::Error,
                    "error: the metaprogram marked the workspace as failed",
                );
                Ok(())
            }
            status => Err(trap(format!("invalid workspace status {status}"))),
        }
    }

    pub(super) fn complete_custom_link(&mut self, id: i64, exit_code: i64) -> Result<(), Trap> {
        match self.workspace(id)?.link.as_mut() {
            Some(link) => {
                link.exit_code = Some(exit_code);
                Ok(())
            }
            None => Err(trap(format!(
                "compiler_custom_link_command_is_complete: workspace {id} is not waiting for a \
                 custom link (it gets READY_FOR_CUSTOM_LINK_COMMAND first, with \
                 `use_custom_link_command` set)"
            ))),
        }
    }

    pub(super) fn begin_intercept(&mut self, id: i64, flags: i64) -> Result<(), Trap> {
        if !crate::platform::HAS_CLOCK
            && flags & (INTERCEPT_REPORT_RUNS | INTERCEPT_REPORT_POLYMORPHS) != 0
        {
            return Err(trap(
                "performance reporting requires a host monotonic clock; this WASM compiler host \
                 does not provide one"
                    .into(),
            ));
        }
        let ws = self.workspace(id)?;
        ws.intercepted = true;
        ws.intercept_flags = flags;
        Ok(())
    }

    pub(super) fn end_intercept(&mut self, id: i64) -> Result<(), Trap> {
        self.workspace(id)?.intercepted = false;
        Ok(())
    }

    /// The install root: the directory above the stdlib, which is the last module directory on
    /// the import path.
    pub(super) fn base_path(&self) -> String {
        let Some(root) = self
            .env
            .options
            .import_paths
            .last()
            .and_then(|stdlib| stdlib.parent())
        else {
            return String::new();
        };
        let mut text = crate::canonicalize(root)
            .unwrap_or_else(|_| root.to_path_buf())
            .display()
            .to_string();
        if !text.ends_with('/') && !text.ends_with('\\') {
            text.push('/');
        }
        text
    }

    /// `add_build_string` scoped by a FILE or IMPORT message: the record it was built from.
    pub(super) fn add_string_to_module(
        &mut self,
        id: i64,
        value: String,
        record: i64,
    ) -> Result<(), Trap> {
        let source = if let Some(Item::Int(file)) = self.records.item(record, "__file", None) {
            ProgramSource::FileString(value, crate::source::FileId(file as u32))
        } else {
            // A FILE message stands for its module (the enclosing import).
            let message = match self.records.item(record, "enclosing_import", None) {
                Some(Item::Ref(import)) => import,
                _ => record,
            };
            let Some(Item::Int(module)) = self.records.item(message, "__module", None) else {
                return Err(trap("add_build_string: the message names no module".into()));
            };
            ProgramSource::ModuleString(value, module as u32)
        };
        self.workspace(id)?.queue_source(source).map_err(trap)
    }

    /// `add_build_string` scoped by a `Code` of the running workspace.
    pub(super) fn add_string_to_code(
        &mut self,
        id: i64,
        source: String,
        code: CodeId,
    ) -> Result<(), Trap> {
        if id != self.current_id() {
            return Err(trap(
                "add_build_string: Code scope must belong to the running workspace".into(),
            ));
        }
        self.workspace(id)?
            .queue_source(ProgramSource::CodeString(source, code))
            .map_err(trap)
    }

    /// Fail unless `id` is the workspace whose compile-time code is running.
    fn running(&mut self, id: i64, what: &str) -> Result<(), Trap> {
        self.workspace(id)?;
        if id == self.current_id() {
            Ok(())
        } else {
            Err(trap(format!("{what} requires the running workspace")))
        }
    }

    pub(super) fn options_snapshot(&mut self, id: i64) -> Result<Option<Snapshot>, Trap> {
        Ok(self.workspace(id)?.options.clone())
    }

    pub(super) fn save_options(&mut self, id: i64, snapshot: Snapshot) -> Result<(), Trap> {
        self.workspace(id)?.options = Some(snapshot);
        Ok(())
    }
}

/// `add_build_file` (`file`) or `add_build_string`: queue a source for workspace `id`.
pub(super) fn add_source(
    shared: &SharedWorkspaces,
    interp: &Interp,
    id: i64,
    value: String,
    file: bool,
) -> Result<(), Trap> {
    let mut reg = shared.borrow_mut();
    let ws = reg.workspace(id)?;
    if ws.stage == Stage::Done {
        return Err(trap(format!(
            "workspace `{}` has already been compiled; add its sources before it completes",
            ws.name
        )));
    }
    let source = if file {
        let fs = reg.env.fs.clone();
        // Resolved now: the metaprogram may change directory before it compiles.
        let cwd = interp.host.working_directory().map_or_else(
            || std::env::current_dir().unwrap_or_default(),
            PathBuf::from,
        );
        let path = fs.canonical(&cwd.join(&value));
        if !fs.is_file(&path) {
            return Err(trap(if fs.is_dir(&path) {
                format!("add_build_file: `{value}` is a directory, not a .jai file")
            } else {
                format!(
                    "add_build_file: file `{value}` does not exist (relative paths start from the \
                     metaprogram's directory, {})",
                    cwd.display()
                )
            }));
        }
        ProgramSource::File(path)
    } else {
        ProgramSource::String(value)
    };
    reg.workspace(id)?
        .queue_source(source.clone())
        .map_err(trap)?;
    let observer = reg.env.observer.take();
    drop(reg);
    if let Some(mut o) = observer {
        o.source_added(id, &source);
        shared.borrow_mut().env.observer = Some(o);
    }
    Ok(())
}

/// A module folder relative to a sandbox's own working directory: the process's is elsewhere.
pub(super) fn import_folder(interp: &Interp, value: String) -> String {
    match interp.host.working_directory() {
        Some(cwd) if Path::new(&value).is_relative() => {
            Path::new(&cwd).join(&value).to_string_lossy().into_owned()
        }
        _ => value,
    }
}

/// The next message of intercepted workspace `id`, compiling it until it has one; `None` once
/// it is done, or for a workspace that would wait on its own compilation.
pub(super) fn next_event(
    shared: &SharedWorkspaces,
    interp: &mut Interp,
    id: i64,
) -> Result<Option<Event>, Trap> {
    if id == shared.borrow().current_id() || id == TOP_LEVEL_WORKSPACE {
        return Ok(None);
    }
    loop {
        let mut reg = shared.borrow_mut();
        let ws = reg.workspace(id)?;
        if let Some(event) = ws.events.pop_front() {
            return Ok(Some(event));
        }
        if ws.stage == Stage::Done {
            return Ok(None);
        }
        drop(reg);
        let mut budget = interp.block_budget;
        let stepped = step(shared, id, &mut budget);
        interp.block_budget = budget;
        stepped.map_err(trap)?;
    }
}

/// `Report` in the Compiler module: `ERROR` (stops the metaprogram), `ERROR_CONTINUABLE`,
/// `WARNING` or `INFO`, at `(file, line, column)` when `file` is not empty.
pub(super) fn report(
    shared: &SharedWorkspaces,
    message: &str,
    (file, line, column): (String, u64, u64),
    mode: u64,
) -> Result<(), Trap> {
    let message = message.trim_end();
    if mode == 0 {
        let mut t = trap(message.to_string());
        t.reported = Some(Box::new((file, line as u32, column as u32)));
        return Err(t);
    }
    let (severity, label) = match mode {
        1 => (ReportSeverity::Error, "error"),
        2 => (ReportSeverity::Warning, "warning"),
        _ => (ReportSeverity::Info, "info"),
    };
    let location = if file.is_empty() {
        String::new()
    } else {
        format!("{file}:{line}:{column}: ")
    };
    let mut reg = shared.borrow_mut();
    (reg.env.report)(severity, &format!("{location}{label}: {message}"));
    if mode == 1 {
        // A continuable error: the metaprogram goes on, but the build has failed.
        let current = reg.current_id();
        reg.workspace(current)?.failed = true;
    }
    Ok(())
}

/// Whether `Code` value `code` is `null`.
pub(super) fn code_is_null(interp: &Interp, code: u64) -> bool {
    interp.codes.get(code as usize).is_some_and(|body| {
        matches!(&**body, crate::ast::CodeBody::Expr(e) if matches!(e.kind, crate::ast::ExprKind::Null))
    })
}

/// `compiler_get_nodes`: the root record of `Code` value `code` and the records of all its
/// expressions.
pub(super) fn code_nodes(
    shared: &SharedWorkspaces,
    interp: &mut Interp,
    code: u64,
) -> Result<(i64, Vec<i64>), Trap> {
    let code = code as usize;
    let Some(body) = interp.codes.get(code).cloned() else {
        return Err(trap("compiler_get_nodes: not a Code value".into()));
    };
    interp.nodes_codes.retain(|&c| c != code);
    interp.nodes_codes.push(code);
    let taken = interp.code_export_cursor.entry(code).or_default();
    if let Some((root, nodes)) = interp.code_exports.get(&code).and_then(|e| e.get(*taken)) {
        *taken += 1;
        return Ok((*root, nodes.clone()));
    }
    // Names and types need the compiler: while nothing observable happened in this
    // compile-time run, ask it to export the code and run again (`call_thunk`).
    if interp.run_effects == Some(interp.effects) {
        interp.export_request = Some(code);
        return Err(trap("compiler_get_nodes: typed export requested".into()));
    }
    let mut reg = shared.borrow_mut();
    Ok(crate::sema::code_export::export_code(
        &mut reg.records,
        &body,
    ))
}

/// `get_type_table`: (address, count) of the descriptors of every type of workspace `id`'s
/// program so far. The running workspace's table was made when its compile-time run began;
/// another workspace's compiler is between steps, so it makes one now.
pub(super) fn type_table(
    shared: &SharedWorkspaces,
    interp: &Interp,
    id: i64,
) -> Result<(u64, u64), Trap> {
    if id < 0 || id == shared.borrow().current_id() {
        return Ok(interp.ct_type_table);
    }
    let taken = shared.borrow_mut().workspace(id)?.compiler.take();
    let Some(mut compiler) = taken else {
        return Err(trap(
            "get_type_table: that workspace has nothing compiled yet".into(),
        ));
    };
    shared.borrow_mut().current.push(id);
    compiler.refresh_type_table();
    shared.borrow_mut().current.pop();
    let found = compiler.ct.interp.ct_type_table;
    shared.borrow_mut().workspace(id)?.compiler = Some(compiler);
    Ok(found)
}

/// `compiler_set_type_info_flags`: applied by the compiler after the running `#run` returns.
pub(super) fn set_type_info_flags(interp: &mut Interp, ty: u64, flags: u32) -> Result<(), Trap> {
    let Some((global, 0)) = interp.global_at(ty) else {
        return Err(trap("compiler_set_type_info_flags: not a type".into()));
    };
    interp.pending_type_flags.push((global, flags));
    Ok(())
}

/// Where the struct whose `Type_Info_Struct` is at `info` was declared: (path, line, column).
pub(super) fn struct_location(interp: &Interp, info: u64) -> Option<(std::rc::Rc<str>, i64, i64)> {
    let (global, 0) = interp.global_at(info)? else {
        return None;
    };
    interp.struct_locations.get(&global).cloned()
}

/// `get_runtime_info`: a snapshot of the running workspace's global data.
pub(super) fn runtime_data(
    shared: &SharedWorkspaces,
    interp: &mut Interp,
    program: &crate::ir::Program,
    id: i64,
) -> Result<u64, Trap> {
    shared
        .borrow_mut()
        .running(id, "get_runtime_info: a global-data snapshot")?;
    interp.runtime_data_snapshot(program)
}

/// `add_data_segment`: a handle for a user segment of the running workspace.
pub(super) fn data_segment(
    shared: &SharedWorkspaces,
    interp: &mut Interp,
    name: Vec<u8>,
    characteristics: u32,
    alignment: i64,
    workspace: i64,
) -> Result<u64, Trap> {
    let mut reg = shared.borrow_mut();
    reg.running(workspace, "add_data_segment: registration")?;
    if characteristics & !0xf != 0 || characteristics & 4 != 0 {
        return Err(trap(
            "add_data_segment: executable or unknown segment characteristics are unavailable"
                .into(),
        ));
    }
    if alignment <= 0 || !(alignment as u64).is_power_of_two() {
        return Err(trap(
            "add_data_segment: alignment must be a positive power of two".into(),
        ));
    }
    // Opaque handles must survive #run freezing and relocation without being copied to an
    // unrelated address. The interpreter adopts this allocation as a real compiler-owned
    // global after the run.
    let handle = interp.register_data_global(&[0; 16], 16, true, 2)?;
    reg.data_segments.push(DataSegment {
        handle,
        _name: name,
        workspace,
        characteristics,
        alignment: alignment as u64,
    });
    Ok(handle)
}

/// `add_global_data`: the address of `bytes` placed in `segment` (a user segment's `handle`)
/// of the running workspace.
pub(super) fn global_data(
    shared: &SharedWorkspaces,
    interp: &mut Interp,
    workspace: i64,
    bytes: &[u8],
    segment: u16,
    handle: u64,
) -> Result<u64, Trap> {
    let mut reg = shared.borrow_mut();
    reg.running(workspace, "add_global_data: registration")?;
    let (align, read_only, zeroed, tag) = match segment {
        0 => (16, false, false, 1),
        2 => (16, true, false, 2),
        3 => (16, false, true, 0),
        0x10 => {
            let descriptor = reg
                .data_segments
                .iter()
                .find(|s| s.handle == handle && s.workspace == workspace)
                .ok_or_else(|| {
                    trap("add_global_data: invalid user segment handle for this workspace".into())
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
    interp.register_data_global(bytes, align, read_only, tag)
}
