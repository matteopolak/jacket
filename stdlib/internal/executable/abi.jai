// Explicit public ABI transcribed from the reviewed executable-formats spec.
vm_prot_t :: s32;
CPU_THREADTYPE_NONE :: cast(cpu_threadtype_t) 0;
CPU_THREADTYPE_INTEL_HTT :: cast(cpu_threadtype_t) 1;
MH_MAGIC :: 0xfeedface;
MH_CIGAM :: 0xcefaedfe;
MH_MAGIC_64 :: 0xfeedfacf;
MH_CIGAM_64 :: 0xcffaedfe;
MH_OBJECT :: 0x1;
MH_EXECUTE :: 0x2;
MH_FVMLIB :: 0x3;
MH_CORE :: 0x4;
MH_PRELOAD :: 0x5;
MH_DYLIB :: 0x6;
MH_DYLINKER :: 0x7;
MH_BUNDLE :: 0x8;
MH_DYLIB_STUB :: 0x9;
MH_DSYM :: 0xa;
MH_KEXT_BUNDLE :: 0xb;
MH_FILESET :: 0xc;
MH_GPU_EXECUTE :: 0xd;
MH_GPU_DYLIB :: 0xe;
MH_NOUNDEFS :: 0x1;
MH_INCRLINK :: 0x2;
MH_DYLDLINK :: 0x4;
MH_BINDATLOAD :: 0x8;
MH_PREBOUND :: 0x10;
MH_SPLIT_SEGS :: 0x20;
MH_LAZY_INIT :: 0x40;
MH_TWOLEVEL :: 0x80;
MH_FORCE_FLAT :: 0x100;
MH_NOMULTIDEFS :: 0x200;
MH_NOFIXPREBINDING :: 0x400;
MH_PREBINDABLE :: 0x800;
MH_ALLMODSBOUND :: 0x1000;
MH_SUBSECTIONS_VIA_SYMBOLS :: 0x2000;
MH_CANONICAL :: 0x4000;
MH_WEAK_DEFINES :: 0x8000;
MH_BINDS_TO_WEAK :: 0x10000;
MH_ALLOW_STACK_EXECUTION :: 0x20000;
MH_ROOT_SAFE :: 0x40000;
MH_SETUID_SAFE :: 0x80000;
MH_NO_REEXPORTED_DYLIBS :: 0x100000;
MH_PIE :: 0x200000;
MH_DEAD_STRIPPABLE_DYLIB :: 0x400000;
MH_HAS_TLV_DESCRIPTORS :: 0x800000;
MH_NO_HEAP_EXECUTION :: 0x1000000;
MH_APP_EXTENSION_SAFE :: 0x02000000;
MH_NLIST_OUTOFSYNC_WITH_DYLDINFO :: 0x04000000;
MH_SIM_SUPPORT :: 0x08000000;
MH_IMPLICIT_PAGEZERO :: 0x10000000;
MH_DYLIB_IN_CACHE :: 0x80000000;
SECTION_TYPE :: 0x000000ff;
SECTION_ATTRIBUTES :: 0xffffff00;
SECTION_ATTRIBUTES_USR :: 0xff000000;
SECTION_ATTRIBUTES_SYS :: 0x00ffff00;
SEG_PAGEZERO :: "__PAGEZERO";
SEG_TEXT :: "__TEXT";
SECT_TEXT :: "__text";
SECT_FVMLIB_INIT0 :: "__fvmlib_init0";
SECT_FVMLIB_INIT1 :: "__fvmlib_init1";
SEG_DATA :: "__DATA";
SECT_DATA :: "__data";
SECT_BSS :: "__bss";
SECT_COMMON :: "__common";
SEG_OBJC :: "__OBJC";
SECT_OBJC_SYMBOLS :: "__symbol_table";
SECT_OBJC_MODULES :: "__module_info";
SECT_OBJC_STRINGS :: "__selector_strs";
SECT_OBJC_REFS :: "__selector_refs";
SEG_ICON :: "__ICON";
SECT_ICON_HEADER :: "__header";
SECT_ICON_TIFF :: "__tiff";
SEG_LINKEDIT :: "__LINKEDIT";
SEG_UNIXSTACK :: "__UNIXSTACK";
SEG_IMPORT :: "__IMPORT";
DYLIB_USE_WEAK_LINK :: 0x01;
DYLIB_USE_REEXPORT :: 0x02;
DYLIB_USE_UPWARD :: 0x04;
DYLIB_USE_DELAYED_INIT :: 0x08;
DYLIB_USE_MARKER :: 0x1a741800;
INDIRECT_SYMBOL_LOCAL :: 0x80000000;
INDIRECT_SYMBOL_ABS :: 0x40000000;
PLATFORM_UNKNOWN :: 0;
PLATFORM_ANY :: 0xFFFFFFFF;
PLATFORM_MACOS :: 1;
PLATFORM_IOS :: 2;
PLATFORM_TVOS :: 3;
PLATFORM_WATCHOS :: 4;
PLATFORM_BRIDGEOS :: 5;
PLATFORM_MACCATALYST :: 6;
PLATFORM_IOSSIMULATOR :: 7;
PLATFORM_TVOSSIMULATOR :: 8;
PLATFORM_WATCHOSSIMULATOR :: 9;
PLATFORM_DRIVERKIT :: 10;
PLATFORM_VISIONOS :: 11;
PLATFORM_VISIONOSSIMULATOR :: 12;
PLATFORM_FIRMWARE :: 13;
PLATFORM_SEPOS :: 14;
PLATFORM_MACOS_EXCLAVECORE :: 15;
PLATFORM_MACOS_EXCLAVEKIT :: 16;
PLATFORM_IOS_EXCLAVECORE :: 17;
PLATFORM_IOS_EXCLAVEKIT :: 18;
PLATFORM_TVOS_EXCLAVECORE :: 19;
PLATFORM_TVOS_EXCLAVEKIT :: 20;
PLATFORM_WATCHOS_EXCLAVECORE :: 21;
PLATFORM_WATCHOS_EXCLAVEKIT :: 22;
PLATFORM_VISIONOS_EXCLAVECORE :: 23;
PLATFORM_VISIONOS_EXCLAVEKIT :: 24;
TOOL_CLANG :: 1;
TOOL_SWIFT :: 2;
TOOL_LD :: 3;
TOOL_LLD :: 4;
TOOL_METAL :: 1024;
TOOL_AIRLLD :: 1025;
TOOL_AIRNT :: 1026;
TOOL_AIRNT_PLUGIN :: 1027;
TOOL_AIRPACK :: 1028;
TOOL_GPUARCHIVER :: 1031;
TOOL_METAL_FRAMEWORK :: 1032;
REBASE_TYPE_POINTER :: 1;
REBASE_TYPE_TEXT_ABSOLUTE32 :: 2;
REBASE_TYPE_TEXT_PCREL32 :: 3;
REBASE_OPCODE_MASK :: 0xF0;
REBASE_IMMEDIATE_MASK :: 0x0F;
REBASE_OPCODE_DONE :: 0x00;
REBASE_OPCODE_SET_TYPE_IMM :: 0x10;
REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB :: 0x20;
REBASE_OPCODE_ADD_ADDR_ULEB :: 0x30;
REBASE_OPCODE_ADD_ADDR_IMM_SCALED :: 0x40;
REBASE_OPCODE_DO_REBASE_IMM_TIMES :: 0x50;
REBASE_OPCODE_DO_REBASE_ULEB_TIMES :: 0x60;
REBASE_OPCODE_DO_REBASE_ADD_ADDR_ULEB :: 0x70;
REBASE_OPCODE_DO_REBASE_ULEB_TIMES_SKIPPING_ULEB :: 0x80;
BIND_TYPE_POINTER :: 1;
BIND_TYPE_TEXT_ABSOLUTE32 :: 2;
BIND_TYPE_TEXT_PCREL32 :: 3;
BIND_SPECIAL_DYLIB_SELF :: 0;
BIND_SPECIAL_DYLIB_MAIN_EXECUTABLE :: -1;
BIND_SPECIAL_DYLIB_FLAT_LOOKUP :: -2;
BIND_SPECIAL_DYLIB_WEAK_LOOKUP :: -3;
BIND_SYMBOL_FLAGS_WEAK_IMPORT :: 0x1;
BIND_SYMBOL_FLAGS_NON_WEAK_DEFINITION :: 0x8;
BIND_OPCODE_MASK :: 0xF0;
BIND_IMMEDIATE_MASK :: 0x0F;
BIND_OPCODE_DONE :: 0x00;
BIND_OPCODE_SET_DYLIB_ORDINAL_IMM :: 0x10;
BIND_OPCODE_SET_DYLIB_ORDINAL_ULEB :: 0x20;
BIND_OPCODE_SET_DYLIB_SPECIAL_IMM :: 0x30;
BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM :: 0x40;
BIND_OPCODE_SET_TYPE_IMM :: 0x50;
BIND_OPCODE_SET_ADDEND_SLEB :: 0x60;
BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB :: 0x70;
BIND_OPCODE_ADD_ADDR_ULEB :: 0x80;
BIND_OPCODE_DO_BIND :: 0x90;
BIND_OPCODE_DO_BIND_ADD_ADDR_ULEB :: 0xA0;
BIND_OPCODE_DO_BIND_ADD_ADDR_IMM_SCALED :: 0xB0;
BIND_OPCODE_DO_BIND_ULEB_TIMES_SKIPPING_ULEB :: 0xC0;
BIND_OPCODE_THREADED :: 0xD0;
BIND_SUBOPCODE_THREADED_SET_BIND_ORDINAL_TABLE_SIZE_ULEB :: 0x00;
BIND_SUBOPCODE_THREADED_APPLY :: 0x01;
EXPORT_SYMBOL_FLAGS_KIND_MASK :: 0x03;
EXPORT_SYMBOL_FLAGS_KIND_REGULAR :: 0x00;
EXPORT_SYMBOL_FLAGS_KIND_THREAD_LOCAL :: 0x01;
EXPORT_SYMBOL_FLAGS_KIND_ABSOLUTE :: 0x02;
EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION :: 0x04;
EXPORT_SYMBOL_FLAGS_REEXPORT :: 0x08;
EXPORT_SYMBOL_FLAGS_STUB_AND_RESOLVER :: 0x10;
EXPORT_SYMBOL_FLAGS_STATIC_RESOLVER :: 0x20;
DICE_KIND_DATA :: 0x0001;
DICE_KIND_JUMP_TABLE8 :: 0x0002;
DICE_KIND_JUMP_TABLE16 :: 0x0003;
DICE_KIND_JUMP_TABLE32 :: 0x0004;
DICE_KIND_ABS_JUMP_TABLE32 :: 0x0005;
R_ABS :: 0;
R_SCATTERED :: 0x80000000;
N_GSYM :: 0x20;
N_FNAME :: 0x22;
N_FUN :: 0x24;
N_STSYM :: 0x26;
N_LCSYM :: 0x28;
N_BNSYM :: 0x2e;
N_AST :: 0x32;
N_OPT :: 0x3c;
N_RSYM :: 0x40;
N_SLINE :: 0x44;
N_ENSYM :: 0x4e;
N_SSYM :: 0x60;
N_SO :: 0x64;
N_OSO :: 0x66;
N_LIB :: 0x68;
N_LSYM :: 0x80;
N_BINCL :: 0x82;
N_SOL :: 0x84;
N_PARAMS :: 0x86;
N_VERSION :: 0x88;
N_OLEVEL :: 0x8A;
N_PSYM :: 0xa0;
N_EINCL :: 0xa2;
N_ENTRY :: 0xa4;
N_LBRAC :: 0xc0;
N_EXCL :: 0xc2;
N_RBRAC :: 0xe0;
N_BCOMM :: 0xe2;
N_ECOMM :: 0xe4;
N_ECOML :: 0xe8;
N_LENG :: 0xfe;
N_PC :: 0x30;
N_STAB :: 0xe0;
N_PEXT :: 0x10;
N_TYPE :: 0x0e;
N_EXT :: 0x01;
N_UNDF :: 0x0;
N_ABS :: 0x2;
N_SECT :: 0xe;
N_PBUD :: 0xc;
N_INDR :: 0xa;
NO_SECT :: 0;
MAX_SECT :: 255;
REFERENCE_TYPE :: 0x7;
REFERENCE_FLAG_UNDEFINED_NON_LAZY :: 0;
REFERENCE_FLAG_UNDEFINED_LAZY :: 1;
REFERENCE_FLAG_DEFINED :: 2;
REFERENCE_FLAG_PRIVATE_DEFINED :: 3;
REFERENCE_FLAG_PRIVATE_UNDEFINED_NON_LAZY :: 4;
REFERENCE_FLAG_PRIVATE_UNDEFINED_LAZY :: 5;
REFERENCED_DYNAMICALLY :: 0x0010;
SELF_LIBRARY_ORDINAL :: 0x0;
MAX_LIBRARY_ORDINAL :: 0xfd;
DYNAMIC_LOOKUP_ORDINAL :: 0xfe;
EXECUTABLE_ORDINAL :: 0xff;
N_NO_DEAD_STRIP :: 0x0020;
N_DESC_DISCARDED :: 0x0020;
N_WEAK_REF :: 0x0040;
N_WEAK_DEF :: 0x0080;
N_REF_TO_WEAK :: 0x0080;
N_ARM_THUMB_DEF :: 0x0008;
N_SYMBOL_RESOLVER :: 0x0100;
N_ALT_ENTRY :: 0x0200;
N_COLD_FUNC :: 0x0400;
cpu_type_t :: s32;
cpu_subtype_t :: s32;
cpu_threadtype_t :: s32;
Coff_Optional_Header_Windows_Specific_Pe32 :: Coff_Optional_Header_Windows_Specific(u32);
Coff_Optional_Header_Windows_Specific_Pe32_Plus :: Coff_Optional_Header_Windows_Specific(u64);
Elf_Header_64 :: Elf_Header(u64);
Elf_Header_32 :: Elf_Header(u32);
Elf_Program_Header_64 :: Elf_Program_Header(u64);
Elf_Program_Header_32 :: Elf_Program_Header(u32);
Elf_Symbol_Id :: #type,isa u32;
MS_DOS_MAGIC_PREFIX :: 0x5a4d;
PE_MAGIC_PREFIX :: 0x00004550;
MAX_IDENTIFIER_LENGTH :: 512;
MAX_CONCURRENT_TOKENS :: 8;

