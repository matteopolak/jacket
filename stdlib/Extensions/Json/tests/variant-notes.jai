// The notes that choose how a Variant is tagged are accepted on a field of a variant, an array or
// a table of variants. The programs in negative/ each break one of the rules.
//
//     jaic stdlib/Extensions/Json/tests/variant-notes.jai -interp

#import "Basic";
#import "Hash_Table";
#import "Extensions/Json";
#import "Extensions/Variant";

Circle :: struct {
    radius: int;
}

Rect :: struct {
    width: int;
    height: int;
}

Doc :: struct {
    internal: Variant(Circle, Rect); @json(tag=kind)
    adjacent: Variant(Circle, Rect); @json(tag=t,content=c)
    untagged: Variant(Circle, Rect); @json(untagged)
    named: Variant(Circle, Rect); @json(tags=a,b)
    list: [..] Variant(Circle, Rect); @json(tag=kind)
    by_name: Table(string, Variant(Circle, Rect)); @json(tags=c,r)
}

main :: () {
    value: Doc;

    text := to_json(value);

    assert(text.count > 0);
    print("ok\n");
}
