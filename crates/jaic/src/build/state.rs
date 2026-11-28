//! Workspace state, the events a metaprogram receives and the registry's shared data.
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use super::option_snapshot;
use super::settings::{BuildEnv, BuildSettings, PendingLink};
use crate::records::Records;
use crate::sema::{Compiler, ProgramSource, ProvidedImport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    /// Collecting files/strings/options; nothing compiled yet.
    Open,
    /// Sources loaded and their compile-time code run; more sources may be
    /// added (after `TYPECHECKED_ALL_WE_CAN`) before code generation.
    Checked,
    /// Output written (or failed); only queued events remain.
    Done,
}

pub(super) struct Workspace {
    pub(super) name: String,
    /// Sources added and not yet loaded into the workspace's compiler.
    pub(super) pending: Vec<ProgramSource>,
    pub(super) settings: BuildSettings,
    pub(super) options: Option<option_snapshot::Snapshot>,
    pub(super) intercepted: bool,
    /// The `Intercept_Flags` the intercepting metaprogram passed to `compiler_begin_intercept`.
    pub(super) intercept_flags: i64,
    pub(super) stage: Stage,
    /// The workspace's compiler between steps (taken out while it runs).
    pub(super) compiler: Option<Box<Compiler>>,
    pub(super) events: VecDeque<Event>,
    pub(super) failed: bool,
    /// `compiler_modify_procedure` calls not applied yet: (body record, statement records).
    pub(super) modifications: Vec<crate::sema::Modification>,
    /// Set from `READY_FOR_CUSTOM_LINK_COMMAND` until the metaprogram's link is reported.
    pub(super) link: Option<PendingLink>,
    /// `PRE_WRITE_EXECUTABLE` went to the intercepting metaprogram: the next step writes the
    /// output, so what the metaprogram does on that message comes first.
    pub(super) awaiting_write: bool,
    /// `remap_import` rules: (importing module, imported name, replacement).
    pub(super) remaps: Vec<(String, String, String)>,
    /// `provide_import` offers.
    pub(super) provided: Vec<ProvidedImport>,
    /// Header records `compiler_make_procedure_live` named, not applied yet.
    pub(super) live: Vec<i64>,
    /// A failed load held back so an intercepting metaprogram can `provide_import` what was
    /// missing; see [`resume_failed_load`].
    pub(super) failed_load: Option<FailedLoad>,
}

/// A workspace whose sources did not load because an import found no module, while a
/// metaprogram was listening for `FAILED_IMPORT`.
pub(super) struct FailedLoad {
    /// The diagnostic, reported if nothing is provided.
    pub(super) text: String,
    /// The sources to load again once something was provided.
    pub(super) sources: Vec<ProgramSource>,
    pub(super) compiler: Box<Compiler>,
    /// How many offers there were when the imports failed.
    pub(super) offers: usize,
}

impl Workspace {
    pub(super) fn queue_source(&mut self, source: ProgramSource) -> Result<(), String> {
        if self.stage == Stage::Done {
            return Err(format!(
                "workspace `{}` has already been compiled; add its sources before it completes",
                self.name
            ));
        }

        self.pending.push(source);
        Ok(())
    }

    pub(super) fn new(name: String) -> Self {
        Workspace {
            name,
            pending: Vec::new(),
            settings: BuildSettings::default(),
            options: None,
            intercepted: false,
            intercept_flags: 0,
            stage: Stage::Open,
            compiler: None,
            events: VecDeque::new(),
            failed: false,
            modifications: Vec::new(),
            link: None,
            awaiting_write: false,
            remaps: Vec::new(),
            provided: Vec::new(),
            live: Vec::new(),
            failed_load: None,
        }
    }
}

/// One compiler message, in primitive form.
#[derive(Clone, Debug, Default)]
pub struct Event {
    /// One of the `EVENT_*` constants.
    pub kind: i64,
    /// Integer payload; its meaning depends on `kind`.
    pub ints: Vec<i64>,
    /// Byte-string payload; its meaning depends on `kind`.
    pub strings: Vec<Vec<u8>>,
}

