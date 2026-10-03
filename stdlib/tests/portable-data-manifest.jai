#import "Basic";
R :: #import "internal/resource-wire/manifest";
S :: #import "String";

main :: () {
    defaults: R.Manifest_Options;

    assert(defaults.dpi_aware && !defaults.require_administrator && defaults.use_segment_heap);

    for bits: 0..7 {
        options := R.Manifest_Options.{ dpi_aware = (bits & 1) != 0, require_administrator = (bits & 2) != 0, use_segment_heap = (bits & 4) != 0 };
        xml := R.generate_manifest(options);

        assert(S.contains(xml, "UTF-8") && !S.contains(xml, "assemblyIdentity"));
        assert(S.contains(xml, "requireAdministrator") == options.require_administrator);
        assert(S.contains(xml, "asInvoker") == !options.require_administrator);
        assert(S.contains(xml, "permonitorv2,permonitor") == options.dpi_aware);
        assert(S.contains(xml, ">unaware<") == !options.dpi_aware);
        assert(S.contains(xml, "SegmentHeap") == options.use_segment_heap);
        print("%\n", xml);
        free(xml);
    }
}
