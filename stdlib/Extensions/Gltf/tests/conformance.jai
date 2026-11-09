// test: skip_on(.WASM, reason = "reads sample models from the file system")

#if OS != .WASM {
    #scope_file
    #import "Basic";
    #import "String";
    #import "Extensions/Testing";
    #import "Extensions/Environment";
    #load "sample-support.inc";

    #scope_export
    sample_assets :: () {
        base := environment_variable("JACKET_OUTPUT_DIR");
        if !base.count base = tprint("%../../../../target", #filepath);

        root := tprint("%/gltf-conformance/samples/Models", base);

        require(file_exists(root), "run `PYTHONPATH=tools python3 tools/gltf_conformance/fetch.py --samples` first");
        run: Sample_Run;

        require(visit_files(root, true, *run, visit_sample));
        print("sample assets: % loaded, % failures; expected-failure list empty\n", run.count, run.failures);
        expect_equal(run.skipped, 3);
        expect_equal(run.count, 267);
        expect_equal(run.failures, 0);
    } @test @slow @only(.NATIVE) @needs(.GLTF_SAMPLES) // Reads 1 GB of models, fetched on request: too much for the interpreter; the focused tests cover it.

    #scope_file
    #import "Extensions/Json";

    Report :: struct {
        issues: Issues;
    }

    Issues :: struct {
        num_errors: s64; @json(rename=numErrors)
    }

    Cross_Check :: struct {
        checked: s64;
        accepted: s64;
    }

    check_invalid :: (info: *File_Visit_Info, run: *Cross_Check) {
        if !ends_with(info.full_name, ".gltf") && !ends_with(info.full_name, ".glb") return;

        report, got := read_entire_file(tprint("%.report.json", info.full_name));

        assert(got);
        defer free(report);
        expected, re := from_json(Report, report);

        assert(re.kind == .NONE);
        defer free_json(*expected);
        if !expected.issues.num_errors return;

        run.checked += 1;
        bytes, ok := read_entire_file(info.full_name);

        assert(ok);
        defer free(bytes);
        slash := find_index_from_right(info.full_name, #char "/");
        folder := slice(info.full_name, 0, slash);
        a, e := gltf_load(bytes, .{ gltf_local_resolve, *folder });

        defer gltf_free(*a);
        defer gltf_error_free(*e);

        if e.kind == .NONE {
            print("ACCEPTED INVALID %\n", info.full_name);
            run.accepted += 1;
        }
    }

    #scope_export
    validator_invalid_samples :: () {
        base := environment_variable("JACKET_OUTPUT_DIR");
        if !base.count base = tprint("%../../../../target", #filepath);

        // Extension semantics and their extension-specific validator fixtures are not checked.
        root := tprint("%/gltf-conformance/validator/test/base/data", base);

        require(file_exists(root), "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");
        run: Cross_Check;

        require(visit_files(root, true, *run, check_invalid));
        print("validator core: % invalid checked, % accepted\n", run.checked, run.accepted);
        expect_equal(run.checked, 190);
        expect_equal(run.accepted, 0);
    } @test
}
