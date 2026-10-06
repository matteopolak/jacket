#import "Basic";

report :: (format: string, args: ..Any) {
    print("report: ");
    print(format, ..args);
}

main :: () {
    name := "jai";
    print("% is % years old\n", name);
    print("done\n", name);
    report("% and %\n", 1);
    line := tprint("%1 and %1\n", name, 2);
    print(line);
}
