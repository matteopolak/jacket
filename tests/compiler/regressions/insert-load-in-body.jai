// `#load` in code that a procedure body inserts loads the file into the module, with its path
// relative to the file that wrote the `#insert`. The inserted code can use what the file declares
// before the `#load` line.
#import "Basic";

generate :: () -> string {
    return "first := loaded_value(); #load \"modules/Loaded_By_Insert.jai\";";
}

main :: () {
    #insert #run generate();

    assert(first == 7);
    assert(loaded_value() == 7);
    assert(also_loaded() == 7);

    print("ok\n");
}

also_loaded :: () -> int {
    return loaded_value();
}
