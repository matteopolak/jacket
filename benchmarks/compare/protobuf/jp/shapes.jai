// jai-protobuf's types and codecs for proto/shapes_flat.proto, in a namespace of their own: the library
// generates top-level `Serialize`, `Deserialize` and message names that would clash with the others.
// Imported by benches/protobuf.jai; only ever built inside the sandbox of tools/bench/library_compare.py.
#import "Basic";
#import "segcore--jai-protobuf";

#insert #run generate_jai(tprint("%/../proto/shapes_flat.proto", #filepath));
