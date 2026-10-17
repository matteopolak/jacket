// Retained original probe, now superseded for the ConvertToApollo handoff.
// The reviewed API uses caller-scoped `return in a typed caller, not a value
// expression from this ordinary no-result macro. No stdlib compatibility claim.
value :: (input: s64) #expand {
    return input * 2;
}

main :: () {
    result := value(21);
    if result != 42 assert(false);
}

#import "Basic";
