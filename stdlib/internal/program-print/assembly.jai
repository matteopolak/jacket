assembly_operand :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if !node || depth > 512 return false;

    kind := syntax_string(node, .ASM_KIND);
    if kind == "declaration" {
        Basic.append(builder, syntax_string(node, .NAME));

        if syntax_int(node, .COLON) {
            Basic.append(builder, ": ");
            Basic.append(builder, syntax_string(node, .CLASS));
        }

        pin := syntax_string(node, .PIN);
        if pin.count {
            Basic.append(builder, " === ");
            Basic.append(builder, pin);
        }

        return true;
    }

    if kind == "memory" {
        Basic.append(builder, "[");
        terms := syntax_child(node, .TERMS);
        if !terms return false;

        if terms.kind == .COMMA_SEPARATED_ARGUMENTS {
            for arg, i: (cast(*C.Code_Comma_Separated_Arguments) terms).arguments if !assembly_term(builder, arg.node, i > 0, depth + 1) return false;
        } else if !assembly_term(builder, terms, false, depth + 1) return false;

        Basic.append(builder, "]");
        if syntax_int(node, .BROADCAST) Basic.append(builder, "!");

        return true;
    }

    return expression(builder, node, false, true, depth + 1);
}

assembly_term :: (builder: *Basic.String_Builder, term: *C.Code_Node, separator: bool, depth: int) -> bool {
    if syntax_int(term, .NEGATE) Basic.append(builder, " - ");
    else if separator Basic.append(builder, " + ");
    if !expression(builder, syntax_child(term, .VALUE), false, true, depth + 1) return false;

    scale := syntax_child(term, .SCALE);
    if scale {
        Basic.append(builder, " * ");
        if !expression(builder, scale, false, true, depth + 1) return false;
    }

    return true;
}

assembly_item :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if depth > 512 return false;
    if syntax_string(node, .ASM_KIND) == "declaration" return assembly_operand(builder, node, depth + 1);

    Basic.append(builder, syntax_string(node, .MNEMONIC));
    suffix := syntax_string(node, .SUFFIX);
    if suffix.count {
        Basic.append(builder, ".");
        Basic.append(builder, suffix);
    }

    size := syntax_child(node, .SIZE);
    if size {
        Basic.append(builder, "?");
        if !expression(builder, size, false, true, depth + 1) return false;
    }

    operands := syntax_child(node, .OPERANDS);
    round := syntax_string(node, .ROUND);
    round_written := false;

    if operands {
        Basic.append(builder, " ");

        if operands.kind == .COMMA_SEPARATED_ARGUMENTS {
            for arg, i: (cast(*C.Code_Comma_Separated_Arguments) operands).arguments {
                if i Basic.append(builder, ", ");
                if !assembly_operand(builder, arg.node, depth + 1) return false;

                if !round_written && round.count && syntax_string(arg.node, .ASM_KIND) != "memory" {
                    Basic.append(builder, round);
                    round_written = true;
                }
            }
        } else {
            if !assembly_operand(builder, operands, depth + 1) return false;

            if round.count && syntax_string(operands, .ASM_KIND) != "memory" {
                Basic.append(builder, round);
                round_written = true;
            }
        }
    }

    if round.count && !round_written return false;

    mask := syntax_child(node, .MASK);
    if mask {
        if syntax_int(node, .ZEROING) Basic.append(builder, " &* ");
        else Basic.append(builder, " & ");
        if !assembly_operand(builder, mask, depth + 1) return false;
    }

    return true;
}

assembly :: (builder: *Basic.String_Builder, node: *C.Code_Node, depth: int) -> bool {
    if !syntax_int(node, .ASM_PRESENT) || depth > 512 return false;

    Basic.append(builder, "#asm ");
    Basic.append(builder, syntax_string(node, .FEATURES));
    Basic.append(builder, " {\n");
    items := syntax_child(node, .ASM_ITEMS);
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
