add_data_segment :: (section_name: string, characteristics := C.Data_Segment_Characteristics.READ | .WRITE, alignment: s32 = 16, w: Workspace = -1) -> (segment: *C.Data_Segment, actual_segment_will_be_created: bool) #compiler {
    handle := __jaic_data_segment(section_name, cast(u32) characteristics, alignment, resolve_workspace(w));
    // The backend groups compatible globals; it does not emit arbitrary named sections.
    return cast(*C.Data_Segment) handle, false;
}

add_global_data :: (data: [] u8, segment: C.Data_Segment_Index, user_segment: *C.Data_Segment = null, w: Workspace = -1) -> [] u8 #compiler {
    result: [] u8;
    __jaic_global_data(resolve_workspace(w), data, cast(u16) segment, user_segment, *result);
    return result;
}

get_runtime_info :: (w: Workspace = -1) -> C.Runtime_Info #compiler {
    if #compile_time {
        result: C.Runtime_Info;
        result.type_table = get_type_table(w);
        result.global_data_info = cast(*C.Global_Data_Info) __jaic_runtime_data(resolve_workspace(w));
        return result;
    }

    // The default is usable in target code without a compile-time workspace query.
    if w != -1 {
        require(w == __jaic_current_workspace(), "get_runtime_info requires the running workspace");
    }

    return __runtime_info;
}

#scope_file
C :: #import "Compiler";

__runtime_info: C.Runtime_Info #elsewhere;
