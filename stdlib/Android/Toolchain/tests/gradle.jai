#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";

main :: () {
    assert(!A.__no_gradle_build_be_verbose);
    A.__no_gradle_build_be_verbose = true;
    assert(A.__no_gradle_build_be_verbose);
    A.__no_gradle_build_be_verbose = false;

    #if OS_IS_UNIX {
        project := "target/jai android gradle fixture";

        assert(F.make_directory_if_it_does_not_exist(project, true));
        wrapper := "target/jai android gradle fixture/gradlew";
        script := "#!/bin/sh\n[ \"$2\" = --no-daemon ] || exit 11\ncase \"$1\" in\n assembleDebug) mkdir -p app/build/outputs/apk/debug; printf apk > app/build/outputs/apk/debug/app-debug.apk;;\n assembleRelease) mkdir -p app/build/outputs/apk/release; printf release > app/build/outputs/apk/release/app-release.apk;;\n installDebug|installRelease) printf %s \"$1\" > requested-install; exit 31;;\n help) exit 0;;\n *) exit 23;;\nesac\n";

        assert(F.write_entire_file(wrapper, script));
        assert(A.run_gradle_task(project, "help", false));
        assert(!A.run_gradle_task(project, "fail", false));
        assert(!A.run_gradle_task(project, "", false));
        assert(!A.run_gradle_task("target/jai absent android project", "help", false));
        assert(!A.gradle_install_apk(project));
        requested, requested_ok := F.read_entire_file("target/jai android gradle fixture/requested-install");

        assert(requested_ok && requested == "installDebug");
        free(requested);
        assert(!A.gradle_install_apk(project, true));
        requested, requested_ok = F.read_entire_file("target/jai android gradle fixture/requested-install");
        assert(requested_ok && requested == "installRelease");
        free(requested);
        success, output := A.android_generate_apk(project);

        assert(success && output == "target/jai android gradle fixture/app/build/outputs/apk/debug/app-debug.apk");
        bytes, read_ok := F.read_entire_file(output);

        assert(read_ok && bytes == "apk");
        free(bytes);
        free(output);
        success, output = A.android_generate_apk(project, true);
        assert(success);
        free(output);
        assert(F.write_entire_file(wrapper, "#!/bin/sh\nexit 9\n"));
        success, output = A.android_generate_apk(project);
        assert(!success && output.count == 0);
        assert(!A.gradle_install_apk(project));
        assert(F.write_entire_file(wrapper, "#!/bin/sh\nexit 0\n"));
        assert(F.file_delete("target/jai android gradle fixture/app/build/outputs/apk/debug/app-debug.apk"));
        success, output = A.android_generate_apk(project);
        assert(!success && output.count == 0);
    } else {
        // Type-check the Windows command-launch branch without executing it on this host.
        if false {
            assert(!A.run_gradle_task("absent project", "help", false));
            assert(!A.gradle_install_apk("absent project"));
            success, output := A.android_generate_apk("absent project");
            assert(!success && output.count == 0);
        }
    }
}
