# MojoShader

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `MOJOSHADER_version :: () -> s32 #foreign libmojoshader`
- `MOJOSHADER_changeset :: () -> *u8 #foreign libmojoshader`
- `MOJOSHADER_malloc :: #type (a0: s32, a1: *void) -> *void #c_call`
- `MOJOSHADER_free :: #type (a0: *void, a1: *void) -> void #c_call`
- `MOJOSHADER_uniform :: struct`
- `MOJOSHADER_constant :: struct`
- `MOJOSHADER_sampler :: struct`
- `MOJOSHADER_samplerMap :: struct`
- `MOJOSHADER_attribute :: struct`
- `MOJOSHADER_swizzle :: struct`
- `MOJOSHADER_symbolTypeInfo :: struct`
- `MOJOSHADER_symbolStructMember :: struct`
- `MOJOSHADER_symbol :: struct`
- `MOJOSHADER_error :: struct`
- `MOJOSHADER_preshaderOperand :: struct`
- `MOJOSHADER_preshaderInstruction :: struct`
- `MOJOSHADER_preshader :: struct`
- `MOJOSHADER_parseData :: struct`
- `MOJOSHADER_maxShaderModel :: (profile: *u8) -> s32 #foreign libmojoshader`
- `MOJOSHADER_parse :: (profile: *u8, mainfn: *u8, tokenbuf: *u8, bufsize: u32, swiz: *MOJOSHADER_swizzle, swizcount: u32, smap: *MOJOSHADER_samplerMap, smapcount: u32, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_parseData #foreign libmojoshader`
- `MOJOSHADER_freeParseData :: (data: *MOJOSHADER_parseData) -> void #foreign libmojoshader`
- `MOJOSHADER_parsePreshader :: (buf: *u8, len: u32, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_preshader #foreign libmojoshader`
- `MOJOSHADER_freePreshader :: (preshader: *MOJOSHADER_preshader) -> void #foreign libmojoshader`
- `MOJOSHADER_preprocessorDefine :: struct`
- `MOJOSHADER_preprocessData :: struct`
- `MOJOSHADER_includeOpen :: #type (a0: MOJOSHADER_includeType, a1: *u8, a2: *u8, a3: **u8, a4: *u32, a5: MOJOSHADER_malloc, a6: MOJOSHADER_free, a7: *void) -> s32 #c_call`
- `MOJOSHADER_includeClose :: #type (a0: *u8, a1: MOJOSHADER_malloc, a2: MOJOSHADER_free, a3: *void) -> void #c_call`
- `MOJOSHADER_preprocess :: (filename: *u8, source: *u8, sourcelen: u32, defines: *MOJOSHADER_preprocessorDefine, define_count: u32, include_open: MOJOSHADER_includeOpen, include_close: MOJOSHADER_includeClose, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_preprocessData #foreign libmojoshader`
- `MOJOSHADER_freePreprocessData :: (data: *MOJOSHADER_preprocessData) -> void #foreign libmojoshader`
- `MOJOSHADER_assemble :: (filename: *u8, source: *u8, sourcelen: u32, comments: **u8, comment_count: u32, symbols: *MOJOSHADER_symbol, symbol_count: u32, defines: *MOJOSHADER_preprocessorDefine, define_count: u32, include_open: MOJOSHADER_includeOpen, include_close: MOJOSHADER_includeClose, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_parseData #foreign libmojoshader`
- `MOJOSHADER_astDataTypeStructMember :: struct`
- `MOJOSHADER_astDataTypeStruct :: struct`
- `MOJOSHADER_astDataTypeArray :: struct`
- `MOJOSHADER_astDataTypeVector :: MOJOSHADER_astDataTypeArray`
- `MOJOSHADER_astDataTypeMatrix :: struct`
- `MOJOSHADER_astDataTypeBuffer :: struct`
- `MOJOSHADER_astDataTypeFunction :: struct`
- `MOJOSHADER_astDataTypeUser :: struct`
- `MOJOSHADER_astDataType :: union`
- `MOJOSHADER_astNodeInfo :: struct`
- `MOJOSHADER_astGeneric :: struct`
- `MOJOSHADER_astExpression :: struct`
- `MOJOSHADER_astArguments :: struct`
- `MOJOSHADER_astExpressionUnary :: struct`
- `MOJOSHADER_astExpressionBinary :: struct`
- `MOJOSHADER_astExpressionTernary :: struct`
- `MOJOSHADER_astExpressionIdentifier :: struct`
- `MOJOSHADER_astExpressionIntLiteral :: struct`
- `MOJOSHADER_astExpressionFloatLiteral :: struct`
- `MOJOSHADER_astExpressionStringLiteral :: struct`
- `MOJOSHADER_astExpressionBooleanLiteral :: struct`
- `MOJOSHADER_astExpressionConstructor :: struct`
- `MOJOSHADER_astExpressionDerefStruct :: struct`
- `MOJOSHADER_astExpressionCallFunction :: struct`
- `MOJOSHADER_astExpressionCast :: struct`
- `MOJOSHADER_astCompilationUnit :: struct`
- `MOJOSHADER_astFunctionParameters :: struct`
- `MOJOSHADER_astFunctionSignature :: struct`
- `MOJOSHADER_astScalarOrArray :: struct`
- `MOJOSHADER_astAnnotations :: struct`
- `MOJOSHADER_astPackOffset :: struct`
- `MOJOSHADER_astVariableLowLevel :: struct`
- `MOJOSHADER_astStructMembers :: struct`
- `MOJOSHADER_astStructDeclaration :: struct`
- `MOJOSHADER_astVariableDeclaration :: struct`
- `MOJOSHADER_astStatement :: struct`
- `MOJOSHADER_astEmptyStatement :: MOJOSHADER_astStatement`
- `MOJOSHADER_astBreakStatement :: MOJOSHADER_astStatement`
- `MOJOSHADER_astContinueStatement :: MOJOSHADER_astStatement`
- `MOJOSHADER_astDiscardStatement :: MOJOSHADER_astStatement`
- `MOJOSHADER_astBlockStatement :: struct`
- `MOJOSHADER_astReturnStatement :: struct`
- `MOJOSHADER_astExpressionStatement :: struct`
- `MOJOSHADER_astIfStatement :: struct`
- `MOJOSHADER_astSwitchCases :: struct`
- `MOJOSHADER_astSwitchStatement :: struct`
- `MOJOSHADER_astWhileStatement :: struct`
- `MOJOSHADER_astDoStatement :: MOJOSHADER_astWhileStatement`
- `MOJOSHADER_astForStatement :: struct`
- `MOJOSHADER_astTypedef :: struct`
- `MOJOSHADER_astTypedefStatement :: struct`
- `MOJOSHADER_astVarDeclStatement :: struct`
- `MOJOSHADER_astStructStatement :: struct`
- `MOJOSHADER_astCompilationUnitFunction :: struct`
- `MOJOSHADER_astCompilationUnitTypedef :: struct`
- `MOJOSHADER_astCompilationUnitStruct :: struct`
- `MOJOSHADER_astCompilationUnitVariable :: struct`
- `MOJOSHADER_astNode :: union`
- `MOJOSHADER_astData :: struct`
- `MOJOSHADER_parseAst :: (srcprofile: *u8, filename: *u8, source: *u8, sourcelen: u32, defs: *MOJOSHADER_preprocessorDefine, define_count: u32, include_open: MOJOSHADER_includeOpen, include_close: MOJOSHADER_includeClose, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_astData #foreign libmojoshader`
- `MOJOSHADER_freeAstData :: (data: *MOJOSHADER_astData) -> void #foreign libmojoshader`
- `MOJOSHADER_irNodeInfo :: struct`
- `MOJOSHADER_irGeneric :: struct`
- `MOJOSHADER_irExprInfo :: struct`
- `MOJOSHADER_irConstant :: struct`
- `MOJOSHADER_irTemp :: struct`
- `MOJOSHADER_irBinOp :: struct`
- `MOJOSHADER_irMemory :: struct`
- `MOJOSHADER_irCall :: struct`
- `MOJOSHADER_irESeq :: struct`
- `MOJOSHADER_irArray :: struct`
- `MOJOSHADER_irConvert :: struct`
- `MOJOSHADER_irSwizzle :: struct`
- `MOJOSHADER_irConstruct :: struct`
- `MOJOSHADER_irExpression :: union`
- `MOJOSHADER_irMove :: struct`
- `MOJOSHADER_irExprStmt :: struct`
- `MOJOSHADER_irJump :: struct`
- `MOJOSHADER_irCJump :: struct`
- `MOJOSHADER_irSeq :: struct`
- `MOJOSHADER_irLabel :: struct`
- `MOJOSHADER_irDiscard :: MOJOSHADER_irGeneric`
- `MOJOSHADER_irStatement :: union`
- `MOJOSHADER_irExprList :: struct`
- `MOJOSHADER_irMisc :: union`
- `MOJOSHADER_irNode :: union`
- `MOJOSHADER_compileData :: struct`
- `MOJOSHADER_compile :: (srcprofile: *u8, filename: *u8, source: *u8, sourcelen: u32, defs: *MOJOSHADER_preprocessorDefine, define_count: u32, include_open: MOJOSHADER_includeOpen, include_close: MOJOSHADER_includeClose, m: MOJOSHADER_malloc, f: MOJOSHADER_free, d: *void) -> *MOJOSHADER_compileData #foreign libmojoshader`
- `MOJOSHADER_freeCompileData :: (data: *MOJOSHADER_compileData) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetProcAddress :: #type (a0: *u8, a1: *void) -> *void #c_call`
- `MOJOSHADER_glContext :: struct`
- `MOJOSHADER_glShader :: struct`
- `MOJOSHADER_glProgram :: struct`
- `MOJOSHADER_glAvailableProfiles :: (lookup: MOJOSHADER_glGetProcAddress, lookup_d: *void, profs: **u8, size: s32, m: MOJOSHADER_malloc, f: MOJOSHADER_free, malloc_d: *void) -> s32 #foreign libmojoshader`
- `MOJOSHADER_glBestProfile :: (lookup: MOJOSHADER_glGetProcAddress, lookup_d: *void, m: MOJOSHADER_malloc, f: MOJOSHADER_free, malloc_d: *void) -> *u8 #foreign libmojoshader`
- `MOJOSHADER_glCreateContext :: (profile: *u8, lookup: MOJOSHADER_glGetProcAddress, lookup_d: *void, m: MOJOSHADER_malloc, f: MOJOSHADER_free, malloc_d: *void) -> *MOJOSHADER_glContext #foreign libmojoshader`
- `MOJOSHADER_glMakeContextCurrent :: (ctx: *MOJOSHADER_glContext) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetError :: () -> *u8 #foreign libmojoshader`
- `MOJOSHADER_glMaxUniforms :: (shader_type: MOJOSHADER_shaderType) -> s32 #foreign libmojoshader`
- `MOJOSHADER_glCompileShader :: (tokenbuf: *u8, bufsize: u32, swiz: *MOJOSHADER_swizzle, swizcount: u32, smap: *MOJOSHADER_samplerMap, smapcount: u32) -> *MOJOSHADER_glShader #foreign libmojoshader`
- `MOJOSHADER_glGetShaderParseData :: (shader: *MOJOSHADER_glShader) -> *MOJOSHADER_parseData #foreign libmojoshader`
- `MOJOSHADER_glLinkProgram :: (vshader: *MOJOSHADER_glShader, pshader: *MOJOSHADER_glShader) -> *MOJOSHADER_glProgram #foreign libmojoshader`
- `MOJOSHADER_glBindProgram :: (program: *MOJOSHADER_glProgram) -> void #foreign libmojoshader`
- `MOJOSHADER_glBindShaders :: (vshader: *MOJOSHADER_glShader, pshader: *MOJOSHADER_glShader) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetVertexShaderUniformF :: (idx: u32, data: *float32, vec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetVertexShaderUniformF :: (idx: u32, data: *float32, vec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetVertexShaderUniformI :: (idx: u32, data: *s32, ivec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetVertexShaderUniformI :: (idx: u32, data: *s32, ivec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetVertexShaderUniformB :: (idx: u32, data: *s32, bcount: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetVertexShaderUniformB :: (idx: u32, data: *s32, bcount: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetPixelShaderUniformF :: (idx: u32, data: *float32, vec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetPixelShaderUniformF :: (idx: u32, data: *float32, vec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetPixelShaderUniformI :: (idx: u32, data: *s32, ivec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetPixelShaderUniformI :: (idx: u32, data: *s32, ivec4count: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetPixelShaderUniformB :: (idx: u32, data: *s32, bcount: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetPixelShaderUniformB :: (idx: u32, data: *s32, bcount: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetLegacyBumpMapEnv :: (sampler: u32, mat00: float32, mat01: float32, mat10: float32, mat11: float32, lscale: float32, loffset: float32) -> void #foreign libmojoshader`
- `MOJOSHADER_glGetVertexAttribLocation :: (usage: MOJOSHADER_usage, index: s32) -> s32 #foreign libmojoshader`
- `MOJOSHADER_glSetVertexAttribute :: (usage: MOJOSHADER_usage, index: s32, size: u32, type: MOJOSHADER_attributeType, normalized: s32, stride: u32, ptr: *void) -> void #foreign libmojoshader`
- `MOJOSHADER_glSetVertexAttribDivisor :: (usage: MOJOSHADER_usage, index: s32, divisor: u32) -> void #foreign libmojoshader`
- `MOJOSHADER_glProgramReady :: () -> void #foreign libmojoshader`
- `MOJOSHADER_glDeleteProgram :: (program: *MOJOSHADER_glProgram) -> void #foreign libmojoshader`
- `MOJOSHADER_glDeleteShader :: (shader: *MOJOSHADER_glShader) -> void #foreign libmojoshader`
- `MOJOSHADER_glDestroyContext :: (ctx: *MOJOSHADER_glContext) -> void #foreign libmojoshader`

