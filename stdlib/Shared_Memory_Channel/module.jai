#assert OS == .MACOS || OS == .LINUX || OS == .WINDOWS, "Shared_Memory_Channel requires native shared-memory mappings";

#load "../internal/ipc/channel-types.jai";
#load "../internal/ipc/channel.jai";
#if OS == .WINDOWS #load "../internal/ipc/channel-windows.jai";
else #load "../internal/ipc/channel-posix.jai";
