//! The C library records the sandbox fills, as member lists placed by the program's
//! `DataLayout`: one list serves every CPU whose C library declares the record the same way,
//! and the alignment of a 64-bit field (4 on i686, 8 on armv7) comes from the layout.
use super::guest::{fill, layout, write_uint};
use crate::layout::Member;
use crate::target::{TargetCpu, TargetOs};

/// What a member of `struct stat` holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stat {
    Dev,
    Ino,
    Mode,
    Nlink,
    Size,
    Blksize,
    Blocks,
    /// The seconds of a `struct timespec` (`st_atim`, `st_mtim`, `st_ctim`, Darwin's
    /// `st_birthtimespec`); its nanoseconds are a `Pad`.
    Seconds,
    /// A member the sandbox leaves zero: an owner, a device number, padding.
    Pad,
}

use Stat::{Blksize, Blocks, Dev, Ino, Mode, Nlink, Pad, Seconds, Size};

/// A `struct timespec` with a 64-bit `time_t`: seconds, then a `long` padded to 8 bytes.
const TIME: [(Stat, u8); 2] = [(Seconds, 8), (Pad, 8)];

/// Linux x86-64; the stdlib's wasm record is the same.
const LINUX_X64: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Nlink, 8),
    (Mode, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Size, 8),
    (Blksize, 8),
    (Blocks, 8),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    (Pad, 8),
    (Pad, 8),
    (Pad, 8),
];

/// The generic Linux record (arm64, riscv64).
const LINUX_GENERIC: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Mode, 4),
    (Nlink, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Pad, 8),
    (Size, 8),
    (Blksize, 4),
    (Pad, 4),
    (Blocks, 8),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    (Pad, 4),
    (Pad, 4),
];

/// 32-bit Linux (i686, armv7) with 64-bit `time_t` and file offsets: what
/// `__stat64_time64` fills.
const LINUX_32: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Mode, 4),
    (Nlink, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Size, 8),
    (Blksize, 4),
    (Blocks, 8),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
];

/// Linux s390x: the times come before the block counts.
const LINUX_S390X: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Nlink, 8),
    (Mode, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Size, 8),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    (Blksize, 8),
    (Blocks, 8),
    (Pad, 8),
    (Pad, 8),
    (Pad, 8),
];

/// Linux powerpc64.
const LINUX_POWERPC64: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Nlink, 8),
    (Mode, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Size, 8),
    (Blksize, 8),
    (Blocks, 8),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    (Pad, 8),
    (Pad, 8),
    (Pad, 8),
];

/// Darwin's 64-bit-inode record.
const DARWIN: &[(Stat, u8)] = &[
    (Dev, 4),
    (Mode, 2),
    (Nlink, 2),
    (Ino, 8),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    TIME[0],
    TIME[1],
    (Size, 8),
    (Blocks, 8),
    (Blksize, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Pad, 8),
];

fn stat_members(darwin: bool, cpu: Option<TargetCpu>) -> &'static [(Stat, u8)] {
    if darwin {
        return DARWIN;
    }
    match cpu {
        Some(TargetCpu::X86 | TargetCpu::Arm) => LINUX_32,
        Some(TargetCpu::Arm64) => LINUX_GENERIC,
        Some(TargetCpu::S390x) => LINUX_S390X,
        Some(TargetCpu::PowerPc64) => LINUX_POWERPC64,
        Some(TargetCpu::X64 | TargetCpu::Wasm | TargetCpu::Avr) | None => LINUX_X64,
    }
}

/// What `stat` reports about one file.
pub(super) struct FileStat {
    pub inode: u64,
    pub mode: u32,
    pub links: u64,
    pub size: u64,
    pub mtime: u64,
}

/// Fill the `struct stat` at `out` for a program on `os` and `cpu` (`None`: the Linux x86-64
/// record), at the program's layout.
pub(super) fn fill_stat(os: TargetOs, cpu: Option<TargetCpu>, out: u64, file: &FileStat) {
    let members = stat_members(os == TargetOs::MacOS, cpu);
    let layout = layout();
    let ints: Vec<Member> = members
        .iter()
        .map(|&(_, bytes)| Member::Int(bytes))
        .collect();
    let record = layout.pack(&ints);
    fill(out, 0, record.size as usize);
    for (&(what, bytes), &offset) in members.iter().zip(&record.offsets) {
        let value = match what {
            Dev => 1,
            Ino => file.inode,
            Mode => u64::from(file.mode),
            Nlink => file.links,
            Size => file.size,
            Blksize => 4096,
            Blocks => file.size.div_ceil(512),
            Seconds => file.mtime,
            Pad => continue,
        };
        write_uint(out + offset, bytes, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::Target;

    /// The record sizes the C libraries declare.
    #[test]
    fn stat_records_have_their_c_library_sizes() {
        for (os, cpu, size) in [
            (TargetOs::Linux, TargetCpu::X64, 144),
            (TargetOs::Linux, TargetCpu::Arm64, 128),
            (TargetOs::Linux, TargetCpu::X86, 108),
            (TargetOs::Linux, TargetCpu::Arm, 112),
            (TargetOs::Linux, TargetCpu::S390x, 144),
            (TargetOs::Linux, TargetCpu::PowerPc64, 144),
            (TargetOs::MacOS, TargetCpu::Arm64, 144),
        ] {
            let layout = Target::new(os, cpu).data_layout();
            let members = stat_members(os == TargetOs::MacOS, Some(cpu));
            let ints: Vec<Member> = members.iter().map(|&(_, b)| Member::Int(b)).collect();
            assert_eq!(layout.pack(&ints).size, size, "{os:?} {cpu:?}");
        }
    }
}
