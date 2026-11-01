# Chat

A small multi-room chat server, terminal client and command bot written in Jai. It exercises Io fibers, TLS, `Print_Color`, `Shared_Memory_Channel`, `Mail`, file recovery and an OS worker for blocking SMTP.

```sh
jaic examples/chat/main.jai -exe "$T/chat"
"$T/chat" serve --file "$T/chat.log"
"$T/chat" client --nick Ada --room lobby
"$T/chat" bot --nick helper --room lobby
"$T/chat" load --clients 4 --rounds 10
"$T/chat" stats
(cd examples/chat && jacket test)
sh examples/chat/tools/smoke.sh
```

Use `--help` on any subcommand for all flags. `serve` defaults to `127.0.0.1:7979`, `chat.log`, history 20, rate 5 commands/second with a burst of 20, and stats channel `jacket-chat`. Only one server may own a history file; there is no file locking. This is a local dogfood app with unauthenticated, case-sensitive nicknames, rather than a public messaging service.

## Protocol

UTF-8 bytes in LF or CRLF terminated lines. Commands are uppercase. A line may have at most 2048 bytes; names are 1–24 ASCII letters, digits, `_` or `-`. Text has 1–1024 bytes and no ASCII control bytes (including tabs and terminal escapes). Space is a literal single separator; spaces inside `text` are retained.

```text
command = "NICK " name | "JOIN " name | "LEAVE" | "HISTORY"
        | "SAY " text | "DM " name " " text | "EMAIL " address
        | "PING" | "QUIT"
```

The server first sends `READY chat/1`. Each command ends with exactly one `OK VERB` or `ERR reason` line. Events may precede that response: `JOINED room nick`, `LEFT room nick`, `MSG room nick text`, `PRIVATE sender text`, or `HISTORY room nick text`. There are no multiline bodies. Clients issue commands sequentially and drain events until their `OK`/`ERR` response. `HISTORY` returns the current room's retained messages, oldest first, then `OK HISTORY`.

`NICK` is required before room commands and can only be set once per connection. A connected nick is exclusive; disconnect makes it reusable. `JOIN` leaves the previous room, `LEAVE` leaves without joining another, and `SAY` requires a room. A client is in at most one room. `DM` delivers to a connected nickname in any room, without persisting it. `QUIT` drains queued responses before closing. A disconnect halfway through a command drops that command. The limiter counts every valid command, including `PING`, rejected state changes and `QUIT`; syntax errors do not consume tokens.

## Client and bot

The POSIX client uses a 24-row ANSI terminal: rows 1–22 scroll messages, row 23 shows help, row 24 holds input. Incoming events save and restore the input cursor. The terminal's usual line editing and echo remain active. Text becomes `SAY text`; `/JOIN room`, `/DM nick text`, `/HISTORY`, `/LEAVE` and `/QUIT` send protocol commands. `Print_Color` chooses a stable color from the nickname. Ctrl-C exits the client. Stdin is opened as an Io stream, so waiting for a line does not block the network fiber. Windows supports scripted clients but has no interactive terminal implementation here.

`client --script PATH` reads protocol lines. `WAIT MS` receives events for up to 10 seconds; `EXPECT-ERROR command` asserts a server rejection. An unexpected error or transport failure exits 1; success exits 0. EOF adds `QUIT` if the script did not include it.

```text
SAY hello everyone
DM Bob hello privately
EXPECT-ERROR JOIN ../invalid
WAIT 500
HISTORY
QUIT
```

The bot implementation is in `bot/`. It answers `!ping` with `pong`, `!echo TEXT` with TEXT and `!help` with its command list. It ignores its own messages and ordinary chat. `--duration MS` bounds a test session; default `-1` runs until disconnection.

## Persistence and limits

Each accepted `SAY` appends `room<TAB>nick<TAB>text<TAB>md5<LF>` and flushes it before broadcasting. MD5 checks accidental corruption; it is not an authentication mechanism. Startup replays the log, keeping only `--history N` messages per room (1–100). A final unterminated record is removed by writing a `.recover` sibling and atomically replacing the log. A corrupt complete record fails startup and preserves the original file. Write failure disables later appends for that process; the rejected message never enters live history. A flush does not promise survival of a power failure.

