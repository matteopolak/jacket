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

#scope_file
libc :: #system_library "libc";
