#import "Basic";
#import "String";
#import "Extensions/Vt";

Capture :: struct {
    text: String_Builder;
}

record :: (data: *void, a: Vt_Action) {
    c := cast(*Capture) data;

    append(*c.text, tprint("%:%", a.kind, a.code));
    for a.params append(*c.text, tprint("/%:%", it.value, it.subparam));

    append(*c.text, tprint("[%]", a.intermediates));
    append(*c.text, tprint("?%", a.private_marker));
    for a.osc append(*c.text, tprint("{%}", it));

    append(*c.text, "|");
}

trace :: (bytes: string, split: int, options: Vt_Options = .{}) -> string {
    p: Vt_Parser;
    p.options = options;
    c: Capture;

    vt_feed(*p, slice(bytes, 0, split), record, *c);
    vt_feed(*p, slice(bytes, split, bytes.count - split), record, *c);
    vt_finish(*p, record, *c);

    return builder_to_string(*c.text);
}
