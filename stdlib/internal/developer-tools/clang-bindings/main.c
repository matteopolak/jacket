/* Independent C-to-Jai staging pipeline using only the public libclang C API.
 * Private parser/emitter used by the Bindings_Generator public facade.
 */
#include <clang-c/Index.h>
#include <ctype.h>
#include <math.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
  char *data;
  size_t count;
  size_t capacity;
  bool failed;
} Text;

typedef struct {
  Text output;
  bool failed;
  const char *library_id;
  bool strip_enums;
  bool alias_enums;
  bool no_library;
  bool no_checks;
  char *enum_prefix;
  size_t enum_prefix_count;
  const char *unwrap_types;
  const char *unwrap_prefixes;
  bool no_comments;
  const char *ignored_macros;
  CXTranslationUnit unit;
  CXIndex index;
  const char *source_path;
  const char **arguments;
  int argument_count;
  Text macro_names;
  bool c_underscore_prefix;
  bool bridge_source;
  Text bridge_names;
  bool cpp_input;
  CXCursor *cpp_declarations;
  size_t cpp_declaration_count;
  size_t cpp_declaration_capacity;
} Generator;

static void append(Text *text, const char *format, ...);

static void source_comment(Generator *g, CXCursor cursor) {
  if (g->no_comments)
    return;
  CXString raw = clang_Cursor_getRawCommentText(cursor);
  const char *text = clang_getCString(raw);
  if (text && *text) {
    append(&g->output, "// ");
    for (const char *p = text; *p; ++p) {
      if (*p == '\n')
        append(&g->output, "\n// ");
      else if (*p != '\r')
        append(&g->output, "%c", *p);
    }
    append(&g->output, "\n");
  }
  clang_disposeString(raw);
}

static bool list_matches(const char *list, const char *name, bool prefix) {
  if (!list)
    return false;
  while (*list) {
    const char *end = strchr(list, ',');
    size_t count = end ? (size_t)(end - list) : strlen(list);
    if ((prefix ? strlen(name) >= count : strlen(name) == count) &&
        strncmp(list, name, count) == 0)
      return true;
    if (!end)
      break;
    list = end + 1;
  }
  return false;
}

static void append(Text *text, const char *format, ...) {
  if (text->failed)
    return;
  va_list args;
  va_start(args, format);
  va_list copy;
  va_copy(copy, args);
  int length = vsnprintf(NULL, 0, format, copy);
  va_end(copy);
  if (length < 0 || (size_t)length > (size_t)-1 - text->count - 1) {
    text->failed = true;
    va_end(args);
    return;
  }
  size_t needed = text->count + (size_t)length + 1;
  if (needed > text->capacity) {
    size_t capacity = needed < 4096 ? 4096 : needed;
    char *data = realloc(text->data, capacity);
    if (!data) {
      text->failed = true;
      va_end(args);
      return;
    }
    text->data = data;
    text->capacity = capacity;
  }
  vsnprintf(text->data + text->count, (size_t)length + 1, format, args);
  text->count += (size_t)length;
  va_end(args);
}

static void fail(Generator *g, CXCursor cursor, const char *reason) {
  CXString name = clang_getCursorSpelling(cursor);
  CXFile file = NULL;
  unsigned line = 0, column = 0;
  clang_getSpellingLocation(clang_getCursorLocation(cursor), &file, &line,
                            &column, NULL);
  if (file) {
    CXString location = clang_getFileName(file);
    fprintf(stderr, "%s:%u:%u: %s (%s)\n", clang_getCString(location), line,
            column, reason,
            clang_getCString(name) ? clang_getCString(name) : "<unnamed>");
    clang_disposeString(location);
  } else
    fprintf(stderr, "<input>:%u:%u: %s (%s)\n", line, column, reason,
            clang_getCString(name) ? clang_getCString(name) : "<unnamed>");
  clang_disposeString(name);
  g->failed = true;
}

static bool seen_cpp_declaration(Generator *g, CXCursor cursor) {
  CXCursor canonical = clang_getCanonicalCursor(cursor);
  if (clang_Cursor_isNull(canonical)) {
    fail(g, cursor, "C++ canonical declaration is unavailable");
    return true;
  }
  for (size_t i = 0; i < g->cpp_declaration_count; ++i)
    if (clang_equalCursors(canonical, g->cpp_declarations[i])) return true;
  if (g->cpp_declaration_count == g->cpp_declaration_capacity) {
    if (g->cpp_declaration_capacity > SIZE_MAX / (2 * sizeof(*g->cpp_declarations))) {
      fail(g, cursor, "C++ declaration identity storage exceeds addressable memory");
      return true;
    }
    size_t capacity = g->cpp_declaration_capacity ? g->cpp_declaration_capacity * 2 : 16;
    CXCursor *allocation = realloc(g->cpp_declarations, capacity * sizeof(*allocation));
    if (!allocation) {
      fail(g, cursor, "C++ declaration identity allocation failed");
      return true;
    }
    g->cpp_declarations = allocation;
    g->cpp_declaration_capacity = capacity;
  }
  g->cpp_declarations[g->cpp_declaration_count++] = canonical;
  return false;
}

