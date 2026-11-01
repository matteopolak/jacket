// sitegen: a small static site generator. Markdown pages and a template go in, HTML comes out, and
// every other file is published under a name that contains a hash of its content.
// README.md is the guide.
//
//     jaic examples/sitegen/main.jai -exe sitegen
//     ./sitegen examples/sitegen/demo public
//     ./sitegen site public --template theme.html --inline-limit 4096 --verbose

#import "Basic";
Args :: #import "Extensions/Args";
#import,dir "site";

Cli :: struct {
    src: string; @positional @required
    out: string; @positional @required
    template: string; @short=t @value=FILE
    inline_limit: s64 = 2048; @short=i @value=BYTES
    verbose: bool; @short=v
}

HELP :: Args.Help(Cli).{
    about = "Builds a static site from a folder of Markdown pages.",
    version = "0.1.0",
    src = "The folder of pages (.md) and assets; names starting with . or _ are not published",
    out = "The folder to write the site to",
    template = "The page template (default: _template.html in the source folder, else a built-in page)",
    inline_limit = "Inline images up to this many bytes as data: URIs",
    verbose = "List every file written",
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    options := Build_Options.{ template_path = cli.template, inline_limit = cli.inline_limit };
    result := build_site(cli.src, cli.out, options);

    for result.warnings print("sitegen: warning: %\n", it, to_standard_error = true);

    if result.error.count > 0 {
        print("sitegen: %\n", result.error, to_standard_error = true);
        exit(1);
    }

    if cli.verbose {
        for result.files print("  %\n", it);
    }

    print("sitegen: wrote % pages and % assets to %\n", result.pages, result.assets, cli.out);
}
