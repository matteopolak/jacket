#scope_module
#import "Basic";

gm_probe_type_supported :: (native: CXType, depth := 0) -> bool {
    if depth >= 64 return false;

    native = clang_getCanonicalType(native);
    // A scalar enum payload can use its canonical integer representation.
    // A nested enum would make the generic pointer constructor register a
    // declaration from the temporary TU, so that nominal shape is unsupported.
    if native.kind == .Enum return depth == 0 && gm_probe_type_supported(clang_getEnumDeclIntegerType(clang_getTypeDeclaration(native)), depth + 1);
    if native.kind == .Pointer return gm_probe_type_supported(clang_getPointeeType(native), depth + 1);

    if native.kind == .ConstantArray {
        element := clang_getCanonicalType(clang_getArrayElementType(native));
        return (element.kind == .Char_S || element.kind == .Char_U) && clang_getArraySize(native) > 0;
    }

    if native.kind == .Int128 || native.kind == .UInt128 return false;

    return gm_scalar_slot(gm_active_generator(), native) != null;
}

gm_macro_probe_input_valid :: (input: *GM_Sessions.Parse_Input) -> bool {
    if !input || input.filename.count <= 0 || !input.filename.data ||
       input.arguments.count < 0 || input.arguments.count > 0x7fffffff ||
       (input.arguments.count > 0 && !input.arguments.data) ||
       input.overlays.count < 0 || input.overlays.count > 0xffffffff ||
       (input.overlays.count > 0 && !input.overlays.data) {
        log_error("Bindings_Generator: macro evaluation requires an exact valid native parse-input receipt\n");
        return false;
    }

    return true;
}

// available=false preserves pending validation; supported=false with available
// true describes an actual unrepresentable/unevaluable expression.
gm_evaluate_macro_declaration :: (declaration: *Declaration) -> literal: *Literal, type: *CType, available: bool, supported: bool {
    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null, null, false, false;

    if session.failed {
        log_error("Bindings_Generator: macro evaluation ownership transaction has already failed\n");
        return null, null, false, false;
    }

    if !declaration || declaration.kind != .MACRO_DEFINITION || declaration.name.count <= 0 || !declaration.name.data {
        log_error("Bindings_Generator: macro evaluation requires a named canonical macro declaration\n");
        return null, null, false, false;
    }

    if !generator.clang_index || !generator.translation_unit {
        log_error("Bindings_Generator: macro evaluation requires a live canonical native index and translation unit\n");
        return null, null, false, false;
    }

    if !gm_macro_probe_input_valid(session.parse_input) return null, null, false, false;

    #if #exists(binding_open_evaluation_probe) {
        unit, cursor, available, supported := binding_open_evaluation_probe(session.parse_input, declaration.name);

        defer if unit clang_disposeTranslationUnit(unit);
        if !available || !supported return null, null, available, supported;

        if !unit || clang_Cursor_isNull(cursor) != 0 {
            log_error("Bindings_Generator: native evaluation probe did not supply a live result cursor\n");
            return null, null, false, false;
        }

        native := clang_getCanonicalType(clang_getCursorType(cursor));
        if !gm_probe_type_supported(native) return null, null, true, false;

        value := gm_native_literal(cursor, clang_getNullCursor(), false);
        if !value return null, null, true, false;
        if native.kind == .Enum native = clang_getEnumDeclIntegerType(clang_getTypeDeclaration(native));

        converted := gm_build_type(native, cursor, false, 0);
        if !converted return null, null, true, false;

        value.location = declaration.location;

        return value, converted, true, true;
    } else {
        log_error("Bindings_Generator: macro evaluation requires the tooling-owned native evaluation probe\n");
        return null, null, false, false;
    }
}
