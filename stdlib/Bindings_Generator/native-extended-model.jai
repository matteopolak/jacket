// Additional native ABI types; no native implementation bodies are reproduced.

#if OS == .WINDOWS {
    // Availability of an entity: available, deprecated, not available or not accessible.
    CXAvailabilityKind :: enum s32 {
        Available :: 0;
        Deprecated :: 1;
        NotAvailable :: 2;
        NotAccessible :: 3;
    }
} else {
    // Availability of an entity: available, deprecated, not available or not accessible.
    CXAvailabilityKind :: enum u32 {
        Available :: 0;
        Deprecated :: 1;
        NotAvailable :: 2;
        NotAccessible :: 3;
    }
}

#if OS == .WINDOWS {
    // Kinds of binary operator expression that a cursor can have.
    CXBinaryOperatorKind :: enum s32 {
        Invalid :: 0;
        PtrMemD :: 1;
        PtrMemI :: 2;
        Mul :: 3;
        Div :: 4;
        Rem :: 5;
        Add :: 6;
        Sub :: 7;
        Shl :: 8;
        Shr :: 9;
        Cmp :: 10;
        LT :: 11;
        GT :: 12;
        LE :: 13;
        GE :: 14;
        EQ :: 15;
        NE :: 16;
        And :: 17;
        Xor :: 18;
        Or :: 19;
        LAnd :: 20;
        LOr :: 21;
        Assign :: 22;
        MulAssign :: 23;
        DivAssign :: 24;
        RemAssign :: 25;
        AddAssign :: 26;
        SubAssign :: 27;
        ShlAssign :: 28;
        ShrAssign :: 29;
        AndAssign :: 30;
        XorAssign :: 31;
        OrAssign :: 32;
        Comma :: 33;
        Last :: 33;
        LAST :: 33; // the name [CX_BinaryOperatorKind] gives it
    }
} else {
    // Kinds of binary operator expression that a cursor can have.
    CXBinaryOperatorKind :: enum u32 {
        Invalid :: 0;
        PtrMemD :: 1;
        PtrMemI :: 2;
        Mul :: 3;
        Div :: 4;
        Rem :: 5;
        Add :: 6;
        Sub :: 7;
        Shl :: 8;
        Shr :: 9;
        Cmp :: 10;
        LT :: 11;
        GT :: 12;
        LE :: 13;
        GE :: 14;
        EQ :: 15;
        NE :: 16;
        And :: 17;
        Xor :: 18;
        Or :: 19;
        LAnd :: 20;
        LOr :: 21;
        Assign :: 22;
        MulAssign :: 23;
        DivAssign :: 24;
        RemAssign :: 25;
        AddAssign :: 26;
        SubAssign :: 27;
        ShlAssign :: 28;
        ShrAssign :: 29;
        AndAssign :: 30;
        XorAssign :: 31;
        OrAssign :: 32;
        Comma :: 33;
        Last :: 33;
        LAST :: 33; // the name [CX_BinaryOperatorKind] gives it
    }
}

#if OS == .WINDOWS {
    // Calling convention of a function type.
    CXCallingConv :: enum s32 {
        Default :: 0;
        C :: 1;
        X86StdCall :: 2;
        X86FastCall :: 3;
        X86ThisCall :: 4;
        X86Pascal :: 5;
        AAPCS :: 6;
        AAPCS_VFP :: 7;
        X86RegCall :: 8;
        IntelOclBicc :: 9;
        Win64 :: 10;
        X86_64Win64 :: 10;
        X86_64SysV :: 11;
        X86VectorCall :: 12;
        Swift :: 13;
        PreserveMost :: 14;
        PreserveAll :: 15;
        AArch64VectorCall :: 16;
        SwiftAsync :: 17;
        AArch64SVEPCS :: 18;
        M68kRTD :: 19;
        PreserveNone :: 20;
        RISCVVectorCall :: 21;
        RISCVVLSCall_32 :: 22;
        RISCVVLSCall_64 :: 23;
        RISCVVLSCall_128 :: 24;
        RISCVVLSCall_256 :: 25;
        RISCVVLSCall_512 :: 26;
        RISCVVLSCall_1024 :: 27;
        RISCVVLSCall_2048 :: 28;
        RISCVVLSCall_4096 :: 29;
        RISCVVLSCall_8192 :: 30;
        RISCVVLSCall_16384 :: 31;
        RISCVVLSCall_32768 :: 32;
        RISCVVLSCall_65536 :: 33;
        Invalid :: 100;
        Unexposed :: 200;
    }
} else {
    // Calling convention of a function type.
    CXCallingConv :: enum u32 {
        Default :: 0;
        C :: 1;
        X86StdCall :: 2;
        X86FastCall :: 3;
        X86ThisCall :: 4;
        X86Pascal :: 5;
        AAPCS :: 6;
        AAPCS_VFP :: 7;
        X86RegCall :: 8;
        IntelOclBicc :: 9;
        Win64 :: 10;
        X86_64Win64 :: 10;
        X86_64SysV :: 11;
        X86VectorCall :: 12;
        Swift :: 13;
        PreserveMost :: 14;
        PreserveAll :: 15;
        AArch64VectorCall :: 16;
        SwiftAsync :: 17;
        AArch64SVEPCS :: 18;
        M68kRTD :: 19;
        PreserveNone :: 20;
        RISCVVectorCall :: 21;
        RISCVVLSCall_32 :: 22;
        RISCVVLSCall_64 :: 23;
        RISCVVLSCall_128 :: 24;
        RISCVVLSCall_256 :: 25;
        RISCVVLSCall_512 :: 26;
        RISCVVLSCall_1024 :: 27;
        RISCVVLSCall_2048 :: 28;
        RISCVVLSCall_4096 :: 29;
        RISCVVLSCall_8192 :: 30;
        RISCVVLSCall_16384 :: 31;
        RISCVVLSCall_32768 :: 32;
        RISCVVLSCall_65536 :: 33;
        Invalid :: 100;
        Unexposed :: 200;
    }
}

