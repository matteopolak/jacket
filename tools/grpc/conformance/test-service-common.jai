// Canonical TestService adapter. GRPC_PROTO_ROOT names the pinned official proto tree.
#import "Basic";
#import "File";
#import "File_Utilities";
G :: #import "Extensions/Grpc";
P :: #import "Extensions/Protobuf";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
Api :: #import "Extensions/Protobuf/Schema"(GRPC = true, PATH = #run tprint("%/src/proto/grpc/testing/test.proto", GRPC_PROTO_ROOT), IMPORT_PATHS = GRPC_PROTO_ROOT);

read_message :: (call: *$T, $Message: Type) -> Message, G.Receive_Kind {
    #if T == G.Call {
        bytes, kind := G.receive_bytes(call);
        if kind != .MESSAGE return .{}, kind;

        defer if bytes.data free(bytes.data);
        message, error := P.decode(Message, bytes);
        if error.kind != .NONE {
            P.free(*error);
            return .{}, .ERROR;
        }

        return message, kind;
    } else return T.receive(call);
}

write_message :: (call: *$T, message: $M, compressed := false) -> G.Status {
    #if T == G.Call {
        bytes, error := P.encode(message);
        if error.kind != .NONE {
            P.free(*error);
            return .{ code = .INTERNAL };
        }

        defer free(bytes.data);

        return G.send_bytes(call, bytes, .{ compress = compressed });
    } else return T.send(call, message, .{ compress = compressed });
}

compressed_request :: (call: *$T) -> bool {
    #if T == G.Call return G.received_compressed(call);
    else return T.received_compressed(call);
}

peer :: (call: *$T) -> string {
    #if T == G.Call return Io.address_to_string(G.peer_address(call));
    else return Io.address_to_string(T.peer_address(call));
}

metadata :: (call: *$T) -> [] G.Metadata_Entry {
    #if T == G.Call return G.initial_metadata(call);
    else return T.metadata(call);
}
