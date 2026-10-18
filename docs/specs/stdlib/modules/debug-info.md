# debug_info

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `parse_codeview_types :: (data: [] u8, expect_codeview_version := true, verbose := false) -> bool, [] *Codeview_Type`
- `dump_codeview_types :: (types: [] *Codeview_Type, ids: [] *Codeview_Type) -> bool`
- `Codeview_Debug_Info :: struct`
- `get_string_from_table :: (info: Codeview_Debug_Info, id: Codeview_String_Table_Id) -> string, success: bool`
- `parse_codeview_symbols :: (info: *Codeview_Debug_Info, data: [] u8, verbose := false) -> success: bool`
- `get_filename_for_file_id :: (info: Codeview_Debug_Info, id: Codeview_File_Checksum_Id) -> string, success: bool`
- `Codeview_Symbol :: struct`
- `Codeview_Symbol_Compile :: struct`
- `Codeview_Symbol_Compile3 :: struct`
- `Codeview_Symbol_Procedure :: struct`
- `is_procedure :: (symbol: *Codeview_Symbol) -> bool`
- `Codeview_Symbol_Regrel :: struct`
- `Codeview_Symbol_Data :: struct`
- `Codeview_Symbol_Buildinfo :: struct`
- `Codeview_Symbol_Frame_Proc :: struct`
- `Codeview_Symbol_Local :: struct`
- `Range_Attribute_Flags :: enum_flags u16`
- `Codeview_Symbol_Defrange_Register :: struct`
- `Codeview_Symbol_Defrange_Framepointer_Rel :: struct`
- `Codeview_Symbol_Defrange_Framepointer_Rel_Full_Scope :: struct`
- `Codeview_Symbol_Defrange_Register_Rel :: struct`
- `get_spilled_udt_member :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> bool`
- `get_offset_parent :: (s: *Codeview_Symbol_Defrange_Register_Rel) -> u16`
- `Codeview_Symbol_Objname :: struct`
- `Codeview_Symbol_Block :: struct`
- `Codeview_Symbol_Constant :: struct`
- `Numeric_Leaf_Value :: struct`
- `Codeview_Symbol_Unamespace :: struct`
- `Codeview_Symbol_Inline_Site :: struct`
- `Codeview_Symbol_Udt :: struct`
- `Codeview_Symbol_Function_List :: struct`
- `Codeview_Binary_Annotation :: struct`
- `Codeview_Address_Range :: struct`
- `Codeview_Address_Gap :: struct`
- `Type_Id :: #type,distinct u32`
- `Item_Id :: #type,isa Type_Id`
- `Codeview_Parser_State :: struct`
- `Codeview_Type_Parser_State :: struct`
- `Codeview_Symbol_Parser_State :: struct`
- `Codeview_Type :: struct`
- `Codeview_Builtin :: struct`
- `Codeview_Struct :: struct`
- `Codeview_Enum :: struct`
- `Codeview_Array :: struct`
- `Codeview_Pointer :: struct`
- `Codeview_Procedure :: struct`
- `Codeview_Method_Function :: struct`
- `Codeview_Overloaded_Method :: struct`
- `Codeview_Method_List :: struct`
- `Codeview_Method :: struct`
- `Codeview_List :: struct`
- `Codeview_Arguments :: struct`
- `Codeview_String_Id :: struct`
- `Codeview_Udt_Source_Line :: struct`
- `Codeview_Function_Id :: struct`
- `Codeview_Alias :: struct`
- `Codeview_Typeserver2 :: struct`
- `Codeview_Modifier :: struct`
- `Codeview_Member :: struct`
- `Codeview_Enumerate :: struct`
- `Codeview_Nested_Type :: struct`
- `Codeview_Line_Info_Sequence :: struct`
- `Codeview_Lines_File_Block :: struct`
- `Codeview_File_Checksum_Id :: #type,isa u32`
- `Codeview_Line_Info :: struct`
- `Codeview_Column_Type :: u16`
- `Codeview_String_Table_Id :: #type,isa u32`
- `Codeview_File_Checksum :: struct`
- `get_line_number :: (using info: Codeview_Line_Info) -> u32`
- `get_line_number_end_delta :: (using info: Codeview_Line_Info) -> u8`
- `line_info_is_for_statement :: (using info: Codeview_Line_Info) -> bool`
- `get_id_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type: Type_Id) -> *Codeview_Type, success: bool`
- `get_type :: (types: [] *Codeview_Type, type: Type_Id) -> *Codeview_Type, success: bool`
- `describe_builtin :: (type_id: Type_Id) -> string`
- `describe_id_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type_id: Type_Id) -> string, success: bool`
- `describe_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type_id: Type_Id) -> string, success: bool`
- `describe_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type: *Codeview_Type) -> string, success: bool`
- `Codeview_Inlinee_Line_Info :: struct`
- `decode_inline_site_line_info :: (info: Codeview_Debug_Info, procedure: Codeview_Symbol_Procedure, site: Codeview_Symbol_Inline_Site) -> [] Codeview_Line_Info_Sequence, success: bool`
- `Guid :: struct`
- `overlay_inlinesites :: (sequences: [] Codeview_Line_Info_Sequence, procedure: Codeview_Symbol_Procedure, inline_sites: [] *Codeview_Symbol_Inline_Site, debug_info: Codeview_Debug_Info) -> bool, [] Codeview_Line_Info_Sequence`
- `overlay_inlinesite :: (sequences: [] Codeview_Line_Info_Sequence, procedure: Codeview_Symbol_Procedure, inline_site: *Codeview_Symbol_Inline_Site, debug_info: Codeview_Debug_Info) -> bool, new_sequences: [] Codeview_Line_Info_Sequence`
- `copy_and_truncate_blocks :: (sequence: *Codeview_Line_Info_Sequence, old_sequence_text_offset: u32)`
- `Codeview_Symbol_Section :: enum u32`
- `Codeview_Symbol_Type :: enum u16`
- `Codeview_Leaf_Type :: enum u16`
- `Codeview_Access :: enum u16`
- `Struct_Properties :: enum_flags u16`
- `Function_Attributes :: enum_flags u8`
- `Codeview_Inlinee_Source_Variant :: enum u32`
- `Codeview_Binary_Annotation_Opcode :: enum u8`
- `Codeview_Builtin_Type :: enum`
- `Codeview_Pointer_Type :: enum u8`
- `Debug_Info :: struct`
- `Compilation_Unit :: struct`
- `Debug_Type :: struct`
- `Subprogram :: struct`
- `Declaration :: struct`
- `Lexical_Block :: struct`
- `Frame_Common_Information_Entry :: struct`
- `Frame_Description_Entry :: struct`
- `Call_Frame_Address_Instruction :: struct`
- `Dwarf_Language :: enum u16`
- `parse_dwarf_debug_info :: (debug_info_data: [] u8, abbrev: [] u8, line_programs: [] u8, debug_string_data: [] u8, verbose := false) -> Debug_Info, success: bool`
- `parse_compilation_unit_v4 :: (using parse_context: *Parse_Context, cu_index: int, offset: int, full_cu_data: [] u8, abbrev: [] u8, line_programs: [] u8) -> Compilation_Unit, success: bool`
- `get_type :: (cu: Compilation_Unit, type: u32) -> *Debug_Type`
- `parse_common_information_entry :: (data: [] u8, offset_in: s64) -> success: bool, Frame_Common_Information_Entry, offset: s64`
- `parse_frame_description_entry :: (cie: Frame_Common_Information_Entry, frame_section_address: u64, data: [] u8, offset_in: s64) -> success: bool, Frame_Description_Entry, offset: s64`
- `decode_cfa_instructions :: (data: [] u8) -> [] Call_Frame_Address_Instruction, success: bool`
- `Dwarf_Attribute_Type :: enum u64`
- `Dwarf_Attribute_Format :: enum`
- `Dwarf_Tag :: enum`
- `Dwarf_Base_Type :: enum`
- `Call_Frame_Address_Instruction_Type :: enum u8`
- `Dwarf_Registers_x64 :: enum u8`
- `Dwarf_File_Id :: #type,isa u32`
- `Line_Info :: struct`
- `Line_Info_Sequence :: struct`
- `File_Info :: struct`
- `Line_Mapping :: struct`
- `Line_Program_Header :: struct`
- `get_filename_for_file_id :: (info: Line_Info, file_id: Dwarf_File_Id) -> string`
- `get_filename :: (info: Line_Info, file_info: File_Info) -> string`
- `get_line_mapping_for_address :: (info: Line_Info, address: u64) -> Line_Mapping, success: bool`
- `get_line_mappings_for_address_range :: (info: Line_Info, start: u64, end_exclusive: u64) -> [] Line_Mapping, success: bool`
- `Free_Page_Map :: struct`
- `init :: (fpm: *Free_Page_Map, cbitsFpm: s32, cbPg: s32)`
- `ensure_room :: (using fpm: *Free_Page_Map, upnMac: UPN, fill: bool)`
- `set_all :: (using fpm: *Free_Page_Map)`
- `is_free_page :: (using fpm: Free_Page_Map, page_number: UPN) -> bool`
- `alloc_page :: (using fpm: *Free_Page_Map, page_number: UPN)`
- `free_page :: (using fpm: *Free_Page_Map, page_number: UPN)`
- `is_fpm_page_number :: (using fpm: Free_Page_Map, pn: UPN) -> bool`
- `Int_Set :: struct`
- `cardinality :: (set: Int_Set) -> u64`
- `is_set :: (set: Int_Set, i: int) -> bool`
- `Msf :: struct`
- `is_msf :: (file_content: string) -> bool`
- `parse_msf :: (file_content: string, path := "", verbose := false) -> success: bool, Msf`
- `get_stream_data :: (msf: Msf, stream_index: UNSN) -> [] u8, success: bool`
- `get_stream_size :: (msf: Msf, stream_index: UNSN) -> s64`
- `is_valid_stream_number :: (msf: Msf, stream_index: UNSN) -> bool`
- `is_valid_user_stream_number :: (msf: Msf, stream_index: UNSN) -> bool`
- `stream_exists :: (msf: Msf, stream_index: UNSN) -> bool`
- `is_valid_page_number :: (msf: Msf, pn: UPN) -> bool`
- `verify_integrity :: (msf: Msf) -> bool`
- `is_pdb :: is_msf`
- `Pdb :: struct`
- `Pdb_Header :: struct`
- `Pdb_Version :: enum u32`
- `parse_pdb :: (file_content: string, path := "", verbose := false) -> success: bool, Pdb`
- `Tpi_Stream_Header :: struct`
- `Tpi_Stream_Version :: enum u32`
- `parse_tpi_stream :: (msf: Msf, stream_index: UNSN, verbose := false) -> [] *Codeview_Type, success: bool`

## Public type members

#### `Codeview_Debug_Info` (struct)