static bool identifier(const char *name) {
  if (!name || (!isalpha((unsigned char)*name) && *name != '_'))
    return false;
  for (const char *p = name; *p; ++p)
    if (!isalnum((unsigned char)*p) && *p != '_')
      return false;
  const char *reserved[] = {
      "if",        "else",      "for",     "while",   "return", "defer",
      "using",     "struct",    "enum",    "union",   "true",   "false",
      "null",      "cast",      "xx",      "context", "int",    "float",
      "bool",      "string",    "void",    "Type",    "Code",   "Any",
      "s8",        "s16",       "s32",     "s64",     "u8",     "u16",
      "u32",       "u64",       "float32", "float64", "break",  "continue",
      "remove",    "case",      "then",    "inline",  "it",     "is_constant",
      "type_info", "interface", "v128"};
  for (size_t i = 0; i < sizeof(reserved) / sizeof(reserved[0]); ++i)
    if (strcmp(name, reserved[i]) == 0)
      return false;
  return true;
}

static void type_text(Generator *g, Text *text, CXType type, CXCursor site,
                      unsigned depth);
static bool bridge_type(CXType type, unsigned depth);

static void function_text(Generator *g, Text *text, CXType type, CXCursor site,
                          unsigned depth) {
  if (clang_getFunctionTypeCallingConv(type) != CXCallingConv_C) {
    fail(g, site,
         "unsupported function calling convention");
    return;
  }
  int count = clang_getNumArgTypes(type);
  if (count < 0) {
    fail(g, site, "function requires a prototype");
    return;
  }
  append(text, "(");
  for (int i = 0; i < count; ++i) {
    if (i)
      append(text, ", ");
    CXString argument_name =
        clang_getCursorSpelling(clang_Cursor_getArgument(site, (unsigned)i));
    const char *name = clang_getCString(argument_name);
    if (name && *name) {
      if (!identifier(name))
        fail(g, site, "reserved parameter name requires an explicit mapping");
      else
        append(text, "%s: ", name);
    } else
      append(text, "arg%d: ", i);
    clang_disposeString(argument_name);
    type_text(g, text, clang_getArgType(type, (unsigned)i), site, depth + 1);
  }
  if (clang_isFunctionTypeVariadic(type)) {
    for (int suffix = 0; suffix <= count; ++suffix) {
      char candidate[64];
      snprintf(candidate, sizeof(candidate), "varargs_%d", suffix);
      bool clash = false;
      for (int i = 0; i < count; ++i) {
        CXString spelling = clang_getCursorSpelling(clang_Cursor_getArgument(site, (unsigned)i));
        const char *name = clang_getCString(spelling);
        if (name && !strcmp(name, candidate)) clash = true;
        clang_disposeString(spelling);
      }
      if (!clash) {
        append(text, "%s%s: ..Any", count ? ", " : "", candidate);
        break;
      }
    }
  }
  append(text, ")");
  CXType result = clang_getResultType(type);
  if (result.kind != CXType_Void) {
    append(text, " -> ");
    type_text(g, text, result, site, depth + 1);
  }
}

static void type_text(Generator *g, Text *text, CXType type, CXCursor site,
                      unsigned depth) {
  if (depth > 64) {
    fail(g, site, "type nesting exceeds private generator limit");
    return;
  }
  if (type.kind == CXType_Elaborated) {
    type_text(g, text, clang_Type_getNamedType(type), site, depth + 1);
    return;
  }
  if (type.kind == CXType_Pointer) {
    CXType pointee = clang_getPointeeType(type);
    if (pointee.kind == CXType_FunctionProto) {
      function_text(g, text, pointee, site, depth + 1);
      append(text, " #c_call");
    } else {
      append(text, "*");
      type_text(g, text, pointee, site, depth + 1);
    }
    return;
  }
  if (type.kind == CXType_ConstantArray) {
    long long count = clang_getArraySize(type);
    if (count <= 0) {
      fail(g, site, "zero-sized arrays cannot be projected");
      return;
    }
    append(text, "[%lld] ", count);
    type_text(g, text, clang_getArrayElementType(type), site, depth + 1);
    return;
  }
  if (type.kind == CXType_Record || type.kind == CXType_Enum ||
      type.kind == CXType_Typedef) {
    CXCursor declaration = clang_getTypeDeclaration(type);
    CXString name = clang_getCursorSpelling(declaration);
    if (type.kind == CXType_Typedef &&
        (list_matches(g->unwrap_types, clang_getCString(name), false) ||
         list_matches(g->unwrap_prefixes, clang_getCString(name), true))) {
      clang_disposeString(name);
      type_text(g, text, clang_getCanonicalType(type), site, depth + 1);
      return;
    }
    if (clang_Location_isInSystemHeader(clang_getCursorLocation(declaration))) {
      fail(g, site,
           "system type requires explicit inclusion or typedef unwrapping");
      clang_disposeString(name);
      return;
    }
    if (!identifier(clang_getCString(name)))
      fail(g, site, "anonymous or reserved type name");
    else
      append(text, "%s", clang_getCString(name));
    clang_disposeString(name);
    return;
  }
  if (type.kind == CXType_Void) {
    append(text, "void");
    return;
  }
  if (type.kind == CXType_Bool) {
    append(text, "bool");
    return;
  }
  if (type.kind == CXType_Float) {
    append(text, "float32");
    return;
  }
  if (type.kind == CXType_Double) {
    append(text, "float64");
    return;
  }
  bool is_signed = type.kind == CXType_Char_S || type.kind == CXType_SChar ||
                   type.kind == CXType_Short || type.kind == CXType_Int ||
                   type.kind == CXType_Long || type.kind == CXType_LongLong;
  bool is_unsigned = type.kind == CXType_Char_U || type.kind == CXType_UChar ||
                     type.kind == CXType_UShort || type.kind == CXType_UInt ||
                     type.kind == CXType_ULong || type.kind == CXType_ULongLong;
  if (is_signed || is_unsigned) {
    long long size = clang_Type_getSizeOf(type);
    if (size == 1 || size == 2 || size == 4 || size == 8)
      append(text, "%c%lld", is_signed ? 's' : 'u', size * 8);
    else
      fail(g, site, "unsupported integer width");
    return;
  }
  fail(g, site, "unsupported C type");
}