Apple_Universal_Binary :: struct {
    content: string;
    files: [] Arch_File;
}

Arch_File :: struct {
    cpu_type: Cpu_Type;
    cpu_subtype: Cpu_Subtype;
    data: [] u8;
}

CPU_STATE :: enum {
    MAX :: 4;
    USER :: 0;
    SYSTEM :: 1;
    IDLE :: 2;
    NICE :: 3;
}

CPU_ARCH :: enum {
    MASK :: 0xff000000;
    ABI64 :: 0x01000000;
    ABI64_32 :: 0x02000000;
}

Cpu_Type :: enum s32 {
    ANY :: -1;
    VAX :: 1;
    MC680x0 :: 6;
    X86 :: 7;
    I386 :: X86;
    X86_64 :: X86 | cast(s32) CPU_ARCH.ABI64;
    MC98000 :: 10;
    HPPA :: 11;
    ARM :: 12;
    ARM64 :: ARM | cast(s32) CPU_ARCH.ABI64;
    ARM64_32 :: ARM | cast(s32) CPU_ARCH.ABI64_32;
    MC88000 :: 13;
    SPARC :: 14;
    I860 :: 15;
    POWERPC :: 18;
    POWERPC64 :: POWERPC | cast(s32) CPU_ARCH.ABI64;
}

Cpu_Subtype :: enum s32 {
    MASK :: -16777216;
    LIB64 :: -2147483648;
    PTRAUTH_ABI :: -2147483648;
    ANY :: -1;
    MULTIPLE :: -1;
    LITTLE_ENDIAN :: 0;
    BIG_ENDIAN :: 1;
    VAX_ALL :: 0;
    VAX780 :: 1;
    VAX785 :: 2;
    VAX750 :: 3;
    VAX730 :: 4;
    UVAXI :: 5;
    UVAXII :: 6;
    VAX8200 :: 7;
    VAX8500 :: 8;
    VAX8600 :: 9;
    VAX8650 :: 10;
    VAX8800 :: 11;
    UVAXIII :: 12;
    MC680x0_ALL :: 1;
    MC68030 :: 1;
    MC68040 :: 2;
    MC68030_ONLY :: 3;
    INTEL_FAMILY_MAX :: 15;
    INTEL_MODEL_ALL :: 0;
    X86_ALL :: 3;
    X86_64_ALL :: 3;
    X86_ARCH1 :: 4;
    X86_64_H :: 8;
    MIPS_ALL :: 0;
    MIPS_R2300 :: 1;
    MIPS_R2600 :: 2;
    MIPS_R2800 :: 3;
    MIPS_R2000a :: 4;
    MIPS_R2000 :: 5;
    MIPS_R3000a :: 6;
    MIPS_R3000 :: 7;
    MC98000_ALL :: 0;
    MC98601 :: 1;
    HPPA_ALL :: 0;
    HPPA_7100 :: 0;
    HPPA_7100LC :: 1;
    MC88000_ALL :: 0;
    MC88100 :: 1;
    MC88110 :: 2;
    SPARC_ALL :: 0;
    I860_ALL :: 0;
    I860_860 :: 1;
    POWERPC_ALL :: 0;
    POWERPC_601 :: 1;
    POWERPC_602 :: 2;
    POWERPC_603 :: 3;
    POWERPC_603e :: 4;
    POWERPC_603ev :: 5;
    POWERPC_604 :: 6;
    POWERPC_604e :: 7;
    POWERPC_620 :: 8;
    POWERPC_750 :: 9;
    POWERPC_7400 :: 10;
    POWERPC_7450 :: 11;
    POWERPC_970 :: 100;
    ARM_ALL :: 0;
    ARM_V4T :: 5;
    ARM_V6 :: 6;
    ARM_V5TEJ :: 7;
    ARM_XSCALE :: 8;
    ARM_V7 :: 9;
    ARM_V7F :: 10;
    ARM_V7S :: 11;
    ARM_V7K :: 12;
    ARM_V8 :: 13;
    ARM_V6M :: 14;
    ARM_V7M :: 15;
    ARM_V7EM :: 16;
    ARM_V8M :: 17;
    ARM_V8M_MAIN :: ARM_V8M;
    ARM_V8M_BASE :: 18;
    ARM_V8_1M_MAIN :: 19;
    ARM64_ALL :: 0;
    ARM64_V8 :: 1;
    ARM64E :: 2;
    ARM64_PTR_AUTH_MASK :: 0x0f000000;
    ARM64_32_ALL :: 0;
    ARM64_32_V8 :: 1;
}

