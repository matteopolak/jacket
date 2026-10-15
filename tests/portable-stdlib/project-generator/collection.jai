// Provenance: reviewed mutable singleton and exact file ownership/reset rules.
using #import "Basic";
P :: #import "Project_Generator";
C :: #import "Compiler";

main :: () {
    assert(P.Project_Generator.project_name == "");
    assert(P.Project_Generator.project_files.count == 0);
    owned := copy_string("alpha/file.jai");

    P.Project_Generator.add_file(owned, true);
    assert(P.Project_Generator.project_files[0].data == owned.data);
    P.Project_Generator.add_file(copy_string("alpha/file.jai"), true);
    assert(P.Project_Generator.project_files.count == 1);
    borrowed := "beta.jai";

    P.Project_Generator.add_file(borrowed, false);
    assert(P.Project_Generator.project_files[1].data != borrowed.data);
    P.Project_Generator.add_file("dir", "other.jai");
    assert(P.Project_Generator.project_files[2] == "dir/other.jai");
    file: C.Message_File;
    file.kind = .FILE;
    file.fully_pathed_filename = "generated.jai";
    file.from_a_string = true;
    P.Project_Generator.process_message(*file);
    assert(P.Project_Generator.project_files.count == 3);
    file.from_a_string = false;
    P.Project_Generator.process_message(*file);
    assert(P.Project_Generator.project_files.count == 4);
    P.Project_Generator.project_name = "keep configuration";
    P.Project_Generator.reset();
    assert(P.Project_Generator.project_files.count == 0);
    assert(P.Project_Generator.project_name == "keep configuration");
    P.Project_Generator.reset();
    print("ok\n");
}
