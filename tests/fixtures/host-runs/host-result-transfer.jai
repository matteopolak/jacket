HOST_OK :: #run,host -> bool {
    return OS == BUILD_OS && CPU == BUILD_CPU;
};

TEXT :: #run,host -> string {
    return "isolated host";
};

FIXED :: #run,host -> [3] int {
    return .[5, 8, 13];
};

VIEW :: #run,host -> [] int {
    values: [3] int = .[2, 3, 7];
    return values;
};

GROW :: #run,host -> [..] int {
    B :: #import "Basic";
    values: [..] int;

    B.array_add(*values, 11);
    B.array_add(*values, 19);

    return values;
};

#assert HOST_OK;
#assert TEXT == "isolated host";
#assert FIXED[2] == 13 && VIEW[0] == 2 && VIEW[2] == 7;
#assert GROW.count == 2 && GROW[0] == 11 && GROW[1] == 19;

main :: () -> int {
    if !HOST_OK return 1;
    if TEXT != "isolated host" return 2;
    if FIXED[2] != 13 return 3;
    if VIEW.count != 3 || VIEW[0] != 2 || VIEW[2] != 7 return 4;
    if GROW.count != 2 || GROW[0] != 11 || GROW[1] != 19 return 5;

    return 0;
}
