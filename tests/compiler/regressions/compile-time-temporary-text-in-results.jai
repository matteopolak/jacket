// Text a `#run` builds in temporary storage and keeps inside a struct or array result is still
// there in the program.
#import "Basic";

Label :: struct {
    text: string;
}

one_label :: () -> Label {
    label: Label;
    label.text = tprint("entry-%", 2);

    return label;
}

many_labels :: () -> [] Label {
    labels := NewArray(2, Label);
    labels[1].text = tprint("entry-%", 3);

    return labels;
}

words :: () -> [] string {
    result := NewArray(2, string);
    result[0] = tprint("a%", 1);
    result[1] = tprint("b%", 2);

    return result;
}

ONE :: #run one_label();
MANY :: #run many_labels();
WORDS :: #run words();

main :: () {
    assert(ONE.text == "entry-2");
    assert(MANY[0].text.count == 0 && MANY[1].text == "entry-3");
    assert(WORDS[0] == "a1" && WORDS[1] == "b2");
    print("ok\n");
}