static void adapter_type_text(Generator *g, Text *text, CXType type, CXCursor site) {
  CXType canonical = clang_getCanonicalType(type);
  if (canonical.kind == CXType_LValueReference) {
    append(text, "*");
    type_text(g, text, clang_getPointeeType(canonical), site, 0);
  } else type_text(g, text, type, site, 0);
}

static void adapter_function_text(Generator *g, CXType type, CXCursor site) {
  int count = clang_getNumArgTypes(type);
  CXType result = clang_getResultType(type);
  bool aggregate = clang_getCanonicalType(result).kind == CXType_Record;
  append(&g->output, "(");
  if (aggregate) {
    // Public native input names are preserved; the extra out parameter must
    // remain distinct even when the source has an argument named out_result.
    unsigned suffix = 0;
    char output_name[64];
    bool collision;
    do {
      snprintf(output_name, sizeof(output_name), "out_result_%u", suffix++);
      collision = false;
      for (int i = 0; i < count; ++i) {
        CXString argument = clang_getCursorSpelling(clang_Cursor_getArgument(site, (unsigned)i));
        const char *name = clang_getCString(argument);
        if (name && strcmp(name, output_name) == 0) collision = true;
        clang_disposeString(argument);
      }
    } while (collision);
    append(&g->output, "%s: *", output_name);
    type_text(g, &g->output, result, site, 0);
  }
  for (int i = 0; i < count; ++i) {
    if (aggregate || i) append(&g->output, ", ");
    CXString argument = clang_getCursorSpelling(clang_Cursor_getArgument(site, (unsigned)i));
    const char *name = clang_getCString(argument);
    if (name && *name) {
      if (!identifier(name)) fail(g, site, "reserved parameter name requires an explicit mapping");
      else append(&g->output, "%s: ", name);
    } else append(&g->output, "arg%d: ", i);
    clang_disposeString(argument);
    adapter_type_text(g, &g->output, clang_getArgType(type, (unsigned)i), site);
  }
  append(&g->output, ")");
  if (!aggregate && clang_getCanonicalType(result).kind != CXType_Void) {
    append(&g->output, " -> ");
    adapter_type_text(g, &g->output, result, site);
  }
}

static enum CXChildVisitResult field(CXCursor cursor, CXCursor parent,
                                     CXClientData data) {
  (void)parent;
  Generator *g = data;
  if (clang_getCursorKind(cursor) != CXCursor_FieldDecl) {
    fail(g, cursor, "unsupported nested record declaration or attribute");
    return CXChildVisit_Continue;
  }
  CXString name = clang_getCursorSpelling(cursor);
  if (!identifier(clang_getCString(name)) || clang_Cursor_isBitField(cursor)) {
    fail(g, cursor, "anonymous, reserved or bitfield member");
  } else {
    append(&g->output, "    %s: ", clang_getCString(name));
    type_text(g, &g->output, clang_getCursorType(cursor), cursor, 0);
    append(&g->output, ";\n");
  }
  clang_disposeString(name);
  return CXChildVisit_Continue;
}

static enum CXChildVisitResult enumerate(CXCursor cursor, CXCursor parent,
                                         CXClientData data) {
  (void)parent;
  Generator *g = data;
  CXString name = clang_getCursorSpelling(cursor);
  if (clang_getCursorKind(cursor) != CXCursor_EnumConstantDecl ||
      !identifier(clang_getCString(name))) {
    fail(g, cursor, "unsupported enum declaration");
  } else {
    const char *original = clang_getCString(name);
    const char *emitted = original;
    if (g->strip_enums && g->enum_prefix_count &&
        strncmp(original, g->enum_prefix, g->enum_prefix_count) == 0 &&
        identifier(original + g->enum_prefix_count))
      emitted += g->enum_prefix_count;
    CXType base = clang_getEnumDeclIntegerType(parent);
    if (base.kind == CXType_UInt || base.kind == CXType_ULong ||
        base.kind == CXType_ULongLong)
      append(&g->output, "    %s :: %llu;\n", emitted,
             clang_getEnumConstantDeclUnsignedValue(cursor));
    else
      append(&g->output, "    %s :: %lld;\n", emitted,
             clang_getEnumConstantDeclValue(cursor));
    if (g->alias_enums && emitted != original)
      append(&g->output, "    %s :: %s;\n", original, emitted);
  }
  clang_disposeString(name);
  return CXChildVisit_Continue;
}

static enum CXChildVisitResult
enum_common_prefix(CXCursor cursor, CXCursor parent, CXClientData data) {
  (void)parent;
  Generator *g = data;
  if (clang_getCursorKind(cursor) != CXCursor_EnumConstantDecl)
    return CXChildVisit_Continue;
  CXString string = clang_getCursorSpelling(cursor);
  const char *name = clang_getCString(string);
  if (!g->enum_prefix) {
    size_t count = strlen(name);
    g->enum_prefix = malloc(count + 1);
    if (!g->enum_prefix)
      fail(g, cursor, "enum prefix allocation failed");
    else {
      memcpy(g->enum_prefix, name, count + 1);
      g->enum_prefix_count = count;
    }
  } else {
    size_t i = 0;
    while (i < g->enum_prefix_count && name[i] && name[i] == g->enum_prefix[i])
      ++i;
    g->enum_prefix_count = i;
  }
  clang_disposeString(string);
  return CXChildVisit_Continue;
}