#if OS == .WINDOWS {
    // Three-way index option: default, enabled or disabled.
    CXChoice :: enum s32 {
        Default :: 0;
        Enabled :: 1;
        Disabled :: 2;
    }
} else {
    // Three-way index option: default, enabled or disabled.
    CXChoice :: enum u32 {
        Default :: 0;
        Enabled :: 1;
        Disabled :: 2;
    }
}

// Result of a code-completion request: `NumResults` completions at `Results`.
CXCodeCompleteResults :: struct {
    Results: *CXCompletionResult;
    NumResults: u32;
}

#if OS == .WINDOWS {
    // Flags selecting what a code-completion request includes, such as macros, code patterns and brief
    // comments.
    CXCodeComplete_Flags :: enum_flags s32 {
        IncludeMacros :: 0x1;
        IncludeCodePatterns :: 0x2;
        IncludeBriefComments :: 0x4;
        SkipPreamble :: 0x8;
        IncludeCompletionsWithFixIts :: 0x10;
    }
} else {
    // Flags selecting what a code-completion request includes, such as macros, code patterns and brief
    // comments.
    CXCodeComplete_Flags :: enum_flags u32 {
        IncludeMacros :: 0x1;
        IncludeCodePatterns :: 0x2;
        IncludeBriefComments :: 0x4;
        SkipPreamble :: 0x8;
        IncludeCompletionsWithFixIts :: 0x10;
    }
}

#if OS == .WINDOWS {
    // Kind of one chunk of a completion string, such as typed text, a placeholder or punctuation.
    CXCompletionChunkKind :: enum s32 {
        Optional :: 0;
        TypedText :: 1;
        Text :: 2;
        Placeholder :: 3;
        Informative :: 4;
        CurrentParameter :: 5;
        LeftParen :: 6;
        RightParen :: 7;
        LeftBracket :: 8;
        RightBracket :: 9;
        LeftBrace :: 10;
        RightBrace :: 11;
        LeftAngle :: 12;
        RightAngle :: 13;
        Comma :: 14;
        ResultType :: 15;
        Colon :: 16;
        SemiColon :: 17;
        Equal :: 18;
        HorizontalSpace :: 19;
        VerticalSpace :: 20;
    }
} else {
    // Kind of one chunk of a completion string, such as typed text, a placeholder or punctuation.
    CXCompletionChunkKind :: enum u32 {
        Optional :: 0;
        TypedText :: 1;
        Text :: 2;
        Placeholder :: 3;
        Informative :: 4;
        CurrentParameter :: 5;
        LeftParen :: 6;
        RightParen :: 7;
        LeftBracket :: 8;
        RightBracket :: 9;
        LeftBrace :: 10;
        RightBrace :: 11;
        LeftAngle :: 12;
        RightAngle :: 13;
        Comma :: 14;
        ResultType :: 15;
        Colon :: 16;
        SemiColon :: 17;
        Equal :: 18;
        HorizontalSpace :: 19;
        VerticalSpace :: 20;
    }
}

#if OS == .WINDOWS {
    // Bit set of the syntactic contexts in which a completion is offered.
    CXCompletionContext :: enum_flags s32 {
        Unexposed :: 0x0;
        AnyType :: 0x1;
        AnyValue :: 0x2;
        ObjCObjectValue :: 0x4;
        ObjCSelectorValue :: 0x8;
        CXXClassTypeValue :: 0x10;
        DotMemberAccess :: 0x20;
        ArrowMemberAccess :: 0x40;
        ObjCPropertyAccess :: 0x80;
        EnumTag :: 0x100;
        UnionTag :: 0x200;
        StructTag :: 0x400;
        ClassTag :: 0x800;
        Namespace :: 0x1000;
        NestedNameSpecifier :: 0x2000;
        ObjCInterface :: 0x4000;
        ObjCProtocol :: 0x8000;
        ObjCCategory :: 0x10000;
        ObjCInstanceMessage :: 0x20000;
        ObjCClassMessage :: 0x40000;
        ObjCSelectorName :: 0x80000;
        MacroName :: 0x100000;
        NaturalLanguage :: 0x200000;
        IncludedFile :: 0x400000;
        Unknown :: 0x7fffff;
    }
} else {
    // Bit set of the syntactic contexts in which a completion is offered.
    CXCompletionContext :: enum_flags u32 {
        Unexposed :: 0x0;
        AnyType :: 0x1;
        AnyValue :: 0x2;
        ObjCObjectValue :: 0x4;
        ObjCSelectorValue :: 0x8;
        CXXClassTypeValue :: 0x10;
        DotMemberAccess :: 0x20;
        ArrowMemberAccess :: 0x40;
        ObjCPropertyAccess :: 0x80;
        EnumTag :: 0x100;
        UnionTag :: 0x200;
        StructTag :: 0x400;
        ClassTag :: 0x800;
        Namespace :: 0x1000;
        NestedNameSpecifier :: 0x2000;
        ObjCInterface :: 0x4000;
        ObjCProtocol :: 0x8000;
        ObjCCategory :: 0x10000;
        ObjCInstanceMessage :: 0x20000;
        ObjCClassMessage :: 0x40000;
        ObjCSelectorName :: 0x80000;
        MacroName :: 0x100000;
        NaturalLanguage :: 0x200000;
        IncludedFile :: 0x400000;
        Unknown :: 0x7fffff;
    }
}

// One code completion: the kind of the completed entity and its completion string.
CXCompletionResult :: struct {
    CursorKind: CXCursorKind;
    CompletionString: CXCompletionString;
}

