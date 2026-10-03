// Public data contract from the reviewed specification; private algorithms are independent.

Location :: struct {
    file: CXFile;
    line: u32;
    column: u32;
}

Comment :: struct {
    location: Location;
    text: string;
}

Declaration :: struct {
    #as using _expression: Expression;
    location: Location;
    name: string;
    output_name: string;
    outputted_jai_name: string;
    type: *CType;
    expression: *Expression;
    parent: *Declaration;
    should_unwrap: bool;
    bitfield_width := -1;
    bit_offset_in_parent := -1;
    field_alignment := 0;
    explicit_field_alignment := 0;
    macro_value_result: enum {
        NEEDS_CHECKING;
        KEEP;
        DISCARD;
    }
    storage_class: CX_StorageClass = .None;
    decl_flags: Declaration_Flags;
    using_except: string;
    comment: Comment;
    library: *Library_Info;
    foreign_name: string;
    size_condition_type: *CType;
}

Block :: struct {
    parent: *Block;
    members: [..] *Declaration;
}

Namespace :: struct {
    #as using declaration: Declaration;
    flags: Flags;
    Flags :: enum_flags u8 {
        FLATTEN :: 0x01;
        IS_EMPTY :: 0x02
    }
    declarations: [..] *Declaration;
}

Struct :: struct {
    #as using declaration: Declaration;
    metadata: [..] *Declaration;
    parents: [..] *Declaration;
    virtual_functions: [..] *Function;
    declarations: [..] *Declaration;
    template_type_params: [..] *Declaration;
    overlays: [..] Declaration_Overlay;
    primary_field: *Declaration;
    flags: Flags;
    Flags :: enum_flags {
        NONE;
        IS_ANON;
        IS_ANON_INLINE_TYPE;
        IS_UNION;
        HAS_CONSTRUCTOR;
        HAS_COPY_CONSTRUCTOR;
        HAS_MOVE_CONSTRUCTOR;
        HAS_DESTRUCTOR;
        NO_STRUCT_CHECKS;
        MIMIC_SPACING;
        OBJC_PROTOCOL;
        OBJC_INTERFACE;
        OBJC_CATEGORY;
        IS_RAW;
        IS_ABSTRACT;
        IS_VTABLE;
        MAY_HAVE_SUBCLASS
    }
}

Declaration_Overlay :: struct {
    place: *Declaration;
    declaration: *Declaration;
}

Struct_Extra_Info :: struct {
    vtable: [..] *Function;
    vtable_struct: *Struct;
    vtable_pointer_declaration: *Declaration;
    vtable_parent: *Struct;
    vtable_parent_info: *Struct_Extra_Info;
    vtable_parent_declaration: *Declaration;
    padding_declaration: *Declaration;
    largest_natural_alignment: int;
    struct_alignment: int;
    has_field: bool;
    has_nonzero_sized_field: bool;
    inherits_from_template: bool;
    modified_declarations: [..] *Declaration;
    all_scope_declarations: [..] *Declaration;
    referenced_type_names: [..] string;
    raw_struct: *Struct;
}

Bitfield :: struct {
    #as using declaration: Declaration;
    members: [..] *Declaration;
    alignment_type: *CType;
    flags: Flags;
    Flags :: enum_flags u8 {
        GENERATE_HELPERS
    }
}

Bitfield_Status :: struct {
    parent: *Struct;
    bitfield: *Bitfield;
    first_decl: *Declaration;
}

Enum :: struct {
    #as using declaration: Declaration;
    enumerates: [..] *Declaration;
    flags: Enum_Flags;
    Enum_Flags :: enum_flags {
        IS_ENUM_FLAGS;
        VALUES_IN_HEX;
        MIMIC_SPACING;
        ALIAS_ORIGINAL_NAMES
    }
}

Typedef :: struct {
    #as using declaration: Declaration;
    unwrap: bool;
    flags: Typedef_Flags;
    Typedef_Flags :: enum_flags {
        ISA;
        DISTINCT
    }
}

Function :: struct {
    #as using declaration: Declaration;
    deleting_foreign_name: string;
    base_foreign_name: string;
    flags: Function_Flags;
    Function_Flags :: enum_flags {
        VIRTUAL;
        PURE_VIRTUAL
    }
}

ComUuid :: struct {
    location: Location;
    uuid: string;
}