static enum CXChildVisitResult field_layout(CXCursor cursor, CXCursor parent,
                                            CXClientData data) {
  Generator *g = data;
  if (clang_getCursorKind(cursor) == CXCursor_FieldDecl) {
    CXString parent_name = clang_getCursorSpelling(parent);
    CXString field_name = clang_getCursorSpelling(cursor);
    long long offset = clang_Cursor_getOffsetOfField(cursor);
    if (offset < 0 || offset % 8 != 0)
      fail(g, cursor, "invalid native field offset");
    else
      append(&g->output, "#assert offset_of(%s, \"%s\") == %lld;\n",
             clang_getCString(parent_name), clang_getCString(field_name),
             offset / 8);
    clang_disposeString(parent_name);
    clang_disposeString(field_name);
  }
  return CXChildVisit_Continue;
}

static enum CXChildVisitResult find_probe_expression(CXCursor cursor, CXCursor parent,
                                                    CXClientData data) {
  (void)parent;
  if (clang_getCursorKind(cursor) == CXCursor_VarDecl ||
      (clang_isExpression(clang_getCursorKind(cursor)) &&
       clang_getCursorType(cursor).kind != CXType_Void)) {
    *(CXCursor *)data = cursor;
    return CXChildVisit_Break;
  }
  return CXChildVisit_Recurse;
}

static enum CXChildVisitResult find_macro_probe(CXCursor cursor, CXCursor parent,
                                               CXClientData data) {
  (void)parent;
  if (clang_getCursorKind(cursor) == CXCursor_FunctionDecl) {
    CXString name = clang_getCursorSpelling(cursor);
    if (strcmp(clang_getCString(name), "__developer_tools_macro_probe") == 0)
      clang_visitChildren(cursor, find_probe_expression, data);
    clang_disposeString(name);
  }
  return CXChildVisit_Continue;
}

typedef struct { CXCursor cursor; unsigned count; } StringChild;

static enum CXChildVisitResult string_child(CXCursor cursor, CXCursor parent,
                                          CXClientData data) {
  (void)parent;
  StringChild *child = data;
  child->cursor = cursor;
  if (child->count < 2) ++child->count;
  return CXChildVisit_Continue;
}

static long long byte_string_extent(CXCursor cursor, unsigned depth) {
  if (depth > 64) return -1;
  enum CXCursorKind kind = clang_getCursorKind(cursor);
  if (kind == CXCursor_StringLiteral) {
    CXType type = clang_getCanonicalType(clang_getCursorType(cursor));
    if (type.kind != CXType_ConstantArray) return -1;
    enum CXTypeKind element = clang_getCanonicalType(clang_getArrayElementType(type)).kind;
    if (element != CXType_Char_S && element != CXType_Char_U) return -1;
    return clang_getArraySize(type);
  }
  if (kind != CXCursor_ParenExpr && kind != CXCursor_UnexposedExpr && kind != CXCursor_VarDecl) return -1;
  StringChild child = {0};
  clang_visitChildren(cursor, string_child, &child);
  return (child.count == 1 || (kind == CXCursor_VarDecl && child.count))
             ? byte_string_extent(child.cursor, depth + 1) : -1;
}

static void byte_string(Text *output, const unsigned char *text) {
  append(output, "\"");
  for (; *text; ++text) {
    if (*text == '\\' || *text == '"') append(output, "\\%c", *text);
    else if (*text < 32 || *text >= 127) append(output, "\\x%02x", *text);
    else append(output, "%c", *text);
  }
  append(output, "\"");
}

