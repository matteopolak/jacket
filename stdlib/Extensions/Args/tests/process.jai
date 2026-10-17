// Real process argv, generated parser, environment values and exit statuses.
#import "Basic";
Args :: #import "Extensions/Args";

Options :: struct {
    count: int = 7; @env=EXTENSIONS_ARGS_COUNT @range(1,20)
}

HELP :: Args.Help(Options).{
    about = "Extensions process contract",
    version = "1.0",
    count = "A bounded count",
};

main :: () {
    options, set := Args.parse(Options, HELP);
    print("count=% set=%\n", options.count, set.count);
}
