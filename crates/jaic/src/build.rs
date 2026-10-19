//! Workspaces: the compiler side of the `Compiler` module.
//!
//! A metaprogram creates workspaces, adds files and strings to them, sets
//! their build options and (optionally) intercepts their compiler messages.
//! Each workspace is compiled by a separate [`Compiler`] sharing the file
//! system; its output (executable, library, object) is written by the
//! embedder's [`OutputBackend`] (LLVM in `jaic-cli`, none in the browser).
//!
//! Jai code reaches this module only through a few `#compiler` primitives
//! (`__jaic_*`, see [`MetaOp`]) with a scalar/string ABI; the public
//! `Compiler` module API (`Build_Options`, Message structs...) is written in Jai
//! on top of them in `stdlib/Compiler/module.jai`.
use crate::builtin_keys::{
    SyntaxKey, WorkspaceFlag, WorkspaceList, WorkspaceNumber, WorkspaceText,
};
use crate::interp::{Interp, Rets, Trap};
use crate::records::{FieldRef, Item};
use crate::sema::{ProgramSource, ProvidedImport, ProvidedKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod calls;
mod drive;
mod host;
mod messages;
mod meta_op;
mod option_snapshot;
mod output;
mod plugin_program;
mod program_data;
mod records;
mod registry;
mod settings;
mod sources;
mod state;
mod workspace;

pub use plugin_program::{PluginImport, PluginProgram, plugin_metaprogram};
pub use sources::parse_code_text;

pub use self::calls::call;
use self::calls::{Call, store, store_pair};
use self::drive::step;
pub use self::meta_op::{LANGUAGE_VERSION, MetaOp, TOP_LEVEL_WORKSPACE, compiler_version};
pub use self::output::finish_all;
pub use self::registry::{
    WASI_RUNTIME_IMPORT, dead_code_setting, import_rules, take_live_requests, take_own_sources,
    wants_wasi_runtime,
};
pub(crate) use self::registry::{isolated_host_environment, runtime_storage_setting};
pub use self::settings::{
    BuildEnv, BuildSettings, LinkObjects, OutputBackend, ReportSeverity, ReportSink,
    UnwrittenOutputHint, WorkspaceObserver,
};
use self::state::{DataSegment, Stage, Workspace};
pub use self::state::{
    EVENT_COMPLETE, EVENT_DEBUG_DUMP, EVENT_ERROR, EVENT_FAILED_IMPORT, EVENT_FILE, EVENT_IMPORT,
    EVENT_PERFORMANCE_REPORT, EVENT_PHASE, EVENT_TYPECHECKED, Event, SharedWorkspaces, Workspaces,
};
pub use crate::builtin_keys::OutputType;