static void numeric_macro(Generator *g, CXCursor cursor) {
  if (clang_Cursor_isMacroFunctionLike(cursor)) {
    fail(g, cursor, "function-like macro requires a generated wrapper");
    return;
  }
  CXString spelling = clang_getCursorSpelling(cursor);
  const char *name = clang_getCString(spelling);
  if (list_matches(g->macro_names.data, name, false)) {
    clang_disposeString(spelling);
    return;
  }
  if (!identifier(name)) {
    fail(g, cursor, "reserved or invalid macro name");
    clang_disposeString(spelling);
    return;
  }
  Text source = {0};
  append(&source, "#include \"");
  for (const char *p = g->source_path; *p; ++p) {
    if (*p == '\\' || *p == '"') append(&source, "\\");
    if ((unsigned char)*p < 32) source.failed = true;
    append(&source, "%c", *p);
  }
  append(&source, "\"\nvoid __developer_tools_macro_probe(void);\n"
                  "void __developer_tools_macro_probe(void) { __typeof__((%s)) "
                  "__developer_tools_macro_value = (%s); (void)__developer_tools_macro_value; }\n", name, name);
  size_t header_size = 0;
  const char *header_contents = clang_getFileContents(
      g->unit, clang_getFile(g->unit, g->source_path), &header_size);
  struct CXUnsavedFile unsaved[] = {
      {"developer-tools-macro-probe.c", source.data, (unsigned long)source.count},
      {g->source_path, header_contents, (unsigned long)header_size}};
  CXTranslationUnit probe = NULL;
  enum CXErrorCode error = CXError_Failure;
  if (!source.failed && header_contents)
    error = clang_parseTranslationUnit2(g->index, unsaved[0].Filename, g->arguments,
                                       g->argument_count, unsaved, 2, 0, &probe);
  bool valid = error == CXError_Success && probe;
  if (probe) {
    for (unsigned i = 0; i < clang_getNumDiagnostics(probe); ++i) {
      CXDiagnostic diagnostic = clang_getDiagnostic(probe, i);
      if (clang_getDiagnosticSeverity(diagnostic) >= CXDiagnostic_Warning) {
        valid = false;
        CXString text = clang_formatDiagnostic(diagnostic, clang_defaultDiagnosticDisplayOptions());
        fprintf(stderr, "%s\n", clang_getCString(text));
        clang_disposeString(text);
      }
      clang_disposeDiagnostic(diagnostic);
    }
  }
  CXCursor variable = clang_getNullCursor();
  CXEvalResult value = NULL;
  if (valid) {
    clang_visitChildren(clang_getTranslationUnitCursor(probe), find_macro_probe, &variable);
    value = clang_Cursor_Evaluate(variable);
  }
  if (value && clang_EvalResult_getKind(value) == CXEval_StrLiteral) {
    const char *text = clang_EvalResult_getAsStr(value);
    long long extent = byte_string_extent(variable, 0);
    if (!text || extent <= 0 || strlen(text) != (size_t)extent - 1) {
      fail(g, cursor, "macro is not a supported byte string constant (wide, embedded NUL or nonliteral)");
    } else {
      append(&g->output, "%s :: ", name);
      byte_string(&g->output, (const unsigned char *)text);
      append(&g->output, ";\n\n");
      if (g->macro_names.count) append(&g->macro_names, ",");
      append(&g->macro_names, "%s", name);
    }
  } else if (!value || (clang_EvalResult_getKind(value) != CXEval_Int &&
                 clang_EvalResult_getKind(value) != CXEval_Float)) {
    fail(g, cursor, "macro is not a supported numeric constant expression");
  } else if (clang_EvalResult_getKind(value) == CXEval_Float &&
             !isfinite(clang_EvalResult_getAsDouble(value))) {
    fail(g, cursor, "nonfinite floating macro requires an explicit bit representation");
  } else {
    append(&g->output, "%s :: cast(", name);
    type_text(g, &g->output, clang_getCanonicalType(clang_getCursorType(variable)), cursor, 0);
    append(&g->output, ") ");
    if (clang_EvalResult_getKind(value) == CXEval_Int) {
      if (clang_EvalResult_isUnsignedInt(value))
        append(&g->output, "%llu", clang_EvalResult_getAsUnsigned(value));
      else
        append(&g->output, "%lld", clang_EvalResult_getAsLongLong(value));
    } else {
      char number[64];
      snprintf(number, sizeof(number), "%.17g", clang_EvalResult_getAsDouble(value));
      append(&g->output, "%s", number);
      if (!strchr(number, '.') && !strchr(number, 'e') && !strchr(number, 'E'))
        append(&g->output, ".0");
    }
    append(&g->output, ";\n\n");
    if (g->macro_names.count) append(&g->macro_names, ",");
    append(&g->macro_names, "%s", name);
  }
  if (value) clang_EvalResult_dispose(value);
  if (probe) clang_disposeTranslationUnit(probe);
  free(source.data);
  clang_disposeString(spelling);
}