## Public type members

#### `MOJOSHADER_uniform` (struct)

- `type: MOJOSHADER_uniformType`
- `index: s32`
- `array_count: s32`
- `constant: s32`
- `name: *u8`

#### `MOJOSHADER_constant` (struct)

- `type: MOJOSHADER_uniformType`
- `index: s32`
- `value: union { f: [4]float32; i: [4]s32; b: s32 }`

#### `MOJOSHADER_sampler` (struct)

- `type: MOJOSHADER_samplerType`
- `index: s32`
- `name: *u8`
- `texbem: s32`

#### `MOJOSHADER_samplerMap` (struct)

- `index: s32`
- `type: MOJOSHADER_samplerType`

#### `MOJOSHADER_attribute` (struct)

- `usage: MOJOSHADER_usage`
- `index: s32`
- `name: *u8`

#### `MOJOSHADER_swizzle` (struct)

- `usage: MOJOSHADER_usage`
- `index: u32`
- `swizzles: [4] u8`

#### `MOJOSHADER_symbolTypeInfo` (struct)

- `parameter_class: MOJOSHADER_symbolClass`
- `parameter_type: MOJOSHADER_symbolType`
- `rows: u32`
- `columns: u32`
- `elements: u32`
- `member_count: u32`
- `members: *MOJOSHADER_symbolStructMember`

