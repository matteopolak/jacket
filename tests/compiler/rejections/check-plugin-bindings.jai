// The disabled factory skips a binding; the enabled factory diagnoses it once.
#import "Compiler";
C :: #import "Check";
Disabled :: #import "Check"(CHECK_BINDINGS = false);

#run {
    workspace := compiler_create_workspace("Original configured binding check");
    header: Code_Procedure_Header;
    header.name = "original_jai_name";
    header.foreign_function_name = "invalid_foreign_name";
    header.type = type_info(#type () -> s32 #c_call);
    header.procedure_flags = .ELSEWHERE;
    typed := Typechecked(Code_Procedure_Header).{ expression = *header };
    headers := Typechecked(Code_Procedure_Header).[typed];
    message: Message_Typechecked;
    message.kind = .TYPECHECKED;
    message.workspace = workspace;
    message.procedure_headers = headers;
    disabled := Disabled.get_plugin();
    disabled.workspace = workspace;
    disabled.message(disabled, cast(*Message) *message);
    enabled := C.get_plugin();
    enabled.workspace = workspace;
    enabled.message(enabled, cast(*Message) *message);
}
