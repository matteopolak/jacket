" Vim syntax file
" Language: Jai
" The fallback highlighting; jailsp's semantic tokens are drawn on top of it. Keep the word
" lists in step with editors/vscode/scripts/build-grammar.ts.

if exists('b:current_syntax')
  finish
endif

syn case match

syn keyword jaiTodo contained TODO FIXME XXX NOTE HACK
syn region jaiBlockComment start=+/\*+ end=+\*/+ contains=jaiBlockComment,jaiTodo,@Spell
syn match jaiLineComment +//.*$+ contains=jaiTodo,@Spell

syn keyword jaiConditional if ifx then else case
syn keyword jaiRepeat for while
syn keyword jaiStatement break continue return remove defer push_context using
syn keyword jaiKeyword struct union enum enum_flags operator inline no_inline interface
syn keyword jaiOperator cast xx
syn match jaiCastFlag +\%(\<\%(cast\|xx\)\>\s*\)\@<=,\s*\%(no_check\|trunc\|truncate\|force\)\>+
syn keyword jaiBoolean true false
syn keyword jaiConstant null
syn keyword jaiBuiltin it it_index context
syn keyword jaiType bool int float float32 float64 string void s8 s16 s32 s64 s128 u8 u16 u32 u64 u128 Type Any Code
syn match jaiBuiltinFunc +\<\%(size_of\|type_of\|type_info\|initializer_of\|is_constant\|align_of\|offset_of\)\>\ze\s*(+

" #import and friends, conditional directives, modifiers, and any other #name.
syn match jaiDirective +#\h\w*+
syn match jaiInclude +#\%(import\|load\|foreign_library\|foreign_system_library\|system_library\|library\)\>+
syn match jaiPreCondit +#\%(if\|ifx\|else\|complete\|through\|ifdef\|endif\)\>+
syn match jaiRun +#\%(run\|insert\|code\|type\|bake_arguments\|bake_constants\|char\|string\)\>+
syn match jaiSpecialDirective +#\%(file\|line\|filepath\|this\|caller_location\|caller_code\|procedure_name\|location\|compile_time\|exists\|bytes\|procedure_of_call\|placeholder\)\>+

" Notes: @foo and @"foo bar".
syn match jaiNote +@\%(\h\w*\|"\%([^"\\]\|\\.\)*"\)+

" Polymorphic names: $T, $$x.
syn match jaiPolymorph +\$\$\?\h\w*+

" Strings. A here-string runs from `#string TAG` to a line starting with TAG.
syn match jaiEscape contained +\\\%([nrt0eabfv%\\"'"]\|x\x\{2}\|d\d\{3}\|u\x\{4}\|U\x\{8}\)+
syn match jaiFormat contained +%\%(\d\+\)\?\%(%\|\.\d\+\)\?+
syn region jaiString start=+"+ skip=+\\.+ end=+"+ contains=jaiEscape,jaiFormat,@Spell
syn region jaiHereString matchgroup=jaiDirective start=+#string\%(\s*,\s*\%(\\.\|\h\w*\)\)*\s*\z(\h\w*\)+ matchgroup=Label end=+^\s*\z1\>+ contains=jaiFormat
syn match jaiChar +#char\s*"\%([^"\\]\|\\.\)*"+ contains=jaiDirective,jaiEscape

" Numbers: 0x, 0b, 0h, separators, floats and exponents.
syn match jaiNumber +\<\d[0-9_]*\>+
syn match jaiNumber +\<0[xX]\x[0-9A-Fa-f_]*\>+
syn match jaiNumber +\<0[bB][01][01_]*\>+
syn match jaiNumber +\<0[hH]\x[0-9A-Fa-f_]*\>+
syn match jaiFloat +\<\d[0-9_]*\.\%(\.\|\h\)\@!\%(\d[0-9_]*\)\?\%([eE][+-]\?\d[0-9_]*\)\?+
syn match jaiFloat +\<\d[0-9_]*[eE][+-]\?\d[0-9_]*\>+
syn match jaiFloat +\%(\w\|)\|\]\|\.\)\@<!\.\d[0-9_]*\%([eE][+-]\?\d[0-9_]*\)\?\>+

" Declarations: `name ::`, `name :=` and `name :`.
syn match jaiProcDecl +\h\w*\ze\s*::\s*\%(inline\s\+\|no_inline\s\+\)\?(+
syn match jaiTypeDecl +\h\w*\ze\s*::\s*\%(struct\|union\|enum\|enum_flags\)\>+
syn match jaiCall +\h\w*\ze\s*(+

syn sync minlines=200

hi def link jaiTodo Todo
hi def link jaiBlockComment Comment
hi def link jaiLineComment Comment
hi def link jaiConditional Conditional
hi def link jaiRepeat Repeat
hi def link jaiStatement Statement
hi def link jaiKeyword Keyword
hi def link jaiOperator Operator
hi def link jaiCastFlag Special
hi def link jaiBoolean Boolean
hi def link jaiConstant Constant
hi def link jaiBuiltin Identifier
hi def link jaiType Type
hi def link jaiBuiltinFunc Function
hi def link jaiDirective PreProc
hi def link jaiInclude Include
hi def link jaiPreCondit PreCondit
hi def link jaiRun Macro
hi def link jaiSpecialDirective Special
hi def link jaiNote Label
hi def link jaiPolymorph Type
hi def link jaiEscape SpecialChar
hi def link jaiFormat SpecialChar
hi def link jaiString String
hi def link jaiHereString String
hi def link jaiChar String
hi def link jaiNumber Number
hi def link jaiFloat Float
hi def link jaiProcDecl Function
hi def link jaiTypeDecl Type
hi def link jaiCall Function

let b:current_syntax = 'jai'