- `types: [] *Codeview_Type`
- `ids: [] *Codeview_Type`
- `symbols: [..] *Codeview_Symbol`
- `string_table: [] u8`
- `file_checksums: [..] Codeview_File_Checksum`
- `line_info: [..] Codeview_Line_Info_Sequence`
- `inlinee_line_info: [..] Codeview_Inlinee_Line_Info`

#### `Codeview_Symbol` (struct)

- `symbol_type: Codeview_Symbol_Type`

Concrete symbol records that declare a leading `using #as base: Codeview_Symbol` store that base member first and expose its `symbol_type` through the `using` member. Where the concrete declaration assigns a fixed tag, the tag values are listed below; procedure records retain the parsed procedure variant. The base begins at aggregate offset zero; later offsets and padding follow the target Jai aggregate ABI. This public record layout is distinct from the serialized CodeView payload layout.

#### `Codeview_Symbol_Compile` (struct)

- `base: Codeview_Symbol` (the common leading storage; its `symbol_type` is fixed to `.COMPILE`)
- `target_cpu: u8`
- `language: u8`
- `flags: u16`
- `compiler_version: string`

#### `Codeview_Symbol_Compile3` (struct)

- `base: Codeview_Symbol` (the common leading storage; its `symbol_type` is fixed to `.COMPILE3`)
- `language:         u8`
- `flags1:           u8`
- `flags2:           u8`
- `reserved:         u8`
- `target_cpu:       u16`
- `fe_version:       Version`
- `be_version:       Version`
- `compiler_version: string`
- `Version :: struct { major: u16; minor: u16; build: u16; qfe: u16; }`

#### `Codeview_Symbol_Procedure` (struct)

- `base: Codeview_Symbol` (common leading storage; `symbol_type` is one of `.LPROC32`, `.GPROC32`, `.LPROC32_ID`, or `.GPROC32_ID` for records recognized by `is_procedure`)
- `parent:             u32`
- `end:                u32`
- `next:               u32`
- `length:             u32`
- `debug_start_offset: u32`
- `debug_end_offset:   u32`
- `type:               Type_Id`
- `offset:             u32`
- `segment_index:      u16`
- `flags:              Flags`
- `name:               string`
- `Flags :: enum_flags u8 { CV_PFLAG_NOFPO :: 0x01; CV_PFLAG_INT :: 0x02; CV_PFLAG_FAR :: 0x04; CV_PFLAG_NEVER :: 0x08; CV_PFLAG_NOTREACHED :: 0x10; CV_PFLAG_CUST_CALL :: 0x20; CV_PFLAG_NOINLINE :: 0x40; CV_PFLAG_OPTDBGINFO :: 0x80; }`
- `inlinesites: [..] *Codeview_Symbol_Inline_Site`

#### `Codeview_Symbol_Regrel` (struct)

- `off:    u32`
- `typind: Type_Id`
- `reg:    u16`
- `name:   string`

#### `Codeview_Symbol_Data` (struct)

- `type:          Type_Id`
- `offset:        u32`
- `segment_index: u16`
- `name:          string`

#### `Codeview_Symbol_Buildinfo` (struct)

- `id: Item_Id #align 2`

#### `Codeview_Symbol_Frame_Proc` (struct)

- `frame_bytes:     u32 #align 2`
- `pad_bytes:       u32 #align 2`
- `pad_offset:      u32 #align 2`
- `save_regs_bytes: u32 #align 2`
- `eh_offset:       u32 #align 2`
- `eh_section_id:   u16 #align 2`
- `flags:           Flags #align 2`
- `Flags :: enum_flags u32 { HAS_ALLOCA :: 0x00000001; HAS_SET_JMP :: 0x00000002; HAS_LONG_JMP :: 0x00000004; HAS_INL_ASM :: 0x00000008; HAS_EH :: 0x00000010; INL_SPEC :: 0x00000020; HAS_SEH :: 0x00000040; NAKED :: 0x00000080; SECURITY_CHECKS :: 0x00000100; ASYNC_EH :: 0x00000200; GS_NO_STACK_ORDERING :: 0x00000400; WAS_INLINED :: 0x00000800; GS_CHECK :: 0x00001000; SAFE_BUFFERS :: 0x00002000; ENCODED_LOCAL_BASE_POINTER_R13 :: 0x0000C000; ENCODED_LOCAL_BASE_POINTER_RSP :: 0x00004000; ENCODED_LOCAL_BASE_POINTER_RBP :: 0x00008000; ENCODED_PARAM_BASE_POINTER_R13 :: 0x00030000; ENCODED_PARAM_BASE_POINTER_RSP :: 0x00010000; ENCODED_PARAM_BASE_POINTER_RBP :: 0x00020000; POGO_ON :: 0x00040000; VALID_COUNTS :: 0x00080000; OPT_SPEED :: 0x00100000; GUARD_CF :: 0x00200000; GUARD_CFW :: 0x00400000; }`

#### `Codeview_Symbol_Local` (struct)

- `type: Type_Id`
- `flags: Flags`
- `name: string`
- `Flags :: enum_flags u16 { IS_PARAM :: 0x0001; ADDR_TAKEN :: 0x0002; COMP_GENX :: 0x0004; IS_AGGREGATE :: 0x0008; IS_AGGREGATED :: 0x0010; IS_ALIASED :: 0x0020; IS_ALIAS :: 0x0040; IS_RET_VALUE :: 0x0080; IS_OPTIMIZED_OUT :: 0x0100; IS_ENREG_GLOB :: 0x0200; IS_ENREG_STAT :: 0x0400; }`

#### `Range_Attribute_Flags` (enum_flags)

- `MAYBE :: 1`

#### `Codeview_Symbol_Defrange_Register` (struct)

- `register:  u16`
- `attribute: Range_Attribute_Flags`
- `range:     Codeview_Address_Range`
- `gaps:      [] Codeview_Address_Gap`

#### `Codeview_Symbol_Defrange_Framepointer_Rel` (struct)

- `frame_pointer_offset: s32`
- `range:                Codeview_Address_Range`
- `gaps:                 [] Codeview_Address_Gap`

#### `Codeview_Symbol_Defrange_Framepointer_Rel_Full_Scope` (struct)

- `frame_pointer_offset: s32`

#### `Codeview_Symbol_Defrange_Register_Rel` (struct)

- `base_register:                        u16`
- `spilled_udt_member_and_offset_parent: u16`
- `base_pointer_offset:                  u32`
- `range:                                Codeview_Address_Range`
- `gaps:                                 [] Codeview_Address_Gap`

#### `Codeview_Symbol_Objname` (struct)

- `signature: u32`
- `name:      string`

#### `Codeview_Symbol_Block` (struct)

- `parent:        u32`
- `end:           u32`
- `length:        u32`
- `offset:        u32`
- `segment_index: u16`
- `name:          string`

#### `Codeview_Symbol_Constant` (struct)

- `type:          Type_Id`
- `value:         Numeric_Leaf_Value`
- `name:          string`

#### `Numeric_Leaf_Value` (struct)

- `type: Type`
- `value: union { f32: float; f64: float64; unsigned: u64; signed: s64; }`

#### `Codeview_Symbol_Unamespace` (struct)

- `name: string`

#### `Codeview_Symbol_Inline_Site` (struct)

- `base: Codeview_Symbol` (common leading storage; `symbol_type` is `.INLINESITE`)
- `parent:     u32`
- `end:        u32`
- `inlinee:    Item_Id`
- `binary_annotations: [..] Codeview_Binary_Annotation`
- `children: [..] *Codeview_Symbol_Inline_Site`

#### `Codeview_Symbol_Udt` (struct)

- `type: Type_Id`
- `name: string`

#### `Codeview_Symbol_Function_List` (struct)

- `functions: [] Function`
- `Function :: struct { func: Type_Id; invocation_count: u32; }`

#### `Codeview_Binary_Annotation` (struct)

- `opcode: Codeview_Binary_Annotation_Opcode`
- `operand1: Operand`
- `operand2: Operand`
- `Operand :: union { unsigned: u32; signed: s32; }`

#### `Codeview_Address_Range` (struct)

- `start_offset: u32`
- `start_section: u16`
- `range_bytes: u16`

#### `Codeview_Address_Gap` (struct)

- `start_offset: u16`
- `range_bytes: u16`

#### `Codeview_Parser_State` (struct)

- `data: [] u8`
- `offset: s64`
- `success: bool`
- `verbose: bool`

#### `Codeview_Type_Parser_State` (struct)

- `types: [..] *Codeview_Type`

#### `Codeview_Symbol_Parser_State` (struct)

- `info: *Codeview_Debug_Info`
- `current_procedure: *Codeview_Symbol_Procedure`
- `open_inlinesites: [..] *Codeview_Symbol_Inline_Site`

#### `Codeview_Type` (struct)

- `kind: Codeview_Leaf_Type`

The concrete typed records that declare `using #as base` embed this common record first under the field name `base`, and expose its `kind` through the `using` member. The `base` member begins at aggregate offset zero; subsequent padding and field offsets follow the target's ordinary Jai aggregate layout because these records declare no explicit packing override. The listed `kind` values are fixed by concrete record type. This is the public aggregate representation used by returned `*Codeview_Type` values; serialized CodeView record bytes are a separate format.

#### `Codeview_Builtin` (struct)

- `base: Codeview_Type` (common leading storage; `kind` is `.BUILTIN`)
- `id: Type_Id`

#### `Codeview_Struct` (struct)

- `base: Codeview_Type` (common leading storage; `kind` is `.STRUCTURE`)
- `num_fields: u16`
- `properties: Struct_Properties`
- `field_list: Type_Id`
- `derived: Type_Id`
- `vshape: u32`
- `size: u64`
- `name: string`
- `unique_name: string`

#### `Codeview_Enum` (struct)

- `num_fields: u16`
- `properties: Struct_Properties`
- `type: Type_Id`
- `field_list: Type_Id`
- `name: string`
- `unique_name: string`

#### `Codeview_Array` (struct)

- `element_type: Type_Id`
- `index_type: Type_Id`
- `size: u64`
- `name: string`

#### `Codeview_Pointer` (struct)

- `type: Type_Id`
- `flags: u32`

#### `Codeview_Procedure` (struct)

- `return_type:        Type_Id`
- `calling_convention: u8`
- `attributes:         u8`
- `num_arguments:      u16`
- `arguments:          Type_Id`

#### `Codeview_Method_Function` (struct)

- `return_type:        Type_Id`
- `class_type:         Type_Id`
- `this_type:          Type_Id`
- `calling_convention: u8`
- `attributes:         u8`
- `num_arguments:      u16`
- `arguments:          Type_Id`
- `this_adjuster:      s32`

#### `Codeview_Overloaded_Method` (struct)

- `count:     u16`
- `list_type: Type_Id`
- `name:      string`

#### `Codeview_Method_List` (struct)

- `elements: [..] Codeview_Method`

#### `Codeview_Method` (struct)

