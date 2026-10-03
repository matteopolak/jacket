QUOTE :: #code {
    generated :: () -> int {
        #if #run enabled {
            Selected :: #import, file "dependency.jai";
            return Selected.answer;
        } else {
            return 1;
        }
    }
};
#load "first.jai";
#load "second.jai";
main :: () -> int { return from_first() + from_second(); }
