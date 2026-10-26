// A project's own Objective_C that declares only what the project uses.
#assert OS == .MACOS, "this fixture is a macOS project";

Project_Class :: struct {
    name: string;
}

describe :: (class: Project_Class) -> string {
    return class.name;
}
