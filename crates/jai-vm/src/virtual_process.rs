//! Virtual POSIX descriptor/process state; no method forks or executes a native process.
use crate::host_effects::{
    HostEffects, HostError, HostOutcome, HostRequest, HostRequestKey, HostResponse,
    ProcessLaunchFailure, ProcessOutput, ProcessTermination, ProgramInvocation,
};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn identity() -> u64 {
    NEXT_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .expect("virtual process identities exhausted")
}

/// A ledger identity, never an OS pid. The ABI adapter must resolve source pid integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u64);
impl ProcessId {
    pub fn abi_value(self) -> Result<i32, ProcessError> {
        i32::try_from(self.0).map_err(|_| ProcessError::Budget("process ABI identities"))
    }
}
/// A process-local slot plus a generation; stale Rust tokens cannot select reused slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileDescriptor {
    process: ProcessId,
    number: i32,
    generation: u64,
}
impl FileDescriptor {
    pub fn number(self) -> i32 {
        self.number
    }
    pub fn process(self) -> ProcessId {
        self.process
    }
}
/// Immutable access mode of an open description; status flags cannot change it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DescriptorAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForkPair {
    pub parent: ProcessId,
    pub child: ProcessId,
}
#[derive(Clone, Copy, Debug)]
pub struct ProcessLimits {
    pub processes: usize,
    pub descriptors: usize,
    pub buffered_bytes: usize,
    pub queued_messages: usize,
    pub transferred_descriptors: usize,
    pub transfer_bytes: usize,
}
impl Default for ProcessLimits {
    fn default() -> Self {
        Self {
            processes: 128,
            descriptors: 4096,
            buffered_bytes: 4 * 1024 * 1024,
            queued_messages: 4096,
            transferred_descriptors: 4096,
            transfer_bytes: 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessError {
    UnknownProcess,
    UnknownDescriptor,
    WrongProcess,
    NotRunning,
    NotChild,
    AlreadyReaped,
    InvalidDescriptor,
    WrongDirection,
    BrokenPipe,
    WouldBlock,
    Unsupported(&'static str),
    Budget(&'static str),
    Host(HostError),
    LiveResources,
}
impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "virtual process: {self:?}")
    }
}
impl std::error::Error for ProcessError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcessEvent {
    Readable(FileDescriptor),
    ChildExited { parent: ProcessId, child: ProcessId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessIo<T> {
    Ready(T),
    Pending(ProcessEvent),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceivedMessage {
    pub bytes: Vec<u8>,
    pub descriptors: Vec<FileDescriptor>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Readiness {
    pub bytes: usize,
    pub eof: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    Pending(ProcessEvent),
    StillRunning,
    Reaped(ProcessTermination),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecOutcome {
    /// The source child continuation must be replaced, never resumed after exec.
    Replaced,
    /// The child remains running, with its descriptors unchanged, so a genuine
    /// source execvp call can return -1 and report the observed OS errno.
    Failed(ProcessLaunchFailure),
    /// Suspend that continuation until the exact host request is serviced.
    Pending(HostRequestKey),
    /// Boundary rejection is preserved. Policy denial must not be converted to errno.
    Rejected(HostError),
}

type OpenId = u64;
type ChannelId = u64;
#[derive(Clone, Debug)]
struct Slot {
    generation: u64,
    open: OpenId,
    close_on_exec: bool,
}
#[derive(Clone, Debug)]
enum ProcessState {
    Running,
    AwaitingHost { key: HostRequestKey },
    Exited(ProcessTermination),
    Reaped,
}
#[derive(Clone, Debug)]
struct Process {
    parent: Option<ProcessId>,
    slots: BTreeMap<i32, Slot>,
    state: ProcessState,
}
#[derive(Clone, Debug)]
struct OpenDescription {
    read: Option<ChannelId>,
    write: Option<ChannelId>,
    socket: bool,
    nonblocking: bool,
    /// Descriptors and in-flight SCM_RIGHTS both keep the description alive.
    references: usize,
}
#[derive(Clone, Debug)]
struct Message {
    bytes: VecDeque<u8>,
    rights: Vec<OpenId>,
}
#[derive(Clone, Debug, Default)]
struct Channel {
    messages: VecDeque<Message>,
    readers: usize,
    writers: usize,
    write_shutdown: bool,
}
/// Shared IPC state is cloned once per VM transaction, not separately for fork branches.
#[derive(Clone, Debug)]
pub struct VirtualProcesses {
    limits: ProcessLimits,
    processes: BTreeMap<ProcessId, Process>,
    opens: HashMap<OpenId, OpenDescription>,
    channels: HashMap<ChannelId, Channel>,
    buffered_bytes: usize,
    queued_messages: usize,
    queued_rights: usize,
}
impl VirtualProcesses {
    pub fn new(limits: ProcessLimits) -> Self {
        Self {
            limits,
            processes: BTreeMap::new(),
            opens: HashMap::new(),
            channels: HashMap::new(),
            buffered_bytes: 0,
            queued_messages: 0,
            queued_rights: 0,
        }
    }
    pub fn create_root(&mut self) -> Result<ProcessId, ProcessError> {
        self.check_process_capacity()?;
        let id = ProcessId(identity());
        id.abi_value()?;
        self.processes.insert(
            id,
            Process {
                parent: None,
                slots: BTreeMap::new(),
                state: ProcessState::Running,
            },
        );
        Ok(id)
    }
    /// Pure fork metadata: the scheduler admits its candidate before cloning.
    /// Process-table entry accounting grows by one; slots grow by this count.
    /// This validates process quota only, not the global descriptor quota.
    pub(crate) fn fork_descriptor_cells(&self, parent: ProcessId) -> Result<usize, ProcessError> {
        let process = self.process(parent)?;
        if !matches!(process.state, ProcessState::Running) {
            return Err(ProcessError::NotRunning);
        }
        self.check_process_capacity()?;
        Ok(process.slots.len())
    }
    /// Only copies the descriptor table. The VM must split its actual continuation/memory.
    pub fn fork(&mut self, parent: ProcessId) -> Result<ForkPair, ProcessError> {
        self.running(parent)?;
        self.check_process_capacity()?;
        let mut slots = self.process(parent)?.slots.clone();
        self.check_descriptor_capacity(slots.len())?;
        let child = ProcessId(identity());
        child.abi_value()?;
        for slot in slots.values_mut() {
            slot.generation = identity();
            self.opens
                .get_mut(&slot.open)
                .expect("live slot")
                .references += 1;
        }
        self.processes.insert(
            child,
            Process {
                parent: Some(parent),
                slots,
                state: ProcessState::Running,
            },
        );
        Ok(ForkPair { parent, child })
    }
    pub fn resolve_pid(&self, parent: ProcessId, abi_pid: i32) -> Result<ProcessId, ProcessError> {
        self.process(parent)?;
        self.processes
            .iter()
            .find_map(|(id, process)| {
                (process.parent == Some(parent) && id.abi_value().ok() == Some(abi_pid))
                    .then_some(*id)
            })
            .ok_or(ProcessError::NotChild)
    }
    /// Converts the current source ABI slot into a checked capability for this process.
    pub fn descriptor(
        &self,
        process: ProcessId,
        number: i32,
    ) -> Result<FileDescriptor, ProcessError> {
        let slot = self
            .process(process)?
            .slots
            .get(&number)
            .ok_or(ProcessError::UnknownDescriptor)?;
        Ok(FileDescriptor {
            process,
            number,
            generation: slot.generation,
        })
    }
    pub fn pipe(&mut self, process: ProcessId) -> Result<[FileDescriptor; 2], ProcessError> {
        self.running(process)?;
        self.check_descriptor_capacity(2)?;
        let channel = identity();
        self.channels.insert(
            channel,
            Channel {
                readers: 1,
                writers: 1,
                ..Channel::default()
            },
        );
        let read = self.open(Some(channel), None, false);
        let write = self.open(None, Some(channel), false);
        Ok([self.install(process, read)?, self.install(process, write)?])
    }
    pub fn socketpair(&mut self, process: ProcessId) -> Result<[FileDescriptor; 2], ProcessError> {
        self.running(process)?;
        self.check_descriptor_capacity(2)?;
        let a = identity();
        let b = identity();
        for channel in [a, b] {
            self.channels.insert(
                channel,
                Channel {
                    readers: 1,
                    writers: 1,
                    ..Channel::default()
                },
            );
        }
        let left = self.open(Some(a), Some(b), true);
        let right = self.open(Some(b), Some(a), true);
        Ok([self.install(process, left)?, self.install(process, right)?])
    }
    pub fn dup2(
        &mut self,
        source: FileDescriptor,
        destination: i32,
    ) -> Result<FileDescriptor, ProcessError> {
        if destination < 0 {
            return Err(ProcessError::InvalidDescriptor);
        }
        self.running(source.process)?;
        let open = self.slot(source)?.open;
        if source.number == destination {
            return Ok(source);
        }
        if let Ok(old) = self.descriptor(source.process, destination) {
            self.close(old)?;
        } else {
            self.check_descriptor_capacity(1)?;
        }
        let slot = Slot {
            generation: identity(),
            open,
            close_on_exec: false,
        };
        self.opens.get_mut(&open).expect("live source").references += 1;
        let fd = FileDescriptor {
            process: source.process,
            number: destination,
            generation: slot.generation,
        };
        self.processes
            .get_mut(&source.process)
            .expect("known process")
            .slots
            .insert(destination, slot);
        Ok(fd)
    }
    pub fn close(&mut self, fd: FileDescriptor) -> Result<(), ProcessError> {
        let open = self.slot(fd)?.open;
        self.processes
            .get_mut(&fd.process)
            .expect("known process")
            .slots
            .remove(&fd.number);
        self.release_open(open);
        Ok(())
    }
    pub fn set_close_on_exec(
        &mut self,
        fd: FileDescriptor,
        enabled: bool,
    ) -> Result<(), ProcessError> {
        self.slot(fd)?;
        self.processes
            .get_mut(&fd.process)
            .expect("known process")
            .slots
            .get_mut(&fd.number)
            .expect("live slot")
            .close_on_exec = enabled;
        Ok(())
    }
    pub fn close_on_exec(&self, fd: FileDescriptor) -> Result<bool, ProcessError> {
        Ok(self.slot(fd)?.close_on_exec)
    }
    /// O_NONBLOCK belongs to the open description and is shared across dup/fork/SCM_RIGHTS.
    pub fn set_nonblocking(
        &mut self,
        fd: FileDescriptor,
        enabled: bool,
    ) -> Result<(), ProcessError> {
        let open = self.slot(fd)?.open;
        self.opens.get_mut(&open).expect("live open").nonblocking = enabled;
        Ok(())
    }
    pub fn nonblocking(&self, fd: FileDescriptor) -> Result<bool, ProcessError> {
        Ok(self.description(fd)?.nonblocking)
    }
    pub(crate) fn access_mode(&self, fd: FileDescriptor) -> Result<DescriptorAccess, ProcessError> {
        let open = self.description(fd)?;
        match (open.read.is_some(), open.write.is_some()) {
            (true, false) => Ok(DescriptorAccess::ReadOnly),
            (false, true) => Ok(DescriptorAccess::WriteOnly),
            (true, true) => Ok(DescriptorAccess::ReadWrite),
            (false, false) => Err(ProcessError::Unsupported("descriptor has no access mode")),
        }
    }
    /// Socket SHUT_WR closes the direction even if inherited aliases remain alive.
    pub fn shutdown_write(&mut self, fd: FileDescriptor) -> Result<(), ProcessError> {
        let open = self.description(fd)?;
        if !open.socket {
            return Err(ProcessError::Unsupported("shutdown on non-socket"));
        }
        let channel = open.write.ok_or(ProcessError::WrongDirection)?;
        self.channels
            .get_mut(&channel)
            .expect("live channel")
            .write_shutdown = true;
        Ok(())
    }
    pub fn write(&mut self, fd: FileDescriptor, bytes: &[u8]) -> Result<usize, ProcessError> {
        self.send(fd, bytes, &[])
    }
    /// The trusted ABI adapter parses SCM_RIGHTS; unsupported ancillary forms reject there.
    pub fn send_rights(
        &mut self,
        fd: FileDescriptor,
        bytes: &[u8],
        rights: &[FileDescriptor],
    ) -> Result<usize, ProcessError> {
        if !self.description(fd)?.socket {
            return Err(ProcessError::Unsupported("SCM_RIGHTS on non-socket"));
        }
        if bytes.is_empty() && !rights.is_empty() {
            return Err(ProcessError::Unsupported("SCM_RIGHTS without stream data"));
        }
        self.send(fd, bytes, rights)
    }
    pub fn read(
        &mut self,
        fd: FileDescriptor,
        count: usize,
    ) -> Result<ProcessIo<Vec<u8>>, ProcessError> {
        match self.receive(fd, count, None)? {
            ProcessIo::Ready(message) => Ok(ProcessIo::Ready(message.bytes)),
            ProcessIo::Pending(event) => Ok(ProcessIo::Pending(event)),
        }
    }
    pub fn recv_rights(
        &mut self,
        fd: FileDescriptor,
        count: usize,
        rights_capacity: usize,
    ) -> Result<ProcessIo<ReceivedMessage>, ProcessError> {
        if !self.description(fd)?.socket {
            return Err(ProcessError::Unsupported("recvmsg on non-socket"));
        }
        self.receive(fd, count, Some(rights_capacity))
    }
    pub fn readiness(&self, fd: FileDescriptor) -> Result<Readiness, ProcessError> {
        let channel = self
            .description(fd)?
            .read
            .ok_or(ProcessError::WrongDirection)?;
        let channel = self.channels.get(&channel).expect("live channel");
        Ok(Readiness {
            bytes: channel
                .messages
                .iter()
                .map(|message| message.bytes.len())
                .sum(),
            eof: channel.writers == 0 || channel.write_shutdown,
        })
    }
    pub fn event_ready(&self, event: ProcessEvent) -> Result<bool, ProcessError> {
        match event {
            ProcessEvent::Readable(fd) => {
                let state = self.readiness(fd)?;
                Ok(state.bytes != 0 || state.eof)
            }
            ProcessEvent::ChildExited { parent, child } => {
                self.child(parent, child)?;
                Ok(matches!(
                    self.process(child)?.state,
                    ProcessState::Exited(_) | ProcessState::Reaped
                ))
            }
        }
    }
    /// Programs are already resolved to an explicitly registered capability and exact argv.
    /// Stdin interaction is unsupported by the current null-stdin host runner.
    /// Host policy/budget errors are boundary failures, not source OS launch errors.
    pub fn exec(
        &mut self,
        process: ProcessId,
        invocation: ProgramInvocation,
        effects: &mut impl HostEffects,
    ) -> Result<ExecOutcome, ProcessError> {
        self.running(process)?;
        self.validate_stdin(process)?;
        match effects.request(HostRequest::RunProgram(invocation)) {
            HostOutcome::Pending(key) => {
                self.processes
                    .get_mut(&process)
                    .expect("known process")
                    .state = ProcessState::AwaitingHost { key };
                Ok(ExecOutcome::Pending(key))
            }
            HostOutcome::Rejected(error) => Ok(ExecOutcome::Rejected(error)),
            HostOutcome::Ready(HostResponse::Process(output)) => {
                self.apply_output(process, output)?;
                Ok(ExecOutcome::Replaced)
            }
            HostOutcome::Ready(HostResponse::ProcessLaunchFailed(failure)) => {
                Ok(ExecOutcome::Failed(failure))
            }
            HostOutcome::Ready(_) => {
                Err(ProcessError::Unsupported("unexpected exec host response"))
            }
        }
    }
    /// Scheduler delivers only the matching serviced observation; no native call occurs here.
    pub fn complete_exec(
        &mut self,
        process: ProcessId,
        key: HostRequestKey,
        outcome: HostOutcome,
    ) -> Result<ExecOutcome, ProcessError> {
        if !matches!(self.process(process)?.state, ProcessState::AwaitingHost { key: expected } if expected == key)
        {
            return Err(ProcessError::NotRunning);
        }
        match outcome {
            HostOutcome::Ready(HostResponse::Process(output)) => {
                self.apply_output(process, output)?;
                Ok(ExecOutcome::Replaced)
            }
            HostOutcome::Ready(HostResponse::ProcessLaunchFailed(failure)) => {
                self.processes
                    .get_mut(&process)
                    .expect("known process")
                    .state = ProcessState::Running;
                Ok(ExecOutcome::Failed(failure))
            }
            HostOutcome::Rejected(error) => {
                self.processes
                    .get_mut(&process)
                    .expect("known process")
                    .state = ProcessState::Running;
                Ok(ExecOutcome::Rejected(error))
            }
            HostOutcome::Pending(same) if same == key => Ok(ExecOutcome::Pending(key)),
            _ => Err(ProcessError::Unsupported("unexpected exec completion")),
        }
    }
    pub fn exit(&mut self, process: ProcessId, code: i32) -> Result<(), ProcessError> {
        self.running(process)?;
        self.close_all(process);
        // POSIX exposes the low eight bits of exit/_exit through wait status.
        self.processes
            .get_mut(&process)
            .expect("known process")
            .state = ProcessState::Exited(ProcessTermination::Exited(code & 0xff));
        Ok(())
    }
    pub fn wait(
        &mut self,
        parent: ProcessId,
        child: ProcessId,
        nohang: bool,
    ) -> Result<WaitOutcome, ProcessError> {
        self.child(parent, child)?;
        match self.process(child)?.state {
            ProcessState::Exited(termination) => {
                self.processes.get_mut(&child).expect("known child").state = ProcessState::Reaped;
                Ok(WaitOutcome::Reaped(termination))
            }
            ProcessState::Reaped => Err(ProcessError::AlreadyReaped),
            _ if nohang => Ok(WaitOutcome::StillRunning),
            _ => Ok(WaitOutcome::Pending(ProcessEvent::ChildExited {
                parent,
                child,
            })),
        }
    }
    pub fn require_quiescent(&self, root: ProcessId) -> Result<(), ProcessError> {
        self.process(root)?;
        if self.processes.iter().any(|(id, process)| {
            !process.slots.is_empty()
                || matches!(process.state, ProcessState::AwaitingHost { .. })
                || (*id != root && !matches!(process.state, ProcessState::Reaped))
        }) || !self.opens.is_empty()
            || !self.channels.is_empty()
            || self.buffered_bytes != 0
            || self.queued_messages != 0
            || self.queued_rights != 0
        {
            return Err(ProcessError::LiveResources);
        }
        Ok(())
    }
    /// Trusted cancellation drops even reference cycles caused by passing sockets as rights.
    pub fn reset(&mut self) {
        self.processes.clear();
        self.opens.clear();
        self.channels.clear();
        self.buffered_bytes = 0;
        self.queued_messages = 0;
        self.queued_rights = 0;
    }
    pub fn retained_bytes(&self) -> usize {
        self.buffered_bytes
    }
    pub fn limits(&self) -> ProcessLimits {
        self.limits
    }
    pub fn parent(&self, process: ProcessId) -> Result<Option<ProcessId>, ProcessError> {
        Ok(self.process(process)?.parent)
    }
    pub fn validate_running(&self, process: ProcessId) -> Result<(), ProcessError> {
        self.running(process)
    }
    /// Charge retained backing and metadata before cloning or walking this world.
    pub fn work_cost(&self) -> Result<u64, ProcessError> {
        let mut cost = 1_u64;
        let mut add = |count: usize| -> Result<(), ProcessError> {
            cost = cost
                .checked_add(u64::try_from(count).map_err(|_| ProcessError::Budget("work cost"))?)
                .ok_or(ProcessError::Budget("work cost"))?;
            Ok(())
        };
        for count in [
            self.processes.len(),
            self.opens.capacity(),
            self.channels.capacity(),
        ] {
            add(count)?;
        }
        for process in self.processes.values() {
            add(process.slots.len())?;
        }
        for channel in self.channels.values() {
            add(channel.messages.capacity())?;
            for message in &channel.messages {
                add(message.bytes.capacity())?;
                add(message.rights.capacity())?;
            }
        }
        Ok(cost)
    }
    /// Cancellation must also retire these requests in the embedding host provider.
    /// Resetting the virtual ledger does not cancel or service any host work itself.
    pub fn pending_requests(&self) -> impl Iterator<Item = (ProcessId, HostRequestKey)> + '_ {
        self.processes
            .iter()
            .filter_map(|(id, process)| match process.state {
                ProcessState::AwaitingHost { key } => Some((*id, key)),
                _ => None,
            })
    }

    fn process(&self, process: ProcessId) -> Result<&Process, ProcessError> {
        self.processes
            .get(&process)
            .ok_or(ProcessError::UnknownProcess)
    }
    fn running(&self, process: ProcessId) -> Result<(), ProcessError> {
        if matches!(self.process(process)?.state, ProcessState::Running) {
            Ok(())
        } else {
            Err(ProcessError::NotRunning)
        }
    }
    fn child(&self, parent: ProcessId, child: ProcessId) -> Result<(), ProcessError> {
        self.process(parent)?;
        if self.process(child)?.parent == Some(parent) {
            Ok(())
        } else {
            Err(ProcessError::NotChild)
        }
    }
    fn slot(&self, fd: FileDescriptor) -> Result<&Slot, ProcessError> {
        let slot = self
            .process(fd.process)?
            .slots
            .get(&fd.number)
            .ok_or(ProcessError::UnknownDescriptor)?;
        if slot.generation != fd.generation {
            return Err(ProcessError::UnknownDescriptor);
        }
        Ok(slot)
    }
    fn description(&self, fd: FileDescriptor) -> Result<&OpenDescription, ProcessError> {
        Ok(self
            .opens
            .get(&self.slot(fd)?.open)
            .expect("live descriptor"))
    }
    fn check_process_capacity(&self) -> Result<(), ProcessError> {
        if self.processes.len() >= self.limits.processes {
            Err(ProcessError::Budget("processes"))
        } else {
            Ok(())
        }
    }
    fn check_descriptor_capacity(&self, additional: usize) -> Result<(), ProcessError> {
        let existing: usize = self
            .processes
            .values()
            .map(|process| process.slots.len())
            .sum();
        if existing
            .checked_add(additional)
            .is_none_or(|count| count > self.limits.descriptors)
        {
            Err(ProcessError::Budget("descriptors"))
        } else {
            Ok(())
        }
    }
    fn open(&mut self, read: Option<ChannelId>, write: Option<ChannelId>, socket: bool) -> OpenId {
        let id = identity();
        self.opens.insert(
            id,
            OpenDescription {
                read,
                write,
                socket,
                nonblocking: false,
                references: 0,
            },
        );
        id
    }
    fn install(
        &mut self,
        process: ProcessId,
        open: OpenId,
    ) -> Result<FileDescriptor, ProcessError> {
        let slots = &self.process(process)?.slots;
        // The embedding reserves conventional stdio slots; it must install explicit
        // virtual stdio bindings before allowing source reads/writes on 0, 1 or 2.
        let mut number = 3_i32;
        while slots.contains_key(&number) {
            number = number
                .checked_add(1)
                .ok_or(ProcessError::Budget("descriptor ABI slots"))?;
        }
        let generation = identity();
        self.opens.get_mut(&open).expect("known open").references += 1;
        self.processes
            .get_mut(&process)
            .expect("known process")
            .slots
            .insert(
                number,
                Slot {
                    generation,
                    open,
                    close_on_exec: false,
                },
            );
        Ok(FileDescriptor {
            process,
            number,
            generation,
        })
    }
    fn send(
        &mut self,
        fd: FileDescriptor,
        bytes: &[u8],
        rights: &[FileDescriptor],
    ) -> Result<usize, ProcessError> {
        let channel_id = self
            .description(fd)?
            .write
            .ok_or(ProcessError::WrongDirection)?;
        let channel = self.channels.get(&channel_id).expect("live channel");
        if bytes.is_empty() && rights.is_empty() {
            return Ok(0);
        }
        if channel.readers == 0 || channel.write_shutdown {
            return Err(ProcessError::BrokenPipe);
        }
        if bytes.len() > self.limits.transfer_bytes {
            return Err(ProcessError::Budget("transfer bytes"));
        }
        if !bytes.is_empty()
            && self.processes.values().any(|process| {
                matches!(process.state, ProcessState::AwaitingHost { .. })
                    && process.slots.get(&0).is_some_and(|slot| {
                        self.opens
                            .get(&slot.open)
                            .is_some_and(|open| open.read == Some(channel_id))
                    })
            })
        {
            return Err(ProcessError::Unsupported(
                "host runner does not accept process stdin",
            ));
        }
        if rights.len() > self.limits.transferred_descriptors {
            return Err(ProcessError::Budget("transferred descriptors"));
        }
        let mut opens = Vec::with_capacity(rights.len());
        for right in rights {
            if right.process != fd.process {
                return Err(ProcessError::WrongProcess);
            }
            opens.push(self.slot(*right)?.open);
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        if self
            .buffered_bytes
            .checked_add(bytes.len())
            .is_none_or(|count| count > self.limits.buffered_bytes)
            || self.queued_messages >= self.limits.queued_messages
            || self
                .queued_rights
                .checked_add(rights.len())
                .is_none_or(|count| count > self.limits.transferred_descriptors)
        {
            return Err(ProcessError::Budget("queued transport"));
        }
        for open in &opens {
            self.opens.get_mut(open).expect("live right").references += 1;
        }
        self.buffered_bytes += bytes.len();
        self.queued_messages += 1;
        self.queued_rights += opens.len();
        self.channels
            .get_mut(&channel_id)
            .expect("live channel")
            .messages
            .push_back(Message {
                bytes: bytes.iter().copied().collect(),
                rights: opens,
            });
        Ok(bytes.len())
    }
    fn receive(
        &mut self,
        fd: FileDescriptor,
        count: usize,
        rights_capacity: Option<usize>,
    ) -> Result<ProcessIo<ReceivedMessage>, ProcessError> {
        let open = self.description(fd)?;
        let channel_id = open.read.ok_or(ProcessError::WrongDirection)?;
        if count > self.limits.transfer_bytes {
            return Err(ProcessError::Budget("transfer bytes"));
        }
        if count == 0 {
            return Ok(ProcessIo::Ready(ReceivedMessage {
                bytes: Vec::new(),
                descriptors: Vec::new(),
            }));
        }
        let channel = self.channels.get(&channel_id).expect("live channel");
        let Some(first) = channel.messages.front() else {
            return if channel.writers == 0 || channel.write_shutdown {
                Ok(ProcessIo::Ready(ReceivedMessage {
                    bytes: Vec::new(),
                    descriptors: Vec::new(),
                }))
            } else if open.nonblocking {
                Err(ProcessError::WouldBlock)
            } else {
                Ok(ProcessIo::Pending(ProcessEvent::Readable(fd)))
            };
        };
        if let Some(capacity) = rights_capacity {
            if first.rights.len() > capacity {
                return Err(ProcessError::Unsupported("truncated SCM_RIGHTS"));
            }
            self.check_descriptor_capacity(first.rights.len())?;
        }
        let channel = self.channels.get_mut(&channel_id).expect("live channel");
        let first = channel.messages.front_mut().expect("nonempty queue");
        let bytes: Vec<_> = first.bytes.drain(..count.min(first.bytes.len())).collect();
        let rights = std::mem::take(&mut first.rights);
        if first.bytes.is_empty() {
            channel.messages.pop_front();
            self.queued_messages -= 1;
        } else {
            // A partially consumed large message must not retain its original
            // backing allocation while only its remaining payload is budgeted.
            first.bytes.shrink_to_fit();
        }
        channel.messages.shrink_to_fit();
        self.buffered_bytes -= bytes.len();
        self.queued_rights -= rights.len();
        let mut descriptors = Vec::new();
        for open in rights {
            if rights_capacity.is_some() {
                descriptors.push(self.install(fd.process, open)?);
            }
            self.release_open(open);
        }
        Ok(ProcessIo::Ready(ReceivedMessage { bytes, descriptors }))
    }
    fn release_open(&mut self, id: OpenId) {
        // Ancillary references may form deep graphs. Cleanup uses bounded heap
        // work instead of recursively consuming the compiler thread stack.
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            let Some(open) = self.opens.get_mut(&id) else {
                continue;
            };
            open.references -= 1;
            if open.references != 0 {
                continue;
            }
            let open = self.opens.remove(&id).expect("known open");
            if let Some(channel) = open.read {
                self.channels.get_mut(&channel).expect("live read").readers -= 1;
            }
            if let Some(channel) = open.write {
                self.channels.get_mut(&channel).expect("live write").writers -= 1;
            }
            for channel_id in [open.read, open.write].into_iter().flatten() {
                let Some(channel) = self.channels.get(&channel_id) else {
                    continue;
                };
                // Bytes and ancillary rights cannot be delivered after the final reader closes.
                if channel.readers == 0 {
                    let messages = std::mem::take(
                        &mut self
                            .channels
                            .get_mut(&channel_id)
                            .expect("known channel")
                            .messages,
                    );
                    for message in messages {
                        self.buffered_bytes -= message.bytes.len();
                        self.queued_messages -= 1;
                        self.queued_rights -= message.rights.len();
                        for right in message.rights {
                            pending.push(right);
                        }
                    }
                }
                if self
                    .channels
                    .get(&channel_id)
                    .is_some_and(|channel| channel.readers == 0 && channel.writers == 0)
                {
                    self.channels.remove(&channel_id);
                }
            }
        }
    }
    fn close_all(&mut self, process: ProcessId) {
        let slots = std::mem::take(
            &mut self
                .processes
                .get_mut(&process)
                .expect("known process")
                .slots,
        );
        for slot in slots.into_values() {
            self.release_open(slot.open);
        }
    }
    fn validate_stdin(&self, process: ProcessId) -> Result<(), ProcessError> {
        if let Ok(stdin) = self.descriptor(process, 0)
            && self.description(stdin)?.read.is_some()
            && self.readiness(stdin)?.bytes != 0
        {
            return Err(ProcessError::Unsupported(
                "host runner does not accept process stdin",
            ));
        }
        Ok(())
    }
    fn apply_output(
        &mut self,
        process: ProcessId,
        output: ProcessOutput,
    ) -> Result<(), ProcessError> {
        self.validate_stdin(process)?;
        // Host observations already exist: validate all virtual mutations before publishing any.
        let mut candidate = self.clone();
        let closable: Vec<_> = candidate
            .process(process)?
            .slots
            .iter()
            .filter_map(|(number, slot)| slot.close_on_exec.then_some(*number))
            .collect();
        for number in closable {
            candidate.close(candidate.descriptor(process, number)?)?;
        }
        for (number, bytes) in [(1, output.stdout), (2, output.stderr)] {
            if !bytes.is_empty() {
                let fd = candidate
                    .descriptor(process, number)
                    .map_err(|_| ProcessError::Unsupported("uncaptured program output"))?;
                // Completed observations can exceed a single C read/write transfer, but not storage.
                for chunk in bytes.chunks(candidate.limits.transfer_bytes.max(1)) {
                    candidate.write(fd, chunk)?;
                }
            }
        }
        candidate.close_all(process);
        candidate
            .processes
            .get_mut(&process)
            .expect("known process")
            .state = ProcessState::Exited(output.termination);
        *self = candidate;
        Ok(())
    }
}

#[cfg(test)]
#[path = "virtual_process/tests.rs"]
mod tests;
