// Reviewed anonymous typedef naming/omission and canonical cursor identity.
#import "Basic";
B :: #import "Bindings_Generator";

records: s64;
enums: s64;
omitted_aliases: s64;
canonical_record: *B.Struct;

inspect :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .STRUCT records += 1;
    if decl.kind == .ENUM enums += 1;

    if decl.kind == .STRUCT && decl.name == "Anonymous_Record" {
        canonical_record = cast(*B.Struct) decl;
        decl.output_name = "Renamed_Record";
    }

    if decl.kind == .TYPEDEF && decl.name == "Record_Alias" {
        assert(B.try_find_struct_in_type(decl.type) == canonical_record);
        assert((decl.decl_flags & .OMIT_FROM_OUTPUT) == 0);
    }

    if decl.kind == .TYPEDEF && (decl.name == "Anonymous_Record" || decl.name == "Anonymous_Union" || decl.name == "Anonymous_Mode") {
        assert((decl.decl_flags & .OMIT_FROM_OUTPUT) != 0);
        nominal: *B.Declaration;
        if decl.type.type_of_struct nominal = cast(*B.Declaration) decl.type.type_of_struct;
        if decl.type.type_of_enum nominal = cast(*B.Declaration) decl.type.type_of_enum;

        assert(nominal != null && nominal.name == decl.name);
        assert(nominal.output_name == (ifx decl.name == "Anonymous_Record" then "Renamed_Record" else decl.output_name));
        omitted_aliases += 1;
    }

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-anonymous.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ identifier = "native", filename = "developer_tools_anonymous" });
    defer array_free(options.libraries);
    options.visitor = inspect;
    options.strip_enum_prefixes = false;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-anonymous-generated.jai", #filepath), log_results = false));
    assert(records == 3 && enums == 1 && omitted_aliases == 3);
    options.visitor = null;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-anonymous-default.jai", #filepath), log_results = false));
    print("ok\n");
}
