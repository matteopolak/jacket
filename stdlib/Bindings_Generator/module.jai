#module_parameters(TARGET_OS: type_of(OS) = xx OS);

#load "../internal/developer-tools/generator-active-state.jai";

#load "active-context.jai";

DO_DEBUG :: false;
DEBUG_ALIGNMENT :: false;
DEBUG_BITFIELDS :: false;
LIBCLANG_VERSION :: 21;
TAB_STRING :: " ";

Declaration_Visitor :: #type (decl: *Declaration, parent_decl: *Declaration) -> Declaration_Visit_Result;
Declaration_Visitor_With_Data :: #type (decl: *Declaration, parent_decl: *Declaration, user_data: *void) -> Declaration_Visit_Result;
Get_Function_Arguments_For_Printing_Func :: #type (func: *Function, type: *Function_Type, mode: Arg_Print_Mode) -> [] *Declaration;

#load "model-flags.jai";
#load "native-model.jai";
#load "native-parser-model.jai";
#load "native-parser-api.jai";
#load "native-extended-model.jai";
#load "native-extended-api.jai";
#load "type-model.jai";
#load "declaration-model.jai";
#load "model-helpers.jai";
#load "generation-options.jai";
#load "naming-helpers.jai";
#load "modifiers.jai";
#load "compatibility-names.jai";
#load "compatibility-tokens.jai";
#load "platform-helpers.jai";
#load "emission-helpers.jai";
#load "../internal/developer-tools/bindings-driver.jai";
#load "../internal/developer-tools/binding-literals.jai";
#load "../internal/developer-tools/binding-tree.jai";
#load "../internal/developer-tools/binding-native-model.jai";
#load "../internal/developer-tools/binding-cpp-model.jai";
#load "../internal/developer-tools/binding-native-options.jai";
#load "../internal/developer-tools/cpp-fragments.jai";
#load "../internal/developer-tools/binding-type-emission.jai";
#load "value-emission.jai";
#load "macro-expressions.jai";
#load "generator-run.jai";
#load "../internal/developer-tools/binding-cursor-source.jai";
#load "../internal/developer-tools/binding-type-queries.jai";
#load "../internal/developer-tools/binding-expression-normalization.jai";
#load "../internal/developer-tools/binding-declaration-visitors.jai";
#load "../internal/developer-tools/binding-model-properties.jai";
#load "../internal/developer-tools/binding-native-properties.jai";
