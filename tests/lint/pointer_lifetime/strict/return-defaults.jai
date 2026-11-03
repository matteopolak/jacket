#import "Basic";

Record :: struct {
    text: string;
}

saved: string;

nullable :: (convert: bool) -> Record {
    result: Record;
    if !convert return result;
    result.text = tprint("temporary");
    return result;
}

mixed_record :: (convert: bool) -> Record {
    result: Record;
    result.text = ifx convert then tprint("temporary") else copy_string("owned");
    return result;
}

mixed_scalar :: (convert: bool) -> string {
    if convert return tprint("temporary");
    return copy_string("owned");
}

copy_mixed_record :: (convert: bool) {
    result := mixed_record(convert);
    saved = copy_string(result.text);
}

copy_mixed_scalar :: (convert: bool) {
    saved = copy_string(mixed_scalar(convert));
}

retain_nullable :: (convert: bool) {
    result := nullable(convert);
    saved = result.text;
}

retain_mixed_record :: (convert: bool) {
    result := mixed_record(convert);
    saved = result.text;
}

retain_mixed_scalar :: (convert: bool) {
    saved = mixed_scalar(convert);
}