static enum CXChildVisitResult declaration(CXCursor cursor, CXCursor parent,
                                           CXClientData data) {
  (void)parent;
  Generator *g = data;
  CXSourceLocation location = clang_getCursorLocation(cursor);
  if (clang_Location_isInSystemHeader(location))
    return CXChildVisit_Continue;
  enum CXCursorKind kind = clang_getCursorKind(cursor);
  if (kind == CXCursor_InclusionDirective)
    return CXChildVisit_Continue;
  if (kind == CXCursor_MacroExpansion)
    return CXChildVisit_Continue;
  if (kind == CXCursor_MacroDefinition) {
    CXFile file = NULL;
    clang_getSpellingLocation(location, &file, NULL, NULL, NULL);
    if (file) {
      CXString name = clang_getCursorSpelling(cursor);
      bool ignored =
          list_matches(g->ignored_macros, clang_getCString(name), false);
      clang_disposeString(name);
      CXToken *tokens = NULL;
      unsigned count = 0;
      clang_tokenize(g->unit, clang_getCursorExtent(cursor), &tokens, &count);
      // An empty definition is a preprocessing flag (including include guards),
      // not a value declaration. Expansion effects are already in Clang's AST.
      if (count > 1 && !ignored) numeric_macro(g, cursor);
      if (tokens)
        clang_disposeTokens(g->unit, tokens, count);
    }
    return CXChildVisit_Continue;
  }
  if (clang_getCursorLanguage(cursor) == CXLanguage_CPlusPlus &&
      kind != CXCursor_FunctionDecl && kind != CXCursor_StructDecl &&
      kind != CXCursor_UnionDecl && kind != CXCursor_EnumDecl && kind != CXCursor_TypedefDecl) {
    fail(g, cursor, "C++ declaration requires pending C++ parsing and bridge support");
    return CXChildVisit_Continue;
  }
  CXString spelling = clang_getCursorSpelling(cursor);
  const char *name = clang_getCString(spelling);
  if (!identifier(name)) {
    fail(g, cursor, "anonymous or reserved declaration name");
    clang_disposeString(spelling);
    return CXChildVisit_Continue;
  }
  CXType type = clang_getCursorType(cursor);
  source_comment(g, cursor);
  if (kind == CXCursor_StructDecl || kind == CXCursor_UnionDecl) {
    if (clang_isCursorDefinition(cursor)) {
      if (g->cpp_input && !clang_isPODType(type)) {
        fail(g, cursor, "non-POD C++ record requires unsupported layout/lifetime handling");
        clang_disposeString(spelling);
        return CXChildVisit_Continue;
      }
      append(&g->output, "%s :: %s {\n", name,
             kind == CXCursor_UnionDecl ? "union" : "struct");
      clang_visitChildren(cursor, field, g);
      append(&g->output, "}\n");
      if (!g->no_checks) {
        append(&g->output, "#assert size_of(%s) == %lld;\n", name,
               clang_Type_getSizeOf(type));
        append(&g->output, "#assert align_of(%s) == %lld;\n", name,
               clang_Type_getAlignOf(type));
        clang_visitChildren(cursor, field_layout, g);
      }
      append(&g->output, "\n");
    } else if (clang_Cursor_isNull(clang_getCursorDefinition(cursor))) {
      append(&g->output,
             "%s :: struct {} // Native incomplete type; pointers only.\n\n",
             name);
    }
  } else if (kind == CXCursor_EnumDecl) {
    append(&g->output, "%s :: enum ", name);
    type_text(g, &g->output, clang_getEnumDeclIntegerType(cursor), cursor, 0);
    append(&g->output, " {\n");
    if (g->strip_enums) {
      clang_visitChildren(cursor, enum_common_prefix, g);
      size_t count = 0;
      for (size_t i = 0; i < g->enum_prefix_count; ++i)
        if (g->enum_prefix[i] == '_')
          count = i + 1;
      g->enum_prefix_count = count;
    }
    clang_visitChildren(cursor, enumerate, g);
    free(g->enum_prefix);
    g->enum_prefix = NULL;
    g->enum_prefix_count = 0;
    append(&g->output, "}\n\n");
  } else if (kind == CXCursor_TypedefDecl) {
    CXType underlying = clang_getTypedefDeclUnderlyingType(cursor);
    CXString underlying_name =
        clang_getCursorSpelling(clang_getTypeDeclaration(underlying));
    if (strcmp(name, clang_getCString(underlying_name)) != 0) {
      append(&g->output, "%s :: ", name);
      type_text(g, &g->output, underlying, cursor, 0);
      append(&g->output, ";\n\n");
    }
    clang_disposeString(underlying_name);
  } else if (kind == CXCursor_FunctionDecl) {
    CXString mangling = clang_Cursor_getMangling(cursor);
    const char *native_name = clang_getCString(mangling);
    const char *c_name = native_name;
    if (g->c_underscore_prefix && c_name && *c_name == '_') ++c_name;
    bool cpp_adapter = g->cpp_input && c_name &&
        (strncmp(c_name, "_Z", 2) == 0 || *c_name == '?');
    if (!c_name || !*c_name) {
      fail(g, cursor, "native linkage name is unavailable");
    } else if (cpp_adapter) {
      int count = clang_getNumArgTypes(type);
      bool supported = count >= 0 && !clang_isFunctionTypeVariadic(type) &&
          clang_getFunctionTypeCallingConv(type) == CXCallingConv_C &&
          bridge_type(clang_getResultType(type), 0) &&
          clang_getCursorLinkage(cursor) == CXLinkage_External &&
          clang_getCursorAvailability(cursor) == CXAvailability_Available &&
          clang_getCursorVisibility(cursor) != CXVisibility_Hidden;
      for (int i = 0; supported && i < count; ++i)
        supported = bridge_type(clang_getArgType(type, (unsigned)i), 0);
      if (!supported) {
        fail(g, cursor, "C++ binding requires an unsupported adapter signature");
      } else if (seen_cpp_declaration(g, cursor)) {
        clang_disposeString(mangling);
        clang_disposeString(spelling);
        return CXChildVisit_Continue;
      } else if (list_matches(g->bridge_names.data, name, false)) {
        fail(g, cursor, "duplicate C bridge output name; overload naming requires a declaration mapping");
      } else {
        if (g->bridge_names.count) append(&g->bridge_names, ",");
        append(&g->bridge_names, "%s", name);
        append(&g->output, "%s :: ", name);
        adapter_function_text(g, type, cursor);
        append(&g->output, " #foreign %s;\n\n", g->library_id);
      }
    } else if (strcmp(c_name, name) != 0) {
      fail(g, cursor, "native linkage requires pending symbol/mangling support");
    } else if (clang_getCursorLinkage(cursor) != CXLinkage_External ||
        clang_isCursorDefinition(cursor)) {
      fail(g, cursor, "inline/static function bodies require a bridge");
    } else {
      append(&g->output, "%s :: ", name);
      function_text(g, &g->output, type, cursor, 0);
      append(&g->output, " #foreign %s;\n\n", g->library_id);
    }
    clang_disposeString(mangling);
  } else
    fail(g, cursor, "unsupported top-level declaration");
  clang_disposeString(spelling);
  return CXChildVisit_Continue;
}

/* Deliberately bounded bridge grammar; libclang supplies canonical type and
 * linkage information. Declarators needing embedded names are not guessed. */
