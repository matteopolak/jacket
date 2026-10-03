// Additional reviewed native ABI reexports. Invocation coverage is tracked separately.

clang_CXCursorSet_contains :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang;

clang_CXCursorSet_insert :: (cset: CXCursorSet, cursor: CXCursor) -> u32 #foreign libclang;

clang_CXIndex_getGlobalOptions :: (unknown0: CXIndex) -> u32 #foreign libclang;

clang_CXIndex_setGlobalOptions :: (unknown0: CXIndex, options: u32) -> void #foreign libclang;

clang_CXIndex_setInvocationEmissionPathOption :: (unknown0: CXIndex, Path: *u8) -> void #foreign libclang;

clang_CXXConstructor_isConvertingConstructor :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXConstructor_isCopyConstructor :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXConstructor_isDefaultConstructor :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXConstructor_isMoveConstructor :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXField_isMutable :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isConst :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isCopyAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isDefaulted :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isDeleted :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isExplicit :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isMoveAssignmentOperator :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isPureVirtual :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isStatic :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXMethod_isVirtual :: (C: CXCursor) -> u32 #foreign libclang;

clang_CXXRecord_isAbstract :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_getBinaryOpcode :: (C: CXCursor) -> CX_BinaryOperatorKind #foreign libclang;

clang_Cursor_getBinaryOpcodeStr :: (Op: CX_BinaryOperatorKind) -> CXString #foreign libclang;

clang_Cursor_getBriefCommentText :: (C: CXCursor) -> CXString #foreign libclang;

clang_Cursor_getCXXManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang;

clang_Cursor_getCommentRange :: (C: CXCursor) -> CXSourceRange #foreign libclang;

clang_Cursor_getGCCAssemblyClobber :: (Cursor: CXCursor, Index: u32) -> CXString #foreign libclang;

clang_Cursor_getGCCAssemblyInput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang;

clang_Cursor_getGCCAssemblyNumClobbers :: (Cursor: CXCursor) -> u32 #foreign libclang;

clang_Cursor_getGCCAssemblyNumInputs :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_Cursor_getGCCAssemblyNumOutputs :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_Cursor_getGCCAssemblyOutput :: (Cursor: CXCursor, Index: u32, Constraint: *CXString, Expr: *CXCursor) -> u32 #foreign libclang;

clang_Cursor_getGCCAssemblyTemplate :: (unknown0: CXCursor) -> CXString #foreign libclang;

clang_Cursor_getMangling :: (unknown0: CXCursor) -> CXString #foreign libclang;

clang_Cursor_getModule :: (C: CXCursor) -> CXModule #foreign libclang;

clang_Cursor_getNumTemplateArguments :: (C: CXCursor) -> s32 #foreign libclang;

clang_Cursor_getObjCDeclQualifiers :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_getObjCManglings :: (unknown0: CXCursor) -> *CXStringSet #foreign libclang;

clang_Cursor_getObjCPropertyAttributes :: (C: CXCursor, reserved: u32) -> u32 #foreign libclang;

clang_Cursor_getObjCPropertyGetterName :: (C: CXCursor) -> CXString #foreign libclang;

clang_Cursor_getObjCPropertySetterName :: (C: CXCursor) -> CXString #foreign libclang;

clang_Cursor_getObjCSelectorIndex :: (unknown0: CXCursor) -> s32 #foreign libclang;

clang_Cursor_getReceiverType :: (C: CXCursor) -> CXType #foreign libclang;

clang_Cursor_getSpellingNameRange :: (unknown0: CXCursor, pieceIndex: u32, options: u32) -> CXSourceRange #foreign libclang;

clang_Cursor_getStorageClass :: (unknown0: CXCursor) -> CX_StorageClass #foreign libclang;

clang_Cursor_getTemplateArgumentKind :: (C: CXCursor, I: u32) -> CXTemplateArgumentKind #foreign libclang;

clang_Cursor_getTemplateArgumentType :: (C: CXCursor, I: u32) -> CXType #foreign libclang;

clang_Cursor_getTemplateArgumentUnsignedValue :: (C: CXCursor, I: u32) -> u64 #foreign libclang;

