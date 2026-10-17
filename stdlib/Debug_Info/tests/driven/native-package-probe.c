// Original loader/parse/demangler regression; official clang-c and dyld APIs.
#include <clang-c/Index.h>
#include <dlfcn.h>
#include <mach-o/dyld.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

struct Native_Signature {
  void *result;
  void **arguments;
  int64_t argument_count, used_bytes;
  int32_t status;
  const char *error;
};

int main(int argc, char **argv) {
  if (argc != 5) return 1;
  void *clang = dlopen(argv[2], RTLD_NOW | RTLD_LOCAL);
  if (!clang) { fprintf(stderr, "libclang package load failed: %s\n", dlerror()); return 2; }
  void *check = dlopen(argv[3], RTLD_NOW | RTLD_LOCAL);
  if (!check) { fprintf(stderr, "Check package load failed: %s\n", dlerror()); return 2; }
  CXIndex (*create)(int, int) = dlsym(clang, "clang_createIndex");
  CXTranslationUnit (*parse)(CXIndex, const char *, const char *const *, int,
                            struct CXUnsavedFile *, unsigned, unsigned) = dlsym(clang, "clang_parseTranslationUnit");
  unsigned (*diagnostics)(CXTranslationUnit) = dlsym(clang, "clang_getNumDiagnostics");
  void (*dispose_unit)(CXTranslationUnit) = dlsym(clang, "clang_disposeTranslationUnit");
  void (*dispose_index)(CXIndex) = dlsym(clang, "clang_disposeIndex");
  const struct Native_Signature *(*signature)(const char *, int64_t, int32_t, int32_t, int32_t) =
      dlsym(check, "developer_check_parse");
  if (!create || !parse || !diagnostics || !dispose_unit || !dispose_index || !signature) return 3;
  CXIndex index = create(0, 0);
  const char *options[] = {"-x", "c", "-std=c11"};
  CXTranslationUnit unit = parse(index, argv[4], options, 3, NULL, 0, 0);
  if (!unit || diagnostics(unit)) return 4;
  const struct Native_Signature *value = signature("_Z8originali", 12, 0, 8, 1);
  if (!value || value->status || value->argument_count != 1 || !value->arguments || value->used_bytes <= 0) return 5;
  unsigned matched = 0;
  for (uint32_t i = 0; i < _dyld_image_count(); ++i) {
    const char *image = _dyld_get_image_name(i);
    const char *name = strrchr(image, '/');
    name = name ? name + 1 : image;
    if (strncmp(name, "libclang.", 9) && strncmp(name, "libLLVM.", 8) &&
        strncmp(name, "libz3.", 6) && strncmp(name, "libzstd.", 8) &&
        strcmp(name, "libdeveloper_tools_check.dylib")) continue;
    if (strncmp(image, argv[1], strlen(argv[1])) || image[strlen(argv[1])] != '/') {
      fprintf(stderr, "dependency loaded outside relocated package: %s\n", name);
      return 6;
    }
    ++matched;
  }
  if (matched < 3) return 7;
  dispose_unit(unit);
  dispose_index(index);
  if (dlclose(check) || dlclose(clang)) return 8;
  return 0;
}
