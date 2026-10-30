// The part of the game and its server that has to agree: the JSON documents, the routes and the
// checksum a finished match is signed with. Both programs import this folder with
// `#import,dir "../shared"`, so a change to a document is a compile error on whichever side
// forgot to follow it.

#load "protocol.jai";
#load "sim.jai";
#load "client.jai";
