// Makes the game's placeholder assets from nothing: a sprite sheet and a bitmap font drawn from
// distance functions, sound effects and a music loop synthesised sample by sample, and the zip
// pack the game loads them from. Every asset is original and tiny, so the repository carries the
// generated files (game2d/game/assets) and this program only needs to run when one changes.
//
//     jaic examples/game2d/tools/assetgen/main.jai -exe assetgen
//     ./assetgen examples/game2d/game/assets            # rewrites the loose files
//     ./assetgen examples/game2d/game/assets --pack pack.zip   # also builds the zip pack
//
// The music loop is encoded to Ogg Vorbis outside this program (see ../README.md): the stdlib
// decodes Vorbis (stb_vorbis) but does not encode it. music.wav is written next to it.

#import "Basic";
#import "Math";
#import "File";
#import "File_Utilities";
#import "String";
#import "Extensions/Image";
#import "Extensions/Deflate";
#import,dir "../../sheet";

#load "draw.jai";
#load "audio.jai";

main :: () {
    args := get_command_line_arguments();

    if args.count < 2 {
        print("usage: assetgen OUTPUT_DIR [--pack FILE.zip]\n");
        exit(2);
    }

    directory := args[1];
    pack_path := "";

    if args.count >= 4 && args[2] == "--pack" pack_path = args[3];

    make_directory_if_it_does_not_exist(directory);

    files: [..] Pack_File;

    write_asset(*files, directory, "sprites.png", encode_sprite_sheet());
    write_asset(*files, directory, "font.png", encode_font());
    write_asset(*files, directory, "shoot.wav", wav_bytes(make_shoot()));
    write_asset(*files, directory, "hit.wav", wav_bytes(make_hit()));
    write_asset(*files, directory, "explode.wav", wav_bytes(make_explosion()));
    write_asset(*files, directory, "pickup.wav", wav_bytes(make_pickup()));
    write_asset(*files, directory, "music.wav", wav_bytes(make_music()));

    // Present only when the Ogg encode has been run once; the pack carries it when it exists.
    ogg_path := join(directory, "/music.ogg");

    if file_exists(ogg_path) {
        ogg, _ := read_entire_file(ogg_path);

        array_add(*files, .{ "music.ogg", cast([] u8) ogg });
    }

    if pack_path {
        if !write_zip(pack_path, files) {
            print("cannot write %\n", pack_path);
            exit(1);
        }

        print("packed % files into %\n", files.count, pack_path);
    }
}

write_asset :: (files: *[..] Pack_File, directory: string, name: string, bytes: [] u8) {
    path := join(directory, "/", name);

    if !write_entire_file(path, cast(string) bytes) {
        print("cannot write %\n", path);
        exit(1);
    }

    print("%: % bytes\n", path, bytes.count);

    // The wav of the music is only the source of the Ogg; the pack gets the Ogg.
    if name != "music.wav" array_add(files, .{ name, bytes });
}

Pack_File :: struct {
    name: string;
    bytes: [] u8;
}

// A zip with deflated entries, as `Zip_File_Directory` reads it: no data descriptors, no zip64.
write_zip :: (path: string, files: [] Pack_File) -> bool {
    out: String_Builder;

    offsets: [..] s64;
    crcs: [..] u32;
    packed_sizes: [..] s64;

    for file: files {
        offset := builder_string_length(*out);
        crc := crc32(file.bytes);
        packed := deflate(file.bytes, .RAW, 9);

        array_add(*offsets, offset);
        array_add(*crcs, crc);
        array_add(*packed_sizes, packed.count);

        put_u32(*out, 0x04034b50);
        put_u16(*out, 20);
        put_u16(*out, 0);
        put_u16(*out, 8);
        put_u16(*out, 0);
        put_u16(*out, 0x21);
        put_u32(*out, crc);
        put_u32(*out, cast(u32) packed.count);
        put_u32(*out, cast(u32) file.bytes.count);
        put_u16(*out, cast(u16) file.name.count);
        put_u16(*out, 0);
        append(*out, file.name);
        append(*out, cast(string) packed);
    }

    directory_start := builder_string_length(*out);

    for file: files {
        put_u32(*out, 0x02014b50);
        put_u16(*out, 20);
        put_u16(*out, 20);
        put_u16(*out, 0);
        put_u16(*out, 8);
        put_u16(*out, 0);
        put_u16(*out, 0x21);
        put_u32(*out, crcs[it_index]);
        put_u32(*out, cast(u32) packed_sizes[it_index]);
        put_u32(*out, cast(u32) file.bytes.count);
        put_u16(*out, cast(u16) file.name.count);
        put_u16(*out, 0);
        put_u16(*out, 0);
        put_u16(*out, 0);
        put_u16(*out, 0);
        put_u32(*out, 0);
        put_u32(*out, cast(u32) offsets[it_index]);
        append(*out, file.name);
    }

    directory_size := builder_string_length(*out) - directory_start;

    put_u32(*out, 0x06054b50);
    put_u16(*out, 0);
    put_u16(*out, 0);
    put_u16(*out, cast(u16) files.count);
    put_u16(*out, cast(u16) files.count);
    put_u32(*out, cast(u32) directory_size);
    put_u32(*out, cast(u32) directory_start);
    put_u16(*out, 0);

    return write_entire_file(path, builder_to_string(*out));
}

put_u16 :: (out: *String_Builder, value: u16) {
    bytes: [2] u8;

    bytes[0] = cast(u8) (value & 0xff);
    bytes[1] = cast(u8) (value >> 8);
    append(out, string.{ 2, bytes.data });
}

put_u32 :: (out: *String_Builder, value: u32) {
    put_u16(out, cast(u16) (value & 0xffff));
    put_u16(out, cast(u16) (value >> 16));
}
