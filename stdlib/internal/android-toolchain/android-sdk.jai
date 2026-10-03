#scope_module
Android_Ndk_Selection :: struct {
    path: string;
    version: [4] u64;
}

android_ndk_host_tag :: () -> string {
    #if BUILD_OS == .WINDOWS return "windows-x86_64";
    else #if BUILD_OS == .MACOS return "darwin-x86_64";
    else return "linux-x86_64";
}

android_ndk_compiler_suffix :: () -> string {
    #if BUILD_OS == .WINDOWS return "bin/clang.exe";
    else return "bin/clang";
}

android_ndk_directory :: (path: string) -> bool {
    directory, success := Android_Ndk_Files.is_directory(path);
    return directory && success;
}

android_ndk_valid :: (root: string) -> bool {
    if root.count == 0 return false;

    toolchain := tool_path_join(root, tprint("toolchains/llvm/prebuilt/%", android_ndk_host_tag()));

    defer free(toolchain);
    sysroot := tool_path_join(toolchain, "sysroot");

    defer free(sysroot);
    compiler := tool_path_join(toolchain, android_ndk_compiler_suffix());

    defer free(compiler);

    return android_ndk_directory(sysroot) && Android_Ndk_Files.file_exists(compiler);
}

android_select_ndk :: (info: *Android_Ndk_Files.File_Visit_Info, selected: *Android_Ndk_Selection) {
    if info.had_error || !info.is_directory || !android_ndk_valid(info.full_name) return;

    version: [4] u64;
    part := 0;

    for byte: info.short_name {
        if byte == #char "." {
            part += 1;
            if part >= 4 return;
        } else if byte >= #char "0" && byte <= #char "9" {
            if version[part] > 1000000000 return;
            version[part] = version[part] * 10 + cast(u64) (byte - #char "0");
        } else return;
    }

    newer := selected.path.count == 0;

    for index: 0..3 {
        if version[index] > selected.version[index] {
            newer = true;
            break;
        }

        if version[index] < selected.version[index] break;
    }

    if newer {
        free(selected.path);
        selected.path = copy_string(info.full_name);
        selected.version = version;
    }
}

android_release_ndk :: (paths: NDK_Paths) {
    free(paths.root);
    free(paths.toolchain);
    free(paths.sysroot);
}

#scope_file
#import "Basic";
Android_Ndk_Files :: #import "File_Utilities";
