//! The C library records a POSIX program reads (`struct stat`, `struct dirent`), as member
//! lists placed by the program's `DataLayout`: one list serves every CPU whose C library
//! declares the record the same way, and the alignment of a 64-bit field (4 on i686, 8 on
//! armv7) comes from the layout.
use super::PosixAbi;
use crate::interp::sandbox::guest::{fill, layout, write_bytes, write_uint};
use crate::layout::Member;
use crate::target::TargetCpu;

/// One of the times `struct stat` holds, as `FileStat::times` orders them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Time {
    Access = 0,
    Modify = 1,
    Change = 2,
    /// Darwin's `st_birthtimespec`.
    Birth = 3,
}

/// What a member of `struct stat` holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stat {
    Dev,
    Ino,
    Mode,
    Nlink,
    Uid,
    Gid,
    Rdev,
    Size,
    Blksize,
    Blocks,
    /// The seconds of a `struct timespec`.
    Seconds(Time),
    /// Its nanoseconds, a `long` padded to 8 bytes.
    Nanoseconds(Time),
    /// Padding and members nothing fills (Darwin's flags and generation number).
    Pad,
}

use Stat::{
    Blksize, Blocks, Dev, Gid, Ino, Mode, Nanoseconds, Nlink, Pad, Rdev, Seconds, Size, Uid,
};

const ACCESS: [(Stat, u8); 2] = [(Seconds(Time::Access), 8), (Nanoseconds(Time::Access), 8)];
const MODIFY: [(Stat, u8); 2] = [(Seconds(Time::Modify), 8), (Nanoseconds(Time::Modify), 8)];
const CHANGE: [(Stat, u8); 2] = [(Seconds(Time::Change), 8), (Nanoseconds(Time::Change), 8)];
const BIRTH: [(Stat, u8); 2] = [(Seconds(Time::Birth), 8), (Nanoseconds(Time::Birth), 8)];

/// Linux x86-64; the stdlib's wasm record is the same.
const LINUX_X64: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Nlink, 8),
    (Mode, 4),
    (Uid, 4),
    (Gid, 4),
    (Pad, 4),
    (Rdev, 8),
    (Size, 8),
    (Blksize, 8),
    (Blocks, 8),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
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
    (Uid, 4),
    (Gid, 4),
    (Rdev, 8),
    (Pad, 8),
    (Size, 8),
    (Blksize, 4),
    (Pad, 4),
    (Blocks, 8),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
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
    (Uid, 4),
    (Gid, 4),
    (Rdev, 8),
    (Size, 8),
    (Blksize, 4),
    (Blocks, 8),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
];

/// Linux s390x: the times come before the block counts.
const LINUX_S390X: &[(Stat, u8)] = &[
    (Dev, 8),
    (Ino, 8),
    (Nlink, 8),
    (Mode, 4),
    (Uid, 4),
    (Gid, 4),
    (Pad, 4),
    (Rdev, 8),
    (Size, 8),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
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
    (Uid, 4),
    (Gid, 4),
    (Pad, 4),
    (Rdev, 8),
    (Size, 8),
    (Blksize, 8),
    (Blocks, 8),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
    (Pad, 8),
    (Pad, 8),
    (Pad, 8),
];

/// Darwin's 64-bit-inode record, the same on x86-64 (`stat$INODE64`) and arm64.
const DARWIN: &[(Stat, u8)] = &[
    (Dev, 4),
    (Mode, 2),
    (Nlink, 2),
    (Ino, 8),
    (Uid, 4),
    (Gid, 4),
    (Rdev, 4),
    ACCESS[0],
    ACCESS[1],
    MODIFY[0],
    MODIFY[1],
    CHANGE[0],
    CHANGE[1],
    BIRTH[0],
    BIRTH[1],
    (Size, 8),
    (Blocks, 8),
    (Blksize, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 4),
    (Pad, 8),
    (Pad, 8),
];

