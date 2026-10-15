// Complete pinned official native SDK, including its own non-Windows adapter.
// This is a host SDK layout check, not a Windows SDK/runtime execution claim.
#include "dxcapi.h"
#include <cstddef>
#include <type_traits>
static_assert(std::is_standard_layout<DxcBuffer>::value, "native buffer layout");
static_assert(sizeof(DxcBuffer) == 24 && alignof(DxcBuffer) == 8, "64-bit SDK buffer ABI");
static_assert(offsetof(DxcBuffer, Ptr) == 0, "native pointer offset");
static_assert(offsetof(DxcBuffer, Size) == 8, "native size offset");
static_assert(offsetof(DxcBuffer, Encoding) == 16, "native encoding offset");
static_assert(sizeof(GUID) == 16, "COM identity size");
static_assert(sizeof(HRESULT) == 4, "native HRESULT size");
static_assert(sizeof(IDxcCompiler3) == 8 && sizeof(IDxcUtils) == 8, "COM interface header");
static_assert(sizeof(IDxcOperationResult) == 8 && sizeof(IDxcResult) == 8, "result interface header");
static_assert(sizeof(IDxcBlob) == 8 && sizeof(IDxcBlobEncoding) == 8, "blob interface header");
