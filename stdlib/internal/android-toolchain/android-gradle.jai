#scope_module
Gradle_File :: #import "File_Utilities";
Gradle_String :: #import "String";
Gradle_Process :: #import "Process";

android_gradle_task :: (project_path: string, task: string, print_output: bool) -> bool {
    if task.count == 0 return false;

    directory, valid := Gradle_File.is_directory(project_path);
    if !valid || !directory return false;

    #if OS == .WINDOWS wrapper_name := "gradlew.bat";
    else wrapper_name := "gradlew";
    wrapper := Gradle_String.path_join(project_path, wrapper_name);

    defer free(wrapper);
    arguments: [..] string;

    defer array_free(arguments);

    if Gradle_File.file_exists(wrapper) {
        #if OS == .WINDOWS {
            array_add(*arguments, "cmd.exe");
            array_add(*arguments, "/c");
            array_add(*arguments, "gradlew.bat");
        } else {
            array_add(*arguments, "/bin/sh");
            array_add(*arguments, "gradlew");
        }
    } else array_add(*arguments, "gradle");

    array_add(*arguments, task);
    array_add(*arguments, "--no-daemon");
    result, output, errors, timed_out := Gradle_Process.run_command(..arguments,
        working_directory = project_path, capture_and_return_output = true, print_captured_output = print_output);

    defer free(output);
    defer free(errors);

    return result.type == .EXITED && result.exit_code == 0 && !timed_out;
}

android_gradle_apk :: (project_path: string, release: bool) -> bool, string {
    if !run_gradle_task(project_path, ifx release then "assembleRelease" else "assembleDebug") return false, "";

    path := Gradle_String.path_join(project_path, ifx release then "app/build/outputs/apk/release/app-release.apk" else "app/build/outputs/apk/debug/app-debug.apk");
    if !Gradle_File.file_exists(path) {
        free(path);
        return false, "";
    }

    return true, path;
}
