#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved: #Context;
    parameter: B.CXCursor;
}

collect :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        if cur.kind == .ParmDecl capture.parameter = cur;
    }

    return .Recurse;
}

main :: () {
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        generator.clang_index = B.clang_createIndex(0, 0);
        source := "void independent_extent(int extent = 3);";
        file := B.CXUnsavedFile.{ Filename = "independent-pointer.cpp", Contents = source.data, Length = cast(u64) source.count };

        assert(B.clang_parseTranslationUnit2(generator.clang_index, file.Filename, null, 0, *file, 1, 0, *generator.translation_unit) == .Success);
        capture := Capture.{ saved = context };

        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), collect, *capture);
        assert(capture.parameter.kind == .ParmDecl);
        integer := B.CType.{ size = 4, number_flags = .SIGNED | ._32BIT };
        fixed := B.CType.{ size = 8, array_element_type = *integer, array_element_count = 2 };
        pointer := B.CType.{ pointer_to = *fixed, flags = .INSERTED, size = size_of(*void) };
        result := B.maybe_convert_pointer_type_to_fixed_array_type(capture.parameter, *pointer);

        assert(result != *pointer && result.array_element_type == *fixed && result.array_element_count == 3 && result.size == 24);
        assert(pointer.pointer_to == *fixed && pointer.flags == .INSERTED && fixed.array_element_count == 2);
    }

    print("ok\n");
}