CPUFAMILY :: enum {
    UNKNOWN :: 0;
    POWERPC_G3 :: 0xcee41549;
    POWERPC_G4 :: 0x77c184ae;
    POWERPC_G5 :: 0xed76d8aa;
    INTEL_6_13 :: 0xaa33392b;
    INTEL_PENRYN :: 0x78ea4fbc;
    INTEL_NEHALEM :: 0x6b5a4cd2;
    INTEL_WESTMERE :: 0x573b5eec;
    INTEL_SANDYBRIDGE :: 0x5490b78c;
    INTEL_IVYBRIDGE :: 0x1f65e835;
    INTEL_HASWELL :: 0x10b282dc;
    INTEL_BROADWELL :: 0x582ed09c;
    INTEL_SKYLAKE :: 0x37fc219f;
    INTEL_KABYLAKE :: 0x0f817246;
    INTEL_ICELAKE :: 0x38435547;
    INTEL_COMETLAKE :: 0x1cf8a03e;
    ARM_9 :: 0xe73283ae;
    ARM_11 :: 0x8ff620d8;
    ARM_XSCALE :: 0x53b005f5;
    ARM_12 :: 0xbd1b0ae9;
    ARM_13 :: 0x0cc90e64;
    ARM_14 :: 0x96077ef1;
    ARM_15 :: 0xa8511bca;
    ARM_SWIFT :: 0x1e2d6381;
    ARM_CYCLONE :: 0x37a09642;
    ARM_TYPHOON :: 0x2c91a47e;
    ARM_TWISTER :: 0x92fb37c8;
    ARM_HURRICANE :: 0x67ceee93;
    ARM_MONSOON_MISTRAL :: 0xe81e7ef6;
    ARM_VORTEX_TEMPEST :: 0x07d34b9f;
    ARM_LIGHTNING_THUNDER :: 0x462504d2;
    ARM_FIRESTORM_ICESTORM :: 0x1b588bb3;
    ARM_BLIZZARD_AVALANCHE :: 0xda33d83d;
    ARM_EVEREST_SAWTOOTH :: 0x8765edea;
    ARM_IBIZA :: 0xfa33415e;
    ARM_PALMA :: 0x72015832;
    ARM_COLL :: 0x2876f5b5;
    ARM_LOBOS :: 0x5f4dea93;
    ARM_DONAN :: 0x6f5129ac;
    ARM_BRAVA :: 0x17d5b93a;
    ARM_TAHITI :: 0x75d4acb9;
    ARM_TUPAI :: 0x204526d0;
    ARM_HIDRA :: 0x1d5a87e8;
    ARM_THERA :: 0xab345f09;
    ARM_TILOS :: 0x01d7a72b;
    INTEL_6_23 :: INTEL_PENRYN;
    INTEL_6_26 :: INTEL_NEHALEM;
}

CPUSUBFAMILY :: enum {
    UNKNOWN :: 0;
    ARM_HP :: 1;
    ARM_HG :: 2;
    ARM_M :: 3;
    ARM_HS :: 4;
    ARM_HC_HD :: 5;
    ARM_HA :: 6;
}

Load_Command_Type :: enum u32 {
    REQ_DYLD :: 0x80000000;
    SEGMENT :: 0x1;
    SYMTAB :: 0x2;
    SYMSEG :: 0x3;
    THREAD :: 0x4;
    UNIXTHREAD :: 0x5;
    LOADFVMLIB :: 0x6;
    IDFVMLIB :: 0x7;
    IDENT :: 0x8;
    FVMFILE :: 0x9;
    PREPAGE :: 0xa;
    DYSYMTAB :: 0xb;
    LOAD_DYLIB :: 0xc;
    ID_DYLIB :: 0xd;
    LOAD_DYLINKER :: 0xe;
    ID_DYLINKER :: 0xf;
    PREBOUND_DYLIB :: 0x10;
    ROUTINES :: 0x11;
    SUB_FRAMEWORK :: 0x12;
    SUB_UMBRELLA :: 0x13;
    SUB_CLIENT :: 0x14;
    SUB_LIBRARY :: 0x15;
    TWOLEVEL_HINTS :: 0x16;
    PREBIND_CKSUM :: 0x17;
    LOAD_WEAK_DYLIB :: 0x18 | REQ_DYLD;
    SEGMENT_64 :: 0x19;
    ROUTINES_64 :: 0x1a;
    UUID :: 0x1b;
    RPATH :: 0x1c | REQ_DYLD;
    CODE_SIGNATURE :: 0x1d;
    SEGMENT_SPLIT_INFO :: 0x1e;
    REEXPORT_DYLIB :: 0x1f | REQ_DYLD;
    LAZY_LOAD_DYLIB :: 0x20;
    ENCRYPTION_INFO :: 0x21;
    DYLD_INFO :: 0x22;
    DYLD_INFO_ONLY :: 0x22 | REQ_DYLD;
    LOAD_UPWARD_DYLIB :: 0x23 | REQ_DYLD;
    VERSION_MIN_MACOSX :: 0x24;
    VERSION_MIN_IPHONEOS :: 0x25;
    FUNCTION_STARTS :: 0x26;
    DYLD_ENVIRONMENT :: 0x27;
    MAIN :: 0x28 | REQ_DYLD;
    DATA_IN_CODE :: 0x29;
    SOURCE_VERSION :: 0x2A;
    DYLIB_CODE_SIGN_DRS :: 0x2B;
    ENCRYPTION_INFO_64 :: 0x2C;
    LINKER_OPTION :: 0x2D;
    LINKER_OPTIMIZATION_HINT :: 0x2E;
    VERSION_MIN_TVOS :: 0x2F;
    VERSION_MIN_WATCHOS :: 0x30;
    NOTE :: 0x31;
    BUILD_VERSION :: 0x32;
    DYLD_EXPORTS_TRIE :: 0x33 | REQ_DYLD;
    DYLD_CHAINED_FIXUPS :: 0x34 | REQ_DYLD;
    FILESET_ENTRY :: 0x35 | REQ_DYLD;
    ATOM_INFO :: 0x36;
    FUNCTION_VARIANTS :: 0x37;
    FUNCTION_VARIANT_FIXUPS :: 0x38;
    TARGET_TRIPLE :: 0x39;
}

Segment_Flags :: enum_flags u32 {
    HIGHVM :: 0x1;
    FVMLIB :: 0x2;
    NORELOC :: 0x4;
    PROTECTED_VERSION_1 :: 0x8;
    READ_ONLY :: 0x10;
}

