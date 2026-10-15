#!/usr/bin/env python3
"""Generate canonical public data shapes from reviewed declarations only.

No implementation bodies, corpus inputs or SDK implementation source are read.
Files are grouped by model responsibility, independently of inventory ordering.
"""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/specs/stdlib/modules/bindings-generator.md"


def aggregate(text, name, platform=None):
    type_text = text
    inventory = text
    if platform:
        type_marker = f"### {platform}-specific type declarations"
        inventory_marker = f"### {platform}-specific declarations"
        type_text = text.split(type_marker, 1)[1]
        inventory = text.split(inventory_marker, 1)[1].split("## Public type members", 1)[0]
        if platform == "Unix":
            type_text = type_text.split("### Windows-specific type declarations", 1)[0]
            inventory = inventory.split("### Windows-specific declarations", 1)[0]
    pattern = rf"^#### `{re.escape(name)}` \([^\n]+\)\n(.*?)(?=^#### |^### |^## |\Z)"
    match = re.search(pattern, type_text, re.MULTILINE | re.DOTALL)
    if not match and platform:
        match = re.search(pattern, text, re.MULTILINE | re.DOTALL)
    if not match:
        raise ValueError(f"Missing reviewed aggregate {name}")
    block = match[1]
    literal = re.search(r"```jai\n(.*?)\n```", block, re.DOTALL)
    if literal:
        return literal[1] + "\n"
    declaration = re.search(rf"^- `{re.escape(name)} :: ([^`]+)`", inventory, re.MULTILINE)
    kind = declaration[1] if declaration else "struct"
    fields = re.findall(r"^- `([^`]+)`", block, re.MULTILINE)
    output = [f"{name} :: {kind} {{"]
    for field in fields:
        if field == "macro_value_result: enum {":
            field = "macro_value_result: enum { NEEDS_CHECKING; KEEP; DISCARD; }"
        if name == "Library_Info" and field.startswith("handle:"):
            field = "#if OS == .WINDOWS { handle: W.HMODULE; } else { handle: *void; }"
        if field.endswith("{"):
            raise ValueError(f"Incomplete reviewed shape {name}: {field}")
        output.append("    " + field + ("" if field.endswith((";", "}")) else ";"))
    output.append("}")
    return "\n".join(output) + "\n"


