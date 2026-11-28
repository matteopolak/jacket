__no_gradle_build_be_verbose := false;

// Builds the project at `project_path` with Gradle and returns `true` and the APK path.
//
// Deprecated; pass an `Android_Build_Options` and an `Android_Project_Config` instead. Runs `assembleRelease`
// when `release` is set, else `assembleDebug`. Returns `false` and an empty string when Gradle fails or the
// APK is missing; the caller frees the path.
android_generate_apk :: (project_path: string, release := false) -> bool, string #deprecated "Pass an Android_Build_Options as the first argument instead." {
    return android_gradle_apk(project_path, release);
}

// Installs the project's debug (or, with `release`, release) build on the device with Gradle.
//
// Runs the `installDebug` or `installRelease` task; see [run_gradle_task].
gradle_install_apk :: (project_path: string, release := false) -> bool {
    return run_gradle_task(project_path, ifx release then "installRelease" else "installDebug");
}

// Runs the Gradle task `task` in `project_path` and returns `true` when it exits with status 0.
//
// Uses the project's `gradlew` script when it exists, else `gradle` from the `PATH`, always with
// `--no-daemon`. Returns `false` when `task` is empty, `project_path` is not a folder, or the run fails or
// times out. `print_output` echoes Gradle's output.
run_gradle_task :: (project_path: string, task: string, print_output := true) -> bool {
    return android_gradle_task(project_path, task, print_output);
}

#load "../../internal/android-toolchain/android-gradle.jai";
