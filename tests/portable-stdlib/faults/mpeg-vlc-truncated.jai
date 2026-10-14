M :: #import "pl_mpeg";

main :: () {
    buffer := M.buffer_create_with_memory(.[], false);
    table := M.Vlc.[.{ 0, 3 }, .{ 0, 7 }];
    M.buffer_read_vlc(buffer, table);
}