static bool bridge_type(CXType type, unsigned depth) {
  if (depth > 64) return false;
  type = clang_getCanonicalType(type);
  switch (type.kind) {
  case CXType_Void: case CXType_Bool: case CXType_Char_U: case CXType_UChar:
  case CXType_Char_S: case CXType_SChar: case CXType_UShort: case CXType_Short:
  case CXType_UInt: case CXType_Int: case CXType_ULong: case CXType_Long:
  case CXType_ULongLong: case CXType_LongLong: case CXType_Float:
  case CXType_Double: case CXType_Enum:
    return true;
  case CXType_Record: {
    CXString spelling = clang_getCursorSpelling(clang_getTypeDeclaration(type));
    const char *name = clang_getCString(spelling);
    bool valid = identifier(name) && clang_isPODType(type);
    clang_disposeString(spelling);
    return valid;
  }
  case CXType_Pointer: case CXType_LValueReference:
    return bridge_type(clang_getPointeeType(type), depth + 1);
  default: return false;
  }
}

static void bridge_declarator(Text *output, CXType type, const char *name) {
  bool reference = clang_getCanonicalType(type).kind == CXType_LValueReference;
  if (reference) type = clang_getPointeeType(clang_getCanonicalType(type));
  CXString spelling = clang_getTypeSpelling(type);
  append(output, "%s%s %s", clang_getCString(spelling), reference ? " *" : "", name);
  clang_disposeString(spelling);
}

static enum CXChildVisitResult bridge_declaration(CXCursor cursor,
                                                  CXCursor parent, CXClientData data) {
  (void)parent;
  Generator *g = data;
  if (clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)))
    return CXChildVisit_Continue;
  /* Scope-path spelling is a separate reviewed-contract question. */
  if (clang_getCursorKind(cursor) != CXCursor_FunctionDecl ||
      clang_getCursorLinkage(cursor) != CXLinkage_External ||
      clang_getCursorAvailability(cursor) != CXAvailability_Available ||
      clang_getCursorVisibility(cursor) == CXVisibility_Hidden)
    return CXChildVisit_Continue;
  CXString spelling = clang_getCursorSpelling(cursor);
  const char *name = clang_getCString(spelling);
  CXString mangling = clang_Cursor_getMangling(cursor);
  const char *native_name = clang_getCString(mangling);
  const char *c_name = native_name;
  if (g->c_underscore_prefix && c_name && *c_name == '_') ++c_name;
  CXType function = clang_getCursorType(cursor);
  CXType result = clang_getResultType(function);
  int count = clang_getNumArgTypes(function);
  bool supported = identifier(name) && c_name && *c_name && strcmp(c_name, name) &&
                   count >= 0 && !clang_isFunctionTypeVariadic(function) &&
                   clang_getFunctionTypeCallingConv(function) == CXCallingConv_C &&
                   bridge_type(result, 0);
  for (int i = 0; supported && i < count; ++i)
    supported = bridge_type(clang_getArgType(function, (unsigned)i), 0);
  if (supported && !seen_cpp_declaration(g, cursor)) {
    if (list_matches(g->bridge_names.data, name, false)) {
      fail(g, cursor, "duplicate C bridge output name; overload naming requires a declaration mapping");
    } else {
      if (g->bridge_names.count) append(&g->bridge_names, ",");
      append(&g->bridge_names, "%s", name);
      CXType canonical_result = clang_getCanonicalType(result);
      bool aggregate = canonical_result.kind == CXType_Record;
      bool reference = canonical_result.kind == CXType_LValueReference;
      bool nothing = canonical_result.kind == CXType_Void;
      append(&g->output, "namespace independently_generated_bridge {\nextern \"C\" ");
      if (aggregate) append(&g->output, "void %s", name);
      else bridge_declarator(&g->output, result, name);
      append(&g->output, "(");
      if (aggregate) {
        CXString result_name = clang_getTypeSpelling(result);
        append(&g->output, "%s *result", clang_getCString(result_name));
        clang_disposeString(result_name);
      }
      for (int i = 0; i < count; ++i) {
        if (aggregate || i) append(&g->output, ", ");
        char argument_name[32];
        snprintf(argument_name, sizeof(argument_name), "argument_%d", i);
        bridge_declarator(&g->output, clang_getArgType(function, (unsigned)i), argument_name);
      }
      append(&g->output, ") {\n  ");
      if (aggregate) append(&g->output, "*result = ");
      else if (!nothing) append(&g->output, "return %s", reference ? "&" : "");
      append(&g->output, "::%s(", name);
      for (int i = 0; i < count; ++i) {
        if (i) append(&g->output, ", ");
        bool ref = clang_getCanonicalType(clang_getArgType(function, (unsigned)i)).kind == CXType_LValueReference;
        append(&g->output, "%sargument_%d", ref ? "*" : "", i);
      }
      append(&g->output, ");\n}\n}\n\n");
    }
  }
  clang_disposeString(mangling);
  clang_disposeString(spelling);
  return CXChildVisit_Continue;
}