#### `MOJOSHADER_symbolStructMember` (struct)

- `name: *u8`
- `info: MOJOSHADER_symbolTypeInfo`

#### `MOJOSHADER_symbol` (struct)

- `name: *u8`
- `register_set: MOJOSHADER_symbolRegisterSet`
- `register_index: u32`
- `register_count: u32`
- `info: MOJOSHADER_symbolTypeInfo`

#### `MOJOSHADER_error` (struct)

- `error: *u8`
- `filename: *u8`
- `error_position: s32`

#### `MOJOSHADER_preshaderOperand` (struct)

- `type: MOJOSHADER_preshaderOperandType`
- `index: u32`
- `array_register_count: u32`
- `array_registers: *u32`

#### `MOJOSHADER_preshaderInstruction` (struct)

- `opcode: MOJOSHADER_preshaderOpcode`
- `element_count: u32`
- `operand_count: u32`
- `operands: [4] MOJOSHADER_preshaderOperand`

#### `MOJOSHADER_preshader` (struct)

- `literal_count: u32`
- `literals: *float64`
- `temp_count: u32`
- `symbol_count: u32`
- `symbols: *MOJOSHADER_symbol`
- `instruction_count: u32`
- `instructions: *MOJOSHADER_preshaderInstruction`
- `register_count: u32`
- `registers: *float32`
- `malloc: MOJOSHADER_malloc`
- `free: MOJOSHADER_free`
- `malloc_data: *void`

