// The site builder behind sitegen: reads a folder of Markdown pages and assets, renders each page
// into a template, and writes the result to an output folder.
//
// `build_site` is the whole pipeline. It reads everything before it writes anything, so a bad
// template leaves the output folder alone. The files here are its steps:
//
// - `paths.jai`: relative-path arithmetic shared by the rest.
// - `assets.jai`: content-hashed names, MIME types and `data:` URIs.
// - `template.jai`: the page template's `{{ placeholders }}`.
// - `pages.jai`: front matter, titles and output names of pages.
// - `build.jai`: the pipeline, and the rewriting of links inside pages.

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Hash_Table";
#import "Sort";
#import "Xar";
#import "Flat_Pool";
#import "md5";
#import "Base64";
#import,dir "../markdown";

#load "paths.jai";
#load "assets.jai";
#load "template.jai";
#load "pages.jai";
#load "build.jai";
