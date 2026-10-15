#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";

main :: () {
    bytes, read_ok := read_entire_file(tprint("%../../target/developer-tools-msf-x64.pdb", #filepath));

    assert(read_ok);
    defer free(bytes);
    ok, pdb := D.parse_pdb(bytes);

    assert(ok);
    defer release_pdb(*pdb);
    assert(D.dump_codeview_types(pdb.tpi, pdb.ipi));
    print("ok\n");
}
