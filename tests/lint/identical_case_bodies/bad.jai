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
        case .PNG;
            return "image";
        case .JPEG;
            return "image";
        case .WAV;
            return "audio";
        case .MP3;
            return "audio";
        case .TXT;
            return "text";
    }

    return "";
}

weight :: (n: int) -> int {
    result := 0;

    if n == {
        case 1;
            result += 1;
            print("light\n");
        case 5;
            result += 5;
        case 2;
            result += 1;
            print("light\n");
    }

    return result;
}

main :: () {
    print("% %\n", describe(.PNG), weight(2));
}
