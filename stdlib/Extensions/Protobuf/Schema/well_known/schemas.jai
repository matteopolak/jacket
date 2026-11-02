#scope_module

// Independently written schema text from the documented google.protobuf type/field contracts:
// https://protobuf.dev/reference/protobuf/google.protobuf/
// Embedded literals make imports independent of installed .proto files.
proto_well_known :: (path: string) -> string {
    if path == "google/protobuf/any.proto" return "syntax='proto3'; package google.protobuf; message Any { string type_url=1; bytes value=2; }";
    if path == "google/protobuf/duration.proto" return "syntax='proto3'; package google.protobuf; message Duration { int64 seconds=1; int32 nanos=2; }";
    if path == "google/protobuf/timestamp.proto" return "syntax='proto3'; package google.protobuf; message Timestamp { int64 seconds=1; int32 nanos=2; }";
    if path == "google/protobuf/empty.proto" return "syntax='proto3'; package google.protobuf; message Empty {}";
    if path == "google/protobuf/field_mask.proto" return "syntax='proto3'; package google.protobuf; message FieldMask { repeated string paths=1; }";
    if path == "google/protobuf/wrappers.proto" return #string END
syntax='proto3';
package google.protobuf;
message DoubleValue { double value=1; }
message FloatValue { float value=1; }
message Int64Value { int64 value=1; }
message UInt64Value { uint64 value=1; }
message Int32Value { int32 value=1; }
message UInt32Value { uint32 value=1; }
message BoolValue { bool value=1; }
message StringValue { string value=1; }
message BytesValue { bytes value=1; }
END;
    if path == "google/protobuf/struct.proto" return #string END
syntax='proto3';
package google.protobuf;
message Struct { map<string,Value> fields=1; }
enum NullValue { NULL_VALUE=0; }
message Value {
  oneof kind {
    NullValue null_value=1;
    double number_value=2;
    string string_value=3;
    bool bool_value=4;
    Struct struct_value=5;
    ListValue list_value=6;
  }
}
message ListValue { repeated Value values=1; }
END;

    return "";
}
