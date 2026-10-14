#import "Basic";
P :: #import "Protocol_For_Memory_Visualization";

#assert P.VIS_PROTOCOL_VERSION == 2;
#assert P.MEMORY_VISUALIZATION_CHANNEL_NAME == "Jai Memory Visualization";
#assert size_of(P.String_Index) == 4;
#assert size_of(P.Summary_Info) == 16;
#assert size_of(P.String_Info) == 12;
#assert size_of(P.Single_Stack_Trace_Info) == 24;
#assert size_of(P.Summary_Common) == 32;
#assert size_of(P.Memory_Debugger_Version_Info) == 24;
#assert size_of(P.Packed_Stack_Trace_Procedure_Info) == 32;
#assert size_of(P.Packed_Summary) == 64;
#assert size_of(P.Allocs_This_Frame) == 32;
#assert size_of(P.Frees_This_Frame) == 40;
#assert size_of(P.Packed_Packed_Stack_Trace) == 40;

main :: () {
    assert(cast(u32) P.Vis_Protocol_Tag.RESET_AND_TELL_VERSION == 1 && cast(u32) P.Vis_Protocol_Tag.UPDATE_USAGE == 5);
    summary: P.Packed_Summary;
    summary.count = 3;
    summary.bytes = 17;
    assert(summary.common.count == 3 && summary.common.bytes == 17);
    index_info := cast(*Type_Info_Variant) type_info(P.String_Index);

    assert(index_info.name == "String_Index");
    packed: P.Packed_Stack_Trace_Procedure_Info;

    assert(cast(s64) *packed.remote_address - cast(s64) *packed == 24);
    trace: P.Single_Stack_Trace_Info;

    assert(cast(s64) *trace.hash - cast(s64) *trace == 8);
    print("ok\n");
}
