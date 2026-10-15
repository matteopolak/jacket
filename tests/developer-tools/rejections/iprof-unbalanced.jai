I :: #import "Iprof"(IMPORT_MODE = .CLIENT, MANUAL_MODE = true, USE_GRAPHICS = false);

main :: () {
    I.init_runtime();
    I.zone_end();
}
