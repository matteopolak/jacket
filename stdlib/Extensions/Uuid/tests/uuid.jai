#import "Basic";
#import "String";
#import "Extensions/Uuid";
U :: #import "Extensions/Uuid";
J :: #import "Extensions/Json";
Map :: #import "Extensions/Hash_Map";
#load "../../test-support/counting-allocator.jai";

Record :: struct {
    id: Uuid; @json(with=uuid_json)
    uuid_json_encode :: U.uuid_json_encode;
    uuid_json_decode :: U.uuid_json_decode;
}

must :: (text: string) -> Uuid {
    value, error := parse_uuid(text);
    assert(error.kind == .NONE);
    return value;
}

uuid_conformance :: () {
    // RFC 9562 A.1-A.6 and B.1-B.2: all layouts are valid opaque UUIDs.
    // Generation of A.1/A.5 (v1/v6) and B.1/B.2 (v8) is outside v1 scope.
    vectors := string.[
        "c232ab00-9414-11ec-b3c8-9f6bdeced846",
        "5df41881-3aed-3515-88a7-2f4a814cf09e",
        "919108f7-52d1-4320-9bac-f847db4148a8",
        "2ed6657d-e927-568b-95e1-2665a8aea6a2",
        "1ec9414c-232a-6b00-b3c8-9f6bdeced846",
        "017f22e2-79b0-7cc3-98c4-dc0c0c07398f",
        "2489e9ad-2ee2-8e00-8ec9-32d5f69181c0",
        "5c146b14-3c52-8afd-938a-375d0df1fbf6"
    ];
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        for text: vectors {
            value := must(text);
            encoded := format_uuid(value);

            assert(encoded == text && uuid_variant(value) == .RFC);
            free(encoded);
            assert(must(tprint("{%}", text)) == value);
            assert(must(tprint("urn:uuid:%", text)) == value);
        }

        assert(must("919108F7-52D1-4320-9BAC-F847DB4148A8") == must(vectors[2]));
        assert(must("00000000-0000-0000-0000-000000000000") == UUID_NIL);
        assert(must("ffffffff-ffff-ffff-ffff-ffffffffffff") == UUID_MAX);
        assert(uuid_variant(UUID_NIL) == .NCS && uuid_variant(UUID_MAX) == .FUTURE);
        ms := UUID_NIL;
        ms.bytes[8] = 0xc0;
        assert(uuid_variant(ms) == .MICROSOFT);

        for text: string.["", "x", "urn:uuid:{919108f7-52d1-4320-9bac-f847db4148a8}", "919108f752d143209bacf847db4148a8", " 919108f7-52d1-4320-9bac-f847db4148a8", "{919108f7-52d1-4320-9bac-f847db4148a8", "919108f7-52d1-4320-9bac-f847db4148a8x"] {
            value, error := parse_uuid(text);
            assert(error.kind != .NONE && error.offset >= 0 && error.offset <= text.count && value == UUID_NIL);
        }

        for i: 0..35 {
            bad := copy_string(vectors[2]);
            bad.data[i] = #char "!";
            _, error := parse_uuid(bad);

            assert(error.kind == .SYNTAX && error.offset == i);
            free(bad);
            _, short_error := parse_uuid(slice(vectors[2], 0, i));

            assert(short_error.kind == .LENGTH && short_error.offset == i);
        }

        builder: String_Builder;

        append(*builder, "id=");
        append_uuid(*builder, must(vectors[2]));
        built := builder_to_string(*builder);

        assert(built == "id=919108f7-52d1-4320-9bac-f847db4148a8");
        free(built);
        namespace := must("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
        a, e3 := uuid_v3(namespace, "www.example.com");
        b, e5 := uuid_v5(namespace, "www.example.com");

        assert(e3.kind == .NONE && a == must(vectors[1]));
        assert(e5.kind == .NONE && b == must(vectors[3]));
        _, limit := uuid_v5(namespace, "long", .{ max_name_size = 3 });

        assert(limit.kind == .LIMIT && limit.offset == 3);
        _, limit3 := uuid_v3(namespace, "long", .{ max_name_size = 3 });

        assert(limit3.kind == .LIMIT);
        raw := must("919108f7-52d1-3320-5bac-f847db4148a8");

        U.set_layout(*raw, 4);
        assert(raw == must(vectors[2]));
        state: u64;
        vector_random := must(vectors[5]);

        state = (cast(u64) 1645557742000 << 12) | 0xcc2;
        vector7, vector_error := U.v7_advance(*state, 1645557742000, vector_random);

        assert(vector_error.kind == .NONE && vector7 == vector_random);
        state = 0;
        raw7 := must(vectors[5]);
        // Counter seed uses ten low random bits, leaving room before overflow.
        raw7.bytes[6] = 0;
        raw7.bytes[7] = 0;
        first, e7 := U.v7_advance(*state, 1645557742000, raw7);

        assert(e7.kind == .NONE && uuid_version(first) == 7);

        for i: 1..100 {
            next, error := U.v7_advance(*state, 1645557742000 - i, raw7);
            assert(error.kind == .NONE && uuid_compare(first, next) < 0);
            first = next;
        }

        state = (cast(u64) 1645557742000 << 12) | 4095;
        before := state;
        _, exhausted := U.v7_advance(*state, 1645557742000, raw7);

        assert(exhausted.kind == .EXHAUSTED && state == before);
        _, resumed := U.v7_advance(*state, 1645557742001, raw7);

        assert(resumed.kind == .NONE);
        _, negative := U.v7_advance(*state, -1, raw7);
        _, overflow := U.v7_advance(*state, 0x1000000000000, raw7);

        assert(negative.kind == .CLOCK && overflow.kind == .CLOCK);

        for 1..100 {
            random, error := uuid_v4();

            assert(error.kind == .NONE && uuid_version(random) == 4 && uuid_variant(random) == .RFC);
            assert(must(format_uuid(random, temp)) == random);
            timed, time_error := uuid_v7();

            assert(time_error.kind == .NONE && uuid_version(timed) == 7);
        }

        map: Map.Map(Uuid, int, uuid_hash, uuid_equal);

        Map.insert(*map, a, 42);
        assert(Map.get(*map, a) == 42 && uuid_hash(a) == uuid_hash(a));
        Map.deinit(*map);
        encoded := J.to_json(Record.{ a });

        assert(encoded == "{\"id\":\"5df41881-3aed-3515-88a7-2f4a814cf09e\"}");
        decoded, error := J.from_json(Record, encoded);

        assert(error.kind == .NONE && decoded.id == a);
        J.free_json(*decoded);
        free(encoded);

        for text: string.["{\"id\":\"bad\"}", "{\"id\":3}", "{\"id\":\"\\u0035df41881-3aed-3515-88a7-2f4a814cf09e\"}"] {
            record, err := J.from_json(Record, text);

            if text == "{\"id\":\"\\u0035df41881-3aed-3515-88a7-2f4a814cf09e\"}" {
                assert(err.kind == .NONE && record.id == a);
            } else assert(err.kind != .NONE);

            J.free_json(*record);
            J.free_json(*err);
        }
    }

    assert(tally.outstanding == 0);
} @test

name_boundaries :: () {
    namespace := must("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    names := string.["", "日本語", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"];
    v3 := string.["c87ee674-4ddc-3efe-a74e-dfe25da5d7b3", "f9e3b804-0a0d-3dd0-a874-0baf0cb21aab", "12adee6c-b187-318d-82d2-f934bf55422b", "66d96f29-a22d-37a1-9666-8af5b81cf1f2", "3a55c73f-2fc6-3ce3-9219-6b3699fdb604"];
    v5 := string.["4ebd0208-8328-5d69-8c44-ec50939c0967", "9786f370-913c-51ea-845e-7f3469bc5966", "7280cc42-274a-5c4a-91fc-ae23f853eeb7", "69349718-028b-5ff0-a00f-c024ded6e6ee", "efd9d9de-780b-5e83-a3e7-30a408967cf8"];
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        // Python uuid reference results across hash-block/padding boundaries and UTF-8.
        for name, i: names {
            a, e3 := uuid_v3(namespace, name);
            b, e5 := uuid_v5(namespace, name);
            assert(e3.kind == .NONE && e5.kind == .NONE);
            assert(a == must(v3[i]) && b == must(v5[i]));
        }
    }

    assert(tally.outstanding == 0);
} @test