int main(int argc, char **argv) {
  if (argc < 4) {
    fprintf(stderr, "usage: clang-bindings input.h output.jai native-library "
                    "[clang args...]\n");
    return 2;
  }
  CXIndex index = clang_createIndex(0, 0);
  if (!index) {
    fprintf(stderr, "libclang index allocation failed\n");
    return 1;
  }
  CXTranslationUnit unit = NULL;
  const char **args = malloc((size_t)(argc + 1) * sizeof(*args));
  if (!args) {
    fprintf(stderr, "libclang argument allocation failed\n");
    clang_disposeIndex(index);
    return 1;
  }
  Generator generator = {0};
  generator.library_id = "native";
  args[0] = "-x";
  args[1] = "c";
  args[2] = "-std=c11";
  int arg_count = 3;
  for (int i = 4; i < argc; ++i) {
    if (strncmp(argv[i], "--jai-library-id=", 17) == 0)
      generator.library_id = argv[i] + 17;
    else if (strcmp(argv[i], "--jai-strip-enums") == 0)
      generator.strip_enums = true;
    else if (strcmp(argv[i], "--jai-alias-enums") == 0)
      generator.alias_enums = true;
    else if (strcmp(argv[i], "--jai-no-library") == 0)
      generator.no_library = true;
    else if (strcmp(argv[i], "--jai-no-checks") == 0)
      generator.no_checks = true;
    else if (strcmp(argv[i], "--jai-no-comments") == 0)
      generator.no_comments = true;
    else if (strcmp(argv[i], "--jai-bridge") == 0)
      generator.bridge_source = true;
    else if (strncmp(argv[i], "--jai-ignore-macros=", 20) == 0)
      generator.ignored_macros = argv[i] + 20;
    else if (strncmp(argv[i], "--jai-unwrap-types=", 19) == 0)
      generator.unwrap_types = argv[i] + 19;
    else if (strncmp(argv[i], "--jai-unwrap-prefixes=", 22) == 0)
      generator.unwrap_prefixes = argv[i] + 22;
    else
      args[arg_count++] = argv[i];
  }
  if (generator.bridge_source) {
    args[1] = "c++";
    args[2] = "-std=c++17";
  }
  for (int i = 0; i < arg_count; ++i) {
    if (strcmp(args[i], "-x") == 0 && i + 1 < arg_count)
      generator.cpp_input = strcmp(args[++i], "c++") == 0;
    else if (strncmp(args[i], "-x", 2) == 0 && args[i][2])
      generator.cpp_input = strcmp(args[i] + 2, "c++") == 0;
  }
  if (!identifier(generator.library_id)) {
    fprintf(stderr, "invalid generated library identifier\n");
    free(args);
    clang_disposeIndex(index);
    return 1;
  }
  enum CXErrorCode error = clang_parseTranslationUnit2(
      index, argv[1], args, arg_count, NULL, 0,
      CXTranslationUnit_DetailedPreprocessingRecord, &unit);
  if (error != CXError_Success || !unit) {
    free(args);
    fprintf(stderr, "libclang parsing failed with status %d\n", error);
    if (unit)
      clang_disposeTranslationUnit(unit);
    clang_disposeIndex(index);
    return 1;
  }
  generator.unit = unit;
  generator.index = index;
  generator.source_path = argv[1];
  generator.arguments = args;
  generator.argument_count = arg_count;
  CXTargetInfo target_info = clang_getTranslationUnitTargetInfo(unit);
  if (!target_info) {
    fprintf(stderr, "could not retrieve native target ABI information\n");
    generator.failed = true;
  } else {
    CXString triple = clang_TargetInfo_getTriple(target_info);
    const char *triple_name = clang_getCString(triple);
    if (!triple_name || !*triple_name) {
      fprintf(stderr, "native target ABI triple is unavailable\n");
      generator.failed = true;
    } else {
      generator.c_underscore_prefix = strstr(triple_name, "-apple-") != NULL;
    }
    clang_disposeString(triple);
    clang_TargetInfo_dispose(target_info);
  }
  for (unsigned i = 0; i < clang_getNumDiagnostics(unit); ++i) {
    CXDiagnostic diagnostic = clang_getDiagnostic(unit, i);
    CXString text = clang_formatDiagnostic(
        diagnostic, clang_defaultDiagnosticDisplayOptions());
    fprintf(stderr, "%s\n", clang_getCString(text));
    if (clang_getDiagnosticSeverity(diagnostic) >= CXDiagnostic_Warning)
      generator.failed = true;
    clang_disposeString(text);
    clang_disposeDiagnostic(diagnostic);
  }
  append(&generator.output,
         "// Independently generated with the public libclang C API.\n");
  if (!generator.no_library && !generator.bridge_source) {
    append(&generator.output, "%s :: #foreign_library \"",
           generator.library_id);
    for (const char *p = argv[3]; *p; ++p) {
      if (*p == '"' || *p == '\\')
        append(&generator.output, "\\%c", *p);
      else if ((unsigned char)*p < 32)
        generator.failed = true;
      else
        append(&generator.output, "%c", *p);
    }
    append(&generator.output, "\";\n\n");
  }
  if (!generator.failed)
    clang_visitChildren(clang_getTranslationUnitCursor(unit), generator.bridge_source ? bridge_declaration : declaration,
                        &generator);
  bool success = !generator.failed && !generator.output.failed && !generator.macro_names.failed && !generator.bridge_names.failed;
  if (success) {
    bool to_stdout = strcmp(argv[2], "-") == 0;
    FILE *output = to_stdout ? stdout : fopen(argv[2], "wb");
    if (!output) {
      perror(argv[2]);
      success = false;
    } else {
      if (fwrite(generator.output.data, 1, generator.output.count, output) !=
          generator.output.count)
        success = false;
      if ((to_stdout ? fflush(output) : fclose(output)) != 0)
        success = false;
      if (!success)
        fprintf(stderr, "could not write complete binding output\n");
    }
  } else
    fprintf(stderr, "binding generation failed; existing output preserved\n");
  free(generator.output.data);
  free(generator.macro_names.data);
  free(generator.bridge_names.data);
  free(generator.cpp_declarations);
  free(args);
  clang_disposeTranslationUnit(unit);
  clang_disposeIndex(index);
  return success ? 0 : 1;
}