#### `MOJOSHADER_parseData` (struct)

- `error_count: s32`
- `errors: *MOJOSHADER_error`
- `profile: *u8`
- `output: *u8`
- `output_len: s32`
- `instruction_count: s32`
- `shader_type: MOJOSHADER_shaderType`
- `major_ver: s32`
- `minor_ver: s32`
- `mainfn: *u8`
- `uniform_count: s32`
- `uniforms: *MOJOSHADER_uniform`
- `constant_count: s32`
- `constants: *MOJOSHADER_constant`
- `sampler_count: s32`
- `samplers: *MOJOSHADER_sampler`
- `attribute_count: s32`
- `attributes: *MOJOSHADER_attribute`
- `output_count: s32`
- `outputs: *MOJOSHADER_attribute`
- `swizzle_count: s32`
- `swizzles: *MOJOSHADER_swizzle`
- `symbol_count: s32`
- `symbols: *MOJOSHADER_symbol`
- `preshader: *MOJOSHADER_preshader`
- `malloc: MOJOSHADER_malloc`
- `free: MOJOSHADER_free`
- `malloc_data: *void`

#### `MOJOSHADER_preprocessorDefine` (struct)

- `identifier: *u8`
- `definition: *u8`

#### `MOJOSHADER_preprocessData` (struct)

