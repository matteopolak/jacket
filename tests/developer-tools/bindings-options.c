#include <clang-c/Index.h>

unsigned developer_tools_check_options(const CXIndexOptions *options) {
    if (options->Size != sizeof(*options)) return 8;
    return options->ExcludeDeclarationsFromPCH | (options->DisplayDiagnostics << 1)
           | (options->StorePreamblesInMemory << 2);
}
