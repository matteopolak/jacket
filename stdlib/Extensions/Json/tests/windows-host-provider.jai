Json :: #import "Extensions/Json";
Tls :: #import "Extensions/Tls"; // jailint: allow(unused_import) the import pulls in the provider under test

Value :: struct {
    label: string;
}

main :: () {
    value: Value;
    Json.free_json(*value);
}