- `error_count: s32`
- `errors: *MOJOSHADER_error`
- `output: *u8`
- `output_len: s32`
- `malloc: MOJOSHADER_malloc`
- `free: MOJOSHADER_free`
- `malloc_data: *void`

#### `MOJOSHADER_astDataTypeStructMember` (struct)

- `datatype: *MOJOSHADER_astDataType`
- `identifier: *u8`

#### `MOJOSHADER_astDataTypeStruct` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `members: *MOJOSHADER_astDataTypeStructMember`
- `member_count: s32`

#### `MOJOSHADER_astDataTypeArray` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `base: *MOJOSHADER_astDataType`
- `elements: s32`

#### `MOJOSHADER_astDataTypeMatrix` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `base: *MOJOSHADER_astDataType`
- `rows: s32`
- `columns: s32`

#### `MOJOSHADER_astDataTypeBuffer` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `base: *MOJOSHADER_astDataType`

#### `MOJOSHADER_astDataTypeFunction` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `retval: *MOJOSHADER_astDataType`
- `params: **MOJOSHADER_astDataType`
- `num_params: s32`
- `intrinsic: s32`

#### `MOJOSHADER_astDataTypeUser` (struct)

- `type: MOJOSHADER_astDataTypeType`
- `details: *MOJOSHADER_astDataType`
- `name: *u8`

#### `MOJOSHADER_astDataType` (union)

- `type: MOJOSHADER_astDataTypeType`
- `array: MOJOSHADER_astDataTypeArray`
- `structure: MOJOSHADER_astDataTypeStruct`
- `vector: MOJOSHADER_astDataTypeVector`
- `matrix: MOJOSHADER_astDataTypeMatrix`
- `buffer: MOJOSHADER_astDataTypeBuffer`
- `user: MOJOSHADER_astDataTypeUser`
- `function: MOJOSHADER_astDataTypeFunction`

#### `MOJOSHADER_astNodeInfo` (struct)

- `type: MOJOSHADER_astNodeType`
- `filename: *u8`
- `line: u32`

#### `MOJOSHADER_astGeneric` (struct)

- `ast: MOJOSHADER_astNodeInfo`

#### `MOJOSHADER_astExpression` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`

#### `MOJOSHADER_astArguments` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `argument: *MOJOSHADER_astExpression`
- `next: *MOJOSHADER_astArguments`

#### `MOJOSHADER_astExpressionUnary` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `operand: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astExpressionBinary` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `left: *MOJOSHADER_astExpression`
- `right: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astExpressionTernary` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `left: *MOJOSHADER_astExpression`
- `center: *MOJOSHADER_astExpression`
- `right: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astExpressionIdentifier` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `identifier: *u8`
- `index: s32`

#### `MOJOSHADER_astExpressionIntLiteral` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `value: s32`

#### `MOJOSHADER_astExpressionFloatLiteral` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `value: float64`

#### `MOJOSHADER_astExpressionStringLiteral` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `_string: *u8`

#### `MOJOSHADER_astExpressionBooleanLiteral` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `value: s32`

#### `MOJOSHADER_astExpressionConstructor` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `args: *MOJOSHADER_astArguments`

#### `MOJOSHADER_astExpressionDerefStruct` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `identifier: *MOJOSHADER_astExpression`
- `member: *u8`
- `isswizzle: s32`
- `member_index: s32`

#### `MOJOSHADER_astExpressionCallFunction` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `identifier: *MOJOSHADER_astExpressionIdentifier`
- `args: *MOJOSHADER_astArguments`

#### `MOJOSHADER_astExpressionCast` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `operand: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astCompilationUnit` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astCompilationUnit`

#### `MOJOSHADER_astFunctionParameters` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `input_modifier: MOJOSHADER_astInputModifier`
- `identifier: *u8`
- `semantic: *u8`
- `interpolation_modifier: MOJOSHADER_astInterpolationModifier`
- `initializer: *MOJOSHADER_astExpression`
- `next: *MOJOSHADER_astFunctionParameters`

#### `MOJOSHADER_astFunctionSignature` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `identifier: *u8`
- `params: *MOJOSHADER_astFunctionParameters`
- `storage_class: MOJOSHADER_astFunctionStorageClass`
- `semantic: *u8`

#### `MOJOSHADER_astScalarOrArray` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `identifier: *u8`
- `isarray: s32`
- `dimension: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astAnnotations` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `initializer: *MOJOSHADER_astExpression`
- `next: *MOJOSHADER_astAnnotations`

