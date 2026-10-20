// A polymorphic struct is not a type until it is given its arguments.
#import "Basic";
Grid :: struct (Cell: Type, W: int) { cells: [W] Cell; }
main :: () {
    g: Grid;
}
