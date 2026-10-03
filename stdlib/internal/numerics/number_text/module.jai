#load "format.jai";

parse_number :: (text: string) -> (success: bool, value: float64) {
    parsed, ok, rest := string_to_float64(text);
    if !ok || eat_spaces(rest).count != 0 return false, 0;
    return true, parsed;
}

#scope_file
#import "Basic";
