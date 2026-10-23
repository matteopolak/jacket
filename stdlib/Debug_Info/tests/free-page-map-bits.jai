#import "Basic";
D :: #import "Debug_Info";

main :: () {
    map: D.Free_Page_Map;

    D.init(*map, 4096, 4096);
    assert(D.is_free_page(map, 63));
    D.alloc_page(*map, 63);
    assert(!D.is_free_page(map, 63));
    D.free_page(*map, 63);
    assert(D.is_free_page(map, 63));
    print("ok\n");
}
