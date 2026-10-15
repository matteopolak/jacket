#import "Basic";
G :: #import "developer-tools-templates-generated";

main :: () {
    explicit: G.Explicit;
    selected: G.Selected;

    explicit.data[2] = 41;
    selected.data[0] = 17;
    assert(explicit.data.count == 3 && explicit.data[2] == 41);
    assert(selected.data.count == 3 && selected.data[0] == 17);
    assert(size_of(G.Selected) == 12 && size_of(G.Distinct) == 4 && size_of(G.Inherited) == 4);
    assert(G.Plain == s32 && G.Distinct != s32);
    print("ok\n");
}
