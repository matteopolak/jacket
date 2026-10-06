// A module procedure that only an unreferenced procedure of the program calls is checked.
#import "Dce_Called";
unused :: () { broken(); }
main :: () { fine(); }
