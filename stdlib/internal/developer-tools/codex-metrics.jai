collect_metrics :: (tc: Typechecked(Code_Procedure_Body)) -> (metrics: Metrics) {
    if !tc.expression return metrics;

    events: [..] Visit.Dumb_Value;

    defer array_free(events);
    Visit.get_values(cast(*Code_Node) tc.expression.block, *events);
    loop_depth: s32;
    conditional_depth: s32;
    cases: s32;
    conditionals: [..] *Code_If;
    conditional_changes: [..] bool;

    defer array_free(conditionals);
    defer array_free(conditional_changes);
    source_file, source_module := get_file_and_module(cast(*Code_Node) tc.expression);

    for event: events {
        node := event.node;
        if node.kind == .FOR || node.kind == .WHILE {
            if event.preorder {
                increment_metric(*loop_depth);
                if loop_depth > metrics.maximum_loop_depth metrics.maximum_loop_depth = loop_depth;
            } else loop_depth -= 1;
        }

        if node.kind == .IF {
            conditional := cast(*Code_If) node;

            if event.preorder {
                chained := false;

                if conditionals.count > 0 {
                    parent := conditionals[conditionals.count - 1];
                    chained = parent.else_block != null && parent.else_block.statements.count == 1 && parent.else_block.statements[0] == node && parent.else_block.l0 == node.l0 && parent.else_block.c0 == node.c0;
                }

                weight := conditional_depth;
                if conditional.if_flags & .IS_SWITCH_STATEMENT weight += 1;
                else if conditional.if_flags & .IS_IFX weight += 1;
                else weight += 2;
                if conditional.else_block weight += 1;

                assert(metrics.if_points <= 0x7fffffff - weight);
                metrics.if_points += weight;
                array_add(*conditionals, conditional);
                array_add(*conditional_changes, !chained);
                if !chained increment_metric(*conditional_depth);

                metrics.maximum_if_depth = max(metrics.maximum_if_depth, conditional_depth);
            } else {
                assert(conditionals.count > 0);
                if conditional_changes[conditional_changes.count - 1] conditional_depth -= 1;
                conditionals.count -= 1;
                conditional_changes.count -= 1;
            }
        }

        if !event.preorder continue;

        if node.kind == {
            case .CASE;
                increment_metric(*cases);
            case .IDENT;
                if is_global_ident(cast(*Code_Ident) node) increment_metric(*metrics.global_reads);
            case .BINARY_OPERATOR;
                binary := cast(*Code_Binary_Operator) node;
                if is_metric_assignment(binary.operator_type) {
                    increment_metric(*metrics.reassignments);

                    if binary.left && binary.left.kind == .IDENT && is_global_ident(cast(*Code_Ident) binary.left) {
                        increment_metric(*metrics.global_writes);
                    }
                }
            case .PROCEDURE_CALL;
                call := cast(*Code_Procedure_Call) node;
                target := metric_call_target(call.resolved_procedure_expression);

                if target && (target.kind == .TYPE_INSTANTIATION || target.kind == .STRUCT || target.kind == .ENUM) continue;

                increment_metric(*metrics.total_calls);

                if !target {
                    increment_metric(*metrics.nonconstant_calls);
                    continue;
                }

                assert(target.kind == .PROCEDURE_HEADER, "Codex: unsupported resolved call target metadata");
                file, module := get_file_and_module(target);

                if module && (module.module_name == "Basic" || module.module_name == "internal/basic") {
                    name := (cast(*Code_Procedure_Header) target).name;
                    if name == "alloc" || name == "Dynamic_New" || name == "NewArray" || name == "alloc_string" || name == "copy_string" || name == "New" || name == "realloc" increment_metric(*metrics.allocations);
                    if name == "free" || name == "realloc" increment_metric(*metrics.frees);
                }

                if file && file == source_file increment_metric(*metrics.same_file_calls);
                if module && module == source_module increment_metric(*metrics.same_module_calls);
        }
    }

    metrics.if_points += cast(s32) Metric_Math.sqrt(cast(float64) cases);
    fraction := Metric_Math.sqrt(cast(float64) cases);
    if fraction - cast(float64) cast(s32) fraction >= 0.5 increment_metric(*metrics.if_points);

    return metrics;
}

#scope_file
#import "Basic";
#import "Compiler";
Visit :: #import "Code_Visit";

increment_metric :: (value: *s32) {
    assert(value.* < 0x7fffffff, "Codex: metric count is not representable");
    value.* += 1;
}

is_metric_assignment :: (op: Operator_Type) -> bool {
    return op == cast(Operator_Type) #char "=" || op == .PLUS_ASSIGN || op == .MINUS_ASSIGN ||
        op == .TIMES_ASSIGN || op == .DIV_ASSIGN || op == .MOD_ASSIGN || op == .SHIFT_LEFT_ASSIGN ||
        op == .SHIFT_RIGHT_ASSIGN || op == .ROTATE_LEFT_ASSIGN || op == .ROTATE_RIGHT_ASSIGN ||
        op == .BITWISE_AND_ASSIGN || op == .BITWISE_OR_ASSIGN || op == .BITWISE_XOR_ASSIGN ||
        op == .LOGICAL_AND_ASSIGN || op == .LOGICAL_OR_ASSIGN;
}

metric_call_target :: (node: *Code_Node) -> *Code_Node {
    // Resolve only metadata links, never execute or descend into a callee's body.
    for 0..63 {
        if !node return null;

        if node.kind == .IDENT {
            declaration := (cast(*Code_Ident) node).resolved_declaration;
            if !declaration return null;
            node = declaration.expression;
        } else if node.kind == .RESOLVED_OVERLOAD {
            node = (cast(*Code_Resolved_Overload) node).result;
        } else if node.kind == .CAST {
            node = (cast(*Code_Cast) node).expression;
        } else return node;
    }

    assert(false, "Codex: cyclic or excessively deep call target metadata");

    return null;
}

Metric_Math :: #import "Math";
