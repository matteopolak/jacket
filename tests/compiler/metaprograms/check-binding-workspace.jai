// Actual canonical Compiler headers, not only manually constructed metadata.
#import "Basic";
#import "Compiler";
C :: #import "developer-tools-check-staged";
N :: #import "developer-tools-check-names";

#run {
    for names, index: N.symbols {
        workspace := compiler_create_workspace("Original native mangled signature checks");
        options := get_build_options(workspace);
        options.output_type = .NO_OUTPUT;
        options.os_target = ifx index == 0 then .MACOS else .WINDOWS;
        options.cpu_target = ifx index == 0 then .ARM64 else .X64;
        set_build_options(options, workspace);
        compiler_begin_intercept(workspace);
        long_type := ifx index == 0 then "s64" else "s32";
        // A symbol existence check is intentionally outside Check's contract.
        source := tprint("native :: #foreign_library \"unused-by-signature-check\";\nrenamed_zero :: () -> s32 #foreign native \"%\";\nrenamed_scalars :: (s32, u16, %, float64, *u8) -> s64 #foreign native \"%\";\nmain :: () {}\n", names.zero, long_type, names.scalars);

        add_build_string(source, workspace);
        checked: s64;

        while true {
            message := compiler_wait_for_message();
            if !message break;

            if message.workspace == workspace && message.kind == .TYPECHECKED {
                for item: (cast(*Message_Typechecked) message).procedure_headers {
                    header := item.expression;
                    if header.name != "renamed_zero" && header.name != "renamed_scalars" continue;

                    print("foreign header metadata: flags=% foreign_name_bytes=% procedure_type_flags=%\n", cast(u32) header.procedure_flags, header.foreign_function_name.count, cast(u32) (cast(*Type_Info_Procedure) header.type).procedure_flags);
                    assert((header.procedure_flags & .ELSEWHERE) != 0);
                    assert(C.verify_elsewhere_binding(header, options.os_target, options.cpu_target));
                    checked += 1;
                }
            }

            if message.workspace == workspace && message.kind == .COMPLETE break;
        }

        compiler_end_intercept(workspace);
        assert(checked == 2);
    }
}

main :: () {
    print("ok\n");
}
