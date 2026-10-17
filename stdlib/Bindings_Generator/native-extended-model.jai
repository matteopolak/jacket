// Additional native ABI types; no native implementation bodies are reproduced.

#if OS == .WINDOWS {
    CXAvailabilityKind :: enum s32 {
        Available :: 0;
        Deprecated :: 1;
        NotAvailable :: 2;
        NotAccessible :: 3;
    }
} else {
    CXAvailabilityKind :: enum u32 {
        Available :: 0;
        Deprecated :: 1;
        NotAvailable :: 2;
        NotAccessible :: 3;
    }
}

#if OS == .WINDOWS {
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
    }
} else {
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
    }
}

#if OS == .WINDOWS {
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
    CXChoice :: enum s32 {
        Default :: 0;
        Enabled :: 1;
        Disabled :: 2;
    }
} else {
    CXChoice :: enum u32 {
        Default :: 0;
        Enabled :: 1;
        Disabled :: 2;
    }
}

CXCodeCompleteResults :: struct {
    Results: *CXCompletionResult;
    NumResults: u32;
}

#if OS == .WINDOWS {
    CXCodeComplete_Flags :: enum_flags s32 {
        IncludeMacros :: 0x1;
        IncludeCodePatterns :: 0x2;
        IncludeBriefComments :: 0x4;
        SkipPreamble :: 0x8;
        IncludeCompletionsWithFixIts :: 0x10;
    }
} else {
    CXCodeComplete_Flags :: enum_flags u32 {
        IncludeMacros :: 0x1;
        IncludeCodePatterns :: 0x2;
        IncludeBriefComments :: 0x4;
        SkipPreamble :: 0x8;
        IncludeCompletionsWithFixIts :: 0x10;
    }
}

#if OS == .WINDOWS {
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

CXCompletionResult :: struct {
    CursorKind: CXCursorKind;
    CompletionString: CXCompletionString;
}

CXCompletionString :: *void;

CXCursorAndRangeVisitor :: struct {
    _context: *void;
    visit: #type (_context: *void, unknown0: CXCursor, unknown1: CXSourceRange) -> CXVisitorResult #c_call;
}

CXCursorSet :: *CXCursorSetImpl;

CXCursorSetImpl :: struct {}

#if OS == .WINDOWS {
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
    CXDiagnosticDisplayOptions :: enum s32 {
        DisplaySourceLocation :: 1;
        DisplayColumn :: 2;
        DisplaySourceRanges :: 4;
        DisplayOption :: 8;
        DisplayCategoryId :: 16;
        DisplayCategoryName :: 32;
    }
} else {
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

CXFileUniqueID :: struct {
    data: [3] u64;
}

#if OS == .WINDOWS {
    CXGlobalOptFlags :: enum s32 {
        None :: 0;
        ThreadBackgroundPriorityForIndexing :: 1;
        ThreadBackgroundPriorityForEditing :: 2;
        ThreadBackgroundPriorityForAll :: 3;
    }
} else {
    CXGlobalOptFlags :: enum u32 {
        None :: 0;
        ThreadBackgroundPriorityForIndexing :: 1;
        ThreadBackgroundPriorityForEditing :: 2;
        ThreadBackgroundPriorityForAll :: 3;
    }
}

CXIdxAttrInfo :: struct {
    kind: CXIdxAttrKind;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

#if OS == .WINDOWS {
    CXIdxAttrKind :: enum s32 {
        Unexposed :: 0;
        IBAction :: 1;
        IBOutlet :: 2;
        IBOutletCollection :: 3;
    }
} else {
    CXIdxAttrKind :: enum u32 {
        Unexposed :: 0;
        IBAction :: 1;
        IBOutlet :: 2;
        IBOutletCollection :: 3;
    }
}

CXIdxBaseClassInfo :: struct {
    base: *CXIdxEntityInfo;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

CXIdxCXXClassDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    bases: **CXIdxBaseClassInfo;
    numBases: u32;
}

CXIdxClientASTFile :: *void;

CXIdxClientContainer :: *void;

CXIdxClientEntity :: *void;

CXIdxClientFile :: *void;

CXIdxContainerInfo :: struct {
    cursor: CXCursor;
}

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
    CXIdxDeclInfoFlags :: enum s32 {
        CXIdxDeclFlag_Skipped :: 1;
    }
} else {
    CXIdxDeclInfoFlags :: enum u32 {
        CXIdxDeclFlag_Skipped :: 1;
    }
}

