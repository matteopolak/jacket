// Module for tests/stdlib/rules-module-parameters.jai: per-instance parameters, a program
// parameter read from a loaded file, a typed parameter, untyped ones and a trailing block.
#module_parameters (LOUD := false, COUNT: s64 = 1, STYLE: Style = .PLAIN, TAG := 0, WIDTH := DEFAULT_WIDTH)(TRACE := false) {
    Style :: enum {
        PLAIN;
        FANCY;
    }
    DEFAULT_WIDTH :: 40;
}

#load "impl.jai";

hits: int;

// Each instance has its own globals, so this tells instances apart.
bump :: () -> int {
    hits += 1;
    return hits;
}

greeting :: (name: string) -> string {
    builder: String_Builder;
    for 1..COUNT {
        if LOUD append(*builder, "HELLO ");
        else append(*builder, "hello ");
        append(*builder, name);
        append(*builder, ";");
    }
    return builder_to_string(*builder);
}

count_type :: () -> Type {
    return type_of(COUNT);
}
tag_type :: () -> Type {
    return type_of(TAG);
}
style_is_fancy :: () -> bool {
    return STYLE == .FANCY;
}

#scope_file
#import "Basic";