fn stat_members(abi: PosixAbi) -> &'static [(Stat, u8)] {
    if abi.darwin() {
        return DARWIN;
    }

    match abi.cpu() {
        TargetCpu::X86 | TargetCpu::Arm => LINUX_32,
        TargetCpu::Arm64 => LINUX_GENERIC,
        TargetCpu::S390x => LINUX_S390X,
        TargetCpu::PowerPc64 => LINUX_POWERPC64,
        TargetCpu::X64 | TargetCpu::Wasm | TargetCpu::Avr => LINUX_X64,
    }
}

/// What `stat` reports about one file. A time is seconds and nanoseconds since 1970.
pub(in crate::interp) struct FileStat {
    pub dev: u64,
    pub inode: u64,
    pub mode: u32,
    pub links: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    /// Access, modification, status change and (Darwin only) birth.
    pub times: [(i64, u32); 4],
}

/// Fill the `struct stat` at `out` for a program of `abi`, at the program's layout.
pub(in crate::interp) fn fill_stat(abi: PosixAbi, out: u64, file: &FileStat) {
    let members = stat_members(abi);
    let layout = layout();
    let ints: Vec<Member> = members
        .iter()
        .map(|&(_, bytes)| Member::Int(bytes))
        .collect();
    let record = layout.pack(&ints);

    fill(out, 0, record.size as usize);

    for (&(what, bytes), &offset) in members.iter().zip(&record.offsets) {
        let value = match what {
            Dev => file.dev,
            Ino => file.inode,
            Mode => u64::from(file.mode),
            Nlink => file.links,
            Uid => u64::from(file.uid),
            Gid => u64::from(file.gid),
            Rdev => file.rdev,
            Size => file.size,
            Blksize => file.blksize,
            Blocks => file.blocks,
            Seconds(time) => file.times[time as usize].0 as u64,
            Nanoseconds(time) => u64::from(file.times[time as usize].1),
            Pad => continue,
        };

        write_uint(out + offset, bytes, value);
    }
}

/// `d_type` of a directory and of a regular file; the numbers are the same on Linux and macOS.
pub(in crate::interp) const DT_DIR: u8 = 4;

pub(in crate::interp) const DT_REG: u8 = 8;

/// One `readdir` result.
pub(in crate::interp) struct DirEntry<'a> {
    pub inode: u64,
    /// Linux's `d_off`, Darwin's `d_seekoff`: where the next entry starts.
    pub next: u64,
    pub kind: u8,
    pub name: &'a [u8],
}

/// The bytes of a `struct dirent` of `abi`: Linux's 64-bit record (what `readdir64`, and
/// `readdir` on a 64-bit CPU, fill) or Darwin's 64-bit-inode one.
pub(in crate::interp) fn dirent_size(abi: PosixAbi) -> usize {
    if abi.darwin() {
        1048
    } else {
        280
    }
}

/// Fill the `struct dirent` at `out` for a program of `abi`. A name longer than the record's
/// is cut short.
pub(in crate::interp) fn fill_dirent(abi: PosixAbi, out: u64, entry: &DirEntry) {
    let size = dirent_size(abi);
    // `d_ino`, `d_off`/`d_seekoff` and `d_reclen` come first in both; Darwin adds `d_namlen`.
    // `d_name` holds 1024 bytes on Darwin and 256 on Linux, its last a zero.
    let (kind_at, name_at, capacity) = if abi.darwin() {
        (20, 21, 1023)
    } else {
        (18, 19, 255)
    };
    let name = &entry.name[..entry.name.len().min(capacity)];

    fill(out, 0, size);
    write_uint(out, 8, entry.inode);
    write_uint(out + 8, 8, entry.next);
    write_uint(out + 16, 2, size as u64);

    if abi.darwin() {
        write_uint(out + 18, 2, name.len() as u64);
    }

    write_uint(out + kind_at, 1, u64::from(entry.kind));
    write_bytes(out + name_at, name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::{Target, TargetOs};

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
            let members = stat_members(PosixAbi::new(os == TargetOs::MacOS, Some(cpu)));
            let ints: Vec<Member> = members.iter().map(|&(_, b)| Member::Int(b)).collect();

            assert_eq!(layout.pack(&ints).size, size, "{os:?} {cpu:?}");
        }
    }
}
