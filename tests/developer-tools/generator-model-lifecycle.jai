#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved: #Context;
    record: B.CXCursor;
    macro: B.CXCursor;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        outer := context.generator;

        if cursor.kind == .StructDecl push_context {
            nested: B.Generator;

            B.init_generator(*nested, .{});
            assert(context.generator == *nested && nested.type_def_u32 != null);
            B.deinit_generator(*nested);
            assert(context.generator == *nested);
        }

        assert(context.generator == outer);
        if cursor.kind == .StructDecl capture.record = cursor;

        if cursor.kind == .MacroDefinition {
            spelling := B.clang_getCursorSpelling(cursor);
            if to_string(B.clang_getCString(spelling)) == "LIFECYCLE_VALUE" capture.macro = cursor;
            B.clang_disposeString(spelling);
        }
    }

    return .Continue;
}

Tracker :: struct {
    underlying: Allocator;
    live: int;
}

tracked :: (mode: Allocator_Mode, bytes: s64, old_bytes: s64, memory: *void, data: *void) -> *void {
    tracker := cast(*Tracker) data;
    result := tracker.underlying.proc(mode, bytes, old_bytes, memory, tracker.underlying.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && !memory && result tracker.live += 1;
    if mode == .FREE && memory tracker.live -= 1;

    return result;
}

main :: () {
    tracker: Tracker;
    tracker.underlying = context.allocator;

    push_context {
        context.allocator = .{ tracked, *tracker };

        generator: B.Generator;

        B.init_generator(*generator, .{});
        context.allocator = tracker.underlying;
        assert(context.generator == *generator);
        assert(!generator.int128_was_referenced);
        assert(generator.pointer_size == size_of(*void));
        assert(generator.type_def_void != null && generator.type_def_bool != null);
        assert(generator.type_def_s8 != null && generator.type_def_u8 != null);
        assert(generator.type_def_s16 != null && generator.type_def_u16 != null);
        assert(generator.type_def_s32 != null && generator.type_def_u32 != null);
        assert(generator.type_def_s64 != null && generator.type_def_u64 != null);
        assert(generator.type_def_s128 != null && generator.type_def_u128 != null);
        assert(generator.type_def_float32 != null && generator.type_def_float64 != null);
        assert(generator.type_def_pointer_to_void != null && generator.type_def_ptr_void != null);
        assert(generator.type_def_objc_class != null && generator.type_def_objc_selector != null);
        assert(generator.type_def_objc_block_pointer != null && generator.type_def_objc_id != null);
        assert(generator.type_def_jai_string != null && generator.type_def_type != null);
        assert(generator.type_def_msvc_destructor_flags != null);
        assert(B.is_16byte_integer_type(generator.type_def_s128));
        assert(B.is_16byte_integer_type(generator.type_def_u128));
        saved_integer := generator.type_def_u32;

        generator.int128_was_referenced = true;

        B.init_generator(*generator, .{});
        assert(generator.type_def_u32 == saved_integer);
        assert(generator.int128_was_referenced);

        generator.clang_index = B.clang_createIndex(0, 0);
        assert(generator.clang_index != null);
        filename := to_c_string(tprint("%generator-model-lifecycle.h", #filepath));

        defer free(filename);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, null, 0, null, 0, 1, *generator.translation_unit) == .Success);
        assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        capture: Capture;
        capture.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
        assert(capture.record.kind == .StructDecl && capture.macro.kind == .MacroDefinition);
        record := B.create_struct(capture.record);

        assert(record != null && record.declarations.count == 3);
        assert(record.declarations[0].type == generator.type_def_char);
        assert(record.declarations[1].type == generator.type_def_bool);
        assert(record.declarations[2].type == generator.type_def_u64);
        macro := B.create_macro_definition(capture.macro);

        assert(macro != null && generator.tokens_to_free.count > 0);
        B.deinit_generator(*generator);
        assert(context.generator == *generator);
        assert(generator.translation_unit == null && generator.clang_index == null);
        assert(generator.tokens_to_free.count == 0);
    }

    assert(tracker.live == 0);

    push_context {
        configured: B.Generator;
        options: B.Generate_Bindings_Options;

        array_add(*options.extra_clang_arguments, "-funsigned-char");
        array_add(*options.extra_clang_arguments, "-x");
        array_add(*options.extra_clang_arguments, "c++");
        defer array_free(options.extra_clang_arguments);
        B.init_generator(*configured, options);
        assert(configured.type_def_char != null);
        assert((configured.type_def_char.number_flags & .SIGNED) == 0);
        B.deinit_generator(*configured);
    }

    push_context {
        strict: B.Generator;
        options: B.Generate_Bindings_Options;

        array_add(*options.extra_clang_arguments, "-std=c89");
        array_add(*options.extra_clang_arguments, "-pedantic-errors");
        defer array_free(options.extra_clang_arguments);
        B.init_generator(*strict, options);
        assert(strict.type_def_bool != null && strict.type_def_s128 != null);
        B.deinit_generator(*strict);
    }

    print("ok\n");
}
