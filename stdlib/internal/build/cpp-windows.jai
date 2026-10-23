#scope_module
// Discover native MSVC or Clang without a developer prompt. Run tools directly:
// source paths and flags never pass through cmd.
default_compiler :: (cpp: bool) -> string {
    native := find_msvc_compiler();
    if native.count > 0 return native;
    return default_clang_compiler(cpp);
}

default_clang_compiler :: (_cpp: bool) -> string {
    name := "clang-cl.exe";

    search_path := environment_tool("PATH", "");
    directories := S.split(search_path, ";");

    defer free(directories.data);

    for directory: directories {
        if directory.count == 0 continue;
        candidate := tprint("%/%", directory, name);
        if U.file_exists(candidate) return candidate;
    }

    for variable: string.["ProgramFiles", "ProgramFiles(x86)"] {
        base := environment_tool(variable, "");
        if base.count == 0 continue;
        candidate := tprint("%/LLVM/bin/%", base, name);
        if U.file_exists(candidate) return candidate;
    }

    installer := environment_tool("ProgramFiles(x86)", "C:/Program Files (x86)");
    vswhere := tprint("%/Microsoft Visual Studio/Installer/vswhere.exe", installer);

    if U.file_exists(vswhere) {
        result, output, errors, timed := Process.run_command(vswhere, "-utf8", "-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Llvm.Clang", "-property", "installationPath", capture_and_return_output = true);

        defer free(output);
        defer free(errors);

        if result.type == .EXITED && result.exit_code == 0 && !timed {
            root := S.trim(output);

            // The directory holds a compiler that runs on this host; --target selects the output CPU.
            candidate := tprint("%/VC/Tools/Llvm/%/bin/%", root, ifx HOST_ARM64 then "ARM64" else "x64", name);

            if U.file_exists(candidate) return candidate;

            candidate = tprint("%/VC/Tools/Llvm/bin/%", root, name);
            if U.file_exists(candidate) return candidate;
        }
    }

    return name;
}

default_archiver :: () -> string {
    compiler := default_compiler(false);
    directory := S.path_strip_filename(compiler);

    if is_native_msvc(compiler) return tprint("%/lib.exe", directory);
    if directory.count > 0 return tprint("%/llvm-lib.exe", directory);

    return "llvm-lib.exe";
}

environment_tool :: (name: string, fallback: string) -> string {
    key, converted := Wide.utf8_to_wide(name);

    defer free(key);
    if !converted return fallback;

    count := get_environment(key, null, 0);
    if count == 0 return fallback;

    buffer := cast(*u16) alloc(cast(s64) count * 2);

    defer free(buffer);
    received := get_environment(key, buffer, count);
    if received == 0 || received >= count return fallback;

    value, ok := Wide.wide_to_utf8(buffer, cast(s32) received);

    defer free(value);
    if !ok return fallback;

    return copy_temporary_string(value);
}

create_intermediate_directory :: (basename: string) -> string, bool {
    for 0..63 {
        random: [4] u32;
        if random_bytes(null, cast(*u8) random.data, cast(u32) size_of(random), 2) < 0 return .{}, false;

        path := sprint("%.build-%-%-%-%", basename, random[0], random[1], random[2], random[3]);
        wide, converted := Wide.utf8_to_wide(path);

        defer free(wide);
        if converted && Native.CreateDirectoryW(wide, null) != 0 return path, true;

        error := Native.GetLastError();

        free(path);
        if !converted || cast(u32) error != WIN32_ALREADY_EXISTS return .{}, false;
    }

    return .{}, false;
}

cleanup_file :: (path: string, failure: *string) -> bool {
    wide, converted := Wide.utf8_to_wide(path);

    defer free(wide);

    if converted {
        if Native.DeleteFileW(wide) != 0 return true;
        error := cast(u32) Native.GetLastError();
        if error == 2 || error == 3 return true;
    }

    return remember_failure("BuildCpp intermediate file cleanup failed", failure);
}

// MSVC and clang-cl share option syntax; GNU-style Clang remains available
// through explicit CC/CXX/compiler paths without rewriting caller arguments.
is_msvc_driver :: (compiler: string) -> bool {
    name := S.path_filename(compiler);
    return S.compare_nocase(name, "cl.exe") == 0 || S.compare_nocase(name, "cl") == 0 ||
           S.compare_nocase(name, "clang-cl.exe") == 0 || S.compare_nocase(name, "clang-cl") == 0;
}

is_native_msvc :: (compiler: string) -> bool {
    name := S.path_filename(compiler);
    return S.compare_nocase(name, "cl.exe") == 0 || S.compare_nocase(name, "cl") == 0;
}