#if OS == .WINDOWS {
    CXIdxEntityCXXTemplateKind :: enum s32 {
        NonTemplate :: 0;
        Template :: 1;
        TemplatePartialSpecialization :: 2;
        TemplateSpecialization :: 3;
    }
} else {
    CXIdxEntityCXXTemplateKind :: enum u32 {
        NonTemplate :: 0;
        Template :: 1;
        TemplatePartialSpecialization :: 2;
        TemplateSpecialization :: 3;
    }
}

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
    CXIdxEntityLanguage :: enum s32 {
        None :: 0;
        C :: 1;
        ObjC :: 2;
        CXX :: 3;
        Swift :: 4;
    }
} else {
    CXIdxEntityLanguage :: enum u32 {
        None :: 0;
        C :: 1;
        ObjC :: 2;
        CXX :: 3;
        Swift :: 4;
    }
}

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
    CXIdxEntityRefKind :: enum s32 {
        Direct :: 1;
        Implicit :: 2;
    }
} else {
    CXIdxEntityRefKind :: enum u32 {
        Direct :: 1;
        Implicit :: 2;
    }
}

CXIdxIBOutletCollectionAttrInfo :: struct {
    attrInfo: *CXIdxAttrInfo;
    objcClass: *CXIdxEntityInfo;
    classCursor: CXCursor;
    classLoc: CXIdxLoc;
}

CXIdxImportedASTFileInfo :: struct {
    file: CXFile;
    module: CXModule;
    loc: CXIdxLoc;
    isImplicit: s32;
}

CXIdxIncludedFileInfo :: struct {
    hashLoc: CXIdxLoc;
    filename: *u8;
    file: CXFile;
    isImport: s32;
    isAngled: s32;
    isModuleImport: s32;
}

CXIdxLoc :: struct {
    ptr_data: [2] *void;
    int_data: u32;
}

CXIdxObjCCategoryDeclInfo :: struct {
    containerInfo: *CXIdxObjCContainerDeclInfo;
    objcClass: *CXIdxEntityInfo;
    classCursor: CXCursor;
    classLoc: CXIdxLoc;
    protocols: *CXIdxObjCProtocolRefListInfo;
}

CXIdxObjCContainerDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    kind: CXIdxObjCContainerKind;
}

#if OS == .WINDOWS {
    CXIdxObjCContainerKind :: enum s32 {
        ForwardRef :: 0;
        Interface :: 1;
        Implementation :: 2;
    }
} else {
    CXIdxObjCContainerKind :: enum u32 {
        ForwardRef :: 0;
        Interface :: 1;
        Implementation :: 2;
    }
}

CXIdxObjCInterfaceDeclInfo :: struct {
    containerInfo: *CXIdxObjCContainerDeclInfo;
    superInfo: *CXIdxBaseClassInfo;
    protocols: *CXIdxObjCProtocolRefListInfo;
}

CXIdxObjCPropertyDeclInfo :: struct {
    declInfo: *CXIdxDeclInfo;
    getter: *CXIdxEntityInfo;
    setter: *CXIdxEntityInfo;
}

CXIdxObjCProtocolRefInfo :: struct {
    protocol: *CXIdxEntityInfo;
    cursor: CXCursor;
    loc: CXIdxLoc;
}

CXIdxObjCProtocolRefListInfo :: struct {
    protocols: **CXIdxObjCProtocolRefInfo;
    numProtocols: u32;
}

CXInclusionVisitor :: #type (included_file: CXFile, inclusion_stack: *CXSourceLocation, include_len: u32, client_data: CXClientData) -> void #c_call;

CXIndexAction :: *void;

#if OS == .WINDOWS {
    CXIndexOptFlags :: enum s32 {
        None :: 0;
        SuppressRedundantRefs :: 1;
        IndexFunctionLocalSymbols :: 2;
        IndexImplicitTemplateInstantiations :: 4;
        SuppressWarnings :: 8;
        SkipParsedBodiesInSession :: 16;
    }
} else {
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
    CXLanguageKind :: enum s32 {
        Invalid :: 0;
        C :: 1;
        ObjC :: 2;
        CPlusPlus :: 3;
    }
} else {
    CXLanguageKind :: enum u32 {
        Invalid :: 0;
        C :: 1;
        ObjC :: 2;
        CPlusPlus :: 3;
    }
}

#if OS == .WINDOWS {
    CXLinkageKind :: enum s32 {
        Invalid :: 0;
        NoLinkage :: 1;
        Internal :: 2;
        UniqueExternal :: 3;
        External :: 4;
    }
} else {
    CXLinkageKind :: enum u32 {
        Invalid :: 0;
        NoLinkage :: 1;
        Internal :: 2;
        UniqueExternal :: 3;
        External :: 4;
    }
}

#if OS == .WINDOWS {
    CXLoadDiag_Error :: enum s32 {
        None :: 0;
        Unknown :: 1;
        CannotLoad :: 2;
        InvalidFile :: 3;
    }
} else {
    CXLoadDiag_Error :: enum u32 {
        None :: 0;
        Unknown :: 1;
        CannotLoad :: 2;
        InvalidFile :: 3;
    }
}

CXModule :: *void;

