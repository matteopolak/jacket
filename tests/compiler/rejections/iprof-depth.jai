I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

main :: () {
    I.init_runtime();
    I.zones[0].name = "original bounded recursion";
    I.zones[0].initted = true;
    for 0..I.MAX_DEPTH I.zone_begin(0);
}
