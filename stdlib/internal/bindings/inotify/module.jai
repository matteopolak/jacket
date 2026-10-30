// The inotify calls and event bits File_Watcher needs, declared here so the watcher never
// depends on a Linux module a project may have replaced.
#assert OS == .LINUX, "inotify requires its native platform";

IN_MODIFY :: 0x00000002;
IN_ATTRIB :: 0x00000004;
IN_CLOSE_WRITE :: 0x00000008;
IN_MOVED_FROM :: 0x00000040;
IN_MOVED_TO :: 0x00000080;
IN_CREATE :: 0x00000100;
IN_DELETE :: 0x00000200;
IN_DELETE_SELF :: 0x00000400;
IN_MOVE_SELF :: 0x00000800;
IN_UNMOUNT :: 0x00002000;
IN_Q_OVERFLOW :: 0x00004000;
IN_IGNORED :: 0x00008000;
IN_ONLYDIR :: 0x01000000;
IN_DONT_FOLLOW :: 0x02000000;
IN_ISDIR :: 0x40000000;

IN_NONBLOCK :: 0x00800;
IN_CLOEXEC :: 0x80000;

inotify_init :: (flags: s32 = 0) -> s32 #foreign libc "inotify_init1";
inotify_add_watch :: (descriptor: s32, path: *u8, mask: u32) -> s32 #foreign libc;
inotify_rm_watch :: (descriptor: s32, watch: s32) -> s32 #foreign libc;

#scope_file
libc :: #library,system "c"; @native(external)