#if OS == .WINDOWS {
    CXNameRefFlags :: enum s32 {
        WantQualifier :: 1;
        WantTemplateArgs :: 2;
        WantSinglePiece :: 4;
    }
} else {
    CXNameRefFlags :: enum u32 {
        WantQualifier :: 1;
        WantTemplateArgs :: 2;
        WantSinglePiece :: 4;
    }
}

#if OS == .WINDOWS {
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
    CXRefQualifierKind :: enum s32 {
        None :: 0;
        LValue :: 1;
        RValue :: 2;
    }
} else {
    CXRefQualifierKind :: enum u32 {
        None :: 0;
        LValue :: 1;
        RValue :: 2;
    }
}

CXRemapping :: *void;

#if OS == .WINDOWS {
    CXReparse_Flags :: enum_flags s32 {
        CXReparse_None :: 0x0;
    }
} else {
    CXReparse_Flags :: enum_flags u32 {
        CXReparse_None :: 0x0;
    }
}

#if OS == .WINDOWS {
    CXResult :: enum s32 {
        Success :: 0;
        Invalid :: 1;
        VisitBreak :: 2;
    }
} else {
    CXResult :: enum u32 {
        Success :: 0;
        Invalid :: 1;
        VisitBreak :: 2;
    }
}

#if OS == .WINDOWS {
    CXSaveError :: enum s32 {
        None :: 0;
        Unknown :: 1;
        TranslationErrors :: 2;
        InvalidTU :: 3;
    }
} else {
    CXSaveError :: enum u32 {
        None :: 0;
        Unknown :: 1;
        TranslationErrors :: 2;
        InvalidTU :: 3;
    }
}

#if OS == .WINDOWS {
    CXSaveTranslationUnit_Flags :: enum_flags s32 {
        CXSaveTranslationUnit_None :: 0x0;
    }
} else {
    CXSaveTranslationUnit_Flags :: enum_flags u32 {
        CXSaveTranslationUnit_None :: 0x0;
    }
}

CXSourceRangeList :: struct {
    count: u32;
    ranges: *CXSourceRange;
}

CXStringSet :: struct {
    Strings: *CXString;
    Count: u32;
}

#if OS == .WINDOWS {
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
    CXTLSKind :: enum s32 {
        None :: 0;
        Dynamic :: 1;
        Static :: 2;
    }
} else {
    CXTLSKind :: enum u32 {
        None :: 0;
        Dynamic :: 1;
        Static :: 2;
    }
}

CXTUResourceUsage :: struct {
    data: *void;
    numEntries: u32;
    entries: *CXTUResourceUsageEntry;
}

#if OS == .WINDOWS {
    CXTUResourceUsageEntry :: struct {
        kind: CXTUResourceUsageKind;
        amount: u32;
    }
} else {
    CXTUResourceUsageEntry :: struct {
        kind: CXTUResourceUsageKind;
        amount: u64;
    }
}

#if OS == .WINDOWS {
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

CXTargetInfoImpl :: struct {}

#if OS == .WINDOWS {
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
    CXTypeNullabilityKind :: enum s32 {
        NonNull :: 0;
        Nullable :: 1;
        Unspecified :: 2;
        Invalid :: 3;
        NullableResult :: 4;
    }
} else {
    CXTypeNullabilityKind :: enum u32 {
        NonNull :: 0;
        Nullable :: 1;
        Unspecified :: 2;
        Invalid :: 3;
        NullableResult :: 4;
    }
}

#if OS == .WINDOWS {
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

CXVersion :: struct {
    Major: s32;
    Minor: s32;
    Subminor: s32;
}

#if OS == .WINDOWS {
    CXVisibilityKind :: enum s32 {
        Invalid :: 0;
        Hidden :: 1;
        Protected :: 2;
        Default :: 3;
    }
} else {
    CXVisibilityKind :: enum u32 {
        Invalid :: 0;
        Hidden :: 1;
        Protected :: 2;
        Default :: 3;
    }
}

#if OS == .WINDOWS {
    CXVisitorResult :: enum s32 {
        Break :: 0;
        Continue :: 1;
    }
} else {
    CXVisitorResult :: enum u32 {
        Break :: 0;
        Continue :: 1;
    }
}

#if OS == .WINDOWS {
    CX_BinaryOperatorKind :: enum s32 {
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
        LAST :: 33;
    }
} else {
    CX_BinaryOperatorKind :: enum u32 {
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
        LAST :: 33;
    }
}

#if OS == .WINDOWS {
    CX_CXXAccessSpecifier :: enum s32 {
        CXXInvalidAccessSpecifier :: 0;
        CXXPublic :: 1;
        CXXProtected :: 2;
        CXXPrivate :: 3;
    }
} else {
    CX_CXXAccessSpecifier :: enum u32 {
        CXXInvalidAccessSpecifier :: 0;
        CXXPublic :: 1;
        CXXProtected :: 2;
        CXXPrivate :: 3;
    }
}

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
