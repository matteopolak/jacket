#import "Basic";

events: *[..] string;

field_text :: (text: string) -> string {
    return copy_string(text,, temp);
}

record :: (text: string) {
    array_add(events, field_text(text));
}

main :: () {
    reset_temporary_storage();
    reported: [..] string;

    events = *reported;
    record("hello");
    assert(reported[0] == "hello");
    events = null;
    free(reported.data);
    reset_temporary_storage();
}

persistent: [..] string;
persistent_events: *[..] string;

record_bad :: () {
    array_add(persistent_events, field_text("bad"));
}

set_persistent :: () {
    persistent_events = *persistent;
    record_bad();
}

returned_events: *[..] string;

record_returned :: () {
    array_add(returned_events, field_text("bad"));
}

return_report :: () -> [..] string {
    local: [..] string;

    returned_events = *local;
    record_returned();
    returned_events = null;

    return local;
}

read_after_reset :: () {
    local: [..] string;

    events = *local;
    record("bad");
    reset_temporary_storage();
    assert(local[0] == "bad");
    events = null;
    free(local.data);
}
