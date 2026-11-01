# kvdb

A small key-value database: an append-only log of checksummed records, an in-memory index of the live keys, compaction, crash recovery, a command line, and a TCP server with a line protocol.

```sh
jaic examples/kvdb/main.jai -exe kvdb
./kvdb set greeting hello
./kvdb get greeting
./kvdb scan gree            # keys that start with "gree", in key order
./kvdb export               # key, tab, value in Base64
./kvdb compact
./kvdb serve --listen 127.0.0.1:7878 &
./kvdb remote --address 127.0.0.1:7878 SET name Ada Lovelace
```

The data file is `kvdb.log` in the current folder; `--file` or `KVDB_FILE` picks another.

## Map

- [main.jai](main.jai): The command line (`Extensions/Args` with subcommands).
- [store/](store): The engine. `record.jai` is the on-disk format and its checksum, `store.jai` the log, the index, recovery and compaction.
- [protocol/](protocol): The line protocol (`commands.jai`), the server and client on `Extensions/Io` fibers, and the line reader they share.
- [tests/](tests): `jacket test` in this folder. Recovery from a log cut at every byte of its last record, damage in the middle of the log, compaction, the record format, the protocol, and a server with several clients over real sockets.
- [tools/smoke.sh](tools/smoke.sh): The same story with the built program. Not part of CI.
- [FRICTION.md](FRICTION.md): What got in the way while writing it.

## How it works

Every `set` or `del` appends one record and hands it to the operating system before the index changes:

```
check(4) kind(1) key size(4) value size(4) key value
```

`check` is the first four bytes of the MD5 digest of everything after it. Opening the store replays the log from the start; the last record for a key wins. The index holds the values, so reads never touch the disk.

A crash can leave half a record at the end. Replay stops at the first record that is cut short, or whole but failing its checksum with nothing after it, and rewrites the log without it (there is no truncate in `File`, so the valid prefix is written beside the log and renamed over it). A record that fails its checksum with more records behind it did not come from a crash: the open fails with `CORRUPT` and the file is left alone.

`compact` writes one record per live key, in key order, to `kvdb.log.compact` and renames it over the log, so a crash during compaction leaves the old log or the new one.

The server runs on one `Io` scheduler: a fiber accepts, a fiber serves each client. A request runs from its first byte to its reply without waiting, so requests never overlap and the store needs no lock. One consequence: `COMPACT` and large writes hold up the other clients.

## Protocol

One request per line; verbs are case insensitive; keys have no spaces.

| Request | Reply |
| --- | --- |
| `GET key` | `VALUE text`, `VALUE64 base64` or `MISSING` |
| `SET key text` | `OK` (the text is the rest of the line) |
| `SET64 key base64` | `OK`, for values with line breaks or binary bytes |
| `DEL key` | `OK` or `MISSING` |
| `SCAN [prefix]` | `ROW key text` or `ROW64 key base64` per key, then `END` |
| `COMPACT` | `OK before after` (log bytes) |
| `STATS` | `STATS keys=N records=N log_bytes=N live_bytes=N` |
| `QUIT` | `OK`, then the connection closes |
| `SHUTDOWN` | `OK`, then the server stops |

Anything else is answered with `ERR message`.

## Things to try

- `kvdb stats` before and after `kvdb compact`: `live bytes` is the size the log will have.
- Cut the log with `head -c` in the middle of its last record and run `kvdb get` on an older key.
- Change a byte near the start of a log with several records and see the open refuse it.
- Change `checksum` in [store/record.jai](store/record.jai) to use another hash module, and run the tests.
