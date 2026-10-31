// Reads the command line into a struct: declare the options as fields and call `parse_arguments`.
//
// ```jai
// Options :: struct { quiet: bool; level := 2; }
//
// ok, options := parse_arguments(Options, args = string.["-level", "5"]);
// assert(ok && options.level == 5 && !options.quiet);
// ```

Is_Set :: Impl.Is_Set;
Argument_Flags :: Impl.Argument_Flags;
Default_Argument_Flags :: Impl.Default_Argument_Flags;
parse_arguments :: Impl.parse_arguments;
show_help :: Impl.show_help;

#scope_file
Impl :: #import "internal/command-line";
