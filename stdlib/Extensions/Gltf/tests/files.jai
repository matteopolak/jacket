// test: skip_on(.WASM, reason = "reads files; gltf_local_resolve needs POSIX directory handles")

#if OS != .WASM {
    #scope_file
    #import "Basic";
    #import "String";
    #import "File";
    #import "Extensions/Environment";
    #import "Extensions/Testing";
    #import "Extensions/Gltf";
    #import,file "../../test-support/counting-allocator.jai";

    #scope_export
    local_resolver_containment :: (t: *Test) {
        folder := tprint("%/asset", temp_dir(t));

        require(make_directory_if_it_does_not_exist(folder));
        require(write_entire_file(tprint("%/ok.bin", folder), "abc"));
        require(write_entire_file(tprint("%/outside.bin", temp_dir(t)), "secret"));
        data, ok := gltf_local_resolve(*folder, "ok%2ebin", 3);

        defer free(data);
        expect(ok && data == "abc");

        for uri: string.["../outside.bin", "%2e%2e/outside.bin", "/outside.bin", "x%00.bin", "file:ok.bin", "bad%q0", "ok.bin?x"] {
            bytes, allowed := gltf_local_resolve(*folder, uri, 100);
            free(bytes);
            expect(!allowed, uri);
        }

        _, too_big := gltf_local_resolve(*folder, "ok.bin", 2);

        expect(!too_big);

        #if OS != .WINDOWS {
            P :: #import "internal/bindings/posix-api";
            link := to_c_string(tprint("%/escape", folder));

            defer free(link);
            require(P.symlink("../outside.bin", link) == 0);
            escaped, allowed := gltf_local_resolve(*folder, "escape", 100);

            free(escaped);
            expect(!allowed);
        }
    } @test

    remaining_validator_regressions :: () {
        base := output_directory();

        root := tprint("%/gltf-conformance/validator/test/base/data", base);

        for name: string.[
            "mesh_data/joints_weights_complex",
            "mesh/unequal_joints",
            "skin/no_joints_in_scene",
            "skin_data/invalid_ibm",
            "animation/channel_duplicate_targets",
            "node/matrix_indecomposable",
            "image/invalid_image_data",
            "image/invalid_data_uri",
            "image/not_found",
            "image/invalid_uri_scheme",
            "image/png_eos",
            "image/invalid_data_uri_mime_type"
        ] {
            path := tprint("%/%.gltf", root, name);
            bytes, ok := read_entire_file(path);

            require(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py data` first");
            defer free(bytes);
            slash := find_index_from_right(path, #char "/");
            folder := slice(path, 0, slash);
            tally: Tally;
            rejected: bool;

            {
                push_allocator(tally_proc, *tally);
                a, e := gltf_load(bytes, .{ gltf_local_resolve, *folder });

                rejected = e.kind != .NONE;
                gltf_free(*a);
                gltf_error_free(*e);
            }

            expect(rejected, name);
            expect_equal(tally.outstanding, 0, name);
        }
    } @test
}
