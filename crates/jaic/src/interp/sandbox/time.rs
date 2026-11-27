//! The virtual clock.

use super::guest::{layout, read_uint, write_uint, write_word};
use super::{LibcFn, SandboxHost, fill, read_i64, read_u64, write_i32, write_u64};
use crate::layout::Member;

impl SandboxHost {
    /// The virtual clock: sleeping, reading the time, and splitting it into a `struct tm`.
    pub(super) fn time_call(&mut self, fun: LibcFn, arg: impl Fn(usize) -> u64) -> Option<u64> {
        Some(match fun {
            LibcFn::Nanosleep | LibcFn::Nanosleep64 => {
                // struct timespec { time_t tv_sec; long tv_nsec }: sleeping only moves the
                // virtual clock. `time_t` is 64 bits on every target the sandbox serves.
                let req = arg(0);
                if req != 0 {
                    let secs = read_u64(req);
                    let nanos = read_uint(req + 8, layout().c.time_long);
                    self.clock_ns = self
                        .clock_ns
                        .saturating_add(secs.saturating_mul(1_000_000_000))
                        .saturating_add(nanos);
                }
                0
            }
            LibcFn::Usleep => {
                self.clock_ns += (arg(0) as u32 as u64) * 1_000;
                0
            }
            LibcFn::Sleep => {
                self.clock_ns += (arg(0) as u32 as u64) * 1_000_000_000;
                0
            }
            LibcFn::ClockGettime | LibcFn::ClockGettime64 => {
                // A fixed epoch plus the virtual clock keeps runs deterministic.
                self.clock_ns += 1_000;
                let ns = self.clock_ns + 1_700_000_000u64 * 1_000_000_000;
                let out = arg(1);
                if out == 0 {
                    return Some(u64::MAX);
                }
                write_u64(out, ns / 1_000_000_000);
                write_uint(out + 8, layout().c.time_long, ns % 1_000_000_000);
                0
            }
            LibcFn::Gettimeofday | LibcFn::Gettimeofday64 => {
                // struct timeval { time_t tv_sec; suseconds_t tv_usec }: both 64 bits on every
                // target the sandbox serves (32-bit Linux links the 64-bit time entry points).
                self.clock_ns += 1_000;
                let ns = self.clock_ns + 1_700_000_000u64 * 1_000_000_000;
                if arg(0) != 0 {
                    write_u64(arg(0), ns / 1_000_000_000);
                    write_u64(arg(0) + 8, ns % 1_000_000_000 / 1_000);
                }
                0
            }
            LibcFn::Time | LibcFn::Time64 => {
                self.clock_ns += 1_000;
                let secs = 1_700_000_000u64 + self.clock_ns / 1_000_000_000;
                if arg(0) != 0 {
                    write_u64(arg(0), secs);
                }
                secs
            }
            LibcFn::GmtimeR | LibcFn::LocaltimeR | LibcFn::GmtimeR64 | LibcFn::LocaltimeR64 => {
                let seconds = read_i64(arg(0));
                fill_tm(seconds, arg(1), layout().c.time_long);
                arg(1)
            }
            _ => return None,
        })
    }
}

/// Days since 1970-01-01 to (year, month 1-12, day).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 {
        mp + 3
    } else {
        mp - 9
    };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// `gmtime_r`/`localtime_r` (the sandbox's zone is UTC): fill a `struct tm`, nine `int`s then
/// a `long tm_gmtoff` of `long` bytes and `const char *tm_zone`, at the program's layout.
fn fill_tm(seconds: i64, out: u64, long: u8) {
    let days = seconds.div_euclid(86_400);
    let rem = seconds.rem_euclid(86_400);
    let (year, month, day) = civil(days);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    const CUMULATIVE: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let yday = CUMULATIVE[(month - 1) as usize] + day - 1 + i64::from(leap && month > 2);
    let fields = [
        rem % 60,
        rem / 60 % 60,
        rem / 3600,
        day,
        month - 1,
        year - 1900,
        (days + 4).rem_euclid(7),
        yday,
        0,
    ];
    let layout = layout();
    let mut members = [Member::Int(4); 11];
    members[9] = Member::Int(long);
    members[10] = Member::Ptr;
    let record = layout.pack(&members);
    fill(out, 0, record.size as usize);
    for (offset, v) in record.offsets.iter().zip(fields) {
        write_i32(out + offset, v as i32);
    }
    write_uint(out + record.offsets[9], long, 0); // tm_gmtoff
    write_word(out + record.offsets[10], 0); // tm_zone
}