clang_Cursor_getTemplateArgumentValue :: (C: CXCursor, I: u32) -> s64 #foreign libclang;

clang_Cursor_getTranslationUnit :: (unknown0: CXCursor) -> CXTranslationUnit #foreign libclang;

clang_Cursor_hasAttrs :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isAnonymous :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isAnonymousRecordDecl :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isDynamicCall :: (C: CXCursor) -> s32 #foreign libclang;

clang_Cursor_isExternalSymbol :: (C: CXCursor, language: *CXString, definedIn: *CXString, isGenerated: *u32) -> u32 #foreign libclang;

clang_Cursor_isFunctionInlined :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isGCCAssemblyHasGoto :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isGCCAssemblyVolatile :: (Cursor: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isInlineNamespace :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isMacroBuiltin :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isMacroFunctionLike :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isObjCOptional :: (C: CXCursor) -> u32 #foreign libclang;

clang_Cursor_isVariadic :: (C: CXCursor) -> u32 #foreign libclang;

clang_EnumDecl_isScoped :: (C: CXCursor) -> u32 #foreign libclang;

clang_EvalResult_getAsStr :: (E: CXEvalResult) -> *u8 #foreign libclang;

clang_File_isEqual :: (file1: CXFile, file2: CXFile) -> s32 #foreign libclang;

clang_File_tryGetRealPathName :: (file: CXFile) -> CXString #foreign libclang;

clang_IndexAction_create :: (CIdx: CXIndex) -> CXIndexAction #foreign libclang;

clang_IndexAction_dispose :: (unknown0: CXIndexAction) -> void #foreign libclang;

clang_Module_getASTFile :: (Module: CXModule) -> CXFile #foreign libclang;

clang_Module_getFullName :: (Module: CXModule) -> CXString #foreign libclang;

clang_Module_getName :: (Module: CXModule) -> CXString #foreign libclang;

clang_Module_getNumTopLevelHeaders :: (unknown0: CXTranslationUnit, Module: CXModule) -> u32 #foreign libclang;

clang_Module_getParent :: (Module: CXModule) -> CXModule #foreign libclang;

clang_Module_getTopLevelHeader :: (unknown0: CXTranslationUnit, Module: CXModule, Index: u32) -> CXFile #foreign libclang;

clang_Module_isSystem :: (Module: CXModule) -> s32 #foreign libclang;

clang_PrintingPolicy_dispose :: (Policy: CXPrintingPolicy) -> void #foreign libclang;

clang_PrintingPolicy_getProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty) -> u32 #foreign libclang;

clang_PrintingPolicy_setProperty :: (Policy: CXPrintingPolicy, Property: CXPrintingPolicyProperty, Value: u32) -> void #foreign libclang;

clang_Range_isNull :: (range: CXSourceRange) -> s32 #foreign libclang;

clang_TargetInfo_dispose :: (Info: CXTargetInfo) -> void #foreign libclang;

clang_TargetInfo_getPointerWidth :: (Info: CXTargetInfo) -> s32 #foreign libclang;

clang_TargetInfo_getTriple :: (Info: CXTargetInfo) -> CXString #foreign libclang;

clang_Type_getCXXRefQualifier :: (T: CXType) -> CXRefQualifierKind #foreign libclang;

clang_Type_getClassType :: (T: CXType) -> CXType #foreign libclang;

clang_Type_getModifiedType :: (T: CXType) -> CXType #foreign libclang;

clang_Type_getNamedType :: (T: CXType) -> CXType #foreign libclang;

clang_Type_getNullability :: (T: CXType) -> CXTypeNullabilityKind #foreign libclang;

clang_Type_getNumObjCProtocolRefs :: (T: CXType) -> u32 #foreign libclang;

clang_Type_getNumObjCTypeArgs :: (T: CXType) -> u32 #foreign libclang;

clang_Type_getNumTemplateArguments :: (T: CXType) -> s32 #foreign libclang;

clang_Type_getObjCEncoding :: (type: CXType) -> CXString #foreign libclang;

