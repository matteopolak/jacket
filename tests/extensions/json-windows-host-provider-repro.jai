Json :: #import "Extensions/Json";
Tls :: #import "Extensions/Tls";

Value :: struct {
    label: string;
}

main :: () {
    value: Value;
    Json.free_json(*value);
}
