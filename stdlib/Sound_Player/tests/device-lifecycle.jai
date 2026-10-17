#import "Basic";
#import "Sound_Player";

main :: () {
    config: Sound_Player_Config;
    config.update_from_a_thread = false;
    devices := get_devices();
    for device: devices assert(device.name.count > 0);

    refreshed := backend_get_devices();
    for device: refreshed assert(device.name.count > 0);
    for device: devices assert(device.name.count > 0, "previous borrowed discovery results remain valid");

    properties := backend_init(config);
    if properties.initted {
        print("native-device-opened\n");
        assert((properties.num_channels == 2 || properties.num_channels == 6 || properties.num_channels == 8) && properties.output_sampling_rate > 0);
        bytes, minimum := backend_count_buffered_bytes();

        assert(bytes == 0 && minimum > 0);
        requested := cast(s64) properties.num_channels * 2 * 256;
        regions, success := backend_lock_fill_regions(requested);

        assert(success && regions.buffer0.count == requested && regions.buffer1.count == 0);
        memset(regions.buffer0.data, 0, regions.buffer0.count);
        backend_release_fill_regions(regions);
        bytes, minimum = backend_count_buffered_bytes();
        assert(bytes >= 0);
        backend_play();
        sleep_milliseconds(20);
        backend_shutdown();
        config.seconds_to_fill_ahead = 0.6;
        properties = backend_init(config);
        assert(properties.initted, "positive finite fill-ahead must be bounded by native capacity, not arbitrarily rejected");
    } else {
        print("native-device-unavailable\n");
        requested := cast(s64) properties.num_channels * 2 * 256;
        regions, success := backend_lock_fill_regions(requested);
        assert(!success && regions.buffer0.count == 0);
    }

    backend_shutdown();
    backend_shutdown();
    print("ok\n");
}
