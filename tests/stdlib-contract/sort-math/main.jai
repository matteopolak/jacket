// Public sort ordering and stable numeric identities from Math.
#import "Basic";
Sort :: #import "Sort";
Math :: #import "Math";

Ranked :: struct {
    rank: s64;
    label: string;
}

main :: () {
    values: [5] s64 = .[9, -2, 9, 0, 4];
    sorted := Sort.quick_sort(values, (a, b) => a - b);

    assert(sorted[0] == -2 && sorted[1] == 0 && sorted[2] == 4);
    assert(sorted[3] == 9 && sorted[4] == 9);
    ranked: [3] Ranked = .[.{ rank = 8, label = "oak" }, .{ rank = -1, label = "ash" }, .{ rank = 3, label = "elm" }];

    Sort.quick_sort(ranked, item => item.rank);
    assert(ranked[0].rank == -1 && ranked[1].rank == 3 && ranked[2].rank == 8);
    assert(Math.PI64 > 3.141592653589 && Math.PI64 < 3.141592653590);
    print("ok\n");
}
