add_relative :: (filename: string, workspace: Workspace) {
    C.add_build_file(filename, workspace);
}

add_at_location :: (filename: string, workspace: Workspace, location: Source_Code_Location) {
    C.add_build_file(filename, workspace, loc = location);
}

#scope_file
C :: #import "Compiler";
