"""Check a negative peer copy's consumed-request and fragmented-prefix contract.

Run against the compatibility copy produced by negative-peer.py. The pinned
original fails the same assertion; retain that red result when refreshing it.
"""

import importlib
import struct
import sys

sys.path.insert(0, sys.argv[1])
http2_base_server = importlib.import_module("http2_base_server")
messages_pb2 = importlib.import_module("messages_pb2")
server = http2_base_server.H2ProtocolBaseServer()
request = messages_pb2.SimpleRequest(response_size=314159)
request.payload.body = b"\0" * 271828
message = request.SerializeToString()
frame = b"\0" + struct.pack("!I", len(message)) + message
server._recv_buffer[1] = frame
assert server.parse_received_data(1).response_size == 314159
# A legal empty DATA END_STREAM must not deliver the same request again.
server._recv_buffer[1] += b""
try:
    repeated = server.parse_received_data(1)
except struct.error:
    repeated = "short prefix raised"
assert repeated is None, "consumed complete request was returned twice"
for count in range(5):
    server._recv_buffer[2] = frame[:count]
    assert server.parse_received_data(2) is None, "fragmented prefix must wait"
print("consumed request + all five fragmented-prefix lengths PASS")
