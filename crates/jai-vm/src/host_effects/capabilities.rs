/// Capability names identify real embedding grants; source strings cannot add one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostCapability {
    FileRead,
    FileWrite,
    Console,
    Clock,
    Processes,
    Cpu,
    Graphics,
    ForeignFunctions,
}
