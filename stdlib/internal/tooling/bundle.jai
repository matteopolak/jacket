make_app_bundle_subdirectories :: (app_bundle_name: string) -> macos_folder_path: string, resources_path: string {
    #if OS == .MACOS {
        macos := B.sprint("%/Contents/MacOS", app_bundle_name);
        resources := B.sprint("%/Contents/Resources", app_bundle_name);

        if execute(string.["/bin/mkdir", "-p", macos, resources]) return macos, resources;

        B.free(macos);
        B.free(resources);

        return "", "";
    } else {
        B.assert(false, "MacOS bundle filesystem operations require macOS");
        return "", "";
    }
}

create_app_bundle :: (app_name: string, app_exe_path: string, icon_file_path: string, resource_directory_to_copy: string, only_copy_subdirectories_and_files: bool, write_plist: bool) -> app_bundle_dir: string, resources_path: string {
    #if OS == .MACOS {
        if app_name.count == 0 || app_exe_path.count == 0 {
            report("bundle requires an application name and executable path\n");
            return "", "";
        }

        bundle: string;
        if S.ends_with(app_name, ".app") bundle = B.copy_string(app_name);
        else bundle = B.sprint("%.app", app_name);

        macos, resources := make_app_bundle_subdirectories(bundle);
        complete := false;

        defer B.free(macos);

        defer {
            if !complete {
                B.free(bundle);
                B.free(resources);
            }
        }

        if macos.count == 0 return "", "";

        executable := tool_path(app_exe_path);

        defer B.free(executable);
        if !execute(string.["/bin/cp", "-p", executable, macos]) return "", "";

        if resource_directory_to_copy.count > 0 {
            source := tool_path(resource_directory_to_copy);

            defer B.free(source);

            if only_copy_subdirectories_and_files {
                contents := B.sprint("%/.", source);
                defer B.free(contents);
                if !execute(string.["/bin/cp", "-Rp", contents, resources]) return "", "";
            } else if !execute(string.["/bin/cp", "-Rp", source, resources]) return "", "";
        }

        if icon_file_path.count > 0 {
            target := B.sprint("%/Icon.icns", resources);

            defer B.free(target);

            if S.ends_with(icon_file_path, ".icns") {
                source := tool_path(icon_file_path);
                defer B.free(source);
                if !execute(string.["/bin/cp", "-p", source, target]) return "", "";
            } else if !convert_image_to_icns(icon_file_path, target) return "", "";
        }

        if write_plist {
            builder: B.String_Builder;

            defer B.free_buffers(*builder);
            plist_start(*builder);
            if !plist_write_key_value(*builder, "CFBundleExecutable", S.path_filename(app_exe_path)) return "", "";
            if !plist_write_key_value(*builder, "CFBundleName", S.path_filename(app_name)) return "", "";
            if !plist_write_key_value(*builder, "CFBundlePackageType", "APPL") return "", "";
            if icon_file_path.count > 0 && !plist_write_key_value(*builder, "CFBundleIconFile", "Icon.icns") return "", "";

            plist_end(*builder);
            data := B.builder_to_string(*builder);

            defer B.free(data);
            path := B.sprint("%/Contents/Info.plist", bundle);

            defer B.free(path);

            if !F.write_entire_file(path, data) {
                report("bundle property-list write failed\n");
                return "", "";
            }
        }

        complete = true;

        return bundle, resources;
    } else {
        B.assert(false, "MacOS bundle filesystem operations require macOS");
        return "", "";
    }
}

convert_image_to_icns :: (input_path: string, output_path: string) -> bool {
    #if OS == .MACOS {
        pattern := B.sprint("%.work-XXXXXX", output_path);

        defer B.free(pattern);
        native := B.to_c_string(pattern);

        defer B.free(native);

        if !Native.mkdtemp(native) {
            report("icon conversion temporary directory creation failed\n");
            return false;
        }

        temporary := B.to_string(native, pattern.count);
        result := convert_icon(input_path, output_path, temporary);

        // Cleanup failure is observable failure even if the output was produced.
        if !execute(string.["/bin/rm", "-rf", temporary]) return false;

        return result;
    } else {
        B.assert(false, "MacOS icon conversion requires macOS");
        return false;
    }
}

#scope_file
B :: #import "Basic";

#if OS == .MACOS {
    S :: #import "String";
    F :: #import "File";
    P :: #import "Process";
    Native :: #import "POSIX";

    tool_path :: (path: string) -> string {
        if S.is_absolute_path(path) return B.copy_string(path);
        return B.sprint("./%", path);
    }

    report :: (message: string) {
        context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    }

    execute :: (args: [] string) -> bool {
        status, output, error, timed := P.run_command(..args, capture_and_return_output = true);

        defer B.free(output);
        defer B.free(error);
        if status.type == .EXITED && status.exit_code == 0 && !timed return true;

        report("bundle tool failed: ");
        report(args[0]);
        report("\n");
        if output.count > 0 report(output);
        if error.count > 0 report(error);

        return false;
    }

    convert_icon :: (input_path: string, output_path: string, temporary: string) -> bool {
        iconset := B.sprint("%/App.iconset", temporary);

        defer B.free(iconset);
        if !execute(string.["/bin/mkdir", "-p", iconset]) return false;

        input := tool_path(input_path);
        output := tool_path(output_path);

        defer B.free(input);
        defer B.free(output);

        for size: s64.[16, 32, 128, 256, 512] {
            for scale: s64.[1, 2] {
                filename: string;
                if scale == 1 filename = B.sprint("%/icon_%x%.png", iconset, size, size);
                else filename = B.sprint("%/icon_%x%@2x.png", iconset, size, size);

                defer B.free(filename);
                pixels := B.sprint("%", size * scale);

                defer B.free(pixels);
                if !execute(string.["/usr/bin/sips", "-z", pixels, pixels, input, "--out", filename]) return false;
            }
        }

        return execute(string.["/usr/bin/iconutil", "-c", "icns", iconset, "-o", output]);
    }
}
