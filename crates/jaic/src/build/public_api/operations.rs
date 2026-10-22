//! Public compiler operations share the workspace engine and declaration-driven ABI.
use super::abi::{put_string, put_view, string};
use super::{Binding, Operation, Schema};
use crate::{
    build::{self, MetaOp, SharedWorkspaces},
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

    pub fn invoke(&self, interp: &mut Interp, op: MetaOp, args: &[u64]) -> Result<Rets, Trap> {
        build::call(&self.shared, op, false, args, self.program, interp)
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
        CreateWorkspace => cx.invoke(interp, MetaOp::WorkspaceCreate, &[cx.arg(0)]),
        DestroyWorkspace => cx.invoke(interp, MetaOp::DestroyWorkspace, &[cx.workspace(0)? as u64]),
        CurrentWorkspace => cx.invoke(interp, MetaOp::CurrentWorkspace, &[]),
        Name => {
            let id = cx.workspace(0)?;
            let name = cx
                .shared
                .borrow_mut()
                .ws(id)
                .map_err(Trap::new)?
                .name
                .clone();
            put_string(interp, cx.result(0)?.0, name.as_bytes())?;
            Ok(Rets::default())
        }
        GetType => Ok(Rets::one(cx.arg(0))),
        GetOptions | SetOptions | SetOptionsDuringCompile => super::options::call(&cx, interp),
        AddFile => {
            let path = cx.location_path(interp, 2, string(interp, cx.arg(0))?)?;
            let slot = temporary_string(interp, path.as_bytes())?;
            cx.invoke(interp, MetaOp::AddFile, &[cx.workspace(1)? as u64, slot])
        }
        AddString => {
            let ws = cx.workspace(1)? as u64;
            let empty = cx.invoke(interp, MetaOp::CodeIsNull, &[cx.arg(2)])?[0] != 0;
            if empty {
                cx.invoke(interp, MetaOp::AddString, &[ws, cx.arg(0)])
            } else {
                cx.invoke(interp, MetaOp::AddStringToCode, &[ws, cx.arg(0), cx.arg(2)])
            }
        }
        AddStringToModule => super::records::add_string(&cx, interp),
        AddLibraryDirectory => cx.invoke(interp, MetaOp::AddLibraryDir, &[cx.arg(0)]),
        RemapImport => cx.invoke(
            interp,
            MetaOp::RemapImport,
            &[cx.workspace(0)? as u64, cx.arg(1), cx.arg(2), cx.arg(3)],
        ),
        ProvideImport => super::records::provide_import(&cx, interp),
        SetTypeInfoFlags => cx.invoke(interp, MetaOp::SetTypeInfoFlags, &[cx.arg(0), cx.arg(1)]),
        SetWorkspaceStatus => cx.invoke(
            interp,
            MetaOp::WorkspaceStatus,
            &[cx.workspace(1)? as u64, cx.arg(0)],
        ),
        BeginIntercept => {
            let ws = cx.workspace(0)?;
            if ws == cx.shared.borrow().current_id() {
                return Err(Trap::new("a workspace cannot wait for its own compilation"));
            }
            if !interp.public_api_state.intercepted.contains(&ws) {
                interp.public_api_state.intercepted.push(ws);
            }
            interp.public_api_state.finished.retain(|&id| id != ws);
            cx.invoke(interp, MetaOp::BeginIntercept, &[ws as u64, cx.arg(1)])
        }
        EndIntercept => {
            let ws = cx.workspace(0)?;
            interp.public_api_state.intercepted.retain(|&id| id != ws);
            cx.invoke(interp, MetaOp::EndIntercept, &[ws as u64])
        }
        WaitForMessage | CustomLinkComplete => super::messages::call(&cx, interp),
        GetNodes | GetCode | RootType | MakeLive | ModifyProcedure => {
            super::records::call(&cx, interp)
        }
        TypeTable => {
            let ws = cx.workspace(0)? as u64;
            cx.invoke(interp, MetaOp::TypeTable, &[ws, cx.result(0)?.0])
        }
        RuntimeInfo => {
            let ws = cx.workspace(0)? as u64;
            let (at, ty) = cx.result(0)?;
            let table = cx.field(ty, at, "type_table")?.0;
            cx.invoke(interp, MetaOp::TypeTable, &[ws, table])?;
            let data = cx.invoke(interp, MetaOp::RuntimeData, &[ws])?[0];
            cx.set_number(interp, ty, at, "global_data_info", data)?;
            Ok(Rets::default())
        }
        AddDataSegment => {
            let result = cx.invoke(
                interp,
                MetaOp::DataSegment,
                &[cx.arg(0), cx.arg(1), cx.arg(2), cx.workspace(3)? as u64],
            )?[0];
            Ok(vec![result, 0].into())
        }
        AddGlobalData => cx.invoke(
            interp,
            MetaOp::GlobalData,
            &[
                cx.workspace(3)? as u64,
                cx.arg(0),
                cx.arg(1),
                cx.arg(2),
                cx.result(0)?.0,
            ],
        ),
        Report => {
            let values = if binding.parameters.len() == 3 {
                let ty = binding.parameters[1];
                let at = cx.arg(1);
                let file = cx.field(ty, at, "fully_pathed_filename")?.0;
                [
                    cx.arg(0),
                    file,
                    cx.number(interp, ty, at, "line_number")?,
                    cx.number(interp, ty, at, "character_number")?,
                    cx.arg(2),
                ]
            } else {
                [cx.arg(3), cx.arg(0), cx.arg(1), cx.arg(2), cx.arg(4)]
            };
            cx.invoke(interp, MetaOp::Report, &values)
        }
        Version => {
            let result = cx.invoke(interp, MetaOp::CompilerVersion, &[cx.result(0)?.0])?;
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
            Ok(result)
        }
        BasePath => cx.invoke(interp, MetaOp::BasePath, &[cx.result(0)?.0]),
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
            let (at, ty) = cx.result(0)?;
            let file = cx.field(ty, at, "fully_pathed_filename")?.0;
            let line = interp.register_data_global(&[0; 8], 8, false, 1)?;
            let col = interp.register_data_global(&[0; 8], 8, false, 1)?;
            cx.invoke(
                interp,
                MetaOp::StructLocation,
                &[cx.arg(1), file, line, col],
            )?;
            cx.set_number(
                interp,
                ty,
                at,
                "line_number",
                interp.read_u64(line).unwrap_or(0),
            )?;
            cx.set_number(
                interp,
                ty,
                at,
                "character_number",
                interp.read_u64(col).unwrap_or(0),
            )?;
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

pub(super) fn temporary_string(interp: &mut Interp, text: &[u8]) -> Result<u64, Trap> {
    let slot = interp.register_data_global(&[0; 16], 8, false, 1)?;
    put_string(interp, slot, text)?;
    Ok(slot)
}

pub(super) fn put_strings(interp: &mut Interp, at: u64, strings: &[String]) -> Result<(), Trap> {
    let array = interp.register_data_global(&vec![0; strings.len() * 16], 8, false, 1)?;
    for (index, text) in strings.iter().enumerate() {
        put_string(interp, array + index as u64 * 16, text.as_bytes())?;
    }
    put_view(interp, at, strings.len() as u64, array)
}
