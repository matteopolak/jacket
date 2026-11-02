#if OS != .WASM {
    #import "Basic";
    #import "Extensions/Testing";
    #import "Extensions/UI";
    Image :: #import "Extensions/Image";

    rgba_images_record_borrowed_pixels :: (t: *Test) {
        ui := headless(.{ width = 32, height = 24 });

        defer free_ui(*ui);
        picture := Image.make_image(2, 1, .RGBA8);

        defer Image.deinit(*picture);
        draw_image(*ui, .{ 0, 0, 20, 10 }, picture);
        assert(ui.draw.cmds.count == 1 && ui.draw.cmds[0].kind == .IMAGE);
        draw_image(*ui, .{ 0, 0, 0, 10 }, picture);
        assert(ui.draw.cmds.count == 1);
        assert(ui.draw.cmds[0].image.pixels.data == picture.pixels.data);
    } @test
}
