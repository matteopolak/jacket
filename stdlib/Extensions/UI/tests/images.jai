#if OS != .WASM {
    #import "Basic";
    #import "Extensions/Testing";
    #import "Extensions/UI";
    Image :: #import "Extensions/Image";

    rgba_images_draw_and_blend :: (t: *Test) {
        ui := headless(.{ width = 32, height = 24 });

        defer free_ui(*ui);
        picture := Image.make_image(2, 1, .RGBA8);

        defer Image.deinit(*picture);
        picture.pixels[0] = 255;
        picture.pixels[3] = 255;
        picture.pixels[6] = 255;
        picture.pixels[7] = 128;
        draw_image(*ui, .{ 0, 0, 20, 10 }, picture);
        assert(ui.draw.cmds.count == 1 && ui.draw.cmds[0].kind == .IMAGE);
        draw_image(*ui, .{ 0, 0, 0, 10 }, picture);
        assert(ui.draw.cmds.count == 1);
        renderer := create_offscreen_renderer(32, 24);
        if !renderer {
            skip(t, "no GPU adapter for image readback");
        }

        defer destroy_renderer(renderer);
        result := render_to_image(renderer, *ui, rgb(0, 0, 0));

        defer array_free(result.pixels);
        assert(result.pixels[4 * (2 * 32 + 2)] > 250);
        assert(result.pixels[4 * (2 * 32 + 15) + 2] >= 127 && result.pixels[4 * (2 * 32 + 15) + 2] <= 129);
    } @test
}
