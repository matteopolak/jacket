Project_Generator: struct {
    project_name := "";
    output_directory := "";
    debug_command := "";
    working_directory := "";
    run_arguments := "";
    build_cmd := #run PG_System.get_path_of_running_executable();
    build_args := "";
    module_path := #filepath;
    project_files: [..] string;

    reset :: () {
        pg_reset();
    }
    add_file :: (file: string, take_ownership: bool) {
        pg_add(file, take_ownership);
    }
    add_file :: (path: string, file: string) {
        pg_add(PG_String.join(path, "/", file), true);
    }
    process_message :: (message: *PG_Compiler.Message) {
        pg_message(message);
    }
    build :: (file: string) {
        pg_build(file);
    }
    intercept_messages :: (w: Workspace) {
        pg_intercept(w);
    }
}

#scope_module
using #import "Basic";
PG_Compiler :: #import "Compiler";
PG_String :: #import "String";
PG_File :: #import "File";
PG_System :: #import "System";
PG_Sort :: #import "Sort";
PG_SHA :: #import "internal/crypto/sha";
#load "../internal/build/project-collection.jai";
#load "../internal/build/project-documents.jai";
