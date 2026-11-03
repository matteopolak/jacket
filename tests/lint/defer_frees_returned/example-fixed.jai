#import "Basic";
#import "String";

full_name :: (first: string) -> string {
    name := join(first, " Smith");
    return name;
}

main :: () {
    name := full_name("Ada");
    print("%\n", name);
    free(name);
}
