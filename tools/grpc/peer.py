#!/usr/bin/env python3
"""Independent grpcio peer using public generic-service APIs and opaque payloads.

python peer.py server --cert server.pem --key server.key --port-file port.txt
python peer.py client --ca ca.pem --target localhost:PORT
"""
import argparse
from concurrent import futures
from pathlib import Path
import signal
import threading
import grpc


def unary(value, context):
    context.send_initial_metadata((("x-initial", "one"),))
    context.set_trailing_metadata((("x-repeat", "one"), ("x-repeat", "two"), ("data-bin", b"\x00\xff")))
    if value in (b"deadline", b"cancel"):
        if value == b"deadline":
            assert 0 < context.time_remaining() < 1
            print("incoming deadline propagated", flush=True)
        ended = threading.Event()
        context.add_callback(ended.set)
        assert ended.wait(10), "client deadline/cancel did not end server context"
        print("incoming " + value.decode() + " terminated", flush=True)
        return b""
    if value == b"error":
        context.abort(grpc.StatusCode.INVALID_ARGUMENT, "bad %\nrequest")
    return value


def server_stream(value, context):
    yield value
    if value == b"late-error":
        context.abort(grpc.StatusCode.INVALID_ARGUMENT, "after data")
    yield b""
    yield value


def client_stream(values, context):
    return b"".join(values)


def bidi(values, context):
    for value in values:
        yield value


def protobuf_codecs(enabled):
    if not enabled:
        return {}
    import echo_pb2
    return {"request_deserializer": lambda raw: echo_pb2.Payload.FromString(raw).data,
            "response_serializer": lambda raw: echo_pb2.Payload(data=raw).SerializeToString()}


def serve(args):
    codecs = protobuf_codecs(args.protobuf)
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=8))
    server.add_generic_rpc_handlers((grpc.method_handlers_generic_handler("fixture.Echo", {
        "Unary": grpc.unary_unary_rpc_method_handler(unary, **codecs),
        "ServerStream": grpc.unary_stream_rpc_method_handler(server_stream, **codecs),
        "ClientStream": grpc.stream_unary_rpc_method_handler(client_stream, **codecs),
        "Bidi": grpc.stream_stream_rpc_method_handler(bidi, **codecs),
    }),))
    credentials = grpc.ssl_server_credentials(((args.key.read_bytes(), args.cert.read_bytes()),))
    port = server.add_secure_port("127.0.0.1:0", credentials)
    if not port:
        raise RuntimeError("failed to bind TLS listener")
    server.start()
    args.port_file.write_text(str(port))
    stopped = threading.Event()
    signal.signal(signal.SIGTERM, lambda *_: stopped.set())
    signal.signal(signal.SIGINT, lambda *_: stopped.set())
    stopped.wait()
    server.stop(0).wait()


def check_unary_metadata(call):
    """Independent verification of initial, repeated ASCII and raw binary trailers."""
    assert ("x-initial", "one") in tuple(call.initial_metadata())
    trailers = tuple(call.trailing_metadata())
    assert [value for name, value in trailers if name == "x-repeat"] == ["one", "two"]
    assert [value for name, value in trailers if name == "data-bin"] == [b"\x00\xff"]


def client(args):
    credentials = grpc.ssl_channel_credentials(args.ca.read_bytes())
    codecs = {}
    if args.protobuf:
        import echo_pb2
        codecs = {"request_serializer": lambda raw: echo_pb2.Payload(data=raw).SerializeToString(),
                  "response_deserializer": lambda raw: echo_pb2.Payload.FromString(raw).data}
    with grpc.secure_channel(args.target, credentials) as channel:
        unary_call = channel.unary_unary("/fixture.Echo/Unary", **codecs)
        result, call = unary_call.with_call(b"hello", metadata=(("x-client", "one"), ("x-client", "two"), ("client-bin", b"\x00\xff")), timeout=10)
        assert result == b"hello"
        check_unary_metadata(call)
        result, call = unary_call.with_call(b"", timeout=10)
        assert result == b""
        check_unary_metadata(call)
        try:
            unary_call(b"error", timeout=10)
            raise AssertionError("application error unexpectedly succeeded")
        except grpc.RpcError as error:
            assert error.code() == grpc.StatusCode.INVALID_ARGUMENT
            assert error.details() == "bad %\nrequest"
            check_unary_metadata(error)
        print("independent unary metadata/application error passed", flush=True)
        assert list(channel.unary_stream("/fixture.Echo/ServerStream", **codecs)(b"hi", timeout=10)) == [b"hi", b"", b"hi"]
        assert channel.stream_unary("/fixture.Echo/ClientStream", **codecs)(iter([b"a", b"", b"b"]), timeout=10) == b"ab"
        assert list(channel.stream_stream("/fixture.Echo/Bidi", **codecs)(iter([b"a", b"", b"b"]), timeout=10)) == [b"a", b"", b"b"]
        try:
            unary_call(b"deadline", timeout=.1)
            raise AssertionError("deadline call unexpectedly completed")
        except grpc.RpcError as error:
            assert error.code() == grpc.StatusCode.DEADLINE_EXCEEDED
        pending = channel.unary_stream("/fixture.Echo/ServerStream", **codecs)(b"cancel", timeout=10)
        assert next(pending) == b"cancel" # proves the server entered this streaming call
        assert pending.cancel()
        try:
            next(pending)
            raise AssertionError("cancel call unexpectedly completed")
        except grpc.RpcError as error:
            assert error.code() == grpc.StatusCode.CANCELLED
        assert unary_call(b"after-cancel", timeout=10) == b"after-cancel"
        print("independent deadline/cancel and sibling progress passed", flush=True)
        large = b"x" * (300 * 1024 + 3)
        assert list(channel.stream_stream("/fixture.Echo/Bidi", **codecs)(iter([large]), timeout=10)) == [large]
    print("all four independent-client modes passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)
    server_parser = sub.add_parser("server")
    server_parser.add_argument("--protobuf", action="store_true")
    for option in ("cert", "key", "port-file"):
        server_parser.add_argument("--" + option, required=True, type=Path)
    client_parser = sub.add_parser("client")
    client_parser.add_argument("--protobuf", action="store_true")
    client_parser.add_argument("--ca", required=True, type=Path)
    client_parser.add_argument("--target", required=True)
    args = parser.parse_args()
    (serve if args.mode == "server" else client)(args)


if __name__ == "__main__":
    main()