CXCompletionString :: *void;

// Callback invoked for each cursor found within a source range; `_context` is passed back to `visit`.
CXCursorAndRangeVisitor :: struct {
    _context: *void;
    visit: #type (_context: *void, unknown0: CXCursor, unknown1: CXSourceRange) -> CXVisitorResult #c_call;
}

CXCursorSet :: *CXCursorSetImpl;

// Opaque target of [CXCursorSet]; has no fields.
CXCursorSetImpl :: struct {}

#if OS == .WINDOWS {
    // Kind of exception specification on a C++ function declaration or type.
    CXCursor_ExceptionSpecificationKind :: enum s32 {
        None :: 0;
        DynamicNone :: 1;
        Dynamic :: 2;
        MSAny :: 3;
        BasicNoexcept :: 4;
        ComputedNoexcept :: 5;
        Unevaluated :: 6;
        Uninstantiated :: 7;
        Unparsed :: 8;
        NoThrow :: 9;
    }
} else {
    // Kind of exception specification on a C++ function declaration or type.
    CXCursor_ExceptionSpecificationKind :: enum u32 {
        None :: 0;
        DynamicNone :: 1;
        Dynamic :: 2;
        MSAny :: 3;
        BasicNoexcept :: 4;
        ComputedNoexcept :: 5;
        Unevaluated :: 6;
        Uninstantiated :: 7;
        Unparsed :: 8;
        NoThrow :: 9;
    }
}

#if OS == .WINDOWS {
    // Flags controlling how a diagnostic is formatted as text.
    CXDiagnosticDisplayOptions :: enum s32 {
        DisplaySourceLocation :: 1;
        DisplayColumn :: 2;
        DisplaySourceRanges :: 4;
        DisplayOption :: 8;
        DisplayCategoryId :: 16;
        DisplayCategoryName :: 32;
    }
} else {
    // Flags controlling how a diagnostic is formatted as text.
    CXDiagnosticDisplayOptions :: enum u32 {
        DisplaySourceLocation :: 1;
        DisplayColumn :: 2;
        DisplaySourceRanges :: 4;
        DisplayOption :: 8;
        DisplayCategoryId :: 16;
        DisplayCategoryName :: 32;
    }
}

CXDiagnosticSet :: *void;

CXFieldVisitor :: #type (C: CXCursor, client_data: CXClientData) -> CXVisitorResult #c_call;

// Identifier that is unique for a file, stored as three 64-bit words.
CXFileUniqueID :: struct {
    data: [3] u64;
}

#if OS == .WINDOWS {
    // Flags for index-wide options: background thread priority for indexing, editing or both.
    CXGlobalOptFlags :: enum s32 {
        None :: 0;
        ThreadBackgroundPriorityForIndexing :: 1;
        ThreadBackgroundPriorityForEditing :: 2;
        ThreadBackgroundPriorityForAll :: 3;
    }
} else {
    // Flags for index-wide options: background thread priority for indexing, editing or both.
    CXGlobalOptFlags :: enum u32 {
        None :: 0;
        ThreadBackgroundPriorityForIndexing :: 1;
        ThreadBackgroundPriorityForEditing :: 2;
        ThreadBackgroundPriorityForAll :: 3;
    }
}

// Attribute attached to a declaration, reported by the indexing API.
CXIdxAttrInfo :: struct {
    kind: CXIdxAttrKind;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

#if OS == .WINDOWS {
    // Kind of attribute that the indexing API reports.
    CXIdxAttrKind :: enum s32 {
        Unexposed :: 0;
        IBAction :: 1;
        IBOutlet :: 2;
        IBOutletCollection :: 3;
    }
} else {
    // Kind of attribute that the indexing API reports.
    CXIdxAttrKind :: enum u32 {
        Unexposed :: 0;
        IBAction :: 1;
        IBOutlet :: 2;
        IBOutletCollection :: 3;
    }
}

// Base class of a class or interface, reported by the indexing API.
CXIdxBaseClassInfo :: struct {
    base: *CXIdxEntityInfo;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

// Indexing information for a C++ class declaration: the declaration and its `numBases` base classes.
CXIdxCXXClassDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    bases: **CXIdxBaseClassInfo;
    numBases: u32;
}

CXIdxClientASTFile :: *void;

CXIdxClientContainer :: *void;

CXIdxClientEntity :: *void;

CXIdxClientFile :: *void;

// Container of a declaration, reported by the indexing API.
CXIdxContainerInfo :: struct {
    cursor: CXCursor;
}

// Indexing information for one declaration, reported to the `indexDeclaration` callback.
CXIdxDeclInfo :: struct {
    entityInfo: *CXIdxEntityInfo;
    cursor: CXCursor;
    loc: CXIdxLoc;
    semanticContainer: *CXIdxContainerInfo;
    lexicalContainer: *CXIdxContainerInfo;
    isRedeclaration: s32;
    isDefinition: s32;
    isContainer: s32;
    declAsContainer: *CXIdxContainerInfo;
    isImplicit: s32;
    attributes: **CXIdxAttrInfo;
    numAttributes: u32;
    flags: u32;
}

#if OS == .WINDOWS {
    // Flags describing a declaration reported by the indexing API.
    CXIdxDeclInfoFlags :: enum s32 {
        CXIdxDeclFlag_Skipped :: 1;
    }
} else {
    // Flags describing a declaration reported by the indexing API.
    CXIdxDeclInfoFlags :: enum u32 {
        CXIdxDeclFlag_Skipped :: 1;
    }
}

