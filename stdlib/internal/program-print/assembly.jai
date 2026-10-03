assembly_operand :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if !node || depth > 512 return false;

    kind := syntax_string(node, "__syntax_asm_kind");
    if kind == "declaration" {
        Basic.append(builder, syntax_string(node, "__syntax_name"));

        if syntax_int(node, "__syntax_colon") {
            Basic.append(builder, ": ");
            Basic.append(builder, syntax_string(node, "__syntax_class"));
        }

        pin := syntax_string(node, "__syntax_pin");
        if pin.count {
            Basic.append(builder, " === ");
            Basic.append(builder, pin);
        }

        return true;
    }

    if kind == "memory" {
        Basic.append(builder, "[");
        terms := syntax_child(node, "__syntax_terms");
        if !terms return false;

        if terms.kind == .COMMA_SEPARATED_ARGUMENTS {
            for arg, i: (cast(*C.Code_Comma_Separated_Arguments) terms).arguments if !assembly_term(builder, arg.node, i > 0, depth + 1) return false;
        } else if !assembly_term(builder, terms, false, depth + 1) return false;

        Basic.append(builder, "]");
        if syntax_int(node, "__syntax_broadcast") Basic.append(builder, "!");

        return true;
    }

    return expression(builder, node, false, true, depth + 1);
}

assembly_term :: (builder: *Basic.String_Builder, term: *C.Code_Node, separator: bool, depth: int) -> bool {
    if syntax_int(term, "__syntax_negate") Basic.append(builder, " - ");
    else if separator Basic.append(builder, " + ");
    if !expression(builder, syntax_child(term, "__syntax_value"), false, true, depth + 1) return false;

    scale := syntax_child(term, "__syntax_scale");
    if scale {
        Basic.append(builder, " * ");
        if !expression(builder, scale, false, true, depth + 1) return false;
    }

    return true;
}

assembly_item :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if depth > 512 return false;
    if syntax_string(node, "__syntax_asm_kind") == "declaration" return assembly_operand(builder, node, depth + 1);

    Basic.append(builder, syntax_string(node, "__syntax_mnemonic"));
    suffix := syntax_string(node, "__syntax_suffix");
    if suffix.count {
        Basic.append(builder, ".");
        Basic.append(builder, suffix);
    }

    size := syntax_child(node, "__syntax_size");
    if size {
        Basic.append(builder, "?");
        if !expression(builder, size, false, true, depth + 1) return false;
    }

    operands := syntax_child(node, "__syntax_operands");
    round := syntax_string(node, "__syntax_round");
    round_written := false;

    if operands {
        Basic.append(builder, " ");

        if operands.kind == .COMMA_SEPARATED_ARGUMENTS {
            for arg, i: (cast(*C.Code_Comma_Separated_Arguments) operands).arguments {
                if i Basic.append(builder, ", ");
                if !assembly_operand(builder, arg.node, depth + 1) return false;

                if !round_written && round.count && syntax_string(arg.node, "__syntax_asm_kind") != "memory" {
                    Basic.append(builder, round);
                    round_written = true;
                }
            }
        } else {
            if !assembly_operand(builder, operands, depth + 1) return false;

            if round.count && syntax_string(operands, "__syntax_asm_kind") != "memory" {
                Basic.append(builder, round);
                round_written = true;
            }
        }
    }

    if round.count && !round_written return false;

    mask := syntax_child(node, "__syntax_mask");
    if mask {
        if syntax_int(node, "__syntax_zeroing") Basic.append(builder, " &* ");
        else Basic.append(builder, " & ");
        if !assembly_operand(builder, mask, depth + 1) return false;
    }

    return true;
}

assembly :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if !syntax_int(node, "__syntax_asm_present") || depth > 512 return false;

    Basic.append(builder, "#asm ");
    Basic.append(builder, syntax_string(node, "__syntax_features"));
    Basic.append(builder, " {\n");
    items := syntax_child(node, "__syntax_asm_items");
    if items {
        if items.kind == .COMMA_SEPARATED_ARGUMENTS {
            for item: (cast(*C.Code_Comma_Separated_Arguments) items).arguments {
                if !assembly_item(builder, item.node, depth + 1) return false;
                Basic.append(builder, ";\n");
            }
        } else {
            if !assembly_item(builder, items, depth + 1) return false;
            Basic.append(builder, ";\n");
        }
    }

    Basic.append(builder, "}");

    return true;
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