/// `Event::kind` of a message about a loaded source file.
pub const EVENT_FILE: i64 = 1;
/// `Event::kind` of a message that the build reached a phase.
pub const EVENT_PHASE: i64 = 2;
/// `Event::kind` of the message that ends a workspace's messages.
pub const EVENT_COMPLETE: i64 = 3;
/// `Event::kind` of a message about an imported module.
pub const EVENT_IMPORT: i64 = 4;
/// `Event::kind` of a message that a code unit was type-checked.
pub const EVENT_TYPECHECKED: i64 = 5;
/// `Event::kind` of a message that an import could not be loaded.
pub const EVENT_FAILED_IMPORT: i64 = 6;
/// `Event::kind` of a compile error message.
pub const EVENT_ERROR: i64 = 7;

/// The text of a `DEBUG_DUMP` message is string 0.
pub const EVENT_DEBUG_DUMP: i64 = 8;

/// `Event::kind` of a performance report message.
pub const EVENT_PERFORMANCE_REPORT: i64 = 9;

/// `Intercept_Flags.DO_PERFORMANCE_REPORT_POLYMORPHS` and `_RUNS` (`stdlib/Compiler/workspace.jai`).
pub(super) const INTERCEPT_REPORT_POLYMORPHS: i64 = 0x1000;

pub(super) const INTERCEPT_REPORT_RUNS: i64 = 0x2000;

pub(super) const PHASE_ALL_SOURCE_CODE_PARSED: i64 = 0;
pub(super) const PHASE_TYPECHECKED_ALL_WE_CAN: i64 = 1;
pub(super) const PHASE_ALL_TARGET_CODE_BUILT: i64 = 2;
pub(super) const PHASE_PRE_WRITE_EXECUTABLE: i64 = 3;
pub(super) const PHASE_POST_WRITE_EXECUTABLE: i64 = 4;
pub(super) const PHASE_READY_FOR_CUSTOM_LINK_COMMAND: i64 = 5;

/// The workspace registry of one top-level compilation.
pub struct Workspaces {
    pub(super) env: BuildEnv,
    /// The workspace whose program is run by the embedder (`jaic -interp` with plugins) instead
    /// of being written.
    pub(super) run_target: Option<i64>,
    pub(super) run_ready: bool,
    /// What the first workspace a plugin metaprogram creates starts from: the build options the
    /// command line asked for.
    pub(super) first_created: Option<BuildSettings>,
    /// Some compile-time code can ask for a type table (`get_type_table`).
    pub(super) type_table_used: bool,
    /// Index = workspace id; 0 and 1 are unused (`jai` reserves workspace 1, so the top-level
    /// program is workspace 2 and the first created workspace is 3).
    pub(super) list: Vec<Workspace>,
    /// Workspace whose compile-time code is running.
    pub(super) current: Vec<i64>,
    /// Strings handed to Jai code; kept alive for the whole compilation.
    pub(super) strings: Vec<Box<[u8]>>,
    /// Record strings handed to Jai code without copying (the record may later drop its own).
    pub(super) kept: Vec<Arc<[u8]>>,
    /// Messages, syntax trees and types exported to metaprograms.
    pub(crate) records: Records,
    /// The structs each program built from the records (`State::builder` indexes it).
    pub(super) builders: Vec<crate::record_build::Builder>,
    pub(super) data_segments: Vec<DataSegment>,
    /// What the build's sources say about reading TYPECHECKED messages.
    pub(crate) typechecked_use: crate::source::TypecheckedUse,
    /// With `record_files` (`jaic -deps`): the files the workspaces' compilers have read, kept
    /// as each step ends because a finished workspace drops its compiler.
    pub(super) record_files: bool,
    pub(super) read_files: std::collections::BTreeSet<std::path::PathBuf>,
}

pub(super) struct DataSegment {
    pub(super) handle: u64,
    pub(super) _name: Vec<u8>,
    pub(super) workspace: i64,
    pub(super) characteristics: u32,
    pub(super) alignment: u64,
}

/// A workspace registry shared by the compiler and the metaprogram that drives it.
pub type SharedWorkspaces = Rc<RefCell<Workspaces>>;