- `attributes:     u16`
- `procedure_type: Type_Id`
- `base_offset:    u32`
- `name:           string`

#### `Codeview_List` (struct)

- `previous:           Type_Id`
- `members:            [..] Codeview_Member`
- `overloaded_methods: [..] Codeview_Overloaded_Method`
- `methods:            [..] Codeview_Method`
- `enumerates:         [..] Codeview_Enumerate`
- `nested:             [..] Codeview_Nested_Type`

#### `Codeview_Arguments` (struct)

- `arguments: [] Type_Id`

#### `Codeview_String_Id` (struct)

- `id: u32`
- `name: string`

#### `Codeview_Udt_Source_Line` (struct)

- `type: Type_Id`
- `src: Type_Id`
- `line: u32`

#### `Codeview_Function_Id` (struct)

- `scope: Type_Id`
- `type: Type_Id`
- `name: string`
- `undocumented: u64`

#### `Codeview_Alias` (struct)

- `type: Type_Id`
- `name: string`

#### `Codeview_Typeserver2` (struct)

- `sig70: Guid`
- `age: u32`
- `name: string`

#### `Codeview_Modifier` (struct)

- `type: Type_Id`
- `modifier: Modifier_Type`
- `Modifier_Type :: enum_flags u16 { CONST :: 0x01; VOLATILE :: 0x02; UNALINGED :: 0x04; }`

#### `Codeview_Member` (struct)

- `access: u16`
- `type: Type_Id`
- `offset: u64`
- `name: string`

#### `Codeview_Enumerate` (struct)

- `access: u16`
- `value: u64`
- `name: string`

#### `Codeview_Nested_Type` (struct)

- `type: Type_Id`
- `name: string`

#### `Codeview_Line_Info_Sequence` (struct)

- `text_offset: u32`
- `segment_index: u16`
- `flags: Flags`
- `length_in_bytes: u32`
- `file_blocks: [..] Codeview_Lines_File_Block`
- `Flags :: enum u16 { HAS_COLUMNS :: 0x0001; }`

#### `Codeview_Lines_File_Block` (struct)

- `file_id: Codeview_File_Checksum_Id`
- `lines: [] Codeview_Line_Info`
- `columns: [] Codeview_Column_Type`

#### `Codeview_Line_Info` (struct)

- `text_offset:             u32`
- `line_and_delta_and_flag: u32`

#### `Codeview_File_Checksum` (struct)

- `id: Codeview_File_Checksum_Id`
- `filename_id: Codeview_String_Table_Id`
- `checksum_type: Checksum_Type`
- `checksum: [] u8`
- `Checksum_Type :: enum u8 { NONE :: 0; MD5 :: 1; SHA1 :: 2; SHA_256 :: 3; }`

#### `Codeview_Inlinee_Line_Info` (struct)

- `inlinee: Type_Id`
- `file_id: Codeview_File_Checksum_Id`
- `source_line: u32`
- `extra_files: [] u32`

#### `Guid` (struct)

- `data1: u32`
- `data2: u16`
- `data3: u16`
- `data4: [8] u8`

#### `Codeview_Symbol_Section` (enum)

- `SYMBOLS              :: 0xF1`
- `LINES                :: 0xF2`
- `STRING_TABLE         :: 0xF3`
- `FILE_CHECKSUMS       :: 0xF4`
- `FRAME_DATA           :: 0xF5`
- `INLINEELINES         :: 0xF6`
- `CROSS_SCOPE_IMPORTS  :: 0xF7`
- `CROSS_SCOPE_EXPORTS  :: 0xF8`
- `IL_LINES             :: 0xF9`
- `FUNC_MDTOKEN_MAP     :: 0xFA`
- `TYPE_MDTOKEN_MAP     :: 0xFB`
- `MERGED_ASSEMBLYINPUT :: 0xFC`
- `COFF_SYMBOL_RVA      :: 0xFD`
- `IGNORE               :: 0x80000000`

#### `Codeview_Symbol_Type` (enum)

- `COMPILE                              :: 0x0001`
- `REGISTER_16t                         :: 0x0002`
- `CONSTANT_16t                         :: 0x0003`
- `UDT_16t                              :: 0x0004`
- `SSEARCH                              :: 0x0005`
- `END                                  :: 0x0006`
- `SKIP                                 :: 0x0007`
- `CVRESERVE                            :: 0x0008`
- `OBJNAME_ST                           :: 0x0009`
- `ENDARG                               :: 0x000a`
- `COBOLUDT_16t                         :: 0x000b`
- `MANYREG_16t                          :: 0x000c`
- `RETURN                               :: 0x000d`
- `ENTRYTHIS                            :: 0x000e`
- `BPREL16                              :: 0x0100`
- `LDATA16                              :: 0x0101`
- `GDATA16                              :: 0x0102`
- `PUB16                                :: 0x0103`
- `LPROC16                              :: 0x0104`
- `GPROC16                              :: 0x0105`
- `THUNK16                              :: 0x0106`
- `BLOCK16                              :: 0x0107`
- `WITH16                               :: 0x0108`
- `LABEL16                              :: 0x0109`
- `CEXMODEL16                           :: 0x010a`
- `VFTABLE16                            :: 0x010b`
- `REGREL16                             :: 0x010c`
- `BPREL32_16t                          :: 0x0200`
- `LDATA32_16t                          :: 0x0201`
- `GDATA32_16t                          :: 0x0202`
- `PUB32_16t                            :: 0x0203`
- `LPROC32_16t                          :: 0x0204`
- `GPROC32_16t                          :: 0x0205`
- `THUNK32_ST                           :: 0x0206`
- `BLOCK32_ST                           :: 0x0207`
- `WITH32_ST                            :: 0x0208`
- `LABEL32_ST                           :: 0x0209`
- `CEXMODEL32                           :: 0x020a`
- `VFTABLE32_16t                        :: 0x020b`
- `REGREL32_16t                         :: 0x020c`
- `LTHREAD32_16t                        :: 0x020d`
- `GTHREAD32_16t                        :: 0x020e`
- `SLINK32                              :: 0x020f`
- `LPROCMIPS_16t                        :: 0x0300`
- `GPROCMIPS_16t                        :: 0x0301`
- `PROCREF_ST                           :: 0x0400`
- `DATAREF_ST                           :: 0x0401`
- `ALIGN                                :: 0x0402`
- `LPROCREF_ST                          :: 0x0403`
- `OEM                                  :: 0x0404`
- `TI16_MAX                             :: 0x1000`
- `REGISTER_ST                          :: 0x1001`
- `CONSTANT_ST                          :: 0x1002`
- `UDT_ST                               :: 0x1003`
- `COBOLUDT_ST                          :: 0x1004`
- `MANYREG_ST                           :: 0x1005`
- `BPREL32_ST                           :: 0x1006`
- `LDATA32_ST                           :: 0x1007`
- `GDATA32_ST                           :: 0x1008`
- `PUB32_ST                             :: 0x1009`
- `LPROC32_ST                           :: 0x100a`
- `GPROC32_ST                           :: 0x100b`
- `VFTABLE32                            :: 0x100c`
- `REGREL32_ST                          :: 0x100d`
- `LTHREAD32_ST                         :: 0x100e`
- `GTHREAD32_ST                         :: 0x100f`
- `LPROCMIPS_ST                         :: 0x1010`
- `GPROCMIPS_ST                         :: 0x1011`
- `FRAMEPROC                            :: 0x1012`
- `COMPILE2_ST                          :: 0x1013`
- `MANYREG2_ST                          :: 0x1014`
- `LPROCIA64_ST                         :: 0x1015`
- `GPROCIA64_ST                         :: 0x1016`
- `LOCALSLOT_ST                         :: 0x1017`
- `PARAMSLOT_ST                         :: 0x1018`
- `ANNOTATION                           :: 0x1019`
- `GMANPROC_ST                          :: 0x101a`
- `LMANPROC_ST                          :: 0x101b`
- `RESERVED1                            :: 0x101c`
- `RESERVED2                            :: 0x101d`
- `RESERVED3                            :: 0x101e`
- `RESERVED4                            :: 0x101f`
- `LMANDATA_ST                          :: 0x1020`
- `GMANDATA_ST                          :: 0x1021`
- `MANFRAMEREL_ST::                     0x1022`
- `MANREGISTER_ST::                     0x1023`
- `MANSLOT_ST                           :: 0x1024`
- `MANMANYREG_ST                        :: 0x1025`
- `MANREGREL_ST                         :: 0x1026`
- `MANMANYREG2_ST::                     0x1027`
- `MANTYPREF                            :: 0x1028`
- `UNAMESPACE_ST                        :: 0x1029`
- `ST_MAX                               :: 0x1100`
- `OBJNAME                              :: 0x1101`
- `THUNK32                              :: 0x1102`
- `BLOCK32                              :: 0x1103`
- `WITH32                               :: 0x1104`
- `LABEL32                              :: 0x1105`
- `REGISTER                             :: 0x1106`
- `CONSTANT                             :: 0x1107`
- `UDT                                  :: 0x1108`
- `COBOLUDT                             :: 0x1109`
- `MANYREG                              :: 0x110a`
- `BPREL32                              :: 0x110b`
- `LDATA32                              :: 0x110c`
- `GDATA32                              :: 0x110d`
- `PUB32                                :: 0x110e`
- `LPROC32                              :: 0x110f`
- `GPROC32                              :: 0x1110`
- `REGREL32                             :: 0x1111`
- `LTHREAD32                            :: 0x1112`
- `GTHREAD32                            :: 0x1113`
- `LPROCMIPS                            :: 0x1114`
- `GPROCMIPS                            :: 0x1115`
- `COMPILE2                             :: 0x1116`
- `MANYREG2                             :: 0x1117`
- `LPROCIA64                            :: 0x1118`
- `GPROCIA64                            :: 0x1119`
- `LOCALSLOT                            :: 0x111a`
- `SLOT                                 :: LOCALSLOT`
- `PARAMSLOT                            :: 0x111b`
- `LMANDATA                             :: 0x111c`
- `GMANDATA                             :: 0x111d`
- `MANFRAMEREL                          :: 0x111e`
- `MANREGISTER                          :: 0x111f`
- `MANSLOT                              :: 0x1120`
- `MANMANYREG                           :: 0x1121`
- `MANREGREL                            :: 0x1122`
- `MANMANYREG2                          :: 0x1123`
- `UNAMESPACE                           :: 0x1124`
- `PROCREF                              :: 0x1125`
- `DATAREF                              :: 0x1126`
- `LPROCREF                             :: 0x1127`
- `ANNOTATIONREF                        :: 0x1128`
- `TOKENREF                             :: 0x1129`
- `GMANPROC                             :: 0x112a`
- `LMANPROC                             :: 0x112b`
- `TRAMPOLINE                           :: 0x112c`
- `MANCONSTANT                          :: 0x112d`
- `ATTR_FRAMEREL                        :: 0x112e`
- `ATTR_REGISTER                        :: 0x112f`
- `ATTR_REGREL                          :: 0x1130`
- `ATTR_MANYREG                         :: 0x1131`
- `SEPCODE                              :: 0x1132`
- `LOCAL_2005                           :: 0x1133`
- `DEFRANGE_2005                        :: 0x1134`
- `DEFRANGE2_2005                       :: 0x1135`
- `SECTION                              :: 0x1136`
- `COFFGROUP                            :: 0x1137`
- `EXPORT                               :: 0x1138`
- `CALLSITEINFO                         :: 0x1139`
- `FRAMECOOKIE                          :: 0x113a`
- `DISCARDED                            :: 0x113b`
- `COMPILE3                             :: 0x113c`
- `ENVBLOCK                             :: 0x113d`
- `LOCAL                                :: 0x113e`
- `DEFRANGE                             :: 0x113f`
- `DEFRANGE_SUBFIELD                    :: 0x1140`
- `DEFRANGE_REGISTER                    :: 0x1141`
- `DEFRANGE_FRAMEPOINTER_REL            :: 0x1142`
- `DEFRANGE_SUBFIELD_REGISTER           :: 0x1143`
- `DEFRANGE_FRAMEPOINTER_REL_FULL_SCOPE :: 0x1144`
- `DEFRANGE_REGISTER_REL                :: 0x1145`
- `LPROC32_ID                           :: 0x1146`
- `GPROC32_ID                           :: 0x1147`
- `LPROCMIPS_ID                         :: 0x1148`
- `GPROCMIPS_ID                         :: 0x1149`
- `LPROCIA64_ID                         :: 0x114a`
- `GPROCIA64_ID                         :: 0x114b`
- `BUILDINFO                            :: 0x114c`
- `INLINESITE                           :: 0x114d`
- `INLINESITE_END                       :: 0x114e`
- `PROC_ID_END                          :: 0x114f`
- `DEFRANGE_HLSL                        :: 0x1150`
- `GDATA_HLSL                           :: 0x1151`
- `LDATA_HLSL                           :: 0x1152`
- `FILESTATIC                           :: 0x1153`
- `LOCAL_DPC_GROUPSHARED                :: 0x1154`
- `LPROC32_DPC                          :: 0x1155`
- `LPROC32_DPC_ID                       :: 0x1156`
- `DEFRANGE_DPC_PTR_TAG                 :: 0x1157`
- `DPC_SYM_TAG_MAP                      :: 0x1158`
- `ARMSWITCHTABLE                       :: 0x1159`
- `CALLEES                              :: 0x115a`
- `CALLERS                              :: 0x115b`
- `POGODATA                             :: 0x115c`
- `INLINESITE2                          :: 0x115d`
- `HEAPALLOCSITE                        :: 0x115e`
- `MOD_TYPEREF                          :: 0x115f`
- `REF_MINIPDB                          :: 0x1160`
- `PDBMAP                               :: 0x1161`
- `GDATA_HLSL32                         :: 0x1162`
- `LDATA_HLSL32                         :: 0x1163`
- `GDATA_HLSL32_EX                      :: 0x1164`
- `LDATA_HLSL32_EX                      :: 0x1165`
- `INLINEES                             :: 0x1168`
- `RECTYPE_MAX                          :: 0x1169` (one past the highest listed symbol record)
- `RECTYPE_LAST                         :: 0x1168` (`RECTYPE_MAX - 1`)
- `RECTYPE_PAD                          :: 0x1269` (`RECTYPE_MAX + 0x100`)

