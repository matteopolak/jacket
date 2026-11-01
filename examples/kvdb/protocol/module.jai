// The network side of kvdb: a line protocol (commands.jai), a server that runs it on fibers
// (server.jai) and a client for it (client.jai). Imported with `#import,dir "../protocol"`; it
// brings the store with it.

#import "Basic";
#import "String";
#import "Base64";
Io :: #import "Extensions/Io";
#import,dir "../store";

#load "lines.jai";
#load "commands.jai";
#load "server.jai";
#load "client.jai";
