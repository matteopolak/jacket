QUOTE :: #code {
    generated :: () -> int {
        #if #run true {
            Selected :: #import, file "dependency.jai";
            return Selected.answer;
        } else {
            Missing :: #import, file "missing.jai";
            return 0;
        }
    }
};
#insert QUOTE;
main :: () -> int { return generated(); }
