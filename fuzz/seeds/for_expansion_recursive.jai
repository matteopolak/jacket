S :: struct { }
for_expansion :: (s: *S, body: Code, flags: For_Flags) #expand { for s { #insert body; } }
main :: () { s: S; for s {} }
