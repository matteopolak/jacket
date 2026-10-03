#scope_module

pg_xml :: (value: string) -> string {
    b: String_Builder;

    defer free_buffers(*b);

    for byte: value {
        if byte == 38 append(*b, "&amp;");
        else if byte == 60 append(*b, "&lt;");
        else if byte == 62 append(*b, "&gt;");
        else if byte == 34 append(*b, "&quot;");
        else if byte == 39 append(*b, "&apos;");
        else append(*b, byte);
    }

    return builder_to_string(*b);
}

pg_windows_path :: (path: string) -> string {
    result := copy_string(path);
    PG_String.replace_chars(result, "/", 92);
    if result.count > 1 && result[1] == 58 && result[0] >= 97 && result[0] <= 122 result[0] -= 32;
    return result;
}

pg_identifier :: (name: string) -> string {
    digest := PG_SHA.sha256(name);
    digest[6] = (digest[6] & 15) | 128;
    digest[8] = (digest[8] & 63) | 128;
    result := alloc_string(38);
    result[0] = 123;
    result[37] = 125;
    digits := "0123456789ABCDEF";
    at: s64 = 1;

    for byte, index: digest {
        if index == 16 break;

        if index == 4 || index == 6 || index == 8 || index == 10 {
            result[at] = 45;
            at += 1;
        }

        result[at] = digits[byte >> 4];
        result[at + 1] = digits[byte & 15];
        at += 2;
    }

    return result;
}

pg_write_if_changed :: (name: string, contents: string) {
    previous, found := PG_File.read_entire_file(name, log_errors = false);
    if found {
        same := previous == contents;
        free(previous);
        if same return;
    }

    if !PG_File.write_entire_file(name, contents) pg_fail("Project generator output file could not be written");
}

pg_write_builder :: (suffix: string, b: *String_Builder) {
    filename := PG_String.join(Project_Generator.project_name, suffix);
    data := builder_to_string(b);

    defer {
        free(filename);
        free(data);
    }

    pg_write_if_changed(filename, data);
}

pg_property :: (b: *String_Builder, name: string, value: string) {
    escaped := pg_xml(value);
    defer free(escaped);
    print_to_builder(b, "    <%>%</%>\n", name, escaped, name);
}

pg_file_tag :: (path: string) -> string {
    if PG_String.ends_with_nocase(path, ".natvis") return "Natvis";
    return "None";
}

pg_has_parent_segment :: (path: string) -> bool {
    begin: s64;

    for byte, index: path if byte == 92 {
        if PG_String.slice(path, begin, index - begin) == ".." return true;
        begin = index + 1;
    }

    return PG_String.slice(path, begin, path.count - begin) == "..";
}

PG_XML_HEADER :: "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<Project ToolsVersion=\"Current\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n";

pg_emit :: () {
    cwd := get_working_directory();
    windows_cwd := pg_windows_path(cwd);
    prefix := PG_String.join(windows_cwd, "\\");

    defer {
        free(cwd);
        free(windows_cwd);
        free(prefix);
    }

    for *path: Project_Generator.project_files {
        normalized := pg_windows_path(path.*);
        if PG_String.begins_with(normalized, prefix) {
            relative := copy_string(PG_String.slice(normalized, prefix.count, normalized.count - prefix.count));
            free(normalized);
            normalized = relative;
        }

        free(path.*);
        path.* = normalized;
    }

    PG_Sort.quick_sort(Project_Generator.project_files, PG_String.compare);
    identifier := pg_identifier(Project_Generator.project_name);

    defer free(identifier);
    pg_emit_solution(identifier);
    pg_emit_project(identifier);
    pg_emit_filters();
    pg_emit_user();
}

pg_emit_solution :: (identifier: string) {
    b: String_Builder;

    defer free_buffers(*b);
    append(*b, "Microsoft Visual Studio Solution File, Format Version 12.00\n# Visual Studio Version 17\n");
    // The project-type GUID identifies C++; the project instance ID is derived.
    print_to_builder(*b, "Project(\"{BC8A1FFA-BEE3-4634-8014-F334798102B3}\") = \"%\", \"%.vcxproj\", \"%\"\nEndProject\n",
        Project_Generator.project_name, Project_Generator.project_name, identifier);
    append(*b, "Global\n GlobalSection(SolutionConfigurationPlatforms) = preSolution\n  Release|x64 = Release|x64\n EndGlobalSection\n GlobalSection(ProjectConfigurationPlatforms) = postSolution\n");
    print_to_builder(*b, "  %.Release|x64.ActiveCfg = Release|x64\n  %.Release|x64.Build.0 = Release|x64\n", identifier, identifier);
    append(*b, " EndGlobalSection\nEndGlobal\n");
    pg_write_builder(".sln", *b);
}

