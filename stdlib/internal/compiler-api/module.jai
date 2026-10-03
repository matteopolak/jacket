// Independent private helpers. Public ABI records live in Compiler.
#load "helpers.jai";
#load "primitives.jai";
#load "workspaces.jai";
#load "records.jai";
#load "registry.jai";
#load "messages.jai";
#load "data.jai";

has_base_type :: (info: *Type_Info, name: string) -> bool {
    current := info;
    // Embedding is finite by value; following only the first #as member models
    // the public aggregate inheritance relationship without walking arbitrary pointers.
    while current && current.type == .STRUCT {
        s := cast(*Type_Info_Struct) current;
        if s.name == name return true;
        if s.members.count == 0 return false;

        member := s.members[0];
        if !(member.flags & .AS) return false;

        current = member.type;
    }

    return false;
}

operator_text :: (op: s32) -> string {
    if op == {
        case 131; return "==";
        case 132; return "!=";
        case 133; return "&&";
        case 134; return "||";
        case 135; return "<=";
        case 136; return ">=";
        case 137; return "<<";
        case 138; return ">>";
        case 139; return "<<<";
        case 140; return ">>>";
        case 145; return "+=";
        case 146; return "-=";
        case 147; return "*=";
        case 148; return "/=";
        case 149; return "%=";
        case 150; return "<<=";
        case 151; return ">>=";
        case 152; return "<<<=";
        case 153; return ">>>=";
        case 154; return "&=";
        case 155; return "|=";
        case 156; return "^=";
        case 157; return "&&=";
        case 158; return "||=";
        case 168; return "<<";
        case 169; return ".*";
        case 500; return "[]";
    }

    if op > 0 && op <= 127 {
        result := Basic.alloc_string(1);
        result[0] = cast(u8) op;
        return result;
    }

    return "";
}

#scope_file
Basic :: #import "Basic";