#### `Codeview_Leaf_Type` (enum)

- `BUILTIN                              :: 0x0000`
- `MODIFIER_16T     :: 0x0001`
- `POINTER_16T      :: 0x0002`
- `ARRAY_16T        :: 0x0003`
- `CLASS_16T        :: 0x0004`
- `STRUCTURE_16T    :: 0x0005`
- `UNION_16T        :: 0x0006`
- `ENUM_16T         :: 0x0007`
- `PROCEDURE_16T    :: 0x0008`
- `MFUNCTION_16T    :: 0x0009`
- `VTSHAPE          :: 0x000a`
- `COBOL0_16T       :: 0x000b`
- `COBOL1           :: 0x000c`
- `BARRAY_16T       :: 0x000d`
- `LABEL            :: 0x000e`
- `NULL             :: 0x000f`
- `NOTTRAN          :: 0x0010`
- `DIMARRAY_16T     :: 0x0011`
- `VFTPATH_16T      :: 0x0012`
- `PRECOMP_16T      :: 0x0013`
- `ENDPRECOMP       :: 0x0014`
- `OEM_16T          :: 0x0015`
- `TYPESERVER_ST    :: 0x0016`
- `SKIP_16T         :: 0x0200`
- `ARGLIST_16T      :: 0x0201`
- `DEFARG_16T       :: 0x0202`
- `LIST             :: 0x0203`
- `FIELDLIST_16T    :: 0x0204`
- `DERIVED_16T      :: 0x0205`
- `BITFIELD_16T     :: 0x0206`
- `METHODLIST_16T   :: 0x0207`
- `DIMCONU_16T      :: 0x0208`
- `DIMCONLU_16T     :: 0x0209`
- `DIMVARU_16T      :: 0x020a`
- `DIMVARLU_16T     :: 0x020b`
- `REFSYM           :: 0x020c`
- `BCLASS_16T       :: 0x0400`
- `VBCLASS_16T      :: 0x0401`
- `IVBCLASS_16T     :: 0x0402`
- `ENUMERATE_ST     :: 0x0403`
- `FRIENDFCN_16T    :: 0x0404`
- `INDEX_16T        :: 0x0405`
- `MEMBER_16T       :: 0x0406`
- `STMEMBER_16T     :: 0x0407`
- `METHOD_16T       :: 0x0408`
- `NESTTYPE_16T     :: 0x0409`
- `VFUNCTAB_16T     :: 0x040a`
- `FRIENDCLS_16T    :: 0x040b`
- `ONEMETHOD_16T    :: 0x040c`
- `VFUNCOFF_16T     :: 0x040d`
- `TI16_MAX         :: 0x1000`
- `MODIFIER         :: 0x1001`
- `POINTER          :: 0x1002`
- `ARRAY_ST         :: 0x1003`
- `CLASS_ST         :: 0x1004`
- `STRUCTURE_ST     :: 0x1005`
- `UNION_ST         :: 0x1006`
- `ENUM_ST          :: 0x1007`
- `PROCEDURE        :: 0x1008`
- `MFUNCTION        :: 0x1009`
- `COBOL0           :: 0x100a`
- `BARRAY           :: 0x100b`
- `DIMARRAY_ST      :: 0x100c`
- `VFTPATH          :: 0x100d`
- `PRECOMP_ST       :: 0x100e`
- `OEM              :: 0x100f`
- `ALIAS_ST         :: 0x1010`
- `OEM2             :: 0x1011`
- `SKIP             :: 0x1200`
- `ARGLIST          :: 0x1201`
- `DEFARG_ST        :: 0x1202`
- `FIELDLIST        :: 0x1203`
- `DERIVED          :: 0x1204`
- `BITFIELD         :: 0x1205`
- `METHODLIST       :: 0x1206`
- `DIMCONU          :: 0x1207`
- `DIMCONLU         :: 0x1208`
- `DIMVARU          :: 0x1209`
- `DIMVARLU         :: 0x120a`
- `BCLASS           :: 0x1400`
- `VBCLASS          :: 0x1401`
- `IVBCLASS         :: 0x1402`
- `FRIENDFCN_ST     :: 0x1403`
- `INDEX            :: 0x1404`
- `MEMBER_ST        :: 0x1405`
- `STMEMBER_ST      :: 0x1406`
- `METHOD_ST        :: 0x1407`
- `NESTTYPE_ST      :: 0x1408`
- `VFUNCTAB         :: 0x1409`
- `FRIENDCLS        :: 0x140a`
- `ONEMETHOD_ST     :: 0x140b`
- `VFUNCOFF         :: 0x140c`
- `NESTTYPEEX_ST    :: 0x140d`
- `MEMBERMODIFY_ST  :: 0x140e`
- `MANAGED_ST       :: 0x140f`
- `ST_MAX           :: 0x1500`
- `TYPESERVER       :: 0x1501`
- `ENUMERATE        :: 0x1502`
- `ARRAY            :: 0x1503`
- `CLASS            :: 0x1504`
- `STRUCTURE        :: 0x1505`
- `UNION            :: 0x1506`
- `ENUM             :: 0x1507`
- `DIMARRAY         :: 0x1508`
- `PRECOMP          :: 0x1509`
- `ALIAS            :: 0x150a`
- `DEFARG           :: 0x150b`
- `FRIENDFCN        :: 0x150c`
- `MEMBER           :: 0x150d`
- `STMEMBER         :: 0x150e`
- `METHOD           :: 0x150f`
- `NESTTYPE         :: 0x1510`
- `ONEMETHOD        :: 0x1511`
- `NESTTYPEEX       :: 0x1512`
- `MEMBERMODIFY     :: 0x1513`
- `MANAGED          :: 0x1514`
- `TYPESERVER2      :: 0x1515`
- `STRIDED_ARRAY    :: 0x1516`
- `HLSL             :: 0x1517`
- `MODIFIER_EX      :: 0x1518`
- `INTERFACE        :: 0x1519`
- `BINTERFACE       :: 0x151a`
- `VECTOR           :: 0x151b`
- `MATRIX           :: 0x151c`
- `VFTABLE          :: 0x151d`
- `ENDOFLEAFRECORD  :: VFTABLE`
- `TYPE_LAST        :: 0x151e` (one past the highest listed type record)
- `TYPE_MAX         :: 0x151d` (`TYPE_LAST - 1`)
- `FUNC_ID          :: 0x1601`
- `MFUNC_ID         :: 0x1602`
- `BUILDINFO        :: 0x1603`
- `SUBSTR_LIST      :: 0x1604`
- `STRING_ID        :: 0x1605`
- `UDT_SRC_LINE     :: 0x1606`
- `UDT_MOD_SRC_LINE :: 0x1607`
- `ID_LAST          :: 0x1608` (one past the highest listed ID record)
- `ID_MAX           :: 0x1607` (`ID_LAST - 1`)
- `NUMERIC          :: 0x8000`
- `CHAR             :: 0x8000`
- `SHORT            :: 0x8001`
- `USHORT           :: 0x8002`
- `LONG             :: 0x8003`
- `ULONG            :: 0x8004`
- `REAL32           :: 0x8005`
- `REAL64           :: 0x8006`
- `REAL80           :: 0x8007`
- `REAL128          :: 0x8008`
- `QUADWORD         :: 0x8009`
- `UQUADWORD        :: 0x800a`
- `REAL48           :: 0x800b`
- `COMPLEX32        :: 0x800c`
- `COMPLEX64        :: 0x800d`
- `COMPLEX80        :: 0x800e`
- `COMPLEX128       :: 0x800f`
- `VARSTRING        :: 0x8010`
- `OCTWORD          :: 0x8017`
- `UOCTWORD         :: 0x8018`
- `DECIMAL          :: 0x8019`
- `DATE             :: 0x801a`
- `UTF8STRING       :: 0x801b`
- `REAL16           :: 0x801c`
- `PAD0             :: 0xf0`
- `PAD1             :: 0xf1`
- `PAD2             :: 0xf2`
- `PAD3             :: 0xf3`
- `PAD4             :: 0xf4`
- `PAD5             :: 0xf5`
- `PAD6             :: 0xf6`
- `PAD7             :: 0xf7`
- `PAD8             :: 0xf8`
- `PAD9             :: 0xf9`
- `PAD10            :: 0xfa`
- `PAD11            :: 0xfb`
- `PAD12            :: 0xfc`
- `PAD13            :: 0xfd`
- `PAD14            :: 0xfe`
- `PAD15            :: 0xff`