find_msvc_compiler :: () -> string {
    // A developer prompt already provides the correctly configured compiler.
    if environment_tool("VCToolsInstallDir", "").count > 0 {
        for host: string.["Hostarm64", "Hostx64"] {
            if !HOST_ARM64 && host == "Hostarm64" continue;
            candidate := tprint("%/bin/%/%/cl.exe", environment_tool("VCToolsInstallDir", ""), host, windows_arch());
            if U.file_exists(candidate) return candidate;
        }
    }

    installer := environment_tool("ProgramFiles(x86)", "C:/Program Files (x86)");
    vswhere := tprint("%/Microsoft Visual Studio/Installer/vswhere.exe", installer);

    if !U.file_exists(vswhere) return "";

    component := "Microsoft.VisualStudio.Component.VC.Tools.x86.x64";

    #if CPU == .ARM64 component = "Microsoft.VisualStudio.Component.VC.Tools.ARM64";

    for host: string.["Hostarm64", "Hostx64"] {
        // Hostarm64 executables cannot run on an x64 host.
        if !HOST_ARM64 && host == "Hostarm64" continue;

        pattern := tprint("VC\\Tools\\MSVC\\**\\bin\\%\\%\\cl.exe", host, windows_arch());
        result, output, errors, timed := Process.run_command(vswhere, "-utf8", "-latest", "-products", "*", "-requires", component, "-find", pattern, capture_and_return_output = true);

        defer free(output);
        defer free(errors);
        if result.type != .EXITED || result.exit_code != 0 || timed continue;

        paths := S.split(output, "\n");

        defer free(paths.data);
        best: string;

        for path: paths {
            candidate := S.trim(path);
            if U.file_exists(candidate) && compare_versions(candidate, best) > 0 best = candidate;
        }

        if best.count > 0 return copy_temporary_string(best);
    }

    return "";
}

// Orders version strings and paths that contain them: runs of digits compare as numbers, so
// `14.100` is newer than `14.99`.
compare_versions :: (a: string, b: string) -> int {
    digit :: (c: u8) -> bool {
        return c >= #char "0" && c <= #char "9";
    }

    i, j := 0, 0;

    while i < a.count && j < b.count {
        if digit(a[i]) && digit(b[j]) {
            // Skip leading zeros, then the longer run is the larger number.
            while i < a.count && a[i] == #char "0" i += 1;
            while j < b.count && b[j] == #char "0" j += 1;

            x_end, y_end := i, j;
            while x_end < a.count && digit(a[x_end]) x_end += 1;
            while y_end < b.count && digit(b[y_end]) y_end += 1;
            if x_end - i != y_end - j return ifx x_end - i > y_end - j then 1 else -1;

            while i < x_end {
                if a[i] != b[j] return ifx a[i] > b[j] then 1 else -1;
                i += 1;
                j += 1;
            }

            continue;
        }

        if a[i] != b[j] return ifx a[i] > b[j] then 1 else -1;

        i += 1;
        j += 1;
    }

    if i < a.count return 1;
    if j < b.count return -1;

    return 0;
}

windows_arch :: () -> string {
    #if CPU == .ARM64 return "arm64";
    else return "x64";
}

append_windows_target :: (args: *[..] string, compiler: string) {
    if is_native_msvc(compiler) return;
    #if CPU == .ARM64 array_add(args, "--target=aarch64-pc-windows-msvc");
    else array_add(args, "--target=x86_64-pc-windows-msvc");
}

append_msvc_sdk :: (args: *[..] string, compiler: string, link: bool) -> bool {
    if !is_native_msvc(compiler) return true; // Clang performs SDK discovery itself.

    // A fully qualified cl path identifies its own C/C++ headers and libraries.
    root := environment_tool("VCToolsInstallDir", "");

    if S.path_strip_filename(compiler).count > 0 {
        resolved, ok := F.get_absolute_path(tprint("%/../../..", S.path_strip_filename(compiler)));
        defer free(resolved);
        if !ok return false;
        root = copy_temporary_string(resolved);
    }

    if root.count == 0 return false;

    sdk := environment_tool("WindowsSdkDir", "");
    if sdk.count == 0 sdk = windows_sdk_root();
    if sdk.count == 0 return false;

    selection: SDK_Selection = .{ root = sdk };
    if !U.visit_files(tprint("%/Include", sdk), false, *selection, select_sdk, visit_files = false, visit_directories = true) return false;

    defer free(selection.version);
    if selection.version.count == 0 return false;

    version := selection.version;

    if link {
        array_add(args, tprint("/LIBPATH:%/lib/%", root, windows_arch()));
        array_add(args, tprint("/LIBPATH:%/Lib/%/um/%", sdk, version, windows_arch()));
        array_add(args, tprint("/LIBPATH:%/Lib/%/ucrt/%", sdk, version, windows_arch()));
    } else {
        array_add(args, tprint("/I%/include", root));

        for directory: string.["ucrt", "shared", "um", "winrt", "cppwinrt"] {
            array_add(args, tprint("/I%/Include/%/%", sdk, version, directory));
        }
    }

    return true;
}

SDK_Selection :: struct {
    root: string;
    version: string;
}

select_sdk :: (info: *U.File_Visit_Info, selection: *SDK_Selection) {
    if !info.is_directory return;

    name := S.path_filename(info.full_name);
    // Require matching headers and target libraries, not just a version folder.
    if !U.file_exists(tprint("%/Include/%/um/Windows.h", selection.root, name)) ||
       !U.file_exists(tprint("%/Include/%/ucrt/stdio.h", selection.root, name)) ||
       !U.file_exists(tprint("%/Lib/%/um/%/kernel32.lib", selection.root, name, windows_arch())) ||
       !U.file_exists(tprint("%/Lib/%/ucrt/%/ucrt.lib", selection.root, name, windows_arch())) return;

    if compare_versions(name, selection.version) > 0 {
        free(selection.version);
        selection.version = copy_string(name);
    }
}

