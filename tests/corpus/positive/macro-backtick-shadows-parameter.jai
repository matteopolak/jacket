#import "Basic";
Thing :: struct { x: int; }
each :: (engine: *Thing, body: Code) #expand {
    `engine := engine;
    engine.x += 10;
    #insert body;
}
main :: () {
    t := Thing.{3};
    each(*t, #code print("%\n", engine.x));
    print("%\n", t.x);
}
