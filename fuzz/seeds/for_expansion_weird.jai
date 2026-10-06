S :: struct { }
for_expansion :: (s: *S, body: Code, flags: For_Flags) #expand { #insert body; #insert body; }
main :: () { s: S; for s { for s { break; } } }
