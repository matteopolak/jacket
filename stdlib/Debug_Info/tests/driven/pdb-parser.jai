#import "Basic";
#import "File";
D :: #import "Debug_Info";
#load "codeview-test-support.jai";
#load "../../../../target/debug-info-pdb-sdk.jai";

main :: () {
    push_context {
        context.allocator = .{ count_allocation, null };
        context.logger = capture_error;
        check_all();
    }

    assert(live_allocations == 0);
    print("ok\n");
}

check_all :: () {
    for architecture: string.["x64", "arm64"] {
        bytes, read_ok := read_entire_file(tprint("%../../../../target/developer-tools-msf-%.pdb", #filepath, architecture));

        assert(read_ok);
        defer free(bytes);
        ok, pdb := D.parse_pdb(bytes);

        assert(ok && pdb.header.version == .VC70 && pdb.header.age == 1 && pdb.contains_id_stream);
        check_pdb_sdk(pdb, architecture);
        defer release_pdb(*pdb);
        assert(!pdb.no_type_merge && !pdb.minimal_debug_info);
        assert(pdb.name_table.buffer.count > 0 && pdb.ipi.count > 0 && pdb.tpi.count > 0);
        assert(D.verify_integrity(pdb.msf));

        for size: s64.[17, 4] {
            before := live_allocations;

            deny_size = size;
            denied = false;
            rejected_ok, rejected := D.parse_pdb(bytes);

            assert(denied && !rejected_ok && rejected.msf.header == null && live_allocations == before);
        }

        tpi, tpi_ok := D.parse_tpi_stream(pdb.msf, 2);

        assert(tpi_ok && tpi.count == pdb.tpi.count);
        defer release_cv_types(tpi);

        for type: pdb.ipi {
            if type.kind == .FUNC_ID {
                function := cast(*D.Codeview_Function_Id) type;
                assert(function.name == "original_debug_entry");
            }
        }

        for prefix: s64.[0, 30, 4095, bytes.count - 1] {
            before := live_allocations;
            rejected_ok, rejected := D.parse_pdb(.{ prefix, bytes.data });
            assert(!rejected_ok && rejected.msf.header == null && live_allocations == before);
        }
    }
}