Section_Flags :: enum_flags u32 {
    REGULAR :: 0x0;
    ZEROFILL :: 0x1;
    CSTRING_LITERALS :: 0x2;
    _4BYTE_LITERALS :: 0x3;
    _8BYTE_LITERALS :: 0x4;
    LITERAL_POINTERS :: 0x5;
    NON_LAZY_SYMBOL_POINTERS :: 0x6;
    LAZY_SYMBOL_POINTERS :: 0x7;
    SYMBOL_STUBS :: 0x8;
    MOD_INIT_FUNC_POINTERS :: 0x9;
    MOD_TERM_FUNC_POINTERS :: 0xa;
    COALESCED :: 0xb;
    GB_ZEROFILL :: 0xc;
    INTERPOSING :: 0xd;
    _16BYTE_LITERALS :: 0xe;
    DTRACE_DOF :: 0xf;
    LAZY_DYLIB_SYMBOL_POINTERS :: 0x10;
    THREAD_LOCAL_REGULAR :: 0x11;
    THREAD_LOCAL_ZEROFILL :: 0x12;
    THREAD_LOCAL_VARIABLES :: 0x13;
    THREAD_LOCAL_VARIABLE_POINTERS :: 0x14;
    THREAD_LOCAL_INIT_FUNCTION_POINTERS :: 0x15;
    INIT_FUNC_OFFSETS :: 0x16;
    ATTR_PURE_INSTRUCTIONS :: 0x80000000;
    ATTR_NO_TOC :: 0x40000000;
    ATTR_STRIP_STATIC_SYMS :: 0x20000000;
    ATTR_NO_DEAD_STRIP :: 0x10000000;
    ATTR_LIVE_SUPPORT :: 0x08000000;
    ATTR_SELF_MODIFYING_CODE :: 0x04000000;
    ATTR_DEBUG :: 0x02000000;
    ATTR_SOME_INSTRUCTIONS :: 0x00000400;
    ATTR_EXT_RELOC :: 0x00000200;
    ATTR_LOC_RELOC :: 0x00000100;
}

mach_header :: struct {
    magic: u32;
    cputype: s32;
    cpusubtype: s32;
    filetype: u32;
    ncmds: u32;
    sizeofcmds: u32;
    flags: u32;
}

mach_header_64 :: struct {
    magic: u32;
    cputype: s32;
    cpusubtype: s32;
    filetype: u32;
    ncmds: u32;
    sizeofcmds: u32;
    flags: u32;
    reserved: u32;
}

load_command :: struct {
    cmd: u32;
    cmdsize: u32;
}

lc_str :: struct {
    offset: u32;
}

segment_command :: struct {
    cmd: u32;
    cmdsize: u32;
    segname: [16] u8;
    vmaddr: u32;
    vmsize: u32;
    fileoff: u32;
    filesize: u32;
    maxprot: s32;
    initprot: s32;
    nsects: u32;
    flags: u32;
}

segment_command_64 :: struct {
    cmd: u32;
    cmdsize: u32;
    segname: [16] u8;
    vmaddr: u64;
    vmsize: u64;
    fileoff: u64;
    filesize: u64;
    maxprot: s32;
    initprot: s32;
    nsects: u32;
    flags: u32;
}

section :: struct {
    sectname: [16] u8;
    segname: [16] u8;
    addr: u32;
    size: u32;
    offset: u32;
    align: u32;
    reloff: u32;
    nreloc: u32;
    flags: u32;
    reserved1: u32;
    reserved2: u32;
}

section_64 :: struct {
    sectname: [16] u8;
    segname: [16] u8;
    addr: u64;
    size: u64;
    offset: u32;
    align: u32;
    reloff: u32;
    nreloc: u32;
    flags: u32;
    reserved1: u32;
    reserved2: u32;
    reserved3: u32;
}

fvmlib :: struct {
    name: lc_str;
    minor_version: u32;
    header_addr: u32;
}

fvmlib_command :: struct {
    cmd: u32;
    cmdsize: u32;
    fvmlib_: fvmlib;
}

dylib :: struct {
    name: lc_str;
    timestamp: u32;
    current_version: u32;
    compatibility_version: u32;
}

dylib_command :: struct {
    cmd: u32;
    cmdsize: u32;
    dylib_: dylib;
}

dylib_use_command :: struct {
    cmd: u32;
    cmdsize: u32;
    nameoff: u32;
    marker: u32;
    current_version: u32;
    compat_version: u32;
    flags: u32;
}

sub_framework_command :: struct {
    cmd: u32;
    cmdsize: u32;
    umbrella: lc_str;
}

sub_client_command :: struct {
    cmd: u32;
    cmdsize: u32;
    client: lc_str;
}

sub_umbrella_command :: struct {
    cmd: u32;
    cmdsize: u32;
    sub_umbrella: lc_str;
}

sub_library_command :: struct {
    cmd: u32;
    cmdsize: u32;
    sub_library: lc_str;
}

prebound_dylib_command :: struct {
    cmd: u32;
    cmdsize: u32;
    name: lc_str;
    nmodules: u32;
    linked_modules: lc_str;
}

dylinker_command :: struct {
    cmd: u32;
    cmdsize: u32;
    name: lc_str;
}

thread_command :: struct {
    cmd: u32;
    cmdsize: u32;
}

routines_command :: struct {
    cmd: u32;
    cmdsize: u32;
    init_address: u32;
    init_module: u32;
    reserved1: u32;
    reserved2: u32;
    reserved3: u32;
    reserved4: u32;
    reserved5: u32;
    reserved6: u32;
}

routines_command_64 :: struct {
    cmd: u32;
    cmdsize: u32;
    init_address: u64;
    init_module: u64;
    reserved1: u64;
    reserved2: u64;
    reserved3: u64;
    reserved4: u64;
    reserved5: u64;
    reserved6: u64;
}

symtab_command :: struct {
    cmd: u32;
    cmdsize: u32;
    symoff: u32;
    nsyms: u32;
    stroff: u32;
    strsize: u32;
}

dysymtab_command :: struct {
    cmd: u32;
    cmdsize: u32;
    ilocalsym: u32;
    nlocalsym: u32;
    iextdefsym: u32;
    nextdefsym: u32;
    iundefsym: u32;
    nundefsym: u32;
    tocoff: u32;
    ntoc: u32;
    modtaboff: u32;
    nmodtab: u32;
    extrefsymoff: u32;
    nextrefsyms: u32;
    indirectsymoff: u32;
    nindirectsyms: u32;
    extreloff: u32;
    nextrel: u32;
    locreloff: u32;
    nlocrel: u32;
}

dylib_table_of_contents :: struct {
    symbol_index: u32;
    module_index: u32;
}

dylib_module :: struct {
    module_name: u32;
    iextdefsym: u32;
    nextdefsym: u32;
    irefsym: u32;
    nrefsym: u32;
    ilocalsym: u32;
    nlocalsym: u32;
    iextrel: u32;
    nextrel: u32;
    iinit_iterm: u32;
    ninit_nterm: u32;
    objc_module_info_addr: u32;
    objc_module_info_size: u32;
}

dylib_module_64 :: struct {
    module_name: u32;
    iextdefsym: u32;
    nextdefsym: u32;
    irefsym: u32;
    nrefsym: u32;
    ilocalsym: u32;
    nlocalsym: u32;
    iextrel: u32;
    nextrel: u32;
    iinit_iterm: u32;
    ninit_nterm: u32;
    objc_module_info_size: u32;
    objc_module_info_addr: u64;
}

dylib_reference :: struct {
    __bitfield: u32;
}

twolevel_hints_command :: struct {
    cmd: u32;
    cmdsize: u32;
    offset: u32;
    nhints: u32;
}

twolevel_hint :: struct {
    __bitfield: u32;
}

prebind_cksum_command :: struct {
    cmd: u32;
    cmdsize: u32;
    cksum: u32;
}

uuid_command :: struct {
    cmd: u32;
    cmdsize: u32;
    uuid: [16] u8;
}

rpath_command :: struct {
    cmd: u32;
    cmdsize: u32;
    path: lc_str;
}

target_triple_command :: struct {
    cmd: u32;
    cmdsize: u32;
    triple: lc_str;
}

linkedit_data_command :: struct {
    cmd: u32;
    cmdsize: u32;
    dataoff: u32;
    datasize: u32;
}

encryption_info_command :: struct {
    cmd: u32;
    cmdsize: u32;
    cryptoff: u32;
    cryptsize: u32;
    cryptid: u32;
}

encryption_info_command_64 :: struct {
    cmd: u32;
    cmdsize: u32;
    cryptoff: u32;
    cryptsize: u32;
    cryptid: u32;
    pad: u32;
}

version_min_command :: struct {
    cmd: u32;
    cmdsize: u32;
    version: u32;
    sdk: u32;
}

build_version_command :: struct {
    cmd: u32;
    cmdsize: u32;
    platform: u32;
    minos: u32;
    sdk: u32;
    ntools: u32;
}

build_tool_version :: struct {
    tool: u32;
    version: u32;
}

dyld_info_command :: struct {
    cmd: u32;
    cmdsize: u32;
    rebase_off: u32;
    rebase_size: u32;
    bind_off: u32;
    bind_size: u32;
    weak_bind_off: u32;
    weak_bind_size: u32;
    lazy_bind_off: u32;
    lazy_bind_size: u32;
    export_off: u32;
    export_size: u32;
}

