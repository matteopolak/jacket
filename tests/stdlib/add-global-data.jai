#import "Basic";
#import "Compiler";

main :: () {
    data :: #run add_global_data(xx "GIF89a-bytes", .READ_ONLY);
    assert(data.count == 12 && data[0] == #char "G" && data[11] == #char "s");
    print("ok\n");
}
