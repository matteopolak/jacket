#import "Basic";

Kind :: enum {
    PNG;
    JPEG;
    WAV;
}

describe :: (kind: Kind) -> string {
    if kind == {
        case .PNG; #through;
        case .JPEG;
            return "image";
        case .WAV;
            return "audio";
    }

    return "";
}

main :: () {
    print("%\n", describe(.PNG));
}