windows_sdk_root :: () -> string {
    subkey, ok := Wide.utf8_to_wide("SOFTWARE\\Microsoft\\Windows Kits\\Installed Roots");

    defer free(subkey);
    if !ok return "";

    value, converted := Wide.utf8_to_wide("KitsRoot10");

    defer free(value);
    if !converted return "";

    for view: u32.[0x10000, 0x20000] {
        bytes: u32;
        if get_registry(cast(*void) 0xffffffff80000002, subkey, value, 2 | view, null, null, *bytes) != 0 || bytes < 2 continue;

        data := cast(*u16) alloc(bytes);

        defer free(data);
        if get_registry(cast(*void) 0xffffffff80000002, subkey, value, 2 | view, null, data, *bytes) != 0 continue;

        result, read := Wide.wide_to_utf8(data);

        defer free(result);
        if read return copy_temporary_string(result);
    }

    return "";
}

HOST_ARM64 :: #run,host -> bool {
    return CPU == .ARM64;
};

append_compile_arguments :: (args: *[..] string, compiler: string, file: string, object: string, type: Build_Type, debug: bool, extra: [] string) -> bool {
    append_windows_target(args, compiler);

    if is_msvc_driver(compiler) {
        array_add(args, "/nologo");
        array_add(args, "/MD");

        if debug {
            array_add(args, "/Od");
            array_add(args, "/Z7");
        } else array_add(args, "/O2");

        if is_cpp(file) array_add(args, "/TP");
        else array_add(args, "/TC");
        if !append_msvc_sdk(args, compiler, false) return false;
        for arg: extra array_add(args, arg);

        array_add(args, "/c");
        array_add(args, file);
        array_add(args, tprint("/Fo%", object));
    } else {
        if debug {
            array_add(args, "-O0");
            array_add(args, "-g");
        } else array_add(args, "-O2");

        for arg: extra array_add(args, arg);

        array_add(args, "-c");
        array_add(args, file);
        array_add(args, "-o");
        array_add(args, object);
    }

    return true;
}

append_link_arguments :: (args: *[..] string, compiler: string, staged: string, intermediates: string, type: Build_Type, debug: bool, extra: [] string, objects: [] string, libraries: [] string) -> bool {
    append_windows_target(args, compiler);

    if is_msvc_driver(compiler) {
        array_add(args, "/nologo");
        array_add(args, "/MD");
        if type == .DYNAMIC_LIBRARY array_add(args, "/LD");
        for arg: extra array_add(args, arg);

        array_add(args, tprint("/Fe%", staged));
        // Objects must precede /link, which forwards the remaining arguments to LINK.
        for object: objects array_add(args, object);
        for library: libraries array_add(args, library);

        array_add(args, "/link");
        if !append_msvc_sdk(args, compiler, true) return false;

        array_add(args, "/NOLOGO");
        array_add(args, "/INCREMENTAL:NO");
        array_add(args, tprint("/IMPLIB:%/import.lib", intermediates));

        if debug {
            array_add(args, "/DEBUG");
            array_add(args, tprint("/PDB:%.pdb", S.path_strip_extension(staged)));
        }
    } else {
        array_add(args, "-Xlinker");
        array_add(args, tprint("/IMPLIB:%/import.lib", intermediates));
        array_add(args, "-Xlinker");
        array_add(args, "/INCREMENTAL:NO");

        if debug {
            array_add(args, "-g");
            array_add(args, "-Xlinker");
            array_add(args, tprint("/PDB:%.pdb", S.path_strip_extension(staged)));
        }

        if type == .DYNAMIC_LIBRARY array_add(args, "-shared");
        for arg: extra array_add(args, arg);

        array_add(args, "-o");
        array_add(args, staged);
        for object: objects array_add(args, object);
        for library: libraries array_add(args, library);
    }

    return true;
}

#scope_file
WIN32_ALREADY_EXISTS :: 183;

kernel :: #system_library "kernel32";
bcrypt :: #system_library "bcrypt";
get_environment :: (name: *u16, buffer: *u16, capacity: u32) -> u32 #foreign kernel "GetEnvironmentVariableW";
random_bytes :: (algorithm: *void, buffer: *u8, count: u32, flags: u32) -> s32 #foreign bcrypt "BCryptGenRandom";

#scope_file
#import "Basic";
Native :: #import "Windows";
Wide :: #import "Windows_Utf8";
U :: #import "File_Utilities";
S :: #import "String";
Process :: #import "Process";

#scope_file
F :: #import "File";

advapi :: #system_library "advapi32";
get_registry :: (key: *void, subkey: *u16, value: *u16, flags: u32, kind: *u32, data: *void, bytes: *u32) -> s32 #foreign advapi "RegGetValueW";
