W :: #import "Windows";

#assert size_of(W.M128A) == 16;
#assert align_of(W.M128A) == 16;
#assert offset_of(W.M128A, "Low") == 0;
#assert offset_of(W.M128A, "High") == 8;
#assert size_of(W.CONTEXT) == 1232;
#assert align_of(W.CONTEXT) == 16;
#assert offset_of(W.CONTEXT, "ContextFlags") == 48;
#assert offset_of(W.CONTEXT, "Rip") == 248;
#assert offset_of(W.CONTEXT, "FltSave") == 256;
#assert offset_of(W.CONTEXT, "Xmm0") == 416;
#assert offset_of(W.CONTEXT, "VectorRegister") == 768;
#assert offset_of(W.CONTEXT, "VectorControl") == 1184;

// Alignment must belong to the canonical type, including embedded records and
// arrays. Adding padding or aligning only an individual local cannot satisfy it.
Vector_Container :: struct {
    prefix: u8;
    values: [2] W.M128A;
}

Context_Container :: struct {
    prefix: u8;
    values: [2] W.CONTEXT;
}

#assert align_of(Vector_Container) == 16;
#assert offset_of(Vector_Container, "values") == 16;
#assert size_of(Vector_Container) == 48;
#assert align_of(Context_Container) == 16;
#assert offset_of(Context_Container, "values") == 16;
#assert size_of(Context_Container) == 2480;

main :: () {
    vector: W.M128A;
    saved_context: W.CONTEXT;

    #assert type_of(vector.Low) == u64;
    #assert type_of(vector.High) == s64;
    #assert type_of(saved_context.VectorRegister) == [26] W.M128A;
    #assert type_of(saved_context.Xmm0) == W.M128A;
}
