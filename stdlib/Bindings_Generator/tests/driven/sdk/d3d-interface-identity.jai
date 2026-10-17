// Source-only Windows check: SDK output parameters must use the canonical
// public compiler interfaces, rather than private duplicate/opaque records.
C :: #import "d3d_compiler";
D11 :: #import "d3d11";
D12 :: #import "d3d12";
W :: #import "Windows";

#assert C.ID3DBlob == C.ID3D10Blob;
#assert type_info(C.ID3DBlob).name == "ID3D10Blob";
#assert D12.D3D12_PRIMITIVE_TOPOLOGY == D12.D3D_PRIMITIVE_TOPOLOGY;
#assert type_info(D12.D3D12_PRIMITIVE_TOPOLOGY).name == "D3D_PRIMITIVE_TOPOLOGY";

module_output :: (graph: *D11.ID3D11FunctionLinkingGraph, instance: **C.ID3D11ModuleInstance, errors: **C.ID3DBlob) -> W.HRESULT {
    return D11.ID3D11FunctionLinkingGraph_CreateModuleInstance(graph, instance, errors);
}

blob_output :: (state: *D12.ID3D12PipelineState, output: **C.ID3DBlob) -> W.HRESULT {
    return D12.ID3D12PipelineState_GetCachedBlob(state, output);
}

main :: () {}
