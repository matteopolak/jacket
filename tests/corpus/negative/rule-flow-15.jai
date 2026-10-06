// `if #complete` on an enum must name every member in some case, even with a default label.
// rules: flow.15
Dir :: enum { NORTH; EAST; SOUTH; WEST; }

main :: () {
    d := Dir.EAST;
    if #complete d == {
        case .NORTH;
        case .EAST;
        case .SOUTH;
        case;
    }
}
