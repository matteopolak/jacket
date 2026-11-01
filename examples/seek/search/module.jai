// The search behind seek: finds the files to look in, scans each one on a pool of threads, and
// returns the output as one string, in path order, whatever the thread count.
//
// `run_search` is the whole job. The files here are its steps:
//
// - `options.jai`: what a search is asked to do, and what it reports.
// - `glob.jai`: the `--glob` patterns.
// - `matcher.jai`: a pattern matched against one line (fixed text or a regular expression).
// - `walk.jai`: the sorted list of files to search.
// - `scan.jai`: one file in, its lines of output out.
// - `run.jai`: the thread pool and the final assembly.

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";
#import "Thread";
Regex :: #import "Extensions/Regex";

#load "options.jai";
#load "glob.jai";
#load "matcher.jai";
#load "walk.jai";
#load "scan.jai";
#load "run.jai";
