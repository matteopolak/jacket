#import "Basic";
#import "File";
#import "Math";
#import "Extensions/Testing";
#import,dir "../viewer";
#load "../fixtures.jai";

formats_and_decoders :: (t: *Test) {
    root := temp_dir(t);

    generate_fixtures(root);
    paths := string.["gradient.png", "gradient.jpg", "checker.bmp", "alpha.tga", "gradient.qoi", "gradient.ppm", "sizes.ico", "animated.gif"];
    formats := string.["PNG", "JPEG", "BMP", "TGA", "QOI", "PNM", "ICO", "GIF"];

    for paths {
        picture := decode_picture(tprint("%/%", root, it), true);

        defer release_picture(*picture);
        expect_equal(picture.format, formats[it_index]);
        expect(picture.image.pixels.count > 0, it);
        expect(picture.error.count == 0, it);

        if picture.format == "ICO" {
            expect(picture.image.width == 256 && picture.image.pixels[0] == 230 && picture.image.pixels[2] == 10);
        } else if picture.format == "GIF" {
            expect(picture.delays.count == 2 && picture.frames.count == 16);
            expect(picture.frames[0] == 255 && picture.frames[8 + 2] == 255);
        } else {
            expect(picture.image.width == 64 && picture.image.height == 32);
            if picture.format != "JPEG" expect(picture.image.pixels[0] == 255);
        }
    }
} @test

bad_data_and_signature :: (t: *Test) {
    expect_equal(format_name(cast([] u8) "GIF89a", "wrong.jpg"), "GIF");
    expect_equal(format_name(cast([] u8) "junk", "image.jpg"), "UNKNOWN");
    root := temp_dir(t);

    require(write_entire_file(tprint("%/bad.png", root), "junk"));
    picture := decode_picture(tprint("%/bad.png", root));

    defer release_picture(*picture);
    expect(picture.error.count > 0 && picture.image.pixels.count == 0);
    missing := decode_picture(tprint("%/absent.png", root));

    defer release_picture(*missing);
    expect(missing.error.count > 0);
} @test

thumbnails_keep_aspect_and_alpha :: (t: *Test) {
    w, h := thumbnail_size(400, 100, 112);

    expect(w == 112 && h == 28);
    w, h = thumbnail_size(1, 10000, 112);
    expect(w == 1 && h == 112);
    w, h = thumbnail_size(0, 100, 112);
    expect(w == 0 && h == 0);
    picture := make_fixture_picture();

    defer Image.deinit(*picture);
    small := thumbnail(picture, 16);

    defer Image.deinit(*small);
    expect(small.width == 16 && small.height == 8);
    expect(small.pixels[3] == 255);
    expect(small.pixels[(small.width - 1) * 4 + 3] == 128);
} @test

cache_evicts_least_recent_and_replaces :: (t: *Test) {
    cache: Cache;
    cache.cap = 32;
    defer free_cache(*cache);
    for 0..2 array_add(*cache.entries, .{});

    a := Image.make_image(2, 2, .RGBA8);
    b := Image.make_image(2, 2, .RGBA8);
    c := Image.make_image(2, 2, .RGBA8);

    expect(admit(*cache, 0, a));
    expect(admit(*cache, 1, b));
    cache.entries[0].stamp = 100;
    expect(admit(*cache, 2, c));
    expect(cache.entries[1].thumb.pixels.count == 0);
    expect(cache.entries[0].thumb.pixels.count == 16 && cache.used == 32);
    expect(admit(*cache, 0, Image.make_image(1, 1, .RGBA8)));
    expect(cache.used == 20);
    oversized := Image.make_image(3, 3, .RGBA8);

    defer Image.deinit(*oversized);
    expect(!admit(*cache, 1, oversized));
    expect(cache.used == 20);
} @test

zoom_anchors_and_clamps :: (t: *Test) {
    view := fit_view(400, 200, 800, 600);

    expect(view.zoom == 2 && view.x == 0 && view.y == 100);
    before_x := (123 - view.x) / view.zoom;
    before_y := (234 - view.y) / view.zoom;

    zoom_at(*view, 1.2, 123, 234);
    expect(abs((123 - view.x) / view.zoom - before_x) < 0.001);
    expect(abs((234 - view.y) / view.zoom - before_y) < 0.001);
    zoom_at(*view, 10000, 123, 234);
    expect(view.zoom == 64);
    zoom_at(*view, 0.000001, 123, 234);
    expect(view.zoom == 0.01);
} @test

worker_batches_respect_cap :: (t: *Test) {
    root := temp_dir(t);

    generate_fixtures(root);
    cache: Cache;
    cache.cap = 112 * 112 * 4;
    defer free_cache(*cache);
    require(open_folder(*cache, root));
    load_thumbnails(*cache, 0, 24);
    expect(cache.used <= cache.cap);
    failed := 0;
    for cache.entries if it.failed failed += 1;

    expect_equal(failed, 1);
    load_thumbnails(*cache, 0, 24);
    expect(cache.used <= cache.cap);
} @test

exports_round_trip :: (t: *Test) {
    image := make_fixture_picture();

    defer Image.deinit(*image);

    for string.["out.png", "out.jpg"] {
        path := tprint("%/%", temp_dir(t), it);

        require(export_picture(image, path));
        back := decode_picture(path);

        defer release_picture(*back);
        expect(back.image.width == 64 && back.image.height == 32);
    }

    expect(!export_picture(image, "unsupported.gif"));
} @test
