#import "Basic";
Observed :: #import "Observed_Basic";

#assert #exists(String_Builder.Buffer.ensured_count) == EXPECTED_ASSERTIONS;
#assert Observed.ASSERTIONS_ENABLED == EXPECTED_ASSERTIONS;

#run print("child configuration accepted\n");

main :: () {}