def main():
    text = SPEC.read_text()
    groups = {
        "type-model.jai": ["CType", "Function_Type", "Expression", "Type_Reference", "Literal", "Macro_Value",
                           "Unary_Operator", "Binary_Operator", "Paren_Expression", "Struct_Literal"],
        "declaration-model.jai": ["Location", "Comment", "Declaration", "Block", "Namespace", "Struct",
                                  "Declaration_Overlay", "Struct_Extra_Info", "Bitfield", "Bitfield_Status",
                                  "Enum", "Typedef", "Function", "ComUuid"],
        "generation-options.jai": ["Generate_Bindings_Options", "Library", "Library_Info", "Bridge_Data",
                                   "Cpp_Functions_Visitor_Data"],
        "model-flags.jai": ["C_Kind", "Number_Flags", "Declaration_Flags", "Strip_Flags", "Mimic_Spacing_Flags",
                            "Print_Decl_Flags", "Arg_Print_Mode", "Declaration_Visit_Result"],
        "native-model.jai": ["CXToken", "CX_StorageClass", "CXTypeLayoutError"],
        "native-parser-model.jai": ["CXString", "CXCursorKind", "CXCursor", "CXTypeKind", "CXType",
                                     "CXErrorCode", "CXTranslationUnitImpl", "CXUnsavedFile", "CXChildVisitResult",
                                     "CXSourceLocation", "CXSourceRange", "CXDiagnosticSeverity", "CXIndexOptions", "CXEvalResultKind"],
    }
    directory = ROOT / "stdlib/Bindings_Generator"
    directory.mkdir(exist_ok=True)
    for filename, names in groups.items():
        source = "// Public data contract from the reviewed specification; private algorithms are independent.\n\n"
        if filename == "native-model.jai":
            source += "CXFile :: *void;\n\n"
        for name in names:
            if filename in {"native-model.jai", "native-parser-model.jai"}:
                unix, windows = aggregate(text, name, "Unix"), aggregate(text, name, "Windows")
                if unix == windows:
                    source += unix + "\n"
                else:
                    source += "#if OS == .WINDOWS {\n" + windows + "} else {\n" + unix + "}\n\n"
            else:
                source += aggregate(text, name) + "\n"
        if filename == "generation-options.jai":
            source += '\n#scope_file\n#if OS == .WINDOWS W :: #import "Windows";\n'
        (directory / filename).write_text(source)
    unix_inventory = text.split("### Unix-specific declarations", 1)[1].split("### Windows-specific declarations", 1)[0]
    windows_inventory = text.split("### Windows-specific declarations", 1)[1].split("## Public type members", 1)[0]
    api_names = ["CINDEX_VERSION_MAJOR", "CINDEX_VERSION_MINOR", "CXIndex", "CXTranslationUnit", "CXClientData",
                 "CXCursorVisitor", "clang_createIndex", "clang_disposeIndex", "clang_parseTranslationUnit2",
                 "clang_disposeTranslationUnit", "clang_getTranslationUnitCursor", "clang_visitChildren",
                 "clang_getCursorType", "clang_getCursorSpelling", "clang_getCString", "clang_disposeString",
                 "clang_Type_getSizeOf", "clang_Type_getAlignOf", "clang_getTypeSpelling", "clang_getResultType",
                 "clang_getArgType", "clang_getNumArgTypes", "clang_Cursor_getNumArguments", "clang_Cursor_getArgument",
                 "clang_getEnumDeclIntegerType", "clang_getEnumConstantDeclValue", "clang_getEnumConstantDeclUnsignedValue",
                 "CXDiagnostic", "clang_getNumDiagnostics", "clang_getDiagnostic", "clang_disposeDiagnostic",
                 "clang_formatDiagnostic", "clang_defaultDiagnosticDisplayOptions", "clang_getDiagnosticSeverity",
                 "clang_getDiagnosticSpelling", "clang_getDiagnosticLocation", "clang_getDiagnosticNumFixIts",
                 "clang_getDiagnosticFixIt", "clang_getDiagnosticNumRanges", "clang_getDiagnosticRange",
                 "clang_getCursorLocation", "clang_Location_isFromMainFile", "clang_Location_isInSystemHeader",
                 "clang_getSpellingLocation", "clang_getFileName", "clang_getCursorSemanticParent",
                 "clang_getTypedefDeclUnderlyingType", "clang_getCanonicalType", "clang_getPointeeType",
                 "clang_getTypeDeclaration", "clang_getArrayElementType", "clang_getArraySize",
                 "clang_Cursor_getOffsetOfField", "clang_isCursorDefinition", "clang_isConstQualifiedType",
                 "clang_isVolatileQualifiedType", "clang_isFunctionTypeVariadic", "clang_Cursor_isBitField",
                 "clang_getFieldDeclBitWidth", "clang_getCursorExtent", "clang_getRangeStart", "clang_getRangeEnd",
                 "clang_Cursor_getRawCommentText", "clang_createIndexWithOptions", "clang_getCursorDefinition",
                 "clang_Cursor_isNull", "clang_tokenize", "clang_disposeTokens", "CXEvalResult",
                 "clang_Cursor_Evaluate", "clang_EvalResult_getKind", "clang_EvalResult_getAsInt",
                 "clang_EvalResult_getAsLongLong", "clang_EvalResult_isUnsignedInt", "clang_EvalResult_getAsUnsigned",
                 "clang_EvalResult_getAsDouble", "clang_EvalResult_dispose", "clang_isExpression",
                 "clang_getFile", "clang_getFileContents", "clang_getNullCursor"]
    source = "// Native parser subset; declarations from the reviewed platform-specific inventories.\n"
    for name in api_names:
        def declaration(inventory):
            result = re.search(rf"^- `({name} :: [^`]+)`", inventory, re.MULTILINE)
            if not result:
                raise ValueError(f"Missing reviewed native declaration {name}")
            return result[1] + ";\n"
        unix, windows = declaration(unix_inventory), declaration(windows_inventory)
        if unix == windows:
            source += unix
        else:
            source += "#if OS == .WINDOWS {\n" + windows + "} else {\n" + unix + "}\n"
    source += '\n#scope_file\nlibclang :: #foreign_library "../internal/developer-tools/clang-bindings/clang";\n'
    (directory / "native-parser-api.jai").write_text(source)
    # Exact compatibility arrays are specified in the reviewed refinement.
    names = ["RESERVED_NAMES", "RESERVED_TYPE_NAMES", "TYPEDEFS_TO_UNWRAP", "TYPEDEF_PREFIXES_TO_UNWRAP", "MACROS_TO_IGNORE"]
    constants = []
    for name in names:
        line = re.search(rf"^- `{name}`: (.*)$", text, re.MULTILINE)[1]
        values = re.findall(r"`([^`]+)`", line)
        constants.append(f"{name} :: string.[" + ", ".join('"' + value + '"' for value in values) + "];\n")
    (directory / "compatibility-names.jai").write_text("\n".join(constants))
    groups = {"common_replacements": [], "windows_replacements": [], "other_replacements": []}
    for line in text.splitlines():
        if not line.startswith("| `"):
            continue
        columns = line.split("|")
        left, right = re.findall(r"`([^`]+)`", columns[1]), re.findall(r"`([^`]+)`", columns[2])
        if len(left) != len(right):
            raise ValueError("Reviewed token replacement columns have different lengths")
        group = "windows_replacements" if "on Windows" in columns[1] else "other_replacements" if "other targets" in columns[1] else "common_replacements"
        groups[group].extend(zip(left, right))
    source = 'TOKENS_TO_REPLACE :: #insert #run generate_replacements_array();\n\n'
    source += 'generate_replacements_array :: () -> string {\n    builder: String_Builder;\n    defer free_buffers(*builder);\n    append(*builder, "[2] string.[");\n'
    source += '    for pair: common_replacements print_to_builder(*builder, "string.[\\\"%\\\", \\\"%\\\"],", pair[0], pair[1]);\n'
    source += '    pairs := ifx TARGET_OS == .WINDOWS then windows_replacements else other_replacements;\n'
    source += '    for pair: pairs print_to_builder(*builder, "string.[\\\"%\\\", \\\"%\\\"],", pair[0], pair[1]);\n'
    source += '    append(*builder, "]");\n    return builder_to_string(*builder);\n}\n\n#scope_file\n#import "Basic";\n'
    for name, pairs in groups.items():
        source += '\n' + name + ' :: [2] string.[' + ', '.join('string.["' + c + '", "' + jai + '"]' for c, jai in pairs) + '];\n'
    (directory / "compatibility-tokens.jai").write_text(source)
    generated = [
        directory / filename for filename in ["type-model.jai", "declaration-model.jai", "generation-options.jai",
                                               "model-flags.jai", "native-model.jai", "compatibility-names.jai",
                                               "compatibility-tokens.jai", "native-parser-model.jai", "native-parser-api.jai"]]
    subprocess.run([str(ROOT / "target/jaifmt"), "--config", str(ROOT / "jaifmt.toml"),
                    *[str(path) for path in generated]], cwd=ROOT, check=True)
    print("Generated thematic public model files and exact reviewed compatibility arrays")


if __name__ == "__main__":
    main()
