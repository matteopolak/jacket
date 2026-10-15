Data :: #import "Probe";
Alias :: Data;
#add_context audit_marker: int;
main :: () {
    note := "UTF16: 🦊";
    a := Data.scalar;
    b := Alias.state;
    item: Data.Number;
    context.audit_marker = a;
    item = context.audit_marker;
    {
        using,only(scalar) Data;
        item = scalar;
    }
    if note == "" { b += 1; }
    Data.state = b;
}