#### `Codeview_Access` (enum)

- `NONE :: 0`
- `PUBLIC :: 3`

#### `Struct_Properties` (enum_flags)

- `PACKED              :: 0x0001`
- `CTOR                :: 0x0002`
- `OVLOPS              :: 0x0004`
- `IS_NESTED           :: 0x0008`
- `NESTED_TYPES        :: 0x0010`
- `OP_ASSIGN           :: 0x0020`
- `OP_CAST             :: 0x0040`
- `FWDREF              :: 0x0080`
- `SCOPED              :: 0x0100`
- `UNIQUE_NAME         :: 0x0200`
- `SEALED              :: 0x0400`
- `HFA_OTHER           :: 0x1800`
- `HFA_FLOAT           :: 0x0800`
- `HFA_DOUBLE          :: 0x1000`
- `INTRINSIC           :: 0x2000`
- `MOCOM_UDT_INTERFACE :: 0xC000`
- `MOCOM_UDT_REF       :: 0x4000`
- `MOCOM_UDT_VALUE     :: 0x8000`

#### `Function_Attributes` (enum_flags)

- `CXX_RETURN_UDT :: 0x01`
- `CTOR           :: 0x02`
- `CTOR_V_BASE    :: 0x03`

#### `Codeview_Inlinee_Source_Variant` (enum)

- `NORMAL :: 0`
- `EX     :: 1`

#### `Codeview_Binary_Annotation_Opcode` (enum)

- `INVALID                            :: 0x00`
- `CODE_OFFSET                        :: 0x01`
- `CHANGE_CODE_OFFSET_BASE            :: 0x02`
- `CHANGE_CODE_OFFSET                 :: 0x03`
- `CHANGE_CODE_LENGTH                 :: 0x04`
- `CHANGE_FILE                        :: 0x05`
- `CHANGE_LINE_OFFSET                 :: 0x06`
- `CHANGE_LINE_END_DELTA              :: 0x07`
- `CHANGE_RANGE_KIND                  :: 0x08`
- `CHANGE_COLUMN_START                :: 0x09`
- `CHANGE_COLUMN_END_DELTA            :: 0x0a`
- `CHANGE_CODE_OFFSET_AND_LINE_OFFSET :: 0x0b`
- `CHANGE_CODE_LENGTH_AND_CODE_OFFSET :: 0x0c`
- `CHANGE_COLUMN_END                  :: 0x0d`

#### `Codeview_Builtin_Type` (enum)

- `NOTYPE  :: 0x0000`
- `ABS     :: 0x0002`
- `SEGMENT :: 0x0002`
- `VOID    :: 0x0003`
- `HRESULT :: 0x0008`
- `CHAR    :: 0x0010`
- `SHORT   :: 0x0011`
- `LONG    :: 0x0012`
- `QUAD    :: 0x0013`
- `UCHAR   :: 0x0020`
- `USHORT  :: 0x0021`
- `ULONG   :: 0x0022`
- `UQUAD   :: 0x0023`
- `FLOAT32 :: 0x0040`
- `FLOAT64 :: 0x0041`
- `FLOAT16 :: 0x0046`
- `S8      :: 0x0068`
- `U8      :: 0x0069`
- `RCHAR   :: 0x0070`
- `WCHAR   :: 0x0071`
- `CHAR16  :: 0x007a`
- `CHAR32  :: 0x007b`
- `S16     :: 0x0072`
- `U16     :: 0x0073`
- `S32     :: 0x0074`
- `U32     :: 0x0075`
- `S64     :: 0x0076`
- `U64     :: 0x0077`
- `BOOL8   :: 0x0030`
- `BOOL16  :: 0x0031`
- `BOOL32  :: 0x0032`
- `BOOL64  :: 0x0033`

#### `Codeview_Pointer_Type` (enum)

- `NEAR         :: 0x00`
- `FAR          :: 0x01`
- `HUGE         :: 0x02`
- `BASE_SEG     :: 0x03`
- `BASE_VAL     :: 0x04`
- `BASE_SEGVAL  :: 0x05`
- `BASE_ADDR    :: 0x06`
- `BASE_SEGADDR :: 0x07`
- `BASE_TYPE    :: 0x08`
- `BASE_SELF    :: 0x09`
- `NEAR32       :: 0x0a`
- `FAR32        :: 0x0b`
- `_64          :: 0x0c`
- `UNUSEDPTR    :: 0x0d`

#### `Debug_Info` (struct)

- `compilation_units: [..] Compilation_Unit`

#### `Compilation_Unit` (struct)

- `producer:       string`
- `language:       Dwarf_Language`
- `name:           string`
- `statement_list: u64`
- `low_pc:         u64`
- `high_pc:        u64`
- `types:          [..] Debug_Type`
- `line_info:      Line_Info`
- `subprograms:    Bucket_Array(Subprogram, 32)`
- `globals:        [..] Declaration`

#### `Debug_Type` (struct)

- `offset:    s64`
- `tag:       Dwarf_Tag`
- `name:      string`
- `size:      u64`
- `base_type: Dwarf_Base_Type`

#### `Subprogram` (struct)

- `name:               string`
- `linkage_name:       string`
- `external:           bool`
- `decl_file_id:       Dwarf_File_Id`
- `decl_line:          u32`
- `type_id:            u32`
- `is_main:            bool`
- `calling_convention: u8`
- `low_pc:             u64`
- `high_pc:            u64`
- `all_blocks:         Bucket_Array(Lexical_Block, 8)`
- `root_block:         *Lexical_Block`

#### `Declaration` (struct)

- `tag:          Dwarf_Tag`
- `name:         string`
- `decl_file_id: Dwarf_File_Id`
- `decl_line:    u32`
- `type_id:      u32`
- `kind: enum { PARAMETER; VARIABLE; }` (implicit values 0 and 1)

#### `Lexical_Block` (struct)

- `low_pc:       u64`
- `high_pc:      u64`
- `declarations: [..] Declaration`
- `children:     [..] *Lexical_Block`

#### `Frame_Common_Information_Entry` (struct)

- `length:                  u64`
- `id:                      u32`
- `version:                 u8`
- `augmentation:            string`
- `eh_data:                 u64`
- `code_alignment_factor:   u64`
- `data_alignment_factor:   s64`
- `return_address_register: u64`
- `augmentation_data:       [] u8`
- `initial_instructions:    [] Call_Frame_Address_Instruction`
- `offset:                  u64`

#### `Frame_Description_Entry` (struct)

- `length:            u64`
- `cie_offset:        u64`
- `pc_begin:          u64`
- `pc_range:          u64`
- `augmentation_data: [] u8`
- `instructions:      [] Call_Frame_Address_Instruction`

#### `Call_Frame_Address_Instruction` (struct)

- `type: Call_Frame_Address_Instruction_Type`
- `arg1: Argument`
- `arg2: Argument`
- `Argument :: union { uvalue: u64; svalue: s64; }`

#### `Dwarf_Language` (enum)

- `C89             :: 0x0001`
- `C               :: 0x0002`
- `ADA83           :: 0x0003`
- `C_PLUS_PLUS     :: 0x0004`
- `COBOL74         :: 0x0005`
- `COBOL85         :: 0x0006`
- `FORTRAN77       :: 0x0007`
- `FORTRAN90       :: 0x0008`
- `PASCAL83        :: 0x0009`
- `MODULA2         :: 0x000a`
- `JAVA            :: 0x000b`
- `C99             :: 0x000c`
- `ADA95           :: 0x000d`
- `FORTRAN95       :: 0x000e`
- `PLI             :: 0x000f`
- `OBJ_C           :: 0x0010`
- `OBJ_C_PLUS_PLUS :: 0x0011`
- `UPC             :: 0x0012`
- `D               :: 0x0013`
- `PYTHON          :: 0x0014`
- `OPEN_CL         :: 0x0015`
- `GO              :: 0x0016`
- `MODULA3         :: 0x0017`
- `HASKELL         :: 0x0018`
- `C_PLUS_PLUS_03  :: 0x0019`
- `C_PLUS_PLUS_11  :: 0x001a`
- `O_CAML          :: 0x001b`
- `RUST            :: 0x001c`
- `C11             :: 0x001d`
- `SWIFT           :: 0x001e`
- `JULIA           :: 0x001f`
- `DYLAN           :: 0x0020`
- `C_PLUS_PLUS_14  :: 0x0021`
- `FORTRAN03       :: 0x0022`
- `FORTRAN08       :: 0x0023`
- `RENDER_SCRIPT   :: 0x0024`
- `BLISS           :: 0x0025`
- `KOTLIN          :: 0x0026`
- `ZIG             :: 0x0027`
- `CRYSTAL         :: 0x0028`
- `C_PLUS_PLUS_17  :: 0x002a`
- `C_PLUS_PLUS_20  :: 0x002b`
- `C17             :: 0x002c`
- `FORTRAN18       :: 0x002d`
- `ADA2005         :: 0x002e`
- `ADA2012         :: 0x002f`
- `HIP             :: 0x0030`
- `ASSEMBLY        :: 0x0031`
- `C_SHARP         :: 0x0032`
- `MOJO            :: 0x0033`
- `GLSL            :: 0x0034`
- `GLSL_ES         :: 0x0035`
- `HLSL            :: 0x0036`
- `OPEN_CL_CPP     :: 0x0037`
- `CPP_FOR_OPEN_CL :: 0x0038`
- `SYCL            :: 0x0039`
- `C_PLUS_PLUS_23  :: 0x003a`
- `ODIN            :: 0x003b`
- `P4              :: 0x003c`
- `METAL           :: 0x003d`
- `C23             :: 0x003e`
- `FORTRAN23       :: 0x003f`
- `RUBY            :: 0x0040`
- `MOVE            :: 0x0041`
- `HYLO            :: 0x0042`
- `V               :: 0x0043`
- `ALGOL68         :: 0x0044`
- `NIM             :: 0x0045`
- `ERLANG          :: 0x0046`
- `ELIXIR          :: 0x0047`
- `GLEAM           :: 0x0048`

