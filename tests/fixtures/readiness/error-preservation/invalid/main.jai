#if Flags.ready {
    selected :: true;
}

#assert selected;

main :: () {}

Flags :: #import,file "flags.jai";
