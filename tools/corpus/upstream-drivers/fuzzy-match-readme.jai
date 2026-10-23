#import "Basic";
#import,dir "isomorphic-tech--fuzzy-match-jai/Fuzzy_Match";

main :: () {
    score := fuzzy_match("Hello there, friend!", "hethe");
    score2 := fuzzy_match("Hello there, friend!", "xyz");
    print("score %\n", score);
    print("no match %\n", score2 == FUZZY_NO_MATCH);
}
