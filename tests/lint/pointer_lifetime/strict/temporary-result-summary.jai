#import "Basic";

Record :: struct {
    text: string;
}

saved: Record;

possible_result :: (temporary: bool) -> Record {
    result: Record;
    result.text = ifx temporary then tprint("temporary") else copy_string("owned");
    return result;
}

consume_result :: (temporary: bool) {
    result := possible_result(temporary);
    print("%", result.text);
}

retain_result :: (temporary: bool) {
    saved = possible_result(temporary);
}
