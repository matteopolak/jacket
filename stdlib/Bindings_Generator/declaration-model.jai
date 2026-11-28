// Public data contract from the specification; private algorithms are independent.

// A position in a parsed source file: the file, and the line and column within it.
Location :: struct {
    file: CXFile;
    line: u32;
    column: u32;
}

// A documentation comment attached to a declaration.
//
// `text` is the raw comment text. `location` is where the comment starts and is zeroed when the comment does
// not begin on an earlier line than the declaration, such as a trailing comment.
Comment :: struct {
    location: Location;
    text: string;
}

// A named entity in the generated model, and the first member of every more specific kind.
//
// Variables, fields and macros are plain `Declaration`s; [Function], [Struct], [Enum], [Typedef], [Namespace]
// and [Bitfield] embed it with `#as using`. `kind` (from [Expression]) says which one a pointer really
// points to, so check it before casting a `*Declaration` to a specific kind.
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

// A scope's declarations in order. `parent` is the enclosing block, or null at the outermost one.
Block :: struct {
    parent: *Block;
    members: [..] *Declaration;
}

// A C++ namespace and the declarations it contains.
Namespace :: struct {
    #as using declaration: Declaration;
    flags: Flags;
    Flags :: enum_flags u8 {
        FLATTEN :: 0x01;
        IS_EMPTY :: 0x02
    }
    declarations: [..] *Declaration;
}

// A struct, union or class, or an Objective-C interface, protocol or category; see [Struct.flags].
//
// `declarations` holds the members and nested declarations, `parents` the base declarations and
// `virtual_functions` the virtual methods.
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

// Pairs `declaration` with the member `place` that fixes where it appears in a [Struct].
//
// [Struct.overlays] holds these; the generator adds one for each vtable pointer declaration.
Declaration_Overlay :: struct {
    place: *Declaration;
    declaration: *Declaration;
}

// Layout and vtable data the generator computes for a [Struct] beyond what the struct records.
//
// [find_or_create_extra_info] returns it. The `vtable_*` fields describe the virtual-method table and its
// inherited parent, `largest_natural_alignment` and `struct_alignment` are in bytes, and
// `padding_declaration`
// is the generated padding member, when one was added.
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
    explicit_layout: bool;
    inherits_from_template: bool;
    modified_declarations: [..] *Declaration;
    all_scope_declarations: [..] *Declaration;
    referenced_type_names: [..] string;
    raw_struct: *Struct;
}

// A run of adjacent C bitfield members emitted as one unit.
//
// `members` holds the bit-field declarations. `alignment_type` is set to a type whose alignment exceeds the
// unit's own, and is null otherwise.
Bitfield :: struct {
    #as using declaration: Declaration;
    members: [..] *Declaration;
    alignment_type: *CType;
    flags: Flags;
    Flags :: enum_flags u8 {
        GENERATE_HELPERS;
        LAYOUT_UNIT
    }
}

// Tracks the bitfield run being built while the members of `parent` are visited.
//
// `bitfield` is the run in progress, or null between runs; `first_decl` is its first member.
Bitfield_Status :: struct {
    parent: *Struct;
    bitfield: *Bitfield;
    first_decl: *Declaration;
}

// A C or C++ enumeration. `enumerates` holds the enumerators in declaration order; see [Enum.flags].
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

// A typedef declaration. The aliased type is the declaration's `type`; see [Typedef.flags].
Typedef :: struct {
    #as using declaration: Declaration;
    unwrap: bool;
    flags: Typedef_Flags;
    Typedef_Flags :: enum_flags {
        ISA;
        DISTINCT
    }
}

// A function or method declaration.
//
// Its signature is the [Function_Type] that `type` refers to, and `foreign_name` (from [Declaration]) is the
// linker symbol; see [Function.flags] for virtual methods.
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

// A COM interface UUID, as text, and the place it was declared.
ComUuid :: struct {
    location: Location;
    uuid: string;
}
