parse_plugin_arguments :: (args: [] string, starting_index := 0) -> (success: bool, plugins: [..] Plugin_To_Create, remaining_args: [] string) {
    if starting_index < 0 || starting_index > args.count return false, .{}, args;

    first := starting_index;
    while first < args.count {
        word := args[first];
        if word == "-" return true, .{}, args;
        if word.count > 0 && word[0] == 43 break;
        first += 1;
    }

    if first == args.count return true, .{}, args;

    ordinary: [..] string;
    created: [..] Plugin_To_Create;

    for index: starting_index..first - 1 array_add(*ordinary, args[index]);

    index := first;
    while index < args.count {
        word := args[index];

        index += 1;

        if word == "-" {
            while index < args.count {
                array_add(*ordinary, args[index]);
                index += 1;
            }

            break;
        }

        if word.count > 0 && word[0] == 43 {
            plugin: Plugin_To_Create;
            plugin.name = .{ count = word.count - 1, data = word.data + 1 };
            array_add(*created, plugin);
        } else array_add(*created[created.count - 1].options, word);
    }

    return true, created, ordinary;
}

#scope_file
using #import "Basic";
