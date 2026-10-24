#import "Basic";

Kind :: enum {
    PNG;
    JPEG;
    WAV;
    MP3;
    TXT;
}

describe :: (kind: Kind) -> string {
    if kind == {
        case .PNG, .JPEG;
            return "image";
        case .WAV;
            return "audio";
        case .MP3;
            return "sound"; // a different word
        case .TXT;
            return "text";
        case;
            return "text";
    }

    return "";
}

fall :: (n: int) -> int {
    result := 0;

    if n == {
        case 1;
            result += 1;
            #through;
        case 2;
            result += 1;
        case 3;
        case 4;
    }

    return result;
}

main :: () {
    print("% %\n", describe(.PNG), fall(2));
}
