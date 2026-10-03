Path :: Paths.Path;
parse_path :: Paths.parse_path;
reduce_path :: Paths.reduce_path;
path_contains :: Paths.path_contains;
path_to_string :: Paths.path_to_string;
copy_to_permanent :: Paths.copy_to_permanent;
free_permanent :: Paths.free_permanent;
File_Visit_Info :: Visit.File_Visit_Info;

#if OS_IS_UNIX {
    visit_files :: Files.posix_visit_files;
    is_directory :: Files.posix_is_directory;
    add_modtime_and_size_if_missing :: Files.posix_add_modtime_and_size_if_missing;
    file_exists :: Files.file_exists;
    file_modtime :: Files.file_modtime;
    file_modtime_and_size :: Files.file_modtime_and_size;
    copy_file :: Files.copy_file;
    copy_directory :: Files.copy_directory;
    file_list :: Files.file_list;
    how_much_of_path_exists_on_the_local_filesystem :: Files.how_much_of_path_exists_on_the_local_filesystem;
    atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: Files.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist;
}

#if OS == .WINDOWS {
    visit_files :: Files.visit_files;
    file_list :: Files.file_list;
    copy_directory :: Files.copy_directory;
    how_much_of_path_exists_on_the_local_filesystem :: Files.how_much_of_path_exists_on_the_local_filesystem;
    file_exists :: Files.file_exists;
    is_directory :: Files.is_directory;
    file_modtime :: Files.file_modtime;
    file_modtime_and_size :: Files.file_modtime_and_size;
    add_modtime_and_size_if_missing :: Files.add_modtime_and_size_if_missing;
    copy_file :: Files.copy_file;
    atomic_file_replace_or_just_rename_if_destination_file_does_not_exist :: Files.atomic_file_replace_or_just_rename_if_destination_file_does_not_exist;
}

#scope_file
Paths :: #import "internal/files/paths";
Visit :: #import "internal/files/visit-info";
#if OS_IS_UNIX Files :: #import "internal/files/filesystem";
#if OS == .WINDOWS Files :: #import "internal/files/filesystem-windows";
