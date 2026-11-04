#import "Basic";

saved: string;

main :: () {
    saved = tprint("valid until process exit");
    print("%", saved);
}

retain :: () {
    saved = tprint("runtime caller controls resets");
}