#### `Dwarf_Attribute_Type` (enum)

- `SIBLING                         :: 0x01`
- `LOCATION                        :: 0x02`
- `NAME                            :: 0x03`
- `ORDERING                        :: 0x09`
- `BYTE_SIZE                       :: 0x0b`
- `BIT_OFFSET                      :: 0x0c`
- `BIT_SIZE                        :: 0x0d`
- `STMT_LIST                       :: 0x10`
- `LOW_PC                          :: 0x11`
- `HIGH_PC                         :: 0x12`
- `LANGUAGE                        :: 0x13`
- `DISCR                           :: 0x15`
- `DISCR_VALUE                     :: 0x16`
- `VISIBILITY                      :: 0x17`
- `IMPORT                          :: 0x18`
- `STRING_LENGTH                   :: 0x19`
- `COMMON_REFERENCE                :: 0x1a`
- `COMP_DIR                        :: 0x1b`
- `CONST_VALUE                     :: 0x1c`
- `CONTAINING_TYPE                 :: 0x1d`
- `DEFAULT_VALUE                   :: 0x1e`
- `INLINE                          :: 0x20`
- `IS_OPTIONAL                     :: 0x21`
- `LOWER_BOUND                     :: 0x22`
- `PRODUCER                        :: 0x25`
- `PROTOTYPED                      :: 0x27`
- `RETURN_ADDR                     :: 0x2a`
- `START_SCOPE                     :: 0x2c`
- `BIT_STRIDE                      :: 0x2e`
- `UPPER_BOUND                     :: 0x2f`
- `ABSTRACT_ORIGIN                 :: 0x31`
- `ACCESSIBILITY                   :: 0x32`
- `ADDRESS_CLASS                   :: 0x33`
- `ARTIFICIAL                      :: 0x34`
- `BASE_TYPES                      :: 0x35`
- `CALLING_CONVENTION              :: 0x36`
- `COUNT                           :: 0x37`
- `DATA_MEMBER_LOCATION            :: 0x38`
- `DECL_COLUMN                     :: 0x39`
- `DECL_FILE                       :: 0x3a`
- `DECL_LINE                       :: 0x3b`
- `DECLARATION                     :: 0x3c`
- `DISCR_LIST                      :: 0x3d`
- `ENCODING                        :: 0x3e`
- `EXTERNAL                        :: 0x3f`
- `FRAME_BASE                      :: 0x40`
- `FRIEND                          :: 0x41`
- `IDENTIFIER_CASE                 :: 0x42`
- `MACRO_INFO                      :: 0x43`
- `NAMELIST_ITEM                   :: 0x44`
- `PRIORITY                        :: 0x45`
- `SEGMENT                         :: 0x46`
- `SPECIFICATION                   :: 0x47`
- `STATIC_LINK                     :: 0x48`
- `TYPE                            :: 0x49`
- `USE_LOCATION                    :: 0x4a`
- `VARIABLE_PARAMETER              :: 0x4b`
- `VIRTUALITY                      :: 0x4c`
- `VTABLE_ELEM_LOCATION            :: 0x4d`
- `ALLOCATED                       :: 0x4e`
- `ASSOCIATED                      :: 0x4f`
- `DATA_LOCATION                   :: 0x50`
- `BYTE_STRIDE                     :: 0x51`
- `ENTRY_PC                        :: 0x52`
- `USE_UTF8                        :: 0x53`
- `EXTENSION                       :: 0x54`
- `RANGES                          :: 0x55`
- `TRAMPOLINE                      :: 0x56`
- `CALL_COLUMN                     :: 0x57`
- `CALL_FILE                       :: 0x58`
- `CALL_LINE                       :: 0x59`
- `DESCRIPTION                     :: 0x5a`
- `BINARY_SCALE                    :: 0x5b`
- `DECIMAL_SCALE                   :: 0x5c`
- `SMALL                           :: 0x5d`
- `DECIMAL_SIGN                    :: 0x5e`
- `DIGIT_COUNT                     :: 0x5f`
- `PICTURE_STRING                  :: 0x60`
- `MUTABLE                         :: 0x61`
- `THREADS_SCALED                  :: 0x62`
- `EXPLICIT                        :: 0x63`
- `OBJECT_POINTER                  :: 0x64`
- `ENDIANITY                       :: 0x65`
- `ELEMENTAL                       :: 0x66`
- `PURE                            :: 0x67`
- `RECURSIVE                       :: 0x68`
- `SIGNATURE                       :: 0x69`
- `MAIN_SUBPROGRAM                 :: 0x6a`
- `DATA_BIT_OFFSET                 :: 0x6b`
- `CONST_EXPR                      :: 0x6c`
- `ENUM_CLASS                      :: 0x6d`
- `LINKAGE_NAME                    :: 0x6e`
- `STRING_LENGTH_BIT_SIZE          :: 0x6f`
- `STRING_LENGTH_BYTE_SIZE         :: 0x70`
- `RANK                            :: 0x71`
- `STR_OFFSETS_BASE                :: 0x72`
- `ADDR_BASE                       :: 0x73`
- `RNGLISTS_BASE                   :: 0x74`
- `DWO_ID                          :: 0x75`
- `DWO_NAME                        :: 0x76`
- `REFERENCE                       :: 0x77`
- `RVALUE_REFERENCE                :: 0x78`
- `MACROS                          :: 0x79`
- `CALL_ALL_CALLS                  :: 0x7a`
- `CALL_ALL_SOURCE_CALLS           :: 0x7b`
- `CALL_ALL_TAIL_CALLS             :: 0x7c`
- `CALL_RETURN_PC                  :: 0x7d`
- `CALL_VALUE                      :: 0x7e`
- `CALL_ORIGIN                     :: 0x7f`
- `CALL_PARAMETER                  :: 0x80`
- `CALL_PC                         :: 0x81`
- `CALL_TAIL_CALL                  :: 0x82`
- `CALL_TARGET                     :: 0x83`
- `CALL_TARGET_CLOBBERED           :: 0x84`
- `CALL_DATA_LOCATION              :: 0x85`
- `CALL_DATA_VALUE                 :: 0x86`
- `NORETURN                        :: 0x87`
- `ALIGNMENT                       :: 0x88`
- `EXPORT_SYMBOLS                  :: 0x89`
- `DELETED                         :: 0x8a`
- `DEFAULTED                       :: 0x8b`
- `LOCLISTS_BASE                   :: 0x8c`
- `MIPS_LOOP_BEGIN                 :: 0x2002`
- `MIPS_TAIL_LOOP_BEGIN            :: 0x2003`
- `MIPS_EPILOG_BEGIN               :: 0x2004`
- `MIPS_LOOP_UNROLL_FACTOR         :: 0x2005`
- `MIPS_SOFTWARE_PIPELINE_DEPTH    :: 0x2006`
- `MIPS_LINKAGE_NAME               :: 0x2007`
- `MIPS_STRIDE                     :: 0x2008`
- `MIPS_ABSTRACT_NAME              :: 0x2009`
- `MIPS_CLONE_ORIGIN               :: 0x200a`
- `MIPS_HAS_INLINES                :: 0x200b`
- `MIPS_STRIDE_BYTE                :: 0x200c`
- `MIPS_STRIDE_ELEM                :: 0x200d`
- `MIPS_PTR_DOPETYPE               :: 0x200e`
- `MIPS_ALLOCATABLE_DOPETYPE       :: 0x200f`
- `MIPS_ASSUMED_SHAPE_DOPETYPE     :: 0x2010`
- `MIPS_ASSUMED_SIZE               :: 0x2011`
- `SF_NAMES                        :: 0x2101`
- `SRC_INFO                        :: 0x2102`
- `MAC_INFO                        :: 0x2103`
- `SRC_COORDS                      :: 0x2104`
- `BODY_BEGIN                      :: 0x2105`
- `BODY_END                        :: 0x2106`
- `GNU_VECTOR                      :: 0x2107`
- `GNU_TEMPLATE_NAME               :: 0x2110`
- `GNU_ODR_SIGNATURE               :: 0x210f`
- `GNU_CALL_SITE_VALUE             :: 0x2111`
- `GNU_CALL_SITE_DATA_VALUE        :: 0x2112`
- `GNU_CALL_SITE_TARGET            :: 0x2113`
- `GNU_CALL_SITE_TARGET_CLOBBERED  :: 0x2114`
- `GNU_TAIL_CALL                   :: 0x2115`
- `GNU_ALL_TAIL_CALL_SITES         :: 0x2116`
- `GNU_ALL_CALL_SITES              :: 0x2117`
- `GNU_ALL_SOURCE_CALL_SITES       :: 0x2118`
- `GNU_MACROS                      :: 0x2119`
- `GNU_DWO_NAME                    :: 0x2130`
- `GNU_DWO_ID                      :: 0x2131`
- `GNU_RANGES_BASE                 :: 0x2132`
- `GNU_ADDR_BASE                   :: 0x2133`
- `GNU_PUBNAMES                    :: 0x2134`
- `GNU_PUBTYPES                    :: 0x2135`
- `GNU_DISCRIMINATOR               :: 0x2136`
- `BORLAND_PROPERTY_READ           :: 0x3b11`
- `BORLAND_PROPERTY_WRITE          :: 0x3b12`
- `BORLAND_PROPERTY_IMPLEMENTS     :: 0x3b13`
- `BORLAND_PROPERTY_INDEX          :: 0x3b14`
- `BORLAND_PROPERTY_DEFAULT        :: 0x3b15`
- `BORLAND_DELPHI_UNIT             :: 0x3b20`
- `BORLAND_DELPHI_CLASS            :: 0x3b21`
- `BORLAND_DELPHI_RECORD           :: 0x3b22`
- `BORLAND_DELPHI_METACLASS        :: 0x3b23`
- `BORLAND_DELPHI_CONSTRUCTOR      :: 0x3b24`
- `BORLAND_DELPHI_DESTRUCTOR       :: 0x3b25`
- `BORLAND_DELPHI_ANONYMOUS_METHOD :: 0x3b26`
- `BORLAND_DELPHI_INTERFACE        :: 0x3b27`
- `BORLAND_DELPHI_ABI              :: 0x3b28`
- `BORLAND_DELPHI_RETURN           :: 0x3b29`
- `BORLAND_DELPHI_FRAMEPTR         :: 0x3b30`
- `BORLAND_CLOSURE                 :: 0x3b31`
- `LLVM_INCLUDE_PATH               :: 0x3e00`
- `LLVM_CONFIG_MACROS              :: 0x3e01`
- `LLVM_ISYSROOT                   :: 0x3e02`
- `LLVM_TAG_OFFSET                 :: 0x3e03`
- `APPLE_OPTIMIZED                 :: 0x3fe1`
- `APPLE_FLAGS                     :: 0x3fe2`
- `APPLE_ISA                       :: 0x3fe3`
- `APPLE_BLOCK                     :: 0x3fe4`
- `APPLE_MAJOR_RUNTIME_VERS        :: 0x3fe5`
- `APPLE_RUNTIME_CLASS             :: 0x3fe6`
- `APPLE_OMIT_FRAME_PTR            :: 0x3fe7`
- `APPLE_PROPERTY_NAME             :: 0x3fe8`
- `APPLE_PROPERTY_GETTER           :: 0x3fe9`
- `APPLE_PROPERTY_SETTER           :: 0x3fea`
- `APPLE_PROPERTY_ATTRIBUTE        :: 0x3feb`
- `APPLE_OBJC_COMPLETE_TYPE        :: 0x3fec`
- `APPLE_PROPERTY                  :: 0x3fed`
- `APPLE_SDK                       :: 0x3fef`