#if OS == .WINDOWS {
    // Template relationship of an indexed entity: not a template, a template, or a partial or full
    // specialization.
    CXIdxEntityCXXTemplateKind :: enum s32 {
        NonTemplate :: 0;
        Template :: 1;
        TemplatePartialSpecialization :: 2;
        TemplateSpecialization :: 3;
    }
} else {
    // Template relationship of an indexed entity: not a template, a template, or a partial or full
    // specialization.
    CXIdxEntityCXXTemplateKind :: enum u32 {
        NonTemplate :: 0;
        Template :: 1;
        TemplatePartialSpecialization :: 2;
        TemplateSpecialization :: 3;
    }
}

// Entity named by a declaration or reference, reported by the indexing API.
CXIdxEntityInfo :: struct {
    kind: CXIdxEntityKind;
    templateKind: CXIdxEntityCXXTemplateKind;
    lang: CXIdxEntityLanguage;
    name: *u8;
    USR: *u8;
    cursor: CXCursor;
    attributes: **CXIdxAttrInfo;
    numAttributes: u32;
}

#if OS == .WINDOWS {
    // Kind of entity that the indexing API reports, such as a function, struct or variable.
    CXIdxEntityKind :: enum s32 {
        Unexposed :: 0;
        Typedef :: 1;
        Function :: 2;
        Variable :: 3;
        Field :: 4;
        EnumConstant :: 5;
        ObjCClass :: 6;
        ObjCProtocol :: 7;
        ObjCCategory :: 8;
        ObjCInstanceMethod :: 9;
        ObjCClassMethod :: 10;
        ObjCProperty :: 11;
        ObjCIvar :: 12;
        Enum :: 13;
        Struct :: 14;
        Union :: 15;
        CXXClass :: 16;
        CXXNamespace :: 17;
        CXXNamespaceAlias :: 18;
        CXXStaticVariable :: 19;
        CXXStaticMethod :: 20;
        CXXInstanceMethod :: 21;
        CXXConstructor :: 22;
        CXXDestructor :: 23;
        CXXConversionFunction :: 24;
        CXXTypeAlias :: 25;
        CXXInterface :: 26;
        CXXConcept :: 27;
    }
} else {
    // Kind of entity that the indexing API reports, such as a function, struct or variable.
    CXIdxEntityKind :: enum u32 {
        Unexposed :: 0;
        Typedef :: 1;
        Function :: 2;
        Variable :: 3;
        Field :: 4;
        EnumConstant :: 5;
        ObjCClass :: 6;
        ObjCProtocol :: 7;
        ObjCCategory :: 8;
        ObjCInstanceMethod :: 9;
        ObjCClassMethod :: 10;
        ObjCProperty :: 11;
        ObjCIvar :: 12;
        Enum :: 13;
        Struct :: 14;
        Union :: 15;
        CXXClass :: 16;
        CXXNamespace :: 17;
        CXXNamespaceAlias :: 18;
        CXXStaticVariable :: 19;
        CXXStaticMethod :: 20;
        CXXInstanceMethod :: 21;
        CXXConstructor :: 22;
        CXXDestructor :: 23;
        CXXConversionFunction :: 24;
        CXXTypeAlias :: 25;
        CXXInterface :: 26;
        CXXConcept :: 27;
    }
}

#if OS == .WINDOWS {
    // Source language of an entity that the indexing API reports.
    CXIdxEntityLanguage :: enum s32 {
        None :: 0;
        C :: 1;
        ObjC :: 2;
        CXX :: 3;
        Swift :: 4;
    }
} else {
    // Source language of an entity that the indexing API reports.
    CXIdxEntityLanguage :: enum u32 {
        None :: 0;
        C :: 1;
        ObjC :: 2;
        CXX :: 3;
        Swift :: 4;
    }
}

// One reference to an entity, reported to the `indexEntityReference` callback.
CXIdxEntityRefInfo :: struct {
    kind: CXIdxEntityRefKind;
    cursor: CXCursor;
    loc: CXIdxLoc;
    referencedEntity: *CXIdxEntityInfo;
    parentEntity: *CXIdxEntityInfo;
    container: *CXIdxContainerInfo;
    role: CXSymbolRole;
}

#if OS == .WINDOWS {
    // Whether a reference reported by the indexing API is direct or implicit.
    CXIdxEntityRefKind :: enum s32 {
        Direct :: 1;
        Implicit :: 2;
    }
} else {
    // Whether a reference reported by the indexing API is direct or implicit.
    CXIdxEntityRefKind :: enum u32 {
        Direct :: 1;
        Implicit :: 2;
    }
}

// Extra information for an `IBOutletCollection` attribute.
CXIdxIBOutletCollectionAttrInfo :: struct {
    attrInfo: *CXIdxAttrInfo;
    objcClass: *CXIdxEntityInfo;
    classCursor: CXCursor;
    classLoc: CXIdxLoc;
}

// Information about an AST file or module imported by the indexed translation unit.
CXIdxImportedASTFileInfo :: struct {
    file: CXFile;
    module: CXModule;
    loc: CXIdxLoc;
    isImplicit: s32;
}

// Information about a file that the preprocessor included.
CXIdxIncludedFileInfo :: struct {
    hashLoc: CXIdxLoc;
    filename: *u8;
    file: CXFile;
    isImport: s32;
    isAngled: s32;
    isModuleImport: s32;
}

// Source location used by the libclang indexing API.
CXIdxLoc :: struct {
    ptr_data: [2] *void;
    int_data: u32;
}

// Indexing information for an Objective-C category declaration.
CXIdxObjCCategoryDeclInfo :: struct {
    containerInfo: *CXIdxObjCContainerDeclInfo;
    objcClass: *CXIdxEntityInfo;
    classCursor: CXCursor;
    classLoc: CXIdxLoc;
    protocols: *CXIdxObjCProtocolRefListInfo;
}

// Indexing information for an Objective-C container declaration, with the container's kind.
CXIdxObjCContainerDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    kind: CXIdxObjCContainerKind;
}

