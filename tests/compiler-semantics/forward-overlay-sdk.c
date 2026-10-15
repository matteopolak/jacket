// Compile against public native SDK headers; this is layout proof, not Windows execution.
#define CINTERFACE
#include <stddef.h>
#include <d3d11.h>
#include <d3d12.h>
_Static_assert(sizeof(D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC) == 16, "D3D11 extent");
_Static_assert(offsetof(D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, ViewDimension) == 0, "D3D11 discriminator");
_Static_assert(offsetof(D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, Texture2D) == 4, "D3D11 alias");
_Static_assert(offsetof(D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, Texture2DArray) == 4, "D3D11 anchor");
_Static_assert(sizeof(D3D12_ROOT_PARAMETER) == 32, "D3D12 extent");
_Static_assert(_Alignof(D3D12_ROOT_PARAMETER) == 8, "D3D12 alignment");
_Static_assert(offsetof(D3D12_ROOT_PARAMETER, ParameterType) == 0, "D3D12 discriminator");
_Static_assert(offsetof(D3D12_ROOT_PARAMETER, Constants) == 8, "D3D12 alias");
_Static_assert(offsetof(D3D12_ROOT_PARAMETER, DescriptorTable) == 8, "D3D12 anchor");
_Static_assert(offsetof(D3D12_ROOT_PARAMETER, ShaderVisibility) == 24, "D3D12 tail");
