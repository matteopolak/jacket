// Explicit private PRETTY_PRINT=false source package. Raw output is gated by
// both reviewed conditions; the ordinary true package has separate output tests.
#import "Basic";
Raw :: #import "rpmalloc"(USE_RAW = true);

probe :: #library,system "rpmalloc_callback_probe";
file_open :: () -> *void #foreign probe "binding_rpmalloc_statistics_file";
file_size :: (file: *void) -> s64 #foreign probe "binding_rpmalloc_statistics_file_size";
file_close :: (file: *void) -> s32 #foreign probe "binding_rpmalloc_statistics_file_close";

main :: () {
    assert(Raw.initialize() == 0);
    memory := Raw.rpmalloc(512);

    assert(memory != null);
    file := file_open();

    assert(file != null);
    Raw.dump_statistics(file);
    assert(file_size(file) == 0);
    assert(file_close(file) == 0);
    Raw.rpfree(memory);
    Raw.finalize();
}
