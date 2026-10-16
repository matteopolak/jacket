#import "Basic";
B :: #import "Bindings_Generator";

data_count, plain_count: int;

group :: (declaration: *B.Declaration) -> string {
    if declaration.name == "JOIN_THIRD" return "Joined";
    return "";
}

data_visitor :: (declaration: *B.Declaration, parent: *B.Declaration, _data: *void) -> B.Declaration_Visit_Result {
    if !parent || parent.kind != .ENUM return .RECURSE;

    expected: string;
    if declaration.name == "SIGNAL_DOWN" expected = "DOWN";
    if declaration.name == "SIGNAL_UP" expected = "UP";
    if declaration.name == "CONNECTION_CLOSED" expected = "CLOSED";
    if declaration.name == "CONNECTION_OPEN" expected = "OPEN";
    if declaration.name == "JOIN_FIRST" expected = "FIRST";
    if declaration.name == "JOIN_SECOND" expected = "SECOND";
    if declaration.name == "JOIN_THIRD" expected = "THIRD";

    assert(expected != "" && declaration.output_name == expected);
    data_count += 1;
    // The second visitor and final model must retain this caller edit.
    if declaration.name == "SIGNAL_DOWN" declaration.output_name = "VisitorDown";

    return .RECURSE;
}

plain_visitor :: (declaration: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if !parent || parent.kind != .ENUM return .RECURSE;

    assert(data_count == 7);
    if declaration.name == "SIGNAL_DOWN" assert(declaration.output_name == "VisitorDown");

    plain_count += 1;

    return .RECURSE;
}

main :: () {
    push_context, defer_pop;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-run-enum-names.hpp", #filepath));
    defer array_free(options.source_files);
    options.convert_macro_value_to_enum_callback = group;
    options.visitor_with_data = data_visitor;
    options.visitor = plain_visitor;
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    succeeded := B.run_generator(*generator, options);

    assert(succeeded && data_count == 7 && plain_count == 7);

    for declaration: generator.global_scope.members {
        if declaration.name != "SignalState" continue;
        enumeration := cast(*B.Enum) declaration;
        assert(enumeration.enumerates[0].output_name == "VisitorDown");
        assert(enumeration.enumerates[1].output_name == "UP");
    }

    print("enum-names ok\n");
}
