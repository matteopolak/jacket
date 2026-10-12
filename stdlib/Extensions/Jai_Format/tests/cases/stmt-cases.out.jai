name :: (v: Kind) -> string {
    if v == {
        case .A; return "a";
        case .B; return "b";
        case .C;
            print("c");
            return "c";

        case .D;
            return "d";

        case .E; #through;
        case .F;
            x := 1;
            return "f";

        case .G;
        case .H;
            return "h";

        case;
            return "";
    }
}

nested :: (v: Kind) {
    if v == {
        case 1;
            if v == {
                case 1;
                    a();
                    b();

                case 2;
                    c();
            }

        case 2; d();
    }
}