linker_option_command :: struct {
    cmd: u32;
    cmdsize: u32;
    count: u32;
}

symseg_command :: struct {
    cmd: u32;
    cmdsize: u32;
    offset: u32;
    size: u32;
}

ident_command :: struct {
    cmd: u32;
    cmdsize: u32;
}

fvmfile_command :: struct {
    cmd: u32;
    cmdsize: u32;
    name: lc_str;
    header_addr: u32;
}

entry_point_command :: struct {
    cmd: u32;
    cmdsize: u32;
    entryoff: u64;
    stacksize: u64;
}

source_version_command :: struct {
    cmd: u32;
    cmdsize: u32;
    version: u64;
}

data_in_code_entry :: struct {
    offset: u32;
    length: u16;
    kind: u16;
}

tlv_descriptor :: struct {
    thunk: #type (unknown0: *tlv_descriptor) -> *void #c_call;
    key: u64;
    offset: u64;
}

note_command :: struct {
    cmd: u32;
    cmdsize: u32;
    data_owner: [16] u8;
    offset: u64;
    size: u64;
}

fileset_entry_command :: struct {
    cmd: u32;
    cmdsize: u32;
    vmaddr: u64;
    fileoff: u64;
    entry_id: lc_str;
    reserved: u32;
}

relocation_info :: struct {
    r_address: s32;
    __bitfield: u32;
}

scattered_relocation_info :: struct {
    __bitfield: u32;
    r_value: s32;
}

reloc_type_generic :: enum u32 {
    VANILLA :: 0;
    PAIR :: 1;
    SECTDIFF :: 2;
    PB_LA_PTR :: 3;
    LOCAL_SECTDIFF :: 4;
    TLV :: 5;
}

nlist :: struct {
    n_un: struct {
        n_strx: u32;
    };
    n_type: u8;
    n_sect: u8;
    n_desc: s16;
    n_value: u32;
}

nlist_64 :: struct {
    n_un: struct {
        n_strx: u32;
    };
    n_type: u8;
    n_sect: u8;
    n_desc: u16;
    n_value: u64;
}

reloc_type_x86_64 :: enum u32 {
    UNSIGNED :: 0;
    SIGNED :: 1;
    BRANCH :: 2;
    GOT_LOAD :: 3;
    GOT :: 4;
    SUBTRACTOR :: 5;
    SIGNED_1 :: 6;
    SIGNED_2 :: 7;
    SIGNED_4 :: 8;
    TLV :: 9;
}

reloc_type_arm64 :: enum u32 {
    UNSIGNED :: 0;
    SUBTRACTOR :: 1;
    BRANCH26 :: 2;
    PAGE21 :: 3;
    PAGEOFF12 :: 4;
    GOT_LOAD_PAGE21 :: 5;
    GOT_LOAD_PAGEOFF12 :: 6;
    POINTER_TO_GOT :: 7;
    TLVP_LOAD_PAGE21 :: 8;
    TLVP_LOAD_PAGEOFF12 :: 9;
    ADDEND :: 10;
    AUTHENTICATED_POINTER :: 11;
}

Coff_Header :: struct {
    machine: Machine_Type;
    num_sections: u16;
    timestamp: u32;
    symbol_table_offset: u32;
    num_symbols: u32;
    optional_header_size: u16;
    characteristics: Header_Characteristics;
}

Coff_Optional_Header_Pe32_Plus :: struct {
    magic: Magic;
    major_linker_version: u8;
    minor_linker_version: u8;
    size_of_code: u32;
    size_of_initialized_data: u32;
    size_of_uninitialized_data: u32;
    address_of_entry_point: u32;
    base_of_code: u32;
    Magic :: enum u16 {
        PE32 :: 0x010b;
        PE32_PLUS :: 0x020b;
    };
}

Coff_Optional_Header_Pe32 :: struct {
    using base: Coff_Optional_Header_Pe32_Plus;
    base_of_data: u32;
}

Coff_Optional_Header_Windows_Specific :: struct(Size_Offset_Type: Type) {
    image_base: Size_Offset_Type;
    section_alignment: u32;
    file_alignment: u32;
    major_operating_system_version: u16;
    minor_operating_system_version: u16;
    major_image_version: u16;
    minor_image_version: u16;
    major_subsystem_version: u16;
    minor_subsystem_version: u16;
    win32_version_value: u32;
    size_of_image: u32;
    size_of_headers: u32;
    check_sum: u32;
    subsystem: Windows_Subsystem_Type;
    dll_characteristics: Windows_Dll_Characteristics;
    size_of_stack_reserve: Size_Offset_Type;
    size_of_stack_commit: Size_Offset_Type;
    size_of_heap_reserve: Size_Offset_Type;
    size_of_heap_commit: Size_Offset_Type;
    loader_flags: u32;
    number_of_rva_and_sizes: u32;
}

Coff_Data_Directory :: struct {
    virtual_address: u32;
    size: u32;
}

Coff_Data_Directory_Index :: enum {
    Export_Table :: 0;
    Import_Table :: 1;
    Resource_Table :: 2;
    Exception_Table :: 3;
    Certificate_Table :: 4;
    Base_Relocation_Table :: 5;
    Debug :: 6;
    Architecture :: 7;
    Global_Ptr :: 8;
    TLS_Table :: 9;
    Load_Config_Table :: 10;
    Bound_Import :: 11;
    IAT :: 12;
    Delay_Import_Descriptor :: 13;
    CLR_Runtime_Header :: 14;
    Reserved :: 15;
}

Coff_Debug_Directory_Entry :: struct {
    characteristics: u32;
    time_date_stamp: u32;
    major_version: u16;
    minor_version: u16;
    type: Coff_Debug_Type;
    size_of_data: u32;
    address_of_raw_data: u32;
    pointer_to_raw_data: u32;
}

Coff_Debug_Type :: enum u32 {
    TYPE_UNKNOWN :: 0;
    TYPE_COFF :: 1;
    TYPE_CODEVIEW :: 2;
    TYPE_FPO :: 3;
    TYPE_MISC :: 4;
    TYPE_EXCEPTION :: 5;
    TYPE_FIXUP :: 6;
    TYPE_OMAP_TO_SRC :: 7;
    TYPE_OMAP_FROM_SRC :: 8;
    TYPE_BORLAND :: 9;
    TYPE_RESERVED10 :: 10;
    TYPE_CLSID :: 11;
    TYPE_REPRO :: 16;
    TYPE_EMBEDDED :: 17;
    TYPE_CRYPTO_HASH :: 19;
    TYPE_EX_DLLCHARACTERISTICS :: 20;
}

Coff_Section_Header :: struct {
    name: [8] u8;
    virtual_size: u32;
    virtual_address: u32;
    size_of_raw_data: u32;
    pointer_to_raw_data: u32;
    pointer_to_relocations: u32;
    pointer_to_line_numbers: u32;
    number_of_relocations: u16;
    number_of_line_numbers: u16;
    characteristics: Section_Characteristics;
}

Machine_Type :: enum u16 {
    UNKNOWN :: 0x0;
    AM33 :: 0x1d3;
    AMD64 :: 0x8664;
    ARM :: 0x1c0;
    ARM64 :: 0xaa64;
    ARMNT :: 0x1c4;
    EBC :: 0xebc;
    I386 :: 0x14c;
    IA64 :: 0x200;
    LOONGARCH32 :: 0x6232;
    LOONGARCH64 :: 0x6264;
    M32R :: 0x9041;
    MIPS16 :: 0x266;
    MIPSFPU :: 0x366;
    MIPSFPU16 :: 0x466;
    POWERPC :: 0x1f0;
    POWERPCFP :: 0x1f1;
    R4000 :: 0x166;
    RISCV32 :: 0x5032;
    RISCV64 :: 0x5064;
    RISCV128 :: 0x5128;
    SH3 :: 0x1a2;
    SH3DSP :: 0x1a3;
    SH4 :: 0x1a6;
    SH5 :: 0x1a8;
    THUMB :: 0x1c2;
    WCEMIPSV2 :: 0x169;
}

Header_Characteristics :: enum_flags u16 {
    RELOCS_STRIPPED :: 0x0001;
    EXECUTABLE_IMAGE :: 0x0002;
    LINE_NUMS_STRIPPED :: 0x0004;
    LOCAL_SYMS_STRIPPED :: 0x0008;
    AGGRESSIVE_WS_TRIM :: 0x0010;
    LARGE_ADDRESS_AWARE :: 0x0020;
    BYTES_REVERSED_LO :: 0x0080;
    _32BIT_MACHINE :: 0x0100;
    DEBUG_STRIPPED :: 0x0200;
    REMOVABLE_RUN_FROM_SWAP :: 0x0400;
    NET_RUN_FROM_SWAP :: 0x0800;
    SYSTEM :: 0x1000;
    DLL :: 0x2000;
    UP_SYSTEM_ONLY :: 0x4000;
    BYTES_REVERSED_HI :: 0x8000;
}

