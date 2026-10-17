#import "Basic";
D :: #import "Debug_Info";

main :: () {
    map: D.Free_Page_Map;

    array_resize(*map.rgw, 1);
    defer array_free(map.rgw);
    map.iwMac = 1;
    D.free_page(*map, 64);
}