#if OS == .WINDOWS {
    // Kind of Objective-C container declaration: forward reference, interface or implementation.
    CXIdxObjCContainerKind :: enum s32 {
        ForwardRef :: 0;
        Interface :: 1;
        Implementation :: 2;
    }
} else {
    // Kind of Objective-C container declaration: forward reference, interface or implementation.
    CXIdxObjCContainerKind :: enum u32 {
        ForwardRef :: 0;
        Interface :: 1;
        Implementation :: 2;
    }
}

// Indexing information for an Objective-C interface declaration.
CXIdxObjCInterfaceDeclInfo :: struct {
    containerInfo: *CXIdxObjCContainerDeclInfo;
    superInfo: *CXIdxBaseClassInfo;
    protocols: *CXIdxObjCProtocolRefListInfo;
}

// Indexing information for an Objective-C property declaration, with its getter and setter entities.
CXIdxObjCPropertyDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    getter: *CXIdxEntityInfo;
    setter: *CXIdxEntityInfo;
}

// One Objective-C protocol referenced by a declaration.
CXIdxObjCProtocolRefInfo :: struct {
    protocol: *CXIdxEntityInfo;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

// List of `numProtocols` Objective-C protocols referenced by a declaration.
CXIdxObjCProtocolRefListInfo :: struct {
    protocols: **CXIdxObjCProtocolRefInfo;
    numProtocols: u32;
}

CXInclusionVisitor :: #type (included_file: CXFile, inclusion_stack: *CXSourceLocation, include_len: u32, client_data: CXClientData) -> void #c_call;

CXIndexAction :: *void;

#if OS == .WINDOWS {
    // Flags controlling what the indexing API reports and how it runs.
    CXIndexOptFlags :: enum s32 {
        None :: 0;
        SuppressRedundantRefs :: 1;
        IndexFunctionLocalSymbols :: 2;
        IndexImplicitTemplateInstantiations :: 4;
        SuppressWarnings :: 8;
        SkipParsedBodiesInSession :: 16;
    }
} else {
    // Flags controlling what the indexing API reports and how it runs.
    CXIndexOptFlags :: enum u32 {
        None :: 0;
        SuppressRedundantRefs :: 1;
        IndexFunctionLocalSymbols :: 2;
        IndexImplicitTemplateInstantiations :: 4;
        SuppressWarnings :: 8;
        SkipParsedBodiesInSession :: 16;
    }
}

#if OS == .WINDOWS {
    // Source language of a cursor.
    CXLanguageKind :: enum s32 {
        Invalid :: 0;
        C :: 1;
        ObjC :: 2;
        CPlusPlus :: 3;
    }
} else {
    // Source language of a cursor.
    CXLanguageKind :: enum u32 {
        Invalid :: 0;
        C :: 1;
        ObjC :: 2;
        CPlusPlus :: 3;
    }
}

#if OS == .WINDOWS {
    // Linkage of an entity: none, internal, unique external or external.
    CXLinkageKind :: enum s32 {
        Invalid :: 0;
        NoLinkage :: 1;
        Internal :: 2;
        UniqueExternal :: 3;
        External :: 4;
    }
} else {
    // Linkage of an entity: none, internal, unique external or external.
    CXLinkageKind :: enum u32 {
        Invalid :: 0;
        NoLinkage :: 1;
        Internal :: 2;
        UniqueExternal :: 3;
        External :: 4;
    }
}

#if OS == .WINDOWS {
    // Error codes returned when loading serialized diagnostics.
    CXLoadDiag_Error :: enum s32 {
        None :: 0;
        Unknown :: 1;
        CannotLoad :: 2;
        InvalidFile :: 3;
    }
} else {
    // Error codes returned when loading serialized diagnostics.
    CXLoadDiag_Error :: enum u32 {
        None :: 0;
        Unknown :: 1;
        CannotLoad :: 2;
        InvalidFile :: 3;
    }
}

CXModule :: *void;

#if OS == .WINDOWS {
    // Flags selecting which parts of a name a range lookup covers: the qualifier, template arguments or a
    // single piece.
    CXNameRefFlags :: enum s32 {
        WantQualifier :: 1;
        WantTemplateArgs :: 2;
        WantSinglePiece :: 4;
    }
} else {
    // Flags selecting which parts of a name a range lookup covers: the qualifier, template arguments or a
    // single piece.
    CXNameRefFlags :: enum u32 {
        WantQualifier :: 1;
        WantTemplateArgs :: 2;
        WantSinglePiece :: 4;
    }
}

#if OS == .WINDOWS {
    // Bit set of Objective-C declaration qualifiers.
    CXObjCDeclQualifierKind :: enum s32 {
        None :: 0;
        In :: 1;
        Inout :: 2;
        Out :: 4;
        Bycopy :: 8;
        Byref :: 16;
        Oneway :: 32;
    }
} else {
    // Bit set of Objective-C declaration qualifiers.
    CXObjCDeclQualifierKind :: enum u32 {
        None :: 0;
        In :: 1;
        Inout :: 2;
        Out :: 4;
        Bycopy :: 8;
        Byref :: 16;
        Oneway :: 32;
    }
}

#if OS == .WINDOWS {
    // Bit set of Objective-C property attributes.
    CXObjCPropertyAttrKind :: enum s32 {
        noattr :: 0;
        readonly :: 1;
        getter :: 2;
        assign :: 4;
        readwrite :: 8;
        retain :: 16;
        copy :: 32;
        nonatomic :: 64;
        setter :: 128;
        atomic :: 256;
        weak :: 512;
        strong :: 1024;
        unsafe_unretained :: 2048;
        class :: 4096;
    }
} else {
    // Bit set of Objective-C property attributes.
    CXObjCPropertyAttrKind :: enum u32 {
        noattr :: 0;
        readonly :: 1;
        getter :: 2;
        assign :: 4;
        readwrite :: 8;
        retain :: 16;
        copy :: 32;
        nonatomic :: 64;
        setter :: 128;
        atomic :: 256;
        weak :: 512;
        strong :: 1024;
        unsafe_unretained :: 2048;
        class :: 4096;
    }
}