#### `MOJOSHADER_astPackOffset` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `ident1: *u8`
- `ident2: *u8`

#### `MOJOSHADER_astVariableLowLevel` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `packoffset: *MOJOSHADER_astPackOffset`
- `register_name: *u8`

#### `MOJOSHADER_astStructMembers` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `semantic: *u8`
- `details: *MOJOSHADER_astScalarOrArray`
- `interpolation_mod: MOJOSHADER_astInterpolationModifier`
- `next: *MOJOSHADER_astStructMembers`

#### `MOJOSHADER_astStructDeclaration` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `name: *u8`
- `members: *MOJOSHADER_astStructMembers`

#### `MOJOSHADER_astVariableDeclaration` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `attributes: s32`
- `datatype: *MOJOSHADER_astDataType`
- `anonymous_datatype: *MOJOSHADER_astStructDeclaration`
- `details: *MOJOSHADER_astScalarOrArray`
- `semantic: *u8`
- `annotations: *MOJOSHADER_astAnnotations`
- `initializer: *MOJOSHADER_astExpression`
- `lowlevel: *MOJOSHADER_astVariableLowLevel`
- `next: *MOJOSHADER_astVariableDeclaration`

#### `MOJOSHADER_astStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`

#### `MOJOSHADER_astBlockStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `statements: *MOJOSHADER_astStatement`

#### `MOJOSHADER_astReturnStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `expr: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astExpressionStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `expr: *MOJOSHADER_astExpression`

#### `MOJOSHADER_astIfStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `attributes: s32`
- `expr: *MOJOSHADER_astExpression`
- `statement: *MOJOSHADER_astStatement`
- `else_statement: *MOJOSHADER_astStatement`

#### `MOJOSHADER_astSwitchCases` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `expr: *MOJOSHADER_astExpression`
- `statement: *MOJOSHADER_astStatement`
- `next: *MOJOSHADER_astSwitchCases`

#### `MOJOSHADER_astSwitchStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `attributes: s32`
- `expr: *MOJOSHADER_astExpression`
- `cases: *MOJOSHADER_astSwitchCases`

#### `MOJOSHADER_astWhileStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `unroll: s32`
- `expr: *MOJOSHADER_astExpression`
- `statement: *MOJOSHADER_astStatement`

#### `MOJOSHADER_astForStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `unroll: s32`
- `var_decl: *MOJOSHADER_astVariableDeclaration`
- `initializer: *MOJOSHADER_astExpression`
- `looptest: *MOJOSHADER_astExpression`
- `counter: *MOJOSHADER_astExpression`
- `statement: *MOJOSHADER_astStatement`

#### `MOJOSHADER_astTypedef` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `datatype: *MOJOSHADER_astDataType`
- `isconst: s32`
- `details: *MOJOSHADER_astScalarOrArray`

#### `MOJOSHADER_astTypedefStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `_type_info: *MOJOSHADER_astTypedef`

#### `MOJOSHADER_astVarDeclStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `declaration: *MOJOSHADER_astVariableDeclaration`

#### `MOJOSHADER_astStructStatement` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astStatement`
- `struct_info: *MOJOSHADER_astStructDeclaration`

#### `MOJOSHADER_astCompilationUnitFunction` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astCompilationUnit`
- `declaration: *MOJOSHADER_astFunctionSignature`
- `definition: *MOJOSHADER_astStatement`
- `index: s32`

#### `MOJOSHADER_astCompilationUnitTypedef` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astCompilationUnit`
- `_type_info: *MOJOSHADER_astTypedef`

#### `MOJOSHADER_astCompilationUnitStruct` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astCompilationUnit`
- `struct_info: *MOJOSHADER_astStructDeclaration`

#### `MOJOSHADER_astCompilationUnitVariable` (struct)

- `ast: MOJOSHADER_astNodeInfo`
- `next: *MOJOSHADER_astCompilationUnit`
- `declaration: *MOJOSHADER_astVariableDeclaration`

#### `MOJOSHADER_astNode` (union)

