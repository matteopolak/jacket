#import "Compiler";
C :: #import "Codex";

main :: () {
    module: Message_Import;
    module.module_name = "Basic";
    file: Message_File;
    file.enclosing_import = *module;
    header: Code_Procedure_Header;
    header.kind = .PROCEDURE_HEADER;
    header.enclosing_load = *file;
    call: Code_Procedure_Call;
    call.kind = .PROCEDURE_CALL;
    call.resolved_procedure_expression = cast(*Code_Node) *header;
    statements := (*Code_Node).[cast(*Code_Node) *call];
    block: Code_Block;
    block.kind = .BLOCK;
    block.statements = .{ statements.count, statements.data };
    body: Code_Procedure_Body;
    body.block = *block;
    tc: Typechecked(Code_Procedure_Body);
    tc.expression = *body;
    C.collect_metrics(tc);
}
