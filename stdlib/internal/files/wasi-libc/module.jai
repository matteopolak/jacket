// C ABI supplied by Extensions/Wasi_Runtime and the interpreter's sandbox.
// Streams and directories are opaque: no host POSIX aggregate layout is assumed.
FILE :: struct {};
DIR :: struct {};
mode_t :: u32;

SEEK_SET :: 0;
SEEK_CUR :: 1;
SEEK_END :: 2;

fopen :: (name: *u8, mode: *u8) -> *FILE #foreign libc;
fclose :: (file: *FILE) -> s32 #foreign libc;
fread :: (data: *void, size: u64, count: u64, file: *FILE) -> u64 #foreign libc;
fwrite :: (data: *void, size: u64, count: u64, file: *FILE) -> u64 #foreign libc;
fflush :: (file: *FILE) -> s32 #foreign libc;
ferror :: (file: *FILE) -> s32 #foreign libc;
fseeko :: (file: *FILE, offset: s64, origin: s32) -> s32 #foreign libc;
ftello :: (file: *FILE) -> s64 #foreign libc;

rename :: (old: *u8, new: *u8) -> s32 #foreign libc;
unlink :: (name: *u8) -> s32 #foreign libc;
link :: (target: *u8, name: *u8) -> s32 #foreign libc;
mkdir :: (name: *u8, mode: mode_t) -> s32 #foreign libc;
rmdir :: (name: *u8) -> s32 #foreign libc;
opendir :: (name: *u8) -> *DIR #foreign libc;
closedir :: (directory: *DIR) -> s32 #foreign libc;
realpath :: (name: *u8, result: *u8) -> *u8 #foreign libc;

time_t :: s64;

// CLOCK_REALTIME and CLOCK_MONOTONIC have the same numbers in WASI as in Linux.
clockid_t :: enum s32 {
    REALTIME :: 0;
    MONOTONIC :: 1;
}

timespec :: struct {
    tv_sec: time_t;
    tv_nsec: s64;
}

tm :: struct {
    tm_sec: s32;
    tm_min: s32;
    tm_hour: s32;
    tm_mday: s32;
    tm_mon: s32;
    tm_year: s32;
    tm_wday: s32;
    tm_yday: s32;
    tm_isdst: s32;
    tm_gmtoff: s64;
    tm_zone: *u8;
}

clock_gettime :: (clock: clockid_t, result: *timespec) -> s32 #foreign libc;
nanosleep :: (requested: *timespec, remaining: *timespec) -> s32 #foreign libc;
gmtime_r :: (time: *time_t, result: *tm) -> *tm #foreign libc;
localtime_r :: (time: *time_t, result: *tm) -> *tm #foreign libc;
exit :: (status: s32) #foreign libc;

// Neither host has time zones, so local time is UTC and `mktime` is `timegm`. Out-of-range
// fields carry into the larger ones, as in C.
timegm :: (broken: *tm) -> time_t {
    year := cast(s64) broken.tm_year + TM_YEAR_BASE;
    month := cast(s64) broken.tm_mon;

    year += month / MONTHS_PER_YEAR;
    month %= MONTHS_PER_YEAR;
    if month < 0 {
        month += MONTHS_PER_YEAR;
        year -= 1;
    }

    // Days since 1970-01-01 of the first of the month, counting years from March 1 so the
    // leap day ends each year.
    if month < 2 year -= 1;

    era := ifx year >= 0 then year / YEARS_PER_ERA else (year - YEARS_PER_ERA - 1) / YEARS_PER_ERA;
    year_of_era := year - era * YEARS_PER_ERA;
    month_from_march := ifx month >= 2 then month - 2 else month + 10;
    day_of_year := (DAYS_PER_MARCH_CYCLE * month_from_march + 2) / MONTHS_PER_MARCH_CYCLE;
    day_of_era := year_of_era * DAYS_PER_YEAR + year_of_era / 4 - year_of_era / 100 + day_of_year;
    days := era * DAYS_PER_ERA + day_of_era - DAYS_FROM_YEAR_ZERO_TO_EPOCH + cast(s64) broken.tm_mday - 1;

    return days * SECONDS_PER_DAY + cast(s64) broken.tm_hour * SECONDS_PER_HOUR + cast(s64) broken.tm_min * 60 + broken.tm_sec;
}

mktime :: (broken: *tm) -> time_t {
    return timegm(broken);
}

#scope_file

TM_YEAR_BASE :: 1900;
MONTHS_PER_YEAR :: 12;
YEARS_PER_ERA :: 400;
DAYS_PER_ERA :: 146097;
DAYS_PER_YEAR :: 365;
DAYS_PER_MARCH_CYCLE :: 153;
MONTHS_PER_MARCH_CYCLE :: 5;
DAYS_FROM_YEAR_ZERO_TO_EPOCH :: 719468;
SECONDS_PER_DAY :: 86400;
SECONDS_PER_HOUR :: 3600;

libc :: #system_library "libc"; @native(external)
