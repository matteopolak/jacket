classify :: (n: int) -> string {
    if n == {
        case 0;
            return "zero";
        // Before a case: case level.
        case 1; #through;
        case 2;
            // Inside a case body: body level.
            return "small";
        case;
            if #complete kind == {
                case .A;  return "a";
                case .B;  return "b";
            }
    }
    return "other";
}