// Availability of an entity on one platform, with the versions where it was introduced, deprecated and
// obsoleted.
CXPlatformAvailability :: struct {
    Platform: CXString;
    Introduced: CXVersion;
    Deprecated: CXVersion;
    Obsoleted: CXVersion;
    Unavailable: s32;
    Message: CXString;
}

CXPrintingPolicy :: *void;

#if OS == .WINDOWS {
    // Property of a printing policy that controls how declarations are printed.
    CXPrintingPolicyProperty :: enum s32 {
        Indentation :: 0;
        SuppressSpecifiers :: 1;
        SuppressTagKeyword :: 2;
        IncludeTagDefinition :: 3;
        SuppressScope :: 4;
        SuppressUnwrittenScope :: 5;
        SuppressInitializers :: 6;
        ConstantArraySizeAsWritten :: 7;
        AnonymousTagLocations :: 8;
        SuppressStrongLifetime :: 9;
        SuppressLifetimeQualifiers :: 10;
        SuppressTemplateArgsInCXXConstructors :: 11;
        Bool :: 12;
        Restrict :: 13;
        Alignof :: 14;
        UnderscoreAlignof :: 15;
        UseVoidForZeroParams :: 16;
        TerseOutput :: 17;
        PolishForDeclaration :: 18;
        Half :: 19;
        MSWChar :: 20;
        IncludeNewlines :: 21;
        MSVCFormatting :: 22;
        ConstantsAsWritten :: 23;
        SuppressImplicitBase :: 24;
        FullyQualifiedName :: 25;
        LastProperty :: 25;
    }
} else {
    // Property of a printing policy that controls how declarations are printed.
    CXPrintingPolicyProperty :: enum u32 {
        Indentation :: 0;
        SuppressSpecifiers :: 1;
        SuppressTagKeyword :: 2;
        IncludeTagDefinition :: 3;
        SuppressScope :: 4;
        SuppressUnwrittenScope :: 5;
        SuppressInitializers :: 6;
        ConstantArraySizeAsWritten :: 7;
        AnonymousTagLocations :: 8;
        SuppressStrongLifetime :: 9;
        SuppressLifetimeQualifiers :: 10;
        SuppressTemplateArgsInCXXConstructors :: 11;
        Bool :: 12;
        Restrict :: 13;
        Alignof :: 14;
        UnderscoreAlignof :: 15;
        UseVoidForZeroParams :: 16;
        TerseOutput :: 17;
        PolishForDeclaration :: 18;
        Half :: 19;
        MSWChar :: 20;
        IncludeNewlines :: 21;
        MSVCFormatting :: 22;
        ConstantsAsWritten :: 23;
        SuppressImplicitBase :: 24;
        FullyQualifiedName :: 25;
        LastProperty :: 25;
    }
}

#if OS == .WINDOWS {
    // Reference qualifier of a C++ member function: none, lvalue (`&`) or rvalue (`&&`).
    CXRefQualifierKind :: enum s32 {
        None :: 0;
        LValue :: 1;
        RValue :: 2;
    }
} else {
    // Reference qualifier of a C++ member function: none, lvalue (`&`) or rvalue (`&&`).
    CXRefQualifierKind :: enum u32 {
        None :: 0;
        LValue :: 1;
        RValue :: 2;
    }
}

CXRemapping :: *void;

#if OS == .WINDOWS {
    // Flags for reparsing a translation unit; only the none value is defined.
    CXReparse_Flags :: enum_flags s32 {
        CXReparse_None :: 0x0;
    }
} else {
    // Flags for reparsing a translation unit; only the none value is defined.
    CXReparse_Flags :: enum_flags u32 {
        CXReparse_None :: 0x0;
    }
}

#if OS == .WINDOWS {
    // Result of a visit operation: success, invalid arguments or the visit was interrupted.
    CXResult :: enum s32 {
        Success :: 0;
        Invalid :: 1;
        VisitBreak :: 2;
    }
} else {
    // Result of a visit operation: success, invalid arguments or the visit was interrupted.
    CXResult :: enum u32 {
        Success :: 0;
        Invalid :: 1;
        VisitBreak :: 2;
    }
}

#if OS == .WINDOWS {
    // Error codes returned when saving a translation unit.
    CXSaveError :: enum s32 {
        None :: 0;
        Unknown :: 1;
        TranslationErrors :: 2;
        InvalidTU :: 3;
    }
} else {
    // Error codes returned when saving a translation unit.
    CXSaveError :: enum u32 {
        None :: 0;
        Unknown :: 1;
        TranslationErrors :: 2;
        InvalidTU :: 3;
    }
}

#if OS == .WINDOWS {
    // Flags for saving a translation unit; only the none value is defined.
    CXSaveTranslationUnit_Flags :: enum_flags s32 {
        CXSaveTranslationUnit_None :: 0x0;
    }
} else {
    // Flags for saving a translation unit; only the none value is defined.
    CXSaveTranslationUnit_Flags :: enum_flags u32 {
        CXSaveTranslationUnit_None :: 0x0;
    }
}

// Array of `count` source ranges at `ranges`.
CXSourceRangeList :: struct {
    count: u32;
    ranges: *CXSourceRange;
}

// Array of `Count` strings at `Strings`.
CXStringSet :: struct {
    Strings: *CXString;
    Count: u32;
}

