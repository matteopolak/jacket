// test: only(.MACOS)
// A Metal source with a syntax error produces no library and an NSError with a non-empty description.
#import "Basic";
#import "Metal";
#import "Objective_C";

main :: () {
    init_metal();

    device := cast(*MTLDevice) MTLCreateSystemDefaultDevice();

    assert(device != null);

    error: *NSError;
    library := MTLDevice.newLibraryWithSource_options_error(device, NSString.getTempString("kernel void broken( {"), null, *error);

    print("library null: %\n", library == null);
    print("error set: %\n", error != null);
    print("description non-empty: %\n", NSString.length(NSError.localizedDescription(error)) > 0);
}
