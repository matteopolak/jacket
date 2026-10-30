// A small `.env` reader, enough to find the database URL when the environment has none. One
// assignment per line:
//
//     # a comment
//     export DATABASE_URL="postgres://app@localhost/app_dev"   # and a trailing comment
//     JACKET_DATABASE_URL='postgres://app@localhost/app_test'
//     OTHER=plain value
//
// Double quotes understand \n, \" and \\. Single quotes keep everything. Unquoted values end at
// the first ` #`. There is no variable substitution and no multi-line value. See
// docs/postgres/typed-queries.md.

#scope_export

// The value `key` gets in the text of a `.env` file, and whether it is assigned at all. The last
// assignment wins. The result lives in temporary storage.
//
// - text: the contents of a `.env` file.
// - key: the variable name.
dotenv_value :: (text: string, key: string) -> string, bool {
    value: string;
    found := false;

    for line: split(text, "\n",, temp) {
        name, assigned, ok := dotenv_assignment(line);

        if ok && name == key {
            value = assigned;
            found = true;
        }
    }

    return value, found;
}

// The database URL the project's `.env` file gives: JACKET_DATABASE_URL, else DATABASE_URL. Empty
// when there is no file or neither is assigned.
//
// - root: the project folder that holds `.env`.
dotenv_database_url :: (root: string) -> string {
    path := join(root, "/.env",, temp);
    if !Typed_Files.file_exists(path) return "";

    text, ok := read_entire_file(path, log_errors = false,, temp);
    if !ok return "";

    url, found := dotenv_value(text, "JACKET_DATABASE_URL");
    if found && url return url;

    url, found = dotenv_value(text, "DATABASE_URL");

    return url;
}

#scope_module

dotenv_assignment :: (line: string) -> name: string, value: string, ok: bool {
    rest := trim(line);

    if !rest.count || rest[0] == #char "#" return "", "", false;

    if begins_with(rest, "export ") || begins_with(rest, "export\t") rest = trim_left(advance(rest, 7));

    equals := find_index_from_left(rest, #char "=");
    if equals <= 0 return "", "", false;

    name := trim(slice(rest, 0, equals));

    for name if it == #char " " || it == #char "\t" return "", "", false;

    return name, dotenv_text(trim_left(advance(rest, equals + 1))), true;
}

// What follows the `=`: a quoted string, or text up to a comment.
dotenv_text :: (text: string) -> string {
    if !text.count return "";

    quote := text[0];

    if quote == #char "'" {
        end := find_index_from_left(advance(text, 1), #char "'");
        if end < 0 return slice(text, 1, text.count - 1);

        return slice(text, 1, end);
    }

    if quote == #char "\"" {
        builder: String_Builder;
        builder.allocator = temp;

        index := 1;

        while index < text.count {
            c := text[index];

            if c == #char "\"" break;

            if c == #char "\\" && index + 1 < text.count {
                index += 1;
                next := text[index];

                if next == #char "n" append(*builder, "\n");
                else append(*builder, slice(text, index, 1));
            } else {
                append(*builder, slice(text, index, 1));
            }

            index += 1;
        }

        return builder_to_string(*builder);
    }

    // Unquoted: a `#` after whitespace starts a comment.
    end := text.count;

    for text {
        if it == #char "#" && it_index > 0 && (text[it_index - 1] == #char " " || text[it_index - 1] == #char "\t") {
            end = it_index;
            break;
        }
    }

    return trim(slice(text, 0, end));
}