#if OS == .WINDOWS {
    // Bit set of the roles a symbol occurrence plays, such as declaration, definition, reference or call.
    CXSymbolRole :: enum s32 {
        None :: 0;
        Declaration :: 1;
        Definition :: 2;
        Reference :: 4;
        Read :: 8;
        Write :: 16;
        Call :: 32;
        Dynamic :: 64;
        AddressOf :: 128;
        Implicit :: 256;
    }
} else {
    // Bit set of the roles a symbol occurrence plays, such as declaration, definition, reference or call.
    CXSymbolRole :: enum u32 {
        None :: 0;
        Declaration :: 1;
        Definition :: 2;
        Reference :: 4;
        Read :: 8;
        Write :: 16;
        Call :: 32;
        Dynamic :: 64;
        AddressOf :: 128;
        Implicit :: 256;
    }
}

#if OS == .WINDOWS {
    // Thread-local storage kind of a variable: none, dynamic or static.
    CXTLSKind :: enum s32 {
        None :: 0;
        Dynamic :: 1;
        Static :: 2;
    }
} else {
    // Thread-local storage kind of a variable: none, dynamic or static.
    CXTLSKind :: enum u32 {
        None :: 0;
        Dynamic :: 1;
        Static :: 2;
    }
}

// Memory used by a translation unit: `numEntries` entries at `entries`.
CXTUResourceUsage :: struct {
    data: *void;
    numEntries: u32;
    entries: *CXTUResourceUsageEntry;
}

#if OS == .WINDOWS {
    // One entry of translation-unit resource usage: a kind and its amount.
    CXTUResourceUsageEntry :: struct {
        kind: CXTUResourceUsageKind;
        amount: u32;
    }
} else {
    // One entry of translation-unit resource usage: a kind and its amount.
    CXTUResourceUsageEntry :: struct {
        kind: CXTUResourceUsageKind;
        amount: u64;
    }
}

#if OS == .WINDOWS {
    // Category of memory that a translation-unit resource-usage entry measures.
    CXTUResourceUsageKind :: enum s32 {
        AST :: 1;
        Identifiers :: 2;
        Selectors :: 3;
        GlobalCompletionResults :: 4;
        SourceManagerContentCache :: 5;
        AST_SideTables :: 6;
        SourceManager_Membuffer_Malloc :: 7;
        SourceManager_Membuffer_MMap :: 8;
        ExternalASTSource_Membuffer_Malloc :: 9;
        ExternalASTSource_Membuffer_MMap :: 10;
        Preprocessor :: 11;
        PreprocessingRecord :: 12;
        SourceManager_DataStructures :: 13;
        Preprocessor_HeaderSearch :: 14;
        MEMORY_IN_BYTES_BEGIN :: 1;
        MEMORY_IN_BYTES_END :: 14;
        First :: 1;
        Last :: 14;
    }
} else {
    // Category of memory that a translation-unit resource-usage entry measures.
    CXTUResourceUsageKind :: enum u32 {
        AST :: 1;
        Identifiers :: 2;
        Selectors :: 3;
        GlobalCompletionResults :: 4;
        SourceManagerContentCache :: 5;
        AST_SideTables :: 6;
        SourceManager_Membuffer_Malloc :: 7;
        SourceManager_Membuffer_MMap :: 8;
        ExternalASTSource_Membuffer_Malloc :: 9;
        ExternalASTSource_Membuffer_MMap :: 10;
        Preprocessor :: 11;
        PreprocessingRecord :: 12;
        SourceManager_DataStructures :: 13;
        Preprocessor_HeaderSearch :: 14;
        MEMORY_IN_BYTES_BEGIN :: 1;
        MEMORY_IN_BYTES_END :: 14;
        First :: 1;
        Last :: 14;
    }
}

CXTargetInfo :: *CXTargetInfoImpl;

// Opaque target of [CXTargetInfo]; has no fields.
CXTargetInfoImpl :: struct {}

#if OS == .WINDOWS {
    // Kind of a template argument, such as a type, declaration, integral value, template, pack or expression.
    CXTemplateArgumentKind :: enum s32 {
        Null :: 0;
        Type :: 1;
        Declaration :: 2;
        NullPtr :: 3;
        Integral :: 4;
        Template :: 5;
        TemplateExpansion :: 6;
        Expression :: 7;
        Pack :: 8;
        Invalid :: 9;
    }
} else {
    // Kind of a template argument, such as a type, declaration, integral value, template, pack or expression.
    CXTemplateArgumentKind :: enum u32 {
        Null :: 0;
        Type :: 1;
        Declaration :: 2;
        NullPtr :: 3;
        Integral :: 4;
        Template :: 5;
        TemplateExpansion :: 6;
        Expression :: 7;
        Pack :: 8;
        Invalid :: 9;
    }
}

#if OS == .WINDOWS {
    // Flags controlling how a translation unit is parsed.
    CXTranslationUnit_Flags :: enum_flags s32 {
        None :: 0x0;
        DetailedPreprocessingRecord :: 0x1;
        Incomplete :: 0x2;
        PrecompiledPreamble :: 0x4;
        CacheCompletionResults :: 0x8;
        ForSerialization :: 0x10;
        CXXChainedPCH :: 0x20;
        SkipFunctionBodies :: 0x40;
        IncludeBriefCommentsInCodeCompletion :: 0x80;
        CreatePreambleOnFirstParse :: 0x100;
        KeepGoing :: 0x200;
        SingleFileParse :: 0x400;
        LimitSkipFunctionBodiesToPreamble :: 0x800;
        IncludeAttributedTypes :: 0x1000;
        VisitImplicitAttributes :: 0x2000;
        IgnoreNonErrorsFromIncludedFiles :: 0x4000;
        RetainExcludedConditionalBlocks :: 0x8000;
    }
} else {
    // Flags controlling how a translation unit is parsed.
    CXTranslationUnit_Flags :: enum_flags u32 {
        None :: 0x0;
        DetailedPreprocessingRecord :: 0x1;
        Incomplete :: 0x2;
        PrecompiledPreamble :: 0x4;
        CacheCompletionResults :: 0x8;
        ForSerialization :: 0x10;
        CXXChainedPCH :: 0x20;
        SkipFunctionBodies :: 0x40;
        IncludeBriefCommentsInCodeCompletion :: 0x80;
        CreatePreambleOnFirstParse :: 0x100;
        KeepGoing :: 0x200;
        SingleFileParse :: 0x400;
        LimitSkipFunctionBodiesToPreamble :: 0x800;
        IncludeAttributedTypes :: 0x1000;
        VisitImplicitAttributes :: 0x2000;
        IgnoreNonErrorsFromIncludedFiles :: 0x4000;
        RetainExcludedConditionalBlocks :: 0x8000;
    }
}

