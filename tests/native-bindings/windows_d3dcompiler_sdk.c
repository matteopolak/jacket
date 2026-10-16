// Independent official-header/import-library control; no Jai declarations used.
#define COBJMACROS
#include <d3dcompiler.h>
#include <string.h>

int main(void) {
  ID3DBlob *code = NULL;
  ID3DBlob *errors = NULL;
  const char shader[] = "float4 main(float4 p:POSITION):SV_Position{return p;}";
  HRESULT status =
      D3DCompile(shader, strlen(shader), "independent-sdk.hlsl", NULL, NULL,
                 "main", "vs_5_0", 0, 0, &code, &errors);
  int valid = SUCCEEDED(status) && code && ID3D10Blob_GetBufferSize(code) > 0;
  if (code)
    ID3D10Blob_Release(code);
  if (errors)
    ID3D10Blob_Release(errors);
  if (!valid)
    return 91;
  code = NULL;
  errors = NULL;
  const char invalid[] = "this is not valid HLSL";
  status = D3DCompile(invalid, strlen(invalid), "independent-invalid.hlsl",
                      NULL, NULL, "main", "vs_5_0", 0, 0, &code, &errors);
  valid = FAILED(status) && errors && ID3D10Blob_GetBufferSize(errors) > 0;
  if (code)
    ID3D10Blob_Release(code);
  if (errors)
    ID3D10Blob_Release(errors);
  return valid ? 0 : 92;
}
