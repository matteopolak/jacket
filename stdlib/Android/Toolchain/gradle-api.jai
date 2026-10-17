__no_gradle_build_be_verbose := false;

android_generate_apk :: (project_path: string, release := false) -> bool, string #deprecated "Pass an Android_Build_Options as the first argument instead." {
    return android_gradle_apk(project_path, release);
}

gradle_install_apk :: (project_path: string, release := false) -> bool {
    return run_gradle_task(project_path, ifx release then "installRelease" else "installDebug");
}

run_gradle_task :: (project_path: string, task: string, print_output := true) -> bool {
    return android_gradle_task(project_path, task, print_output);
}

#load "../../internal/android-toolchain/android-gradle.jai";
