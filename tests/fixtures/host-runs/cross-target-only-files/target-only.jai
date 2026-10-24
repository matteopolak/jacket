#library,system,link_always "NoSuchTargetFrameworkForTests";

#scope_file
Basic :: #import "Basic";

#run {
    arguments: [..] string;
    defer Basic.array_free(arguments);
    Basic.array_add(*arguments, "-Wl,-x");
    Basic.assert(arguments.count == 1);
}
