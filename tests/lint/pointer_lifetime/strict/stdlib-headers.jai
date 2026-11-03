#import "Basic";
T :: #import "Thread";

Owner :: struct {
    mutex: T.Mutex;
    count: int;
}

safe_header :: (owner: *Owner) -> int {
    T.destroy(*owner.mutex);
    return owner.count;
}

bad_header :: (owner: *Owner) -> int {
    free(owner);
    return owner.count;
}

Arrays :: struct {
    entries: [..] int;
    count: int;
}

safe_array_reuse :: (owner: *Arrays) -> int {
    array_reset(*owner.entries);
    array_add(*owner.entries, 1);
    return owner.count;
}

Builder_Owner :: struct {
    builder: String_Builder;
    count: int;
}

safe_builder_reuse :: (owner: *Builder_Owner) -> int {
    free_buffers(*owner.builder);
    append(*owner.builder, "replacement");
    result := owner.count;

    free_buffers(*owner.builder);

    return result;
}

H :: #import "Extensions/Http";

safe_http_error_header :: (error: *H.Client_Error) -> bool {
    H.free_error(error);
    return error.kind == .NONE;
}

safe_http_response_header :: (response: *H.Client_Response) -> int {
    H.close(response);
    return response.status;
}
