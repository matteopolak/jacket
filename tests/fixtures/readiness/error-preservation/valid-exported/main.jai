#if Flags.ready {
    selected :: 42;
}

#assert selected == 42;

main :: () {}

Flags :: #import,file "flags.jai";