Windows_Subsystem_Type :: enum u16 {
    UNKNOWN :: 0;
    NATIVE :: 1;
    WINDOWS_GUI :: 2;
    WINDOWS_CUI :: 3;
    OS2_CUI :: 5;
    POSIX_CUI :: 7;
    NATIVE_WINDOWS :: 8;
    WINDOWS_CE_GUI :: 9;
    EFI_APPLICATION :: 10;
    EFI_BOOT_SERVICE_DRIVER :: 11;
    EFI_RUNTIME_DRIVER :: 12;
    EFI_ROM :: 13;
    XBOX :: 14;
    WINDOWS_BOOT_APPLICATION :: 16;
}

Windows_Dll_Characteristics :: enum_flags u16 {
    HIGH_ENTROPY_VA :: 0x0020;
    DYNAMIC_BASE :: 0x0040;
    FORCE_INTEGRITY :: 0x0080;
    NX_COMPAT :: 0x0100;
    NO_ISOLATION :: 0x0200;
    NO_SEH :: 0x0400;
    NO_BIND :: 0x0800;
    APPCONTAINER :: 0x1000;
    WDM_DRIVER :: 0x2000;
    GUARD_CF :: 0x4000;
    TERMINAL_SERVER_AWARE :: 0x8000;
}

Section_Characteristics :: enum_flags u32 {
    TYPE_NO_PAD :: 0x00000008;
    CNT_CODE :: 0x00000020;
    CNT_INITIALIZED_DATA :: 0x00000040;
    CNT_UNINITIALIZED_DATA :: 0x00000080;
    LNK_OTHER :: 0x00000100;
    LNK_INFO :: 0x00000200;
    LNK_REMOVE :: 0x00000800;
    LNK_COMDAT :: 0x00001000;
    GPREL :: 0x00008000;
    MEM_PURGEABLE :: 0x00020000;
    MEM_16BIT :: 0x00020000;
    MEM_LOCKED :: 0x00040000;
    MEM_PRELOAD :: 0x00080000;
    ALIGN_1BYTES :: 0x00100000;
    ALIGN_2BYTES :: 0x00200000;
    ALIGN_4BYTES :: 0x00300000;
    ALIGN_8BYTES :: 0x00400000;
    ALIGN_16BYTES :: 0x00500000;
    ALIGN_32BYTES :: 0x00600000;
    ALIGN_64BYTES :: 0x00700000;
    ALIGN_128BYTES :: 0x00800000;
    ALIGN_256BYTES :: 0x00900000;
    ALIGN_512BYTES :: 0x00A00000;
    ALIGN_1024BYTES :: 0x00B00000;
    ALIGN_2048BYTES :: 0x00C00000;
    ALIGN_4096BYTES :: 0x00D00000;
    ALIGN_8192BYTES :: 0x00E00000;
    LNK_NRELOC_OVFL :: 0x01000000;
    MEM_DISCARDABLE :: 0x02000000;
    MEM_NOT_CACHED :: 0x04000000;
    MEM_NOT_PAGED :: 0x08000000;
    MEM_SHARED :: 0x10000000;
    MEM_EXECUTE :: 0x20000000;
    MEM_READ :: 0x40000000;
    MEM_WRITE :: 0x80000000;
}

Pdata_Entry :: struct {
    begin_address: u32;
    end_address: u32;
    unwind_info_address: u32;
}

Coff_Relocation :: struct {
    virtual_address: u32 #align 1;
    symbol_index: u32 #align 1;
    type: Coff_Relocation_Type_x64 #align 1;
}

Coff_Relocation_Type_x64 :: enum u16 {
    ABSOLUTE :: 0x0000;
    ADDR64 :: 0x0001;
    ADDR32 :: 0x0002;
    ADDR32NB :: 0x0003;
    REL32 :: 0x0004;
    REL32_1 :: 0x0005;
    REL32_2 :: 0x0006;
    REL32_3 :: 0x0007;
    REL32_4 :: 0x0008;
    REL32_5 :: 0x0009;
    SECTION :: 0x000A;
    SECREL :: 0x000B;
    SECREL7 :: 0x000C;
    TOKEN :: 0x000D;
    SREL32 :: 0x000E;
    PAIR :: 0x000F;
    SSPAN32 :: 0x0010;
}

Coff_Symbol :: struct {
    name: union {
        short_name: [8] u8;
        long_name: struct {
            zeroes: u32 #align 1;
            offset: u32 #align 1;
        };
    } #align 1;
    value: u32 #align 1;
    section_number: u16 #align 1;
    type: u16 #align 1;
    storage_class: Coff_Symbol_Storage_Class #align 1;
    number_of_aux_symbols: u8 #align 1;
}

Coff_Symbol_Storage_Class :: enum u8 {
    END_OF_FUNCTION :: 0xFF;
    NULL :: 0;
    AUTOMATIC :: 1;
    EXTERNAL :: 2;
    STATIC :: 3;
    REGISTER :: 4;
    EXTERNAL_DEF :: 5;
    LABEL :: 6;
    UNDEFINED_LABEL :: 7;
    MEMBER_OF_STRUCT :: 8;
    ARGUMENT :: 9;
    STRUCT_TAG :: 10;
    MEMBER_OF_UNION :: 11;
    UNION_TAG :: 12;
    TYPE_DEFINITION :: 13;
    UNDEFINED_STATIC :: 14;
    ENUM_TAG :: 15;
    MEMBER_OF_ENUM :: 16;
    REGISTER_PARAM :: 17;
    BIT_FIELD :: 18;
    BLOCK :: 100;
    FUNCTION :: 101;
    END_OF_STRUCT :: 102;
    FILE :: 103;
    SECTION :: 104;
    WEAK_EXTERNAL :: 105;
    CLR_TOKEN :: 107;
}

Unwind_Info_Header :: struct {
    version_flags: u8;
    prolog_size: u8;
    num_unwind_codes: u8;
    frame_register: u8;
}

Unwind_Info :: struct {
    version: u8;
    flags: Unwind_Info_Flags;
    prolog_size: u8;
    frame_register: u8;
    scaled_frame_register_offset: u8;
    unwind_codes: [] u16;
    extra: union {
        handler: *Unwind_Info_Exception_Handler;
        chained: *Pdata_Entry;
    };
}

Unwind_Info_Flags :: enum_flags u8 {
    NHANDLER :: 0x0;
    EHANDLER :: 0x1;
    UHANDLER :: 0x2;
    CHAININFO :: 0x4;
}

Unwind_Info_Exception_Handler :: struct {
    address: u32;
}

Unwind_Code :: struct {
    prolog_offset: u8;
    code_info: u8;
}

Unwind_Operation_Code :: enum u8 {
    PUSH_NONVOL :: 0;
    ALLOC_LARGE :: 1;
    ALLOC_SMALL :: 2;
    SET_FPREG :: 3;
    SAVE_NONVOL :: 4;
    SAVE_NONVOL_FAR :: 5;
    SAVE_XMM128 :: 8;
    SAVE_XMM128_FAR :: 9;
    PUSH_MACHFRAME :: 10;
}

Unwind_Operation_Info_Register :: enum {
    RAX :: 0;
    RCX :: 1;
    RDX :: 2;
    RBX :: 3;
    RSP :: 4;
    RBP :: 5;
    RSI :: 6;
    RDI :: 7;
    R8 :: 8;
    R9 :: 9;
    R10 :: 10;
    R11 :: 11;
    R12 :: 12;
    R13 :: 13;
    R14 :: 14;
    R15 :: 15;
}

Elf :: struct {
    header: *Elf_Header_64;
    program_header_table: [] Elf_Program_Header_64;
    sections: [] Elf_Section_Header;
    section_names: string;
    text_section: *Elf_Section_Header;
    debug_info_section_index: int;
    debug_line_section_index: int;
    debug_info_section: *Elf_Section_Header;
    debug_abbrev_section: *Elf_Section_Header;
    debug_str_section: *Elf_Section_Header;
    debug_line_section: *Elf_Section_Header;
    debug_loc_section: *Elf_Section_Header;
    debug_ranges_section: *Elf_Section_Header;
    text: [] u8;
    symbols: [] Elf_Symbol;
    symbol_names: string;
    dynamic_symbols: [] Elf_Symbol;
    dynamic_symbol_names: string;
    content: string;
}

