// The body of an inserted procedure is checked once something calls it.
#import "Basic";

#insert "update :: (value: int) { Missing_Buffer[0] = value; }\n";

main :: () {
    update(3);
}