The server caps rooms at 32, remembered nicknames at 256, concurrent connections at 256, and each output queue at 128 lines. A client whose queue fills is disconnected. One output fiber serializes each client's traffic; a two-second write deadline bounds stalled peers. Room changes and disk writes run without fiber suspension, preserving mutation order. On this Mac, disk flush can briefly stall the scheduler. SIGINT/SIGTERM close the listener, cancel readers, drain bounded output, join handlers and finish queued email before reporting `STOPPED`. Signal handling is POSIX; Windows supports the protocol but not this shutdown path.

## TLS

```sh
openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=localhost \
  -addext subjectAltName=DNS:localhost -keyout "$T/key.pem" -out "$T/cert.pem"
"$T/chat" serve --tls-cert "$T/cert.pem" --tls-key "$T/key.pem" --file "$T/tls.log"
"$T/chat" client --nick Ada --roots "$T/cert.pem"
```

Providing both server flags enables TLS for that listener. Client, bot and load use `--roots CA.pem` to enable TLS, check the chain and verify the name `localhost`. The TLS configuration outlives its streams. Remote hostname selection is an open limitation. TLS depends on the repository's native mbedTLS libraries.

## Stats and offline mentions

`chat stats --channel NAME` creates the single-reader shared-memory channel, then waits up to four seconds for a snapshot. The server polls for that reader once a second and publishes `connections=N messages/s=N total=N`. The rate is the number of accepted room messages in the preceding monitor interval (nominally one second). The reader releases and unlinks its channel on exit. Do not run multiple readers with the same name. POSIX shared memory must be allowed by the host sandbox.

`EMAIL ada@example.test` registers an address for the connection's nickname. If that nick subsequently disconnects, room messages containing a token `@Ada` accumulate into a digest (at most 32 pending mentions per user). Configure `serve --smtp-host HOST --smtp-port PORT` to send them. Addresses, remembered offline nicknames and unsent digests live only in memory; room history is the only persisted state. A worker sends one digest per user per batch, with a two-second SMTP timeout; the server checks for another batch every second. SMTP is unauthenticated plaintext in this example. Failed deliveries are logged and dropped, with no retry queue.

`chat smtp-sink --port 2525 --file "$T/digest.eml"` accepts one email on loopback and writes its DATA. The smoke test uses only this Jai sink, never an external SMTP service. `Mail` depends on native libcurl.

## Load, tests and extension points

`load --clients N --rounds N` starts simultaneous Io clients, joins the `load` room and measures elapsed time from sending each `SAY` to receiving its acknowledgement. It reports accepted messages/s and sorted sample p50/p99 latencies in milliseconds, and fails if a connection or command fails. Each worker has a ten-second deadline. Large runs require a higher server `--rate`; default runs fit its initial burst.

`chat/model.jai` owns parsing, bounded histories, checksummed persistence and the token bucket. `server.jai` owns connections and room command execution; `transport.jai` bridges TCP and TLS; `client.jai` owns terminal/script/load clients; `services.jai` owns SMTP and stats. Extend the protocol in `parse_request`, `execute`, tests and the grammar above. Keep all queued strings owned, and join writer fibers before freeing a peer. Do not add yielding operations inside room mutation or persistence without introducing serialization.

Tests cover grammar, rate refill, mention boundaries, bounded isolated histories, room caps, nickname exclusivity, join/leave routing, private messages, recovery and corrupt records. The smoke orchestrates the actual binary, client scripts, interactive stdin through a pseudo-terminal, bot, load tool, stats and SMTP sink; it restarts history after an incomplete append and exercises TLS with generated certificates and an untrusted CA. It skips stats only when an OS probe reports permission denied or unavailable shared memory, and skips TLS when `openssl` is absent. `CHAT_KEEP_WORK=1` retains transcripts beneath `$T`; build output otherwise lives in a temporary directory beneath `$T` (or `$TMPDIR` when T is unset). `JAIC` selects the compiler. Python 3 is required only for smoke orchestration.