Elf_Header :: struct(Offset_Type: Type) {
    ident: struct {
        magic: u32;
        class: Elf_Class;
        data: Elf_Endianness;
        version: u8;
        os_abi: Elf_Abi;
        abi_version: u8;
        pad: [7] u8;
    };
    type: Elf_Type;
    machine: Elf_Machine;
    version: u32;
    entry: Offset_Type;
    program_header_offset: Offset_Type;
    section_header_offset: Offset_Type;
    flags: u32;
    ehsize: u16;
    phentsize: u16;
    phnum: u16;
    section_header_entry_size: u16;
    num_section_headers: u16;
    section_names_section_index: u16;
}

Elf_Class :: enum u8 {
    _32_BIT :: 1;
    _64_BIT :: 2;
}

Elf_Endianness :: enum u8 {
    LSB :: 1;
    MSB :: 1;
}

Elf_Abi :: enum u8 {
    SYSTEM_V :: 0x00;
    LINUX :: 0x03;
    FREEBSD :: 0x09;
}

Elf_Type :: enum u16 {
    NONE :: 0x00;
    REL :: 0x01;
    EXEC :: 0x02;
    DYN :: 0x03;
    CORE :: 0x04;
    LOOS :: 0xFE00;
    HIOS :: 0xFEFF;
    LOPROC :: 0xFF00;
    HIPROC :: 0xFFFF;
}

Elf_Machine :: enum u16 {
    AMD_x64 :: 0x3E;
    ARM_v8 :: 0xB7;
}

Elf_Section_Header :: struct {
    name_offset: u32;
    type: Elf_Section_Type;
    flags: Elf_Section_Flags;
    address: u64;
    offset: u64;
    size: u64;
    link: u32;
    info: u32;
    addralign: u64;
    entsize: u64;
}

Elf_Section_Type :: enum u32 {
    NULL :: 0x00;
    PROGBITS :: 0x01;
    SYMTAB :: 0x02;
    STRTAB :: 0x03;
    RELA :: 0x04;
    HASH :: 0x05;
    DYNAMIC :: 0x06;
    NOTE :: 0x07;
    NOBITS :: 0x08;
    REL :: 0x09;
    SHLIB :: 0x0A;
    DYNSYM :: 0x0B;
    INIT_ARRAY :: 0x0E;
    FINI_ARRAY :: 0x0F;
    PREINIT_ARRAY :: 0x10;
    GROUP :: 0x11;
    SYMTAB_SHNDX :: 0x12;
    NUM :: 0x13;
    LOOS :: 0x60000000;
    LLVM_ADDRSIG :: 0x6FFF4C03;
}

Elf_Section_Flags :: enum_flags u64 {
    WRITE :: 0x01;
    ALLOC :: 0x02;
    EXECINSTR :: 0x04;
    MERGE :: 0x10;
    STRINGS :: 0x20;
    INFO_LINK :: 0x40;
    LINK_ORDER :: 0x80;
    OS_NONCONFORMING :: 0x0100;
    GROUP :: 0x0200;
    TLS :: 0x0400;
    MASKOS :: 0x0ff00000;
    MASKPROC :: 0xf0000000;
    ORDERED :: 0x4000000;
    EXCLUDE :: 0x8000000;
}

Elf_Segment_Type :: enum u32 {
    NULL :: 0;
    LOAD :: 1;
    DYNAMIC :: 2;
    INTERP :: 3;
    NOTE :: 4;
    SHLIB :: 5;
    PHDR :: 6;
    TLS :: 7;
    NUM :: 8;
    LOOS :: 0x60000000;
    GNU_EH_FRAME :: 0x6474e550;
    GNU_STACK :: 0x6474e551;
    GNU_RELRO :: 0x6474e552;
    GNU_PROPERTY :: 0x6474e553;
    LOSUNW :: 0x6ffffffa;
    SUNWBSS :: 0x6ffffffa;
    SUNWSTACK :: 0x6ffffffb;
    HISUNW :: 0x6fffffff;
    HIOS :: 0x6fffffff;
    LOPROC :: 0x70000000;
    HIPROC :: 0x7fffffff;
}

Elf_Segment_Flags :: enum_flags u32 {
    EXEC :: (1 << 0);
    WRITE :: (1 << 1);
    READ :: (1 << 2);
    MASKOS :: 0x0ff00000;
    MASKPROC :: 0xf0000000;
}

Elf_Symbol :: struct {
    name_offset: u32;
    info: u8;
    other: u8;
    section_index: u16;
    value: u64;
    size: u64;
}

Elf_Symbol_Binding :: enum {
    LOCAL :: 0;
    GLOBAL :: 1;
    WEAK :: 2;
    NUM :: 3;
    GNU_UNIQUE :: 10;
}

Elf_Symbol_Type :: enum {
    NOTYPE :: 0;
    OBJECT :: 1;
    FUNC :: 2;
    SECTION :: 3;
    FILE :: 4;
    COMMON :: 5;
    TLS :: 6;
    NUM :: 7;
    GNU_IFUNC :: 10;
}

Elf_Symbol_Visibility :: enum u8 {
    DEFAULT :: 0;
    INTERNAL :: 1;
    HIDDEN :: 2;
    PROTECTED :: 3;
}

Elf_Special_Section_Indices :: enum u16 {
    UNDEF :: 0;
    BEFORE :: 0xff00;
    AFTER :: 0xff01;
    ABS :: 0xfff1;
    COMMON :: 0xfff2;
    XINDEX :: 0xffff;
}

Elf_Relocation :: struct {
    address: u64;
    type: Elf_Relocation_Type;
    symbol_id: Elf_Symbol_Id;
    addend: s64;
}

Elf_Relocation_Type :: enum u32 {
    AMD64_64 :: 1;
    AMD64_PC32 :: 2;
    AMD64_PLT32 :: 4;
    AMD64_JUMP_SLOT :: 7;
    AMD64_RELATIVE :: 8;
    AMD64_GOTPCREL :: 9;
    AMD64_32 :: 10;
    AMD64_32S :: 11;
    X86_64_ORBIS_GOTPCREL_LOAD :: 0x28;
    AARCH64_ABS64 :: 257;
    AARCH64_ABS32 :: 258;
    AARCH64_ABS16 :: 259;
    AARCH64_PREL64 :: 260;
    AARCH64_PREL32 :: 261;
    AARCH64_PREL16 :: 262;
    AARCH64_PLT32 :: 314;
    AARCH64_GLOB_DAT :: 1025;
    AARCH64_JUMP_SLOT :: 1026;
    AARCH64_RELATIVE :: 1027;
}

Elf_Dynamic_64 :: struct {
    tag: Elf_Dynamic_Tag;
    value: u64;
}

Elf_Dynamic_Tag :: enum s64 {
    NULL :: 0;
    NEEDED :: 1;
    PLTRELSZ :: 2;
    PLTGOT :: 3;
    HASH :: 4;
    STRTAB :: 5;
    SYMTAB :: 6;
    RELA :: 7;
    RELASZ :: 8;
    RELAENT :: 9;
    STRSZ :: 10;
    SYMENT :: 11;
    INIT :: 12;
    FINI :: 13;
    SONAME :: 14;
    RPATH :: 15;
    SYMBOLIC :: 16;
    REL :: 17;
    RELSZ :: 18;
    RELENT :: 19;
    PLTREL :: 20;
    DEBUG :: 21;
    TEXTREL :: 22;
    JMPREL :: 23;
    BIND_NOW :: 24;
    INIT_ARRAY :: 25;
    FINI_ARRAY :: 26;
    INIT_ARRAYSZ :: 27;
    FINI_ARRAYSZ :: 28;
    RUNPATH :: 29;
    FLAGS :: 30;
    ENCODING :: 32;
    PREINIT_ARRAY :: 32;
    PREINIT_ARRAYSZ :: 33;
    SYMTAB_SHNDX :: 34;
    RELRSZ :: 35;
    RELR :: 36;
    RELRENT :: 37;
    NUM :: 38;
    LOOS :: 0x6000000d;
    HIOS :: 0x6ffff000;
    LOPROC :: 0x70000000;
    HIPROC :: 0x7fffffff;
    GNU_HASH :: 0x6ffffef5;
    VERSYM :: 0x6ffffff0;
    RELACOUNT :: 0x6ffffff9;
    VERNEED :: 0x6ffffffe;
    VERNEEDNUM :: 0x6fffffff;
}

Elf_Dynamic_Flags :: enum_flags u64 {
    ORIGIN :: 0x00000001;
    SYMBOLIC :: 0x00000002;
    TEXTREL :: 0x00000004;
    BIND_NOW :: 0x00000008;
    STATIC_TLS :: 0x00000010;
}

Mach_O :: struct {
    header: *Mach_Header;
    load_commands: [] *Load_Command;
    sections: [..] *Section_64;
    symbols: [..] Mach_Symbol;
    debug_info_section: *Section_64;
    debug_abbrev_section: *Section_64;
    debug_str_section: *Section_64;
    debug_line_section: *Section_64;
    content: string;
}

