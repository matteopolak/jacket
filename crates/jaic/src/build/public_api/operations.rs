//! Public compiler operations share the workspace engine and declaration-driven ABI.
use super::abi::{put_string, put_view, string};
use super::{Binding, Operation, Schema};
use crate::{
    build::{SharedWorkspaces, Workspaces, engine},
    fxhash::HashMap,
    interp::{Interp, Rets, Trap},
    ir,
    types::TypeId,
};
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct State {
    pub builder: Option<usize>,
    pub planned: crate::fxhash::HashSet<u64>,
    pub intercepted: Vec<i64>,
    pub cursor: usize,
    pub finished: Vec<i64>,
    pub last_complete: u64,
    pub link_phases: HashMap<i64, (u64, TypeId, Rc<Schema>)>,
    pub schemas: Vec<super::binding::Shared>,
}

pub(super) struct Call<'a> {
    pub binding: &'a Binding,
    pub args: &'a [u64],
    pub program: &'a ir::Program,
    pub shared: SharedWorkspaces,
}

impl Call<'_> {
    pub fn arg(&self, index: usize) -> u64 {
        self.args[index + usize::from(self.binding.context)]
    }

    pub fn schema(&self) -> &Schema {
        &self.binding.schema
    }

    pub fn result(&self, index: usize) -> Result<(u64, TypeId), Trap> {
        let Some(&(ty, memory)) = self.binding.returns.get(index) else {
            return Err(Trap::new("compiler operation result signature mismatch"));
        };
        if !memory {
            return Err(Trap::new("compiler operation requires an aggregate result"));
        }
        let offset = usize::from(self.binding.context)
            + self.binding.parameters.len()
            + self.binding.returns[..index]
                .iter()
                .filter(|(_, memory)| *memory)
                .count();
        Ok((self.args[offset], ty))
    }

    pub fn workspace(&self, index: usize) -> Result<i64, Trap> {
        let id = self.arg(index) as i64;
        let id = if id == -1 || id == 0 {
            self.shared.borrow().current_id()
        } else {
            id
        };
        self.shared.borrow_mut().ws(id).map_err(Trap::new)?;
        Ok(id)
    }

    /// The engine's registry.
    pub fn registry(&self) -> std::cell::RefMut<'_, Workspaces> {
        self.shared.borrow_mut()
    }

    /// The `string` argument `index`.
    pub fn string(&self, interp: &Interp, index: usize) -> Result<String, Trap> {
        string(interp, self.arg(index))
    }

    pub fn field(&self, record: TypeId, address: u64, path: &str) -> Result<(u64, TypeId), Trap> {
        let mut at = address;
        let mut ty = record;
        for name in path.split('.') {
            let (offset, field) = self.schema().field(ty, name)?;
            at += offset;
            ty = field;
        }
        Ok((at, ty))
    }

    pub fn number(
        &self,
        interp: &Interp,
        record: TypeId,
        address: u64,
        path: &str,
    ) -> Result<u64, Trap> {
        let (at, ty) = self.field(record, address, path)?;
        self.schema().read_number(interp, ty, at)
    }

    pub fn set_number(
        &self,
        interp: &mut Interp,
        record: TypeId,
        address: u64,
        path: &str,
        value: u64,
    ) -> Result<(), Trap> {
        let (at, ty) = self.field(record, address, path)?;
        self.schema().write_number(interp, ty, at, value)
    }

    pub fn text(
        &self,
        interp: &Interp,
        record: TypeId,
        address: u64,
        path: &str,
    ) -> Result<String, Trap> {
        string(interp, self.field(record, address, path)?.0)
    }

    pub fn location_path(
        &self,
        interp: &Interp,
        index: usize,
        path: String,
    ) -> Result<String, Trap> {
        if path.is_empty()
            || std::path::Path::new(&path).is_absolute()
            || path.as_bytes().get(1) == Some(&b':')
        {
            return Ok(path);
        }
        let file = self.text(
            interp,
            self.binding.parameters[index],
            self.arg(index),
            "fully_pathed_filename",
        )?;
        Ok(std::path::Path::new(&file)
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .join(path)
            .to_string_lossy()
            .into_owned())
    }
}