#### `Dwarf_Attribute_Format` (enum)

- `ADDR           :: 0x01`
- `BLOCK2         :: 0x03`
- `BLOCK4         :: 0x04`
- `DATA2          :: 0x05`
- `DATA4          :: 0x06`
- `DATA8          :: 0x07`
- `STRING         :: 0x08`
- `BLOCK          :: 0x09`
- `BLOCK1         :: 0x0a`
- `DATA1          :: 0x0b`
- `FLAG           :: 0x0c`
- `SDATA          :: 0x0d`
- `STRP           :: 0x0e`
- `UDATA          :: 0x0f`
- `REF_ADDR       :: 0x10`
- `REF1           :: 0x11`
- `REF2           :: 0x12`
- `REF4           :: 0x13`
- `REF8           :: 0x14`
- `REF_UDATA      :: 0x15`
- `INDIRECT       :: 0x16`
- `SEC_OFFSET     :: 0x17`
- `EXPRLOC        :: 0x18`
- `FLAG_PRESENT   :: 0x19`
- `REF_SIG8       :: 0x20`
- `STRX           :: 0x1a`
- `ADDRX          :: 0x1b`
- `REF_SUP4       :: 0x1c`
- `STRP_SUP       :: 0x1d`
- `DATA16         :: 0x1e`
- `LINE_STRP      :: 0x1f`
- `IMPLICIT_CONST :: 0x21`
- `LOCLISTX       :: 0x22`
- `RNGLISTX       :: 0x23`
- `REF_SUP8       :: 0x24`
- `STRX1          :: 0x25`
- `STRX2          :: 0x26`
- `STRX3          :: 0x27`
- `STRX4          :: 0x28`
- `ADDRX1         :: 0x29`
- `ADDRX2         :: 0x2a`
- `ADDRX3         :: 0x2b`
- `ADDRX4         :: 0x2c`
- `GNU_ADDR_INDEX :: 0x1f01`
- `GNU_STR_INDEX  :: 0x1f02`
- `GNU_REF_ALT    :: 0x1f20`
- `GNU_STRP_ALT   :: 0x1f21`

#### `Dwarf_Tag` (enum)

- `NULL                         :: 0x0000`
- `ARRAY_TYPE                   :: 0x0001`
- `CLASS_TYPE                   :: 0x0002`
- `ENTRY_POINT                  :: 0x0003`
- `ENUMERATION_TYPE             :: 0x0004`
- `FORMAL_PARAMETER             :: 0x0005`
- `IMPORTED_DECLARATION         :: 0x0008`
- `LABEL                        :: 0x000a`
- `LEXICAL_BLOCK                :: 0x000b`
- `MEMBER                       :: 0x000d`
- `POINTER_TYPE                 :: 0x000f`
- `REFERENCE_TYPE               :: 0x0010`
- `COMPILE_UNIT                 :: 0x0011`
- `STRING_TYPE                  :: 0x0012`
- `STRUCTURE_TYPE               :: 0x0013`
- `SUBROUTINE_TYPE              :: 0x0015`
- `TYPEDEF                      :: 0x0016`
- `UNION_TYPE                   :: 0x0017`
- `UNSPECIFIED_PARAMETERS       :: 0x0018`
- `VARIANT                      :: 0x0019`
- `COMMON_BLOCK                 :: 0x001a`
- `COMMON_INCLUSION             :: 0x001b`
- `INHERITANCE                  :: 0x001c`
- `INLINED_SUBROUTINE           :: 0x001d`
- `MODULE                       :: 0x001e`
- `PTR_TO_MEMBER_TYPE           :: 0x001f`
- `SET_TYPE                     :: 0x0020`
- `SUBRANGE_TYPE                :: 0x0021`
- `WITH_STMT                    :: 0x0022`
- `ACCESS_DECLARATION           :: 0x0023`
- `BASE_TYPE                    :: 0x0024`
- `CATCH_BLOCK                  :: 0x0025`
- `CONST_TYPE                   :: 0x0026`
- `CONSTANT                     :: 0x0027`
- `ENUMERATOR                   :: 0x0028`
- `FILE_TYPE                    :: 0x0029`
- `FRIEND                       :: 0x002a`
- `NAMELIST                     :: 0x002b`
- `NAMELIST_ITEM                :: 0x002c`
- `PACKED_TYPE                  :: 0x002d`
- `SUBPROGRAM                   :: 0x002e`
- `TEMPLATE_TYPE_PARAMETER      :: 0x002f`
- `TEMPLATE_VALUE_PARAMETER     :: 0x0030`
- `THROWN_TYPE                  :: 0x0031`
- `TRY_BLOCK                    :: 0x0032`
- `VARIANT_PART                 :: 0x0033`
- `VARIABLE                     :: 0x0034`
- `VOLATILE_TYPE                :: 0x0035`
- `DWARF_PROCEDURE              :: 0x0036`
- `RESTRICT_TYPE                :: 0x0037`
- `INTERFACE_TYPE               :: 0x0038`
- `NAMESPACE                    :: 0x0039`
- `IMPORTED_MODULE              :: 0x003a`
- `UNSPECIFIED_TYPE             :: 0x003b`
- `PARTIAL_UNIT                 :: 0x003c`
- `IMPORTED_UNIT                :: 0x003d`
- `CONDITION                    :: 0x003f`
- `SHARED_TYPE                  :: 0x0040`
- `TYPE_UNIT                    :: 0x0041`
- `RVALUE_REFERENCE_TYPE        :: 0x0042`
- `TEMPLATE_ALIAS               :: 0x0043`
- `COARRAY_TYPE                 :: 0x0044`
- `GENERIC_SUBRANGE             :: 0x0045`
- `DYNAMIC_TYPE                 :: 0x0046`
- `ATOMIC_TYPE                  :: 0x0047`
- `CALL_SITE                    :: 0x0048`
- `CALL_SITE_PARAMETER          :: 0x0049`
- `SKELETON_UNIT                :: 0x004a`
- `IMMUTABLE_TYPE               :: 0x004b`
- `MIPS_LOOP                    :: 0x4081`
- `FORMAT_LABEL                 :: 0x4101`
- `FUNCTION_TEMPLATE            :: 0x4102`
- `CLASS_TEMPLATE               :: 0x4103`
- `GNU_TEMPLATE_TEMPLATE_PARAM  :: 0x4106`
- `GNU_TEMPLATE_PARAMETER_PACK  :: 0x4107`
- `GNU_FORMAL_PARAMETER_PACK    :: 0x4108`
- `GNU_CALL_SITE                :: 0x4109`
- `GNU_CALL_SITE_PARAMETER      :: 0x410a`
- `APPLE_PROPERTY               :: 0x4200`
- `BORLAND_PROPERTY             :: 0xb000`
- `BORLAND_DELPHI_STRING        :: 0xb001`
- `BORLAND_DELPHI_DYNAMIC_ARRAY :: 0xb002`
- `BORLAND_DELPHI_SET           :: 0xb003`
- `BORLAND_DELPHI_VARIANT       :: 0xb004`

#### `Dwarf_Base_Type` (enum)

- `ADDRESS         :: 0x01`
- `BOOLEAN         :: 0x02`
- `COMPLEX_FLOAT   :: 0x03`
- `FLOAT           :: 0x04`
- `SIGNED          :: 0x05`
- `SIGNED_CHAR     :: 0x06`
- `UNSIGNED        :: 0x07`
- `UNSIGNED_CHAR   :: 0x08`
- `IMAGINARY_FLOAT :: 0x09`
- `PACKED_DECIMAL  :: 0x0a`
- `NUMERIC_STRING  :: 0x0b`
- `EDITED          :: 0x0c`
- `SIGNED_FIXED    :: 0x0d`
- `UNSIGNED_FIXED  :: 0x0e`
- `DECIMAL_FLOAT   :: 0x0f`
- `UTF             :: 0x10`
- `LO_USER         :: 0x80`
- `HI_USER         :: 0xff`

#### `Call_Frame_Address_Instruction_Type` (enum)

- `ADVANCE_LOC        :: 0x40`
- `OFFSET             :: 0x80`
- `RESTORE            :: 0xC0`
- `NOP                :: 0x00`
- `SET_LOC            :: 0x01`
- `ADVANCE_LOC1       :: 0x02`
- `ADVANCE_LOC2       :: 0x03`
- `ADVANCE_LOC4       :: 0x04`
- `OFFSET_EXTENDED    :: 0x05`
- `RESTORE_EXTENDED   :: 0x06`
- `UNDEFINED          :: 0x07`
- `SAME_VALUE         :: 0x08`
- `REGISTER           :: 0x09`
- `REMEMBER_STATE     :: 0x0a`
- `RESTORE_STATE      :: 0x0b`
- `DEF_CFA            :: 0x0c`
- `DEF_CFA_REGISTER   :: 0x0d`
- `DEF_CFA_OFFSET     :: 0x0e`
- `DEF_CFA_EXPRESSION :: 0x0f`
- `OFFSET_EXTENDED_SF :: 0x10`
- `DEF_CFA_SF         :: 0x11`
- `DEF_CFA_OFFSET_SF  :: 0x12`
- `VAL_OFFSET         :: 0x13`
- `VAL_OFFSET_SF      :: 0x14`
- `VAL_EXPRESSION     :: 0x15`
- `LO_USER            :: 0x1b`
- `HI_USER            :: 0x3e`

#### `Dwarf_Registers_x64` (enum)

- `RAX   :: 0`
- `RDX   :: 1`
- `RCX   :: 2`
- `RBX   :: 3`
- `RSI   :: 4`
- `RDI   :: 5`
- `RBP   :: 6`
- `RSP   :: 7`
- `R8    :: 8`
- `R9    :: 9`
- `R10   :: 10`
- `R11   :: 11`
- `R12   :: 12`
- `R13   :: 13`
- `R14   :: 14`
- `R15   :: 15`
- `RA    :: 16`
- `XMM0  :: 17`
- `XMM1  :: 18`
- `XMM2  :: 19`
- `XMM3  :: 20`
- `XMM4  :: 21`
- `XMM5  :: 22`
- `XMM6  :: 23`
- `XMM7  :: 24`
- `XMM8  :: 25`
- `XMM9  :: 26`
- `XMM10 :: 27`
- `XMM11 :: 28`
- `XMM12 :: 29`
- `XMM13 :: 30`
- `XMM14 :: 31`
- `XMM15 :: 32`