#if OS == .WINDOWS {
    // Nullability of a pointer type: non-null, nullable, unspecified, invalid or nullable result.
    CXTypeNullabilityKind :: enum s32 {
        NonNull :: 0;
        Nullable :: 1;
        Unspecified :: 2;
        Invalid :: 3;
        NullableResult :: 4;
    }
} else {
    // Nullability of a pointer type: non-null, nullable, unspecified, invalid or nullable result.
    CXTypeNullabilityKind :: enum u32 {
        NonNull :: 0;
        Nullable :: 1;
        Unspecified :: 2;
        Invalid :: 3;
        NullableResult :: 4;
    }
}

#if OS == .WINDOWS {
    // Kinds of unary operator expression that a cursor can have.
    CXUnaryOperatorKind :: enum s32 {
        Invalid :: 0;
        PostInc :: 1;
        PostDec :: 2;
        PreInc :: 3;
        PreDec :: 4;
        AddrOf :: 5;
        Deref :: 6;
        Plus :: 7;
        Minus :: 8;
        Not :: 9;
        LNot :: 10;
        Real :: 11;
        Imag :: 12;
        Extension :: 13;
        Coawait :: 14;
    }
} else {
    // Kinds of unary operator expression that a cursor can have.
    CXUnaryOperatorKind :: enum u32 {
        Invalid :: 0;
        PostInc :: 1;
        PostDec :: 2;
        PreInc :: 3;
        PreDec :: 4;
        AddrOf :: 5;
        Deref :: 6;
        Plus :: 7;
        Minus :: 8;
        Not :: 9;
        LNot :: 10;
        Real :: 11;
        Imag :: 12;
        Extension :: 13;
        Coawait :: 14;
    }
}

// Version number with major, minor and subminor parts.
CXVersion :: struct {
    Major: s32;
    Minor: s32;
    Subminor: s32;
}

#if OS == .WINDOWS {
    // Symbol visibility of an entity: invalid, hidden, protected or default.
    CXVisibilityKind :: enum s32 {
        Invalid :: 0;
        Hidden :: 1;
        Protected :: 2;
        Default :: 3;
    }
} else {
    // Symbol visibility of an entity: invalid, hidden, protected or default.
    CXVisibilityKind :: enum u32 {
        Invalid :: 0;
        Hidden :: 1;
        Protected :: 2;
        Default :: 3;
    }
}

#if OS == .WINDOWS {
    // Value a visitor callback returns to stop (`Break`) or continue (`Continue`) the traversal.
    CXVisitorResult :: enum s32 {
        Break :: 0;
        Continue :: 1;
    }
} else {
    // Value a visitor callback returns to stop (`Break`) or continue (`Continue`) the traversal.
    CXVisitorResult :: enum u32 {
        Break :: 0;
        Continue :: 1;
    }
}

// The older name of [CXBinaryOperatorKind], which `clang_Cursor_getBinaryOpcode` uses; its last member is `LAST` there.
CX_BinaryOperatorKind :: CXBinaryOperatorKind;

#if OS == .WINDOWS {
    // C++ access specifier of a member: invalid, public, protected or private.
    CX_CXXAccessSpecifier :: enum s32 {
        CXXInvalidAccessSpecifier :: 0;
        CXXPublic :: 1;
        CXXProtected :: 2;
        CXXPrivate :: 3;
    }
} else {
    // C++ access specifier of a member: invalid, public, protected or private.
    CX_CXXAccessSpecifier :: enum u32 {
        CXXInvalidAccessSpecifier :: 0;
        CXXPublic :: 1;
        CXXProtected :: 2;
        CXXPrivate :: 3;
    }
}

// Callbacks that the indexing API invokes while it walks a translation unit.
IndexerCallbacks :: struct {
    abortQuery: #type (client_data: CXClientData, reserved: *void) -> s32 #c_call;
    diagnostic: #type (client_data: CXClientData, unknown0: CXDiagnosticSet, reserved: *void) -> void #c_call;
    enteredMainFile: #type (client_data: CXClientData, mainFile: CXFile, reserved: *void) -> CXIdxClientFile #c_call;
    ppIncludedFile: #type (client_data: CXClientData, unknown0: *CXIdxIncludedFileInfo) -> CXIdxClientFile #c_call;
    importedASTFile: #type (client_data: CXClientData, unknown0: *CXIdxImportedASTFileInfo) -> CXIdxClientASTFile #c_call;
    startedTranslationUnit: #type (client_data: CXClientData, reserved: *void) -> CXIdxClientContainer #c_call;
    indexDeclaration: #type (client_data: CXClientData, unknown0: *CXIdxDeclInfo) -> void #c_call;
    indexEntityReference: #type (client_data: CXClientData, unknown0: *CXIdxEntityRefInfo) -> void #c_call;
}
