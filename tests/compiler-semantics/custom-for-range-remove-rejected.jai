// A numeric loop has no removable array, including when its body is inserted by a macro.
Sequence :: struct {}

for_expansion :: (sequence: Sequence, body: Code, flags: For_Flags) #expand {
    for `it: 0..3 {
        #insert body;
    }
}

main :: () {
    sequence: Sequence;
    for sequence remove;
}
