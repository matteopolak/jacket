// A local procedure cannot read a variable of the procedure around it.
#import "Basic";
wrapper :: () {
    val := 10;
    helper :: () {
        print("%\n", val);
    }
    helper();
}
main :: () { wrapper(); }
