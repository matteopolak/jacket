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
    if path == "google/protobuf/source_context.proto" return "syntax='proto3'; package google.protobuf; message SourceContext { string file_name=1; }";
    if path == "google/protobuf/type.proto" return #string END
syntax='proto3';
package google.protobuf;
import 'google/protobuf/any.proto';
import 'google/protobuf/source_context.proto';
enum Syntax { SYNTAX_PROTO2=0; SYNTAX_PROTO3=1; SYNTAX_EDITIONS=2; }
message Type {
  string name=1; repeated Field fields=2; repeated string oneofs=3; repeated Option options=4;
  SourceContext source_context=5; Syntax syntax=6; string edition=7;
}
message Field {
  enum Kind {
    TYPE_UNKNOWN=0; TYPE_DOUBLE=1; TYPE_FLOAT=2; TYPE_INT64=3; TYPE_UINT64=4; TYPE_INT32=5;
    TYPE_FIXED64=6; TYPE_FIXED32=7; TYPE_BOOL=8; TYPE_STRING=9; TYPE_GROUP=10; TYPE_MESSAGE=11;
    TYPE_BYTES=12; TYPE_UINT32=13; TYPE_ENUM=14; TYPE_SFIXED32=15; TYPE_SFIXED64=16;
    TYPE_SINT32=17; TYPE_SINT64=18;
  }
  enum Cardinality { CARDINALITY_UNKNOWN=0; CARDINALITY_OPTIONAL=1; CARDINALITY_REQUIRED=2; CARDINALITY_REPEATED=3; }
  Kind kind=1; Cardinality cardinality=2; int32 number=3; string name=4; string type_url=6;
  int32 oneof_index=7; bool packed=8; repeated Option options=9; string json_name=10; string default_value=11;
}
message Enum {
  string name=1; repeated EnumValue enumvalue=2; repeated Option options=3;
  SourceContext source_context=4; Syntax syntax=5; string edition=6;
}
message EnumValue { string name=1; int32 number=2; repeated Option options=3; }
message Option { string name=1; Any value=2; }
END;
    if path == "google/protobuf/api.proto" return #string END
syntax='proto3';
package google.protobuf;
import 'google/protobuf/source_context.proto';
import 'google/protobuf/type.proto';
message Api {
  string name=1; repeated Method methods=2; repeated Option options=3; string version=4;
  SourceContext source_context=5; repeated Mixin mixins=6; Syntax syntax=7;
}
message Method {
  string name=1; string request_type_url=2; bool request_streaming=3; string response_type_url=4;
  bool response_streaming=5; repeated Option options=6; Syntax syntax=7;
}
message Mixin { string name=1; string root=2; }
END;
    // Only the option messages, so schemas can declare custom options. Put the full
    // descriptor.proto on the import path to use the descriptor types themselves.
    if path == "google/protobuf/descriptor.proto" return #string END
syntax='proto2';
package google.protobuf;
message FileOptions { extensions 1000 to max; }
message MessageOptions { extensions 1000 to max; }
message FieldOptions { extensions 1000 to max; }
message OneofOptions { extensions 1000 to max; }
message EnumOptions { extensions 1000 to max; }
message EnumValueOptions { extensions 1000 to max; }
message ServiceOptions { extensions 1000 to max; }
message MethodOptions { extensions 1000 to max; }
message ExtensionRangeOptions { extensions 1000 to max; }
message FeatureSet { extensions 1000 to max; }
END;

    return "";
}