#### `Line_Info` (struct)

- `dirs: [..] string`
- `files: [..] File_Info`
- `sequences: [..] Line_Info_Sequence`

#### `Line_Info_Sequence` (struct)

- `addresses: [..] u64`
- `mappings: [..] Line_Mapping`

#### `File_Info` (struct)

- `dir_id: u64`
- `path: string`

#### `Line_Mapping` (struct)

- `file_id: Dwarf_File_Id`
- `line: u32`
- `column: u32`
- `is_end: bool`
- `flags: Flags`
- `Flags :: enum_flags { STATEMENT; BASIC_BLOCK; PROLOGUE_END; EPILOGUE_BEGIN; }`

#### `Line_Program_Header` (struct)

- `size: u32`
- `version: u16`
- `header_length: u32 #align 2`
- `minimum_instruction_length: u8`
- `maximum_operations_per_instruction: u8`
- `default_is_stmt: bool`
- `line_base: s8`
- `line_range: u8`
- `opcode_base: u8`

#### `Free_Page_Map` (struct)

- `BPW :: 8 * size_of(u64)`
- `iwMac: u32`
- `iwRover: u32`
- `cbPg: s32`
- `fBigMsf: bool`
- `rgw: [..] s64`
- `wFill: s64`

#### `Int_Set` (struct)

- `BPW :: 32`
- `words: [] u32`

#### `Msf` (struct)

- `header: *Msf_Header_Big`
- `parms: Msf_Parms`
- `stream_table: Stream_Table`
- `fpm: Free_Page_Map`
- `fpm_freed: Free_Page_Map`
- `siPnList: Stream_Info`
- `content: string`

#### Contextual layout dependencies

The following shapes are required to interpret public `Parse_Context`, `Msf`, and `Pdb` signatures or fields. Types identified as module-contextual below are dependencies of those public aggregate shapes; this does not add independent exports to the declaration inventory.

`Parse_Context` is the contextual argument type named by `parse_compilation_unit_v4`:

- `debug_info: Debug_Info`
- `debug_string_data: [] u8`
- `depth: int`
- `verbose: bool`
- `current_compilation_unit: *Compilation_Unit`
- `subprogram_stack: [..] *Subprogram`
- `block_stack: [..] *Lexical_Block`

Its active compilation-unit and traversal stacks are parser context; `parse_compilation_unit_v4` sets and clears the active-unit pointer during the call. The supplied debug-string bytes and line-program bytes are borrowed inputs. Do not assume `Parse_Context` is a separately exported utility type beyond its appearance in that signature.

`Msf` contains these contextual MSF types:

- `UPN` and `UNSN` are aliases of `u32` (universal page number and unified stream number).
- `Msf_Header_Big` is a union of `pg: Pg` and the named `data` header view. `Pg` contains `rgb: [4096] u8`. The `data` view contains `magic: [30] u8`, `page_size: s32`, `valid_fpm_page_no: UPN`, `page_count: UPN`, `st: Stream_Info_Persist`, and `mpspnpnSt: [2] u32`. `Stream_Info_Persist` contains `byte_count: s32` and `stream_page_numbers: s32`. The header magic is the 30-byte MSF signature already described by the format.
- `Msf_Parms` has fields `cbPg: s32`, `page_size_in_bytes: u32`, `mask_cb_pg_mod: u32`, `cbitsFpm: s32`, `cpgFileGrowth: s32`, `pnMax: UPN`, `cpnFpm: UPN`, `pnFpm0: UPN`, `pnFpm1: UPN`, and `pnDataMin: UPN`.
- `Stream_Table` contains `page_size_in_bytes: u32` and `stream_infos: [..] Stream_Info`.
- `Stream_Info` contains `byte_count: s32 = -1` and `stream_page_numbers: [..] UPN`; `-1` means no stream data.

`Pdb` uses contextual `Name_Table` and `GUID` fields. `Name_Table` is declared in file scope and is not an importer-visible type. Its embedded referent has `buffer: [] u8` and `string_to_name_map: Pdb_Map(String_Offset, Name_Index)`; this is contextual shape only, not an exported type contract. The private map record has `keys: [] Key`, `values: [] Value`, `present: Int_Set`, and `deleted: Int_Set`; the instantiated key/value types are `String_Offset` (`s32`) and `Name_Index` (`u32`). These map and alias declarations are also file-scope; do not expose them as standalone imports. Callers can treat the `Pdb.name_table` value as opaque unless using APIs that accept it. `GUID` has `Data1: u32`, `Data2: u16`, `Data3: u16`, and `Data4: [8] u8`. `parse_msf` retains a view of the caller's input bytes and its header pointer refers into that view, so the input must stay alive while the returned `Msf` uses it. Stream and parser arrays use the active allocator; there is no public aggregate deinitializer. `get_stream_data` returns a newly allocated byte array for the caller to release. `parse_pdb` returns allocated parser arrays and a name-table buffer; its embedded `Msf.content` still refers to the supplied file bytes. The PDB/MSF aggregates have no public deinitializer, so ownership/release beyond allocator-managed storage is not separately specified.

## DWARF frame parser behavior

`parse_common_information_entry` supports the version-1 CIE form with a 32-bit identifier equal to zero. A 64-bit length marker is read, but the 32-bit identifier model means this is not full DWARF64 support. This is the `.eh_frame` CIE convention; `.debug_frame` CIE identifiers are not accepted. CIE augmentation strings and augmentation-data slices borrow the input section. Decoded instruction arrays are allocated using the active allocator.

`parse_frame_description_entry` reads the encoded 32-bit backward distance from the FDE's CIE-pointer field, resolves it relative to that field, and stores the resulting section offset in `Frame_Description_Entry.cie_offset`. The parser checks that this resolved offset equals the supplied CIE's `offset`; `cie_offset` is therefore not the raw encoded distance. Its CIE-pointer handling follows `.eh_frame`; `.debug_frame` FDE/CIE pointer conventions are unsupported. The returned augmentation-data slice borrows the input section, while the decoded instruction array uses the active allocator.

For FDE initial-location decoding, the `R` CIE augmentation supplies the pointer encoding. The supported forms are absolute 8-byte address (`0x00`) and signed 4-byte PC-relative address (`0x1b`); other format/application combinations fail. The PC-relative base is the section address of the encoded initial-location field. The range uses the corresponding unsigned width (8 bytes for `0x00`, 4 bytes for `0x1b`) and is not relative. Without an `R` augmentation the zero encoding selects absolute 8-byte addresses and ranges. The CIE parser accepts only version 1 and does not retain a separate `address_size` field.

`decode_cfa_instructions` accepts only instructions whose operands fit its two scalar `Argument` union slots. It does not decode DWARF expression blocks; expression-bearing instructions and `SET_LOC` are unsupported and produce failure. Operand byte order is the parser's little-endian section order. The decoder has no separate address-size or CIE parameter, so its accepted instruction subset does not infer or apply CIE-specific address-size state.

`is_procedure` recognizes the four procedure-record tags listed in `Codeview_Symbol_Procedure.base.symbol_type`; other symbol tags return false. A procedure stores its address range, type, flags, name, and its associated inline-site pointers in the public fields listed above. An inline site embeds `Codeview_Symbol` first, fixes the tag to `.INLINESITE`, and stores its parent/end references, inlinee item identifier, binary annotations, and child inline sites. These declarations establish Jai member order and the leading base member; exact padding and later offsets follow the target's Jai aggregate ABI, not CodeView's serialized byte layout.

`decode_inline_site_line_info` requires a matching entry in `Codeview_Debug_Info.inlinee_line_info`; if none matches the site's inlinee identifier it returns failure. Decoded code offsets and lengths are associated with the containing procedure's segment. Annotation state can change file, line, statement status, and columns; the returned public line shape carries start columns, while column-end deltas are not represented. An unfinished final line sequence is a failure. Zero-line/zero-length annotation streams do not have a guaranteed successful result.

`overlay_inlinesites` processes the supplied sites in order and returns failure with an empty result if any site cannot be decoded or placed. `overlay_inlinesite` requires each decoded inline range to be contained within an existing sequence; it splits that sequence around the inline range, inserts the inline sequence, and applies child sites to the replacement regions. It reports failure when a range is not contained. `copy_and_truncate_blocks` duplicates the file-block array and narrows its line/column views to the requested sequence interval, adjusting returned line offsets relative to the new start. The line and column arrays themselves can remain views into the source storage, so callers must not assume deep-copy independence or that the source sequences are unchanged.

#### `Pdb` (struct)

- `header: Pdb_Header`
- `signature: GUID`
- `name_table: Name_Table`
- `contains_id_stream: bool`
- `no_type_merge:      bool`
- `minimal_debug_info: bool`
- `tpi: [] *Codeview_Type`
- `ipi: [] *Codeview_Type`
- `msf: Msf`

#### `Pdb_Header` (struct)

- `version: Pdb_Version`
- `signature: u32`
- `age: u32`

#### `Pdb_Version` (enum)

- `VC2      :: 19941610`
- `VC4      :: 19950623`
- `VC41     :: 19950814`
- `VC50     :: 19960307`
- `VC98     :: 19970604`
- `VC70     :: 20000404`
- `VC70Dep  :: 19990604`
- `VC80     :: 20030901`
- `VC110    :: 20091201`
- `VC140    :: 20140508`

#### `Tpi_Stream_Header` (struct)

- `version:                    Tpi_Stream_Version`
- `header_size:                u32`
- `type_index_begin:           u32`
- `type_index_end:             u32`
- `type_record_bytes:          u32`
- `hash_stream_index:          u16`
- `hash_aux_stream_index:      u16`
- `hash_key_size:              u32`
- `num_hash_buckets:           u32`
- `hash_value_buffer_offset:   s32`
- `hash_value_buffer_length:   u32`
- `index_offset_buffer_offset: s32`
- `index_offset_buffer_length: u32`
- `hash_adj_buffer_offset:     s32`
- `hash_adj_buffer_length:     u32`

#### `Tpi_Stream_Version` (enum)

- `V40 :: 19950410`
- `V41 :: 19951122`
- `V50 :: 19961031`
- `V70 :: 19990903`
- `V80 :: 20040203`

## Behavioral contract and constraints

The module parses DWARF and CodeView debug information into Jai-side records for inspecting source locations, types, and symbols. The parser is incomplete. Unsupported sections/encodings may not be represented; malformed-input failure behavior is not a stable contract. Parsed data lifetime follows the parser's returned structures and allocator fields.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: no explicit `#library` name identified.