- `ast: MOJOSHADER_astNodeInfo`
- `generic: MOJOSHADER_astGeneric`
- `expression: MOJOSHADER_astExpression`
- `arguments: MOJOSHADER_astArguments`
- `unary: MOJOSHADER_astExpressionUnary`
- `binary: MOJOSHADER_astExpressionBinary`
- `ternary: MOJOSHADER_astExpressionTernary`
- `identifier: MOJOSHADER_astExpressionIdentifier`
- `intliteral: MOJOSHADER_astExpressionIntLiteral`
- `floatliteral: MOJOSHADER_astExpressionFloatLiteral`
- `stringliteral: MOJOSHADER_astExpressionStringLiteral`
- `boolliteral: MOJOSHADER_astExpressionBooleanLiteral`
- `constructor: MOJOSHADER_astExpressionConstructor`
- `derefstruct: MOJOSHADER_astExpressionDerefStruct`
- `callfunc: MOJOSHADER_astExpressionCallFunction`
- `_cast: MOJOSHADER_astExpressionCast`
- `compunit: MOJOSHADER_astCompilationUnit`
- `params: MOJOSHADER_astFunctionParameters`
- `funcsig: MOJOSHADER_astFunctionSignature`
- `soa: MOJOSHADER_astScalarOrArray`
- `annotations: MOJOSHADER_astAnnotations`
- `packoffset: MOJOSHADER_astPackOffset`
- `varlowlevel: MOJOSHADER_astVariableLowLevel`
- `structmembers: MOJOSHADER_astStructMembers`
- `structdecl: MOJOSHADER_astStructDeclaration`
- `vardecl: MOJOSHADER_astVariableDeclaration`
- `stmt: MOJOSHADER_astStatement`
- `emptystmt: MOJOSHADER_astEmptyStatement`
- `breakstmt: MOJOSHADER_astBreakStatement`
- `contstmt: MOJOSHADER_astContinueStatement`
- `discardstmt: MOJOSHADER_astDiscardStatement`
- `blockstmt: MOJOSHADER_astBlockStatement`
- `returnstmt: MOJOSHADER_astReturnStatement`
- `exprstmt: MOJOSHADER_astExpressionStatement`
- `ifstmt: MOJOSHADER_astIfStatement`
- `cases: MOJOSHADER_astSwitchCases`
- `switchstmt: MOJOSHADER_astSwitchStatement`
- `whilestmt: MOJOSHADER_astWhileStatement`
- `dostmt: MOJOSHADER_astDoStatement`
- `forstmt: MOJOSHADER_astForStatement`
- `typdef: MOJOSHADER_astTypedef`
- `typedefstmt: MOJOSHADER_astTypedefStatement`
- `vardeclstmt: MOJOSHADER_astVarDeclStatement`
- `structstmt: MOJOSHADER_astStructStatement`
- `funcunit: MOJOSHADER_astCompilationUnitFunction`
- `typedefunit: MOJOSHADER_astCompilationUnitTypedef`
- `structunit: MOJOSHADER_astCompilationUnitStruct`
- `varunit: MOJOSHADER_astCompilationUnitVariable`

#### `MOJOSHADER_astData` (struct)

- `error_count: s32`
- `errors: *MOJOSHADER_error`
- `source_profile: *u8`
- `ast: *MOJOSHADER_astNode`
- `malloc: MOJOSHADER_malloc`
- `free: MOJOSHADER_free`
- `malloc_data: *void`
- `opaque: *void`

#### `MOJOSHADER_irNodeInfo` (struct)

- `type: MOJOSHADER_irNodeType`
- `filename: *u8`
- `line: u32`

#### `MOJOSHADER_irGeneric` (struct)

- `ir: MOJOSHADER_irNodeInfo`

#### `MOJOSHADER_irExprInfo` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `type: MOJOSHADER_astDataTypeType`
- `elements: s32`

#### `MOJOSHADER_irConstant` (struct)

- `info: MOJOSHADER_irExprInfo`
- `anon_struct_10 :: union { ival: [16]s32; fval: [16]float32 }`
- `value: anon_struct_10`

#### `MOJOSHADER_irTemp` (struct)

- `info: MOJOSHADER_irExprInfo`
- `index: s32`

#### `MOJOSHADER_irBinOp` (struct)

- `info: MOJOSHADER_irExprInfo`
- `op: MOJOSHADER_irBinOpType`
- `left: *MOJOSHADER_irExpression`
- `right: *MOJOSHADER_irExpression`

#### `MOJOSHADER_irMemory` (struct)

- `info: MOJOSHADER_irExprInfo`
- `index: s32`

#### `MOJOSHADER_irCall` (struct)

- `info: MOJOSHADER_irExprInfo`
- `index: s32`
- `args: *MOJOSHADER_irExprList`

#### `MOJOSHADER_irESeq` (struct)

- `info: MOJOSHADER_irExprInfo`
- `stmt: *MOJOSHADER_irStatement`
- `expr: *MOJOSHADER_irExpression`

