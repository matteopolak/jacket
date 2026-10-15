// Reviewed D3D compiler Jai-string overloads: defines intentionally ignored,
// copied result bytes/diagnostics versus caller-owned file COM blobs.
#import "Basic";
D :: #import "d3d_compiler";
W :: #import "Windows";

main :: () {
    defines: [1] string = .["THIS_IS_NOT_A_DEFINE_GRAMMAR"];
    code, errors, status := D.D3DCompile("float4 main() : SV_Target { return 1; }", "memory.hlsl", defines, null, "main", "ps_5_0", 0, 0);

    assert(status >= 0 && code.count > 0);
    free(code);
    free(errors);
    code, errors, status = D.D3DCompile("invalid shader source", "memory.hlsl", defines, null, "main", "ps_5_0", 0, 0);
    assert(status < 0 && errors.count > 0);
    free(code);
    free(errors);
    file_code, file_errors, file_status := D.D3DCompileFromFile("__clean_bindings_missing_shader__.hlsl", defines, null, "main", "ps_5_0", 0, 0);

    assert(file_status < 0);
    if file_code W.IUnknown_Release(cast(*W.IUnknown) file_code);
    if file_errors W.IUnknown_Release(cast(*W.IUnknown) file_errors);
}