pub(crate) fn call(
    binding: &Binding,
    args: &[u64],
    program: &ir::Program,
    interp: &mut Interp,
) -> Result<Rets, Trap> {
    let shared = interp
        .workspaces
        .clone()
        .ok_or_else(|| Trap::new("this build has no compiler workspaces (Compiler module)"))?;
    // Hooks reach the compiler, which only the thread holding the baton may touch.
    #[cfg(not(target_arch = "wasm32"))]
    debug_assert!(
        interp.holds_baton(),
        "{:?} called without the baton",
        binding.operation
    );
    if binding.operation.is_effect() {
        interp.effects += 1;
    }
    let cx = Call {
        binding,
        args,
        program,
        shared,
    };
    use Operation::{
        AddDataSegment, AddFile, AddGlobalData, AddLibraryDirectory, AddString, AddStringToModule,
        BasePath, BeginIntercept, CommandLine, CreateWorkspace, CurrentWorkspace,
        CustomLinkComplete, DestroyWorkspace, DeveloperDebug, EndIntercept, GetCode, GetNodes,
        GetOptions, GetType, MakeLive, MemoryBreakpoint, ModifyProcedure, Name, ProvideImport,
        RemapImport, Report, RootType, RuntimeInfo, SetOptions, SetOptionsDuringCompile,
        SetTypeInfoFlags, SetWorkspaceStatus, StructLocation, TypeTable, Version, WaitForMessage,
    };
    match binding.operation {
        CreateWorkspace => {
            let name = cx.string(interp, 0)?;
            Ok(Rets::one(cx.registry().create_workspace(name) as u64))
        }
        DestroyWorkspace => {
            let id = cx.workspace(0)?;
            cx.registry().destroy_workspace(id).map_err(Trap::new)?;
            Ok(Rets::default())
        }
        CurrentWorkspace => Ok(Rets::one(cx.shared.borrow().current_id() as u64)),
        Name => {
            let id = cx.workspace(0)?;
            let name = cx.registry().workspace(id)?.name.clone();
            put_string(interp, cx.result(0)?.0, name.as_bytes())?;
            Ok(Rets::default())
        }
        GetType => Ok(Rets::one(cx.arg(0))),
        GetOptions | SetOptions | SetOptionsDuringCompile => super::options::call(&cx, interp),
        AddFile => {
            let path = cx.location_path(interp, 2, cx.string(interp, 0)?)?;
            engine::add_source(&cx.shared, interp, cx.workspace(1)?, path, true)?;
            Ok(Rets::default())
        }
        AddString => {
            let ws = cx.workspace(1)?;
            let text = cx.string(interp, 0)?;
            if engine::code_is_null(interp, cx.arg(2)) {
                engine::add_source(&cx.shared, interp, ws, text, false)?;
            } else {
                if cx.arg(2) >= interp.codes.len() as u64 {
                    return Err(Trap::new("add_build_string: invalid Code scope"));
                }
                let code = crate::sema::value::CodeId(cx.arg(2) as u32);
                cx.registry().add_string_to_code(ws, text, code)?;
            }
            Ok(Rets::default())
        }
        AddStringToModule => super::records::add_string(&cx, interp),
        AddLibraryDirectory => {
            crate::interp::add_library_dir(std::path::PathBuf::from(cx.string(interp, 0)?));
            Ok(Rets::default())
        }
        RemapImport => {
            let ws = cx.workspace(0)?;
            let rule = (
                cx.string(interp, 1)?,
                cx.string(interp, 2)?,
                cx.string(interp, 3)?,
            );
            cx.registry().remap_import(ws, rule)?;
            Ok(Rets::default())
        }
        ProvideImport => super::records::provide_import(&cx, interp),
        SetTypeInfoFlags => {
            engine::set_type_info_flags(interp, cx.arg(0), cx.arg(1) as u32)?;
            Ok(Rets::default())
        }
        SetWorkspaceStatus => {
            let ws = cx.workspace(1)?;
            cx.registry().set_status(ws, cx.arg(0))?;
            Ok(Rets::default())
        }
        BeginIntercept => {
            let ws = cx.workspace(0)?;
            if ws == cx.shared.borrow().current_id() {
                return Err(Trap::new("a workspace cannot wait for its own compilation"));
            }
            if !interp.public_api_state.intercepted.contains(&ws) {
                interp.public_api_state.intercepted.push(ws);
            }
            interp.public_api_state.finished.retain(|&id| id != ws);
            cx.registry().begin_intercept(ws, cx.arg(1) as i64)?;
            Ok(Rets::default())
        }
        EndIntercept => {
            let ws = cx.workspace(0)?;
            interp.public_api_state.intercepted.retain(|&id| id != ws);
            cx.registry().end_intercept(ws)?;
            Ok(Rets::default())
        }
        WaitForMessage | CustomLinkComplete => super::messages::call(&cx, interp),
        GetNodes | GetCode | RootType | MakeLive | ModifyProcedure => {
            super::records::call(&cx, interp)
        }
        TypeTable => {
            let (table, count) = engine::type_table(&cx.shared, interp, cx.workspace(0)?)?;
            put_view(interp, cx.result(0)?.0, count, table)?;
            Ok(Rets::default())
        }
        RuntimeInfo => {
            let ws = cx.workspace(0)?;
            let (at, ty) = cx.result(0)?;
            let (table, count) = engine::type_table(&cx.shared, interp, ws)?;
            put_view(interp, cx.field(ty, at, "type_table")?.0, count, table)?;
            let data = engine::runtime_data(&cx.shared, interp, cx.program, ws)?;
            cx.set_number(interp, ty, at, "global_data_info", data)?;
            Ok(Rets::default())
        }
        AddDataSegment => {
            let name = bytes_of(interp, cx.arg(0))?;
            let handle = engine::data_segment(
                &cx.shared,
                interp,
                name,
                cx.arg(1) as u32,
                i64::from(cx.arg(2) as i32),
                cx.workspace(3)?,
            )?;
            Ok(vec![handle, 0].into())
        }
        AddGlobalData => {
            let ws = cx.workspace(3)?;
            let data = bytes_of(interp, cx.arg(0))?;
            let address =
                engine::global_data(&cx.shared, interp, ws, &data, cx.arg(1) as u16, cx.arg(2))?;
            put_view(interp, cx.result(0)?.0, data.len() as u64, address)?;
            Ok(Rets::default())
        }
        Report => {
            let (message, location, mode) = if binding.parameters.len() == 3 {
                let ty = binding.parameters[1];
                let at = cx.arg(1);
                let location = (
                    cx.text(interp, ty, at, "fully_pathed_filename")?,
                    cx.number(interp, ty, at, "line_number")?,
                    cx.number(interp, ty, at, "character_number")?,
                );
                (cx.string(interp, 0)?, location, cx.arg(2))
            } else {
                let location = (cx.string(interp, 0)?, cx.arg(1), cx.arg(2));
                (cx.string(interp, 3)?, location, cx.arg(4))
            };
            engine::report(&cx.shared, &message, location, mode)?;
            Ok(Rets::default())
        }
        Version => {
            let text = crate::build::compiler_version();
            keep_string(&cx, interp, cx.result(0)?.0, text.as_bytes())?;
            if cx.arg(0) != 0 {
                let ty = cx.schema().pointed(binding.parameters[0])?;
                let version = crate::build::LANGUAGE_VERSION;
                for (name, value) in ["major", "minor", "micro"]
                    .into_iter()
                    .zip([version.0, version.1, version.2])
                {
                    cx.set_number(interp, ty, cx.arg(0), name, u64::from(value))?;
                }
            }
            Ok(Rets::default())
        }
        BasePath => {
            let base = cx.shared.borrow().base_path();
            keep_string(&cx, interp, cx.result(0)?.0, base.as_bytes())?;
            Ok(Rets::default())
        }
        CommandLine => {
            let values = cx.shared.borrow().env.command_line.clone();
            put_strings(interp, cx.result(0)?.0, &values)?;
            Ok(Rets::default())
        }
        StructLocation => {
            if cx.workspace(0)? != cx.shared.borrow().current_id() {
                return Err(Trap::new(
                    "foreign workspace struct locations require matching descriptors",
                ));
            }
            // An unknown struct's location is zero.
            let (at, ty) = cx.result(0)?;
            let (line, column) = match engine::struct_location(interp, cx.arg(1)) {
                Some((path, line, column)) => {
                    let file = cx.field(ty, at, "fully_pathed_filename")?.0;
                    keep_string(&cx, interp, file, path.as_bytes())?;
                    (line as u64, column as u64)
                }
                None => (0, 0),
            };
            cx.set_number(interp, ty, at, "line_number", line)?;
            cx.set_number(interp, ty, at, "character_number", column)?;
            Ok(Rets::default())
        }
        MemoryBreakpoint => Err(Trap::new(
            "compiler memory breakpoints are unavailable in this compiler",
        )),
        DeveloperDebug => Err(Trap::new(
            "developer_debug requires compiler debugger support, which is unavailable",
        )),
    }
}

/// The bytes of the `string` at `at`.
fn bytes_of(interp: &Interp, at: u64) -> Result<Vec<u8>, Trap> {
    let (count, data) = super::abi::view(interp, at)?;
    let count = usize::try_from(count).map_err(|_| Trap::new("invalid string size"))?;
    super::abi::bytes(interp, data, count)
}

/// Store `text` as a `string` at `at`, its bytes owned by the registry for the whole build.
fn keep_string(cx: &Call, interp: &mut Interp, at: u64, text: &[u8]) -> Result<(), Trap> {
    let (count, data) = cx.registry().keep_string(text);
    put_view(interp, at, count, data)
}

pub(super) fn put_strings(interp: &mut Interp, at: u64, strings: &[String]) -> Result<(), Trap> {
    let array = interp.register_data_global(&vec![0; strings.len() * 16], 8, false, 1)?;
    for (index, text) in strings.iter().enumerate() {
        put_string(interp, array + index as u64 * 16, text.as_bytes())?;
    }
    put_view(interp, at, strings.len() as u64, array)
}
