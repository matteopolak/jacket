#scope_module
#import "Basic";

gm_aggregate_expression :: (cur: CXCursor, children: [] CXCursor) -> *Expression {
    session := gm_current_session();
    if !session return null;

    type := gm_build_type(clang_getCursorType(cur), cur, true, 0);
    underlying := gm_underlying(type);

    if !underlying || !underlying.type_of_struct {
        gm_construction_error("aggregate initializer requires a represented record type");
        return null;
    }

    values: [..] *Expression;

    if underlying.template_instantiation_params.count > 0 {
        gm_construction_error("template aggregate initializers require complete instantiated type-name rendering");
        return null;
    }

    defer array_free(values);

    for child: children {
        value := gm_build_expression(child, cur);
        if !value return null;
        array_add(*values, value);
    }

    result := GM_Sessions.new_owned(session, Struct_Literal);
    result.kind = .STRUCT_LITERAL;
    result.type_name = GM_Sessions.copy_text(session, gm_name(cast(*Declaration) underlying.type_of_struct));
    result.values = GM_Sessions.copy_array(session, values);

    return cast(*Expression) result;
}