clang_Type_getObjCObjectBaseType :: (T: CXType) -> CXType #foreign libclang;

clang_Type_getObjCProtocolDecl :: (T: CXType, i: u32) -> CXCursor #foreign libclang;

clang_Type_getObjCTypeArg :: (T: CXType, i: u32) -> CXType #foreign libclang;

clang_Type_getOffsetOf :: (T: CXType, S: *u8) -> s64 #foreign libclang;

clang_Type_getTemplateArgumentAsType :: (T: CXType, i: u32) -> CXType #foreign libclang;

clang_Type_getValueType :: (CT: CXType) -> CXType #foreign libclang;

clang_Type_isTransparentTagTypedef :: (T: CXType) -> u32 #foreign libclang;

clang_Type_visitFields :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang;

clang_codeCompleteAt :: (TU: CXTranslationUnit, complete_filename: *u8, complete_line: u32, complete_column: u32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> *CXCodeCompleteResults #foreign libclang;

clang_codeCompleteGetContainerKind :: (Results: *CXCodeCompleteResults, IsIncomplete: *u32) -> CXCursorKind #foreign libclang;

clang_codeCompleteGetContainerUSR :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang;

clang_codeCompleteGetContexts :: (Results: *CXCodeCompleteResults) -> u64 #foreign libclang;

clang_codeCompleteGetDiagnostic :: (Results: *CXCodeCompleteResults, Index: u32) -> CXDiagnostic #foreign libclang;

clang_codeCompleteGetNumDiagnostics :: (Results: *CXCodeCompleteResults) -> u32 #foreign libclang;

clang_codeCompleteGetObjCSelector :: (Results: *CXCodeCompleteResults) -> CXString #foreign libclang;

clang_constructUSR_ObjCCategory :: (class_name: *u8, category_name: *u8) -> CXString #foreign libclang;

clang_constructUSR_ObjCClass :: (class_name: *u8) -> CXString #foreign libclang;

clang_constructUSR_ObjCIvar :: (name: *u8, classUSR: CXString) -> CXString #foreign libclang;

clang_constructUSR_ObjCMethod :: (name: *u8, isInstanceMethod: u32, classUSR: CXString) -> CXString #foreign libclang;

clang_constructUSR_ObjCProperty :: (property: *u8, classUSR: CXString) -> CXString #foreign libclang;

clang_constructUSR_ObjCProtocol :: (protocol_name: *u8) -> CXString #foreign libclang;

clang_createCXCursorSet :: () -> CXCursorSet #foreign libclang;

clang_createTranslationUnit :: (CIdx: CXIndex, ast_filename: *u8) -> CXTranslationUnit #foreign libclang;

clang_createTranslationUnit2 :: (CIdx: CXIndex, ast_filename: *u8, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang;

clang_createTranslationUnitFromSourceFile :: (CIdx: CXIndex, source_filename: *u8, num_clang_command_line_args: s32, clang_command_line_args: **u8, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile) -> CXTranslationUnit #foreign libclang;

clang_defaultCodeCompleteOptions :: () -> u32 #foreign libclang;

clang_defaultEditingTranslationUnitOptions :: () -> u32 #foreign libclang;

clang_defaultReparseOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang;

clang_defaultSaveOptions :: (TU: CXTranslationUnit) -> u32 #foreign libclang;

clang_disposeCXCursorSet :: (cset: CXCursorSet) -> void #foreign libclang;

clang_disposeCXPlatformAvailability :: (availability: *CXPlatformAvailability) -> void #foreign libclang;

clang_disposeCXTUResourceUsage :: (usage: CXTUResourceUsage) -> void #foreign libclang;

clang_disposeCodeCompleteResults :: (Results: *CXCodeCompleteResults) -> void #foreign libclang;

clang_disposeDiagnosticSet :: (Diags: CXDiagnosticSet) -> void #foreign libclang;

clang_disposeOverriddenCursors :: (overridden: *CXCursor) -> void #foreign libclang;

clang_disposeSourceRangeList :: (ranges: *CXSourceRangeList) -> void #foreign libclang;

clang_disposeStringSet :: (set: *CXStringSet) -> void #foreign libclang;

clang_enableStackTraces :: () -> void #foreign libclang;

clang_equalLocations :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang;

clang_equalRanges :: (range1: CXSourceRange, range2: CXSourceRange) -> u32 #foreign libclang;

clang_equalTypes :: (A: CXType, B: CXType) -> u32 #foreign libclang;

clang_executeOnThread :: (fn: #type (unknown0: *void) -> void #c_call, user_data: *void, stack_size: u32) -> void #foreign libclang;

clang_findIncludesInFile :: (TU: CXTranslationUnit, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang;

clang_findReferencesInFile :: (cursor: CXCursor, file: CXFile, visitor: CXCursorAndRangeVisitor) -> CXResult #foreign libclang;

clang_getAddressSpace :: (T: CXType) -> u32 #foreign libclang;

clang_getAllSkippedRanges :: (tu: CXTranslationUnit) -> *CXSourceRangeList #foreign libclang;

clang_getBinaryOperatorKindSpelling :: (kind: CXBinaryOperatorKind) -> CXString #foreign libclang;

clang_getBuildSessionTimestamp :: () -> u64 #foreign libclang;

clang_getCXTUResourceUsage :: (TU: CXTranslationUnit) -> CXTUResourceUsage #foreign libclang;

clang_getCXXAccessSpecifier :: (unknown0: CXCursor) -> CX_CXXAccessSpecifier #foreign libclang;

clang_getCanonicalCursor :: (unknown0: CXCursor) -> CXCursor #foreign libclang;

clang_getChildDiagnostics :: (D: CXDiagnostic) -> CXDiagnosticSet #foreign libclang;

clang_getClangVersion :: () -> CXString #foreign libclang;

clang_getCompletionAnnotation :: (completion_string: CXCompletionString, annotation_number: u32) -> CXString #foreign libclang;

clang_getCompletionAvailability :: (completion_string: CXCompletionString) -> CXAvailabilityKind #foreign libclang;

clang_getCompletionBriefComment :: (completion_string: CXCompletionString) -> CXString #foreign libclang;

clang_getCompletionChunkCompletionString :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionString #foreign libclang;

clang_getCompletionChunkKind :: (completion_string: CXCompletionString, chunk_number: u32) -> CXCompletionChunkKind #foreign libclang;

clang_getCompletionChunkText :: (completion_string: CXCompletionString, chunk_number: u32) -> CXString #foreign libclang;

clang_getCompletionFixIt :: (results: *CXCodeCompleteResults, completion_index: u32, fixit_index: u32, replacement_range: *CXSourceRange) -> CXString #foreign libclang;

clang_getCompletionNumAnnotations :: (completion_string: CXCompletionString) -> u32 #foreign libclang;

clang_getCompletionNumFixIts :: (results: *CXCodeCompleteResults, completion_index: u32) -> u32 #foreign libclang;

clang_getCompletionParent :: (completion_string: CXCompletionString, kind: *CXCursorKind) -> CXString #foreign libclang;

clang_getCompletionPriority :: (completion_string: CXCompletionString) -> u32 #foreign libclang;

clang_getCursorAvailability :: (cursor: CXCursor) -> CXAvailabilityKind #foreign libclang;

clang_getCursorBinaryOperatorKind :: (cursor: CXCursor) -> CXBinaryOperatorKind #foreign libclang;

clang_getCursorCompletionString :: (cursor: CXCursor) -> CXCompletionString #foreign libclang;

clang_getCursorDisplayName :: (unknown0: CXCursor) -> CXString #foreign libclang;

clang_getCursorExceptionSpecificationType :: (C: CXCursor) -> s32 #foreign libclang;

clang_getCursorKindSpelling :: (Kind: CXCursorKind) -> CXString #foreign libclang;

clang_getCursorLanguage :: (cursor: CXCursor) -> CXLanguageKind #foreign libclang;

clang_getCursorLexicalParent :: (cursor: CXCursor) -> CXCursor #foreign libclang;

clang_getCursorLinkage :: (cursor: CXCursor) -> CXLinkageKind #foreign libclang;

clang_getCursorPlatformAvailability :: (cursor: CXCursor, always_deprecated: *s32, deprecated_message: *CXString, always_unavailable: *s32, unavailable_message: *CXString, availability: *CXPlatformAvailability, availability_size: s32) -> s32 #foreign libclang;

clang_getCursorPrettyPrinted :: (Cursor: CXCursor, Policy: CXPrintingPolicy) -> CXString #foreign libclang;

clang_getCursorPrintingPolicy :: (unknown0: CXCursor) -> CXPrintingPolicy #foreign libclang;

clang_getCursorReferenceNameRange :: (C: CXCursor, NameFlags: u32, PieceIndex: u32) -> CXSourceRange #foreign libclang;

clang_getCursorReferenced :: (unknown0: CXCursor) -> CXCursor #foreign libclang;

clang_getCursorResultType :: (C: CXCursor) -> CXType #foreign libclang;

clang_getCursorTLSKind :: (cursor: CXCursor) -> CXTLSKind #foreign libclang;

clang_getCursorUSR :: (unknown0: CXCursor) -> CXString #foreign libclang;

clang_getCursorUnaryOperatorKind :: (cursor: CXCursor) -> CXUnaryOperatorKind #foreign libclang;

clang_getCursorVisibility :: (cursor: CXCursor) -> CXVisibilityKind #foreign libclang;

clang_getDeclObjCTypeEncoding :: (C: CXCursor) -> CXString #foreign libclang;

clang_getDefinitionSpellingAndExtent :: (unknown0: CXCursor, startBuf: **u8, endBuf: **u8, startLine: *u32, startColumn: *u32, endLine: *u32, endColumn: *u32) -> void #foreign libclang;

clang_getDiagnosticCategory :: (unknown0: CXDiagnostic) -> u32 #foreign libclang;

clang_getDiagnosticCategoryName :: (Category: u32) -> CXString #foreign libclang;

clang_getDiagnosticCategoryText :: (unknown0: CXDiagnostic) -> CXString #foreign libclang;

clang_getDiagnosticInSet :: (Diags: CXDiagnosticSet, Index: u32) -> CXDiagnostic #foreign libclang;

clang_getDiagnosticOption :: (Diag: CXDiagnostic, Disable: *CXString) -> CXString #foreign libclang;

clang_getDiagnosticSetFromTU :: (Unit: CXTranslationUnit) -> CXDiagnosticSet #foreign libclang;

clang_getElementType :: (T: CXType) -> CXType #foreign libclang;

clang_getExceptionSpecificationType :: (T: CXType) -> s32 #foreign libclang;

clang_getExpansionLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang;

clang_getFileLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang;

clang_getFileTime :: (SFile: CXFile) -> time_t #foreign libclang;

clang_getFileUniqueID :: (file: CXFile, outID: *CXFileUniqueID) -> s32 #foreign libclang;

clang_getFullyQualifiedName :: (CT: CXType, Policy: CXPrintingPolicy, WithGlobalNsPrefix: u32) -> CXString #foreign libclang;

clang_getFunctionTypeCallingConv :: (T: CXType) -> CXCallingConv #foreign libclang;

clang_getIBOutletCollectionType :: (unknown0: CXCursor) -> CXType #foreign libclang;

clang_getIncludedFile :: (cursor: CXCursor) -> CXFile #foreign libclang;

clang_getInclusions :: (tu: CXTranslationUnit, visitor: CXInclusionVisitor, client_data: CXClientData) -> void #foreign libclang;

clang_getInstantiationLocation :: (location: CXSourceLocation, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang;

clang_getLocation :: (tu: CXTranslationUnit, file: CXFile, line: u32, column: u32) -> CXSourceLocation #foreign libclang;

clang_getLocationForOffset :: (tu: CXTranslationUnit, file: CXFile, offset: u32) -> CXSourceLocation #foreign libclang;

clang_getModuleForFile :: (unknown0: CXTranslationUnit, unknown1: CXFile) -> CXModule #foreign libclang;

clang_getNonReferenceType :: (CT: CXType) -> CXType #foreign libclang;

clang_getNullLocation :: () -> CXSourceLocation #foreign libclang;

clang_getNullRange :: () -> CXSourceRange #foreign libclang;

clang_getNumCompletionChunks :: (completion_string: CXCompletionString) -> u32 #foreign libclang;

clang_getNumDiagnosticsInSet :: (Diags: CXDiagnosticSet) -> u32 #foreign libclang;

clang_getNumElements :: (T: CXType) -> s64 #foreign libclang;

clang_getNumOverloadedDecls :: (cursor: CXCursor) -> u32 #foreign libclang;

clang_getOffsetOfBase :: (Parent: CXCursor, Base: CXCursor) -> s64 #foreign libclang;

clang_getOverloadedDecl :: (cursor: CXCursor, index: u32) -> CXCursor #foreign libclang;

clang_getOverriddenCursors :: (cursor: CXCursor, overridden: **CXCursor, num_overridden: *u32) -> void #foreign libclang;

clang_getPresumedLocation :: (location: CXSourceLocation, filename: *CXString, line: *u32, column: *u32) -> void #foreign libclang;

clang_getRange :: (begin: CXSourceLocation, end: CXSourceLocation) -> CXSourceRange #foreign libclang;

clang_getRemappings :: (unknown0: *u8) -> CXRemapping #foreign libclang;

clang_getRemappingsFromFileList :: (unknown0: **u8, unknown1: u32) -> CXRemapping #foreign libclang;

clang_getSkippedRanges :: (tu: CXTranslationUnit, file: CXFile) -> *CXSourceRangeList #foreign libclang;

clang_getSpecializedCursorTemplate :: (C: CXCursor) -> CXCursor #foreign libclang;

clang_getTUResourceUsageName :: (kind: CXTUResourceUsageKind) -> *u8 #foreign libclang;

clang_getTemplateCursorKind :: (C: CXCursor) -> CXCursorKind #foreign libclang;

clang_getToken :: (TU: CXTranslationUnit, Location: CXSourceLocation) -> *CXToken #foreign libclang;

clang_getTranslationUnitSpelling :: (CTUnit: CXTranslationUnit) -> CXString #foreign libclang;

clang_getTranslationUnitTargetInfo :: (CTUnit: CXTranslationUnit) -> CXTargetInfo #foreign libclang;

clang_getTypeKindSpelling :: (K: CXTypeKind) -> CXString #foreign libclang;

clang_getTypePrettyPrinted :: (CT: CXType, cxPolicy: CXPrintingPolicy) -> CXString #foreign libclang;

clang_getTypedefName :: (CT: CXType) -> CXString #foreign libclang;

clang_getUnaryOperatorKindSpelling :: (kind: CXUnaryOperatorKind) -> CXString #foreign libclang;

clang_getUnqualifiedType :: (CT: CXType) -> CXType #foreign libclang;

clang_hashCursor :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_indexLoc_getCXSourceLocation :: (loc: CXIdxLoc) -> CXSourceLocation #foreign libclang;

clang_indexLoc_getFileLocation :: (loc: CXIdxLoc, indexFile: *CXIdxClientFile, file: *CXFile, line: *u32, column: *u32, offset: *u32) -> void #foreign libclang;

clang_indexSourceFile :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang;

clang_indexSourceFileFullArgv :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, out_TU: *CXTranslationUnit, TU_options: u32) -> s32 #foreign libclang;

clang_indexTranslationUnit :: (unknown0: CXIndexAction, client_data: CXClientData, index_callbacks: *IndexerCallbacks, index_callbacks_size: u32, index_options: u32, unknown1: CXTranslationUnit) -> s32 #foreign libclang;

clang_index_getCXXClassDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxCXXClassDeclInfo #foreign libclang;

clang_index_getClientContainer :: (unknown0: *CXIdxContainerInfo) -> CXIdxClientContainer #foreign libclang;

clang_index_getClientEntity :: (unknown0: *CXIdxEntityInfo) -> CXIdxClientEntity #foreign libclang;

clang_index_getIBOutletCollectionAttrInfo :: (unknown0: *CXIdxAttrInfo) -> *CXIdxIBOutletCollectionAttrInfo #foreign libclang;

clang_index_getObjCCategoryDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCCategoryDeclInfo #foreign libclang;

clang_index_getObjCContainerDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCContainerDeclInfo #foreign libclang;

clang_index_getObjCInterfaceDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCInterfaceDeclInfo #foreign libclang;

clang_index_getObjCPropertyDeclInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCPropertyDeclInfo #foreign libclang;

clang_index_getObjCProtocolRefListInfo :: (unknown0: *CXIdxDeclInfo) -> *CXIdxObjCProtocolRefListInfo #foreign libclang;

clang_index_isEntityObjCContainerKind :: (unknown0: CXIdxEntityKind) -> s32 #foreign libclang;

clang_index_setClientContainer :: (unknown0: *CXIdxContainerInfo, unknown1: CXIdxClientContainer) -> void #foreign libclang;

clang_index_setClientEntity :: (unknown0: *CXIdxEntityInfo, unknown1: CXIdxClientEntity) -> void #foreign libclang;

clang_isAttribute :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isBeforeInTranslationUnit :: (loc1: CXSourceLocation, loc2: CXSourceLocation) -> u32 #foreign libclang;

clang_isDeclaration :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isFileMultipleIncludeGuarded :: (tu: CXTranslationUnit, file: CXFile) -> u32 #foreign libclang;

clang_isInvalid :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isInvalidDeclaration :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_isPODType :: (T: CXType) -> u32 #foreign libclang;

clang_isPreprocessing :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isReference :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isRestrictQualifiedType :: (T: CXType) -> u32 #foreign libclang;

clang_isStatement :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isTranslationUnit :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isUnexposed :: (unknown0: CXCursorKind) -> u32 #foreign libclang;

clang_isVirtualBase :: (unknown0: CXCursor) -> u32 #foreign libclang;

clang_loadDiagnostics :: (file: *u8, error: *CXLoadDiag_Error, errorString: *CXString) -> CXDiagnosticSet #foreign libclang;

clang_parseTranslationUnit :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32) -> CXTranslationUnit #foreign libclang;

clang_parseTranslationUnit2FullArgv :: (CIdx: CXIndex, source_filename: *u8, command_line_args: **u8, num_command_line_args: s32, unsaved_files: *CXUnsavedFile, num_unsaved_files: u32, options: u32, out_TU: *CXTranslationUnit) -> CXErrorCode #foreign libclang;

clang_remap_dispose :: (unknown0: CXRemapping) -> void #foreign libclang;

clang_remap_getFilenames :: (unknown0: CXRemapping, unknown1: u32, unknown2: *CXString, unknown3: *CXString) -> void #foreign libclang;

clang_remap_getNumFiles :: (unknown0: CXRemapping) -> u32 #foreign libclang;

clang_reparseTranslationUnit :: (TU: CXTranslationUnit, num_unsaved_files: u32, unsaved_files: *CXUnsavedFile, options: u32) -> s32 #foreign libclang;

clang_saveTranslationUnit :: (TU: CXTranslationUnit, FileName: *u8, options: u32) -> s32 #foreign libclang;

clang_sortCodeCompletionResults :: (Results: *CXCompletionResult, NumResults: u32) -> void #foreign libclang;

clang_suspendTranslationUnit :: (unknown0: CXTranslationUnit) -> u32 #foreign libclang;

clang_toggleCrashRecovery :: (isEnabled: u32) -> void #foreign libclang;

clang_visitCXXBaseClasses :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang;

clang_visitCXXMethods :: (T: CXType, visitor: CXFieldVisitor, client_data: CXClientData) -> u32 #foreign libclang;

#scope_file
libclang :: #foreign_library "../internal/developer-tools/clang-bindings/clang";

#if OS == .WINDOWS {
    time_t :: s64;
} else {
    Native_POSIX :: #import "POSIX";

    time_t :: Native_POSIX.time_t;
}
