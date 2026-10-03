compiler_create_workspace :: (name: string) -> s64 #compiler;
main :: () -> int { return cast(int) #run compiler_create_workspace("authored-no-effects"); }
