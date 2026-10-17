// docs/specs/stdlib/modules/minimal-metaprogram.md: deliberately limited heuristic.
#import "Basic";
M :: #import "Minimal_Metaprogram";

main :: () {
    if M.is_absolute_path("") exit(2);
    if M.is_absolute_path("C:") exit(3);
    if M.is_absolute_path("relative/file.jai") exit(4);
    if M.is_absolute_path("a") exit(5);
    if !M.is_absolute_path("/file.jai") exit(6);
    if !M.is_absolute_path("\\file.jai") exit(7);
    if !M.is_absolute_path("C:x") exit(8);
    if !M.is_absolute_path("?:x") exit(9);
    if !M.is_absolute_path("C:/file.jai") exit(10);
    if !M.is_absolute_path("C:\\file.jai") exit(11);

    write_string("ok\n");
}
