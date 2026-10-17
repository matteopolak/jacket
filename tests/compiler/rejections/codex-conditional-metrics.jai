#import "Compiler";
C :: #import "Codex";

main :: () {
    condition: Code_If;
    condition.kind = .IF;
    statements := (*Code_Node).[cast(*Code_Node) *condition];
    block: Code_Block;
    block.kind = .BLOCK;
    block.statements = .{ statements.count, statements.data };
    body: Code_Procedure_Body;
    body.block = *block;
    tc: Typechecked(Code_Procedure_Body);
    tc.expression = *body;
    C.collect_metrics(tc);
}