#### `MOJOSHADER_irArray` (struct)

- `info: MOJOSHADER_irExprInfo`
- `array: *MOJOSHADER_irExpression`
- `element: *MOJOSHADER_irExpression`

#### `MOJOSHADER_irConvert` (struct)

- `info: MOJOSHADER_irExprInfo`
- `expr: *MOJOSHADER_irExpression`

#### `MOJOSHADER_irSwizzle` (struct)

- `info: MOJOSHADER_irExprInfo`
- `expr: *MOJOSHADER_irExpression`
- `channels: [4] u8`

#### `MOJOSHADER_irConstruct` (struct)

- `info: MOJOSHADER_irExprInfo`
- `args: *MOJOSHADER_irExprList`

#### `MOJOSHADER_irExpression` (union)

- `ir: MOJOSHADER_irNodeInfo`
- `info: MOJOSHADER_irExprInfo`
- `constant: MOJOSHADER_irConstant`
- `temp: MOJOSHADER_irTemp`
- `binop: MOJOSHADER_irBinOp`
- `memory: MOJOSHADER_irMemory`
- `call: MOJOSHADER_irCall`
- `eseq: MOJOSHADER_irESeq`
- `array: MOJOSHADER_irArray`
- `convert: MOJOSHADER_irConvert`
- `swizzle: MOJOSHADER_irSwizzle`
- `construct: MOJOSHADER_irConstruct`

#### `MOJOSHADER_irMove` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `dst: *MOJOSHADER_irExpression`
- `src: *MOJOSHADER_irExpression`
- `writemask: s32`

#### `MOJOSHADER_irExprStmt` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `expr: *MOJOSHADER_irExpression`

#### `MOJOSHADER_irJump` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `label: s32`

#### `MOJOSHADER_irCJump` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `cond: MOJOSHADER_irConditionType`
- `left: *MOJOSHADER_irExpression`
- `right: *MOJOSHADER_irExpression`
- `iftrue: s32`
- `iffalse: s32`

#### `MOJOSHADER_irSeq` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `first: *MOJOSHADER_irStatement`
- `next: *MOJOSHADER_irStatement`

#### `MOJOSHADER_irLabel` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `index: s32`

#### `MOJOSHADER_irStatement` (union)

- `ir: MOJOSHADER_irNodeInfo`
- `generic: MOJOSHADER_irGeneric`
- `move: MOJOSHADER_irMove`
- `expr: MOJOSHADER_irExprStmt`
- `jump: MOJOSHADER_irJump`
- `cjump: MOJOSHADER_irCJump`
- `seq: MOJOSHADER_irSeq`
- `label: MOJOSHADER_irLabel`
- `discard: MOJOSHADER_irDiscard`

#### `MOJOSHADER_irExprList` (struct)

- `ir: MOJOSHADER_irNodeInfo`
- `expr: *MOJOSHADER_irExpression`
- `next: *MOJOSHADER_irExprList`

#### `MOJOSHADER_irMisc` (union)

- `ir: MOJOSHADER_irNodeInfo`
- `generic: MOJOSHADER_irGeneric`
- `exprlist: MOJOSHADER_irExprList`

#### `MOJOSHADER_irNode` (union)

- `ir: MOJOSHADER_irNodeInfo`
- `generic: MOJOSHADER_irGeneric`
- `expr: MOJOSHADER_irExpression`
- `stmt: MOJOSHADER_irStatement`
- `misc: MOJOSHADER_irMisc`

#### `MOJOSHADER_compileData` (struct)

- `error_count: s32`
- `errors: *MOJOSHADER_error`
- `warning_count: s32`
- `warnings: *MOJOSHADER_error`
- `source_profile: *u8`
- `output: *u8`
- `output_len: s32`
- `symbol_count: s32`
- `symbols: *MOJOSHADER_symbol`
- `malloc: MOJOSHADER_malloc`
- `free: MOJOSHADER_free`
- `malloc_data: *void`

#### `MOJOSHADER_glContext` (struct)

No public fields; opaque handle marker.

#### `MOJOSHADER_glShader` (struct)

No public fields; opaque handle marker.

#### `MOJOSHADER_glProgram` (struct)

No public fields; opaque handle marker.

## Behavioral contract and constraints

This package binds MojoShader, which translates shader bytecode to target shader languages. The Jai declarations expose native result structures and library functions; returned parse data must be released with MojoShader's matching free routine. Parse/translation errors are conveyed by MojoShader diagnostics/results. The native library is platform-specific and its version is not pinned by the module declarations.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: `libmojoshader`, `mojoshader`.
