// Checking a body nothing calls runs the #run directives inside it.
#import "Basic";
unused :: () {
    #run assert(false, "the #run of an unreferenced body ran");
}
main :: () {}
