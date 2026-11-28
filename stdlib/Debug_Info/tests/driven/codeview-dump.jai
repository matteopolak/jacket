#import "Basic";
Env_Output :: #import "Extensions/Environment";
#import "File";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";

main :: () {
    bytes, read_ok := read_entire_file(tprint("%/developer-tools-msf-x64.pdb", Env_Output.output_directory()));

    assert(read_ok);
    defer free(bytes);
    ok, pdb := D.parse_pdb(bytes);

    assert(ok);
    defer release_pdb(*pdb);
    assert(D.dump_codeview_types(pdb.tpi, pdb.ipi));
    print("ok\n");
}