pg_emit_project :: (identifier: string) {
    b: String_Builder;

    defer free_buffers(*b);
    append(*b, PG_XML_HEADER);
    append(*b, "  <ItemGroup Label=\"ProjectConfigurations\"><ProjectConfiguration Include=\"Release|x64\"><Configuration>Release</Configuration><Platform>x64</Platform></ProjectConfiguration></ItemGroup>\n  <PropertyGroup Label=\"Globals\">\n");
    pg_property(*b, "ProjectGuid", identifier);
    pg_property(*b, "Keyword", "MakeFileProj");
    append(*b, "  </PropertyGroup>\n  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.Default.props\" />\n  <PropertyGroup Label=\"Configuration\"><ConfigurationType>Makefile</ConfigurationType></PropertyGroup>\n  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.props\" />\n  <PropertyGroup>\n");
    command := PG_String.join("\"", Project_Generator.build_cmd, "\" ", Project_Generator.build_args);

    defer free(command);
    pg_property(*b, "NMakeBuildCommandLine", command);
    pg_property(*b, "NMakeReBuildCommandLine", command);
    pg_property(*b, "NMakeOutput", Project_Generator.debug_command);
    append(*b, "  </PropertyGroup>\n  <ItemGroup>\n");

    for path: Project_Generator.project_files {
        escaped := pg_xml(path);
        print_to_builder(*b, "    <% Include=\"%\" />\n", pg_file_tag(path), escaped);
        free(escaped);
    }

    append(*b, "  </ItemGroup>\n  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.targets\" />\n</Project>\n");
    pg_write_builder(".vcxproj", *b);
}

pg_emit_filters :: () {
    folders: [..] string;

    defer {
        for folder: folders free(folder);
        array_free(folders);
    }

    for path: Project_Generator.project_files {
        if pg_has_parent_segment(path) continue;

        for byte, index: path if byte == 92 && index > 0 {
            folder := PG_String.slice(path, 0, index);
            present := false;

            for known: folders if known == folder {
                present = true;
                break;
            }

            if !present array_add(*folders, copy_string(folder));
        }
    }

    PG_Sort.quick_sort(folders, PG_String.compare);
    b: String_Builder;

    defer free_buffers(*b);
    append(*b, PG_XML_HEADER);
    append(*b, "  <ItemGroup>\n");

    for folder: folders {
        escaped := pg_xml(folder);
        identifier := pg_identifier(folder);

        print_to_builder(*b, "    <Filter Include=\"%\"><UniqueIdentifier>%</UniqueIdentifier></Filter>\n", escaped, identifier);
        free(escaped);
        free(identifier);
    }

    append(*b, "  </ItemGroup>\n  <ItemGroup>\n");

    for path: Project_Generator.project_files {
        escaped := pg_xml(path);
        tag := pg_file_tag(path);
        folder := PG_String.path_strip_filename(path);

        if folder.count > 0 && !pg_has_parent_segment(path) {
            folder = PG_String.slice(folder, 0, folder.count - 1);
            escaped_folder := pg_xml(folder);
            print_to_builder(*b, "    <% Include=\"%\"><Filter>%</Filter></%>\n", tag, escaped, escaped_folder, tag);
            free(escaped_folder);
        } else print_to_builder(*b, "    <% Include=\"%\" />\n", tag, escaped);

        free(escaped);
    }

    append(*b, "  </ItemGroup>\n</Project>\n");
    pg_write_builder(".vcxproj.filters", *b);
}

pg_emit_user :: () {
    b: String_Builder;

    defer free_buffers(*b);
    append(*b, PG_XML_HEADER);
    append(*b, "  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='Release|x64'\">\n");
    pg_property(*b, "LocalDebuggerCommand", Project_Generator.debug_command);
    path := pg_windows_path(Project_Generator.working_directory);
    relative := path.count == 0 || (path[0] != 92 && !(path.count >= 3 && path[1] == 58));
    working := path;

    if relative working = PG_String.join("$(SolutionDir)", path);

    pg_property(*b, "LocalDebuggerWorkingDirectory", working);
    if relative free(working);

    free(path);
    pg_property(*b, "LocalDebuggerCommandArguments", Project_Generator.run_arguments);
    pg_property(*b, "DebuggerFlavor", "WindowsLocalDebugger");
    append(*b, "  </PropertyGroup>\n</Project>\n");
    pg_write_builder(".vcxproj.user", *b);
}