Mach_Header :: struct {
    magic: u32;
    cputype: Cpu_Type;
    cpusubtype: Cpu_Subtype;
    filetype: u32;
    ncmds: u32;
    sizeofcmds: u32;
    flags: Mach_Header_Flags;
    reserved: u32;
}

Mach_Symbol :: struct {
    name: string;
    type: union {
        symbolic_type: Mach_Symbolic_Symbol_Type;
        type: Mach_Symbol_Type;
    };
    flags: Mach_Symbol_Flags;
    description: u16;
    value: u64;
    section: u8;
}

Mach_Symbol_Type :: enum u8 {
    UNDEFINED :: N_UNDF;
    ABSOLUTE :: N_ABS;
    SECTION :: N_SECT;
    PREBOUND :: N_PBUD;
    INDIRECT :: N_INDR;
}

Mach_Symbolic_Symbol_Type :: enum u8 {
    GLOBAL :: N_GSYM;
    F77_NAME :: N_FNAME;
    PROC_NAME :: N_FUN;
    DATA_VAR :: N_STSYM;
    BSS_VAR :: N_LCSYM;
    BEGIN_NSECT :: N_BNSYM;
    AST :: N_AST;
    GCC_OPT :: N_OPT;
    REGISTER :: N_RSYM;
    SRC_LINE :: N_SLINE;
    END_NSECT :: N_ENSYM;
    STRUCT :: N_SSYM;
    SOURCE_FILE :: N_SO;
    OBJECT_FILE :: N_OSO;
    LOCAL :: N_LSYM;
    BEGIN_COMMON :: N_BCOMM;
    END_COMMON :: N_ECOMM;
    END_COMMON_LOCAL :: N_ECOML;
    LENGTH :: N_LENG;
}

Mach_Symbol_Flags :: enum_flags u8 {
    EXTERNAL;
    PRIVATE_EXTERNAL;
    SYMBOLIC;
}

Load_Command :: struct {
    cmd: Load_Command_Type;
    cmdsize: u32;
}

Segment_Command_64 :: struct {
    using base: Load_Command;
    segname: [16] u8;
    vmaddr: u64;
    vmsize: u64;
    fileoff: u64;
    filesize: u64;
    maxprot: vm_prot_t;
    initprot: vm_prot_t;
    nsects: u32;
    flags: Segment_Flags;
}

Section_64 :: struct {
    sectname: [16] u8;
    segname: [16] u8;
    addr: u64;
    size: u64;
    offset: u32;
    align: u32;
    reloff: u32;
    nreloc: u32;
    flags: Section_Flags;
    reserved1: u32;
    reserved2: u32;
    reserved3: u32;
}

Symtab_Command :: struct {
    using base: Load_Command;
    symoff: u32;
    nsyms: u32;
    stroff: u32;
    strsize: u32;
}

Uuid_Command :: struct {
    using base: Load_Command;
    uuid: [16] u8;
}

Build_Version_Command :: struct {
    using base: Load_Command;
    platform: Mach_Platform;
    minos: u32;
    sdk: u32;
    ntools: u32;
}

Mach_Header_Flags :: enum_flags u32 {
    NOUNDEFS :: 0x1;
    INCRLINK :: 0x2;
    DYLDLINK :: 0x4;
    BINDATLOAD :: 0x8;
    PREBOUND :: 0x10;
    SPLIT_SEGS :: 0x20;
    LAZY_INIT :: 0x40;
    TWOLEVEL :: 0x80;
    FORCE_FLAT :: 0x100;
    NOMULTIDEFS :: 0x200;
    NOFIXPREBINDING :: 0x400;
    PREBINDABLE :: 0x800;
    ALLMODSBOUND :: 0x1000;
    SUBSECTIONS_VIA_SYMBOLS :: 0x2000;
    CANONICAL :: 0x4000;
    WEAK_DEFINES :: 0x8000;
    BINDS_TO_WEAK :: 0x10000;
    ALLOW_STACK_EXECUTION :: 0x20000;
    ROOT_SAFE :: 0x40000;
    SETUID_SAFE :: 0x80000;
    NO_REEXPORTED_DYLIBS :: 0x100000;
    PIE :: 0x200000;
    DEAD_STRIPPABLE_DYLIB :: 0x400000;
    HAS_TLV_DESCRIPTORS :: 0x800000;
    NO_HEAP_EXECUTION :: 0x1000000;
    NLIST_OUTOFSYNC_WITH_DYLDINFO :: 0x04000000;
    SIM_SUPPORT :: 0x08000000;
    DYLIB_IN_CACHE :: 0x80000000;
}

Mach_Platform :: enum u32 {
    UNKNOWN :: 0;
    ANY :: 0xFFFFFFFF;
    MACOS :: 1;
    IOS :: 2;
    TVOS :: 3;
    WATCHOS :: 4;
    BRIDGEOS :: 5;
    MACCATALYST :: 6;
    IOSSIMULATOR :: 7;
    TVOSSIMULATOR :: 8;
    WATCHOSSIMULATOR :: 9;
    DRIVERKIT :: 10;
    VISIONOS :: 11;
    VISIONOSSIMULATOR :: 12;
    FIRMWARE :: 13;
    SEPOS :: 14;
    MACOS_EXCLAVECORE :: 15;
    MACOS_EXCLAVEKIT :: 16;
    IOS_EXCLAVECORE :: 17;
    IOS_EXCLAVEKIT :: 18;
    TVOS_EXCLAVECORE :: 19;
    TVOS_EXCLAVEKIT :: 20;
    WATCHOS_EXCLAVECORE :: 21;
    WATCHOS_EXCLAVEKIT :: 22;
    VISIONOS_EXCLAVECORE :: 23;
    VISIONOS_EXCLAVEKIT :: 24;
}

Ms_Dos_Header :: struct {
    magic: u16;
    cblp: u16;
    cp: u16;
    crlc: u16;
    cparhdr: u16;
    minalloc: u16;
    maxalloc: u16;
    ss: u16;
    sp: u16;
    csum: u16;
    ip: u16;
    cs: u16;
    lfarlc: u16;
    ovno: u16;
    res: [4] u16;
    oemid: u16;
    oeminfo: u16;
    res2: [10] u16;
    lfanew: u32;
}

Token_Type :: enum s16 {
    IDENT :: 256;
    STRING :: 258;
    TRIPLE_MINUS;
    TRIPLE_DOT;
    END_OF_INPUT;
    ERROR;
}

Value_Flags :: enum_flags u32 {
    HERE_STRING :: 0x1;
    NUMBER :: 0x2;
    HEX :: 0x4;
    BINARY :: 0x8;
    FLOAT :: 0x10;
    REQUIRES_FLOAT64 :: 0x10_0000;
    DEFAULTS_TO_FLOAT64 :: 0x20_0000;
    REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS :: 0x40_0000;
    OVERFLOWED :: 0x1000_0000;
}

Token :: struct {
    type: Token_Type = .ERROR;
    l0, c0: s32;
    l1, c1: s32 = -1;
    string_value: string;
    value_flags: Value_Flags;
    preceeding_whitespace: s32;
}

Lexer :: struct {
    current_line_number: s32;
    current_external_file_error_report_line_number: s32;
    current_character_index: s32;
    tokens: [MAX_CONCURRENT_TOKENS] Token;
    token_array_cursor: s32;
    num_incoming_tokens: s32;
    input: string;
    input_cursor: s64;
    should_free_input := false;
    total_lines_processed: s64;
    nonblank_lines_processed: s64;
    preceeding_whitespace: s32;
    previous_token_line_number: s64;
    eof_token: Token;
    user_data: *void;
    scratch_buffer: [MAX_IDENTIFIER_LENGTH] u8;
    reported_error := false;
}

Hex_Digit_Type :: enum u8 {
    ASCII_8_BIT :: 0;
    UNICODE_16 :: 1;
    UNICODE_32 :: 2;
}

Text_Based_Dylib :: struct {
    tbd_version: int;
    targets: [] string;
    install_name: string;
    exports: [] Target_Exports;
}

Target_Exports :: struct {
    targets: [] string;
    symbols: [] string;
    re_exports: [] string;
}

Elf_Program_Header :: struct(Size_Type: Type) {
    type: Elf_Segment_Type;
    #if Size_Type == u64 {
        flags: Elf_Segment_Flags;
    }
    offset: Size_Type;
    vaddr: Size_Type;
    paddr: Size_Type;
    filesz: Size_Type;
    memsz: Size_Type;
    #if Size_Type == u32 {
        flags: Elf_Segment_Flags;
    }
    align: Size_Type;
}
