// Tool-bootstrap implementation of reviewed File_Utilities APIs. Staged only,
// not a claim that the foundation worker's canonical module is integrated.
is_directory :: (path: string) -> (directory: bool, success: bool) {
    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 return false, false;

    return P.S_ISDIR(stat.st_mode), true;
}

visit_files :: (dir_name: string, recursive: bool, user_data: $T,
                proc: (info: *File_Visit_Info, user_data: T), visit_files := true,
                visit_directories := false, visit_symlinks := true,
                follow_directory_symlinks := true) -> bool {
    ancestors: [..] Visit_Identity;

    defer array_free(ancestors);

    return visit_directory(dir_name, recursive, user_data, proc, visit_files,
                           visit_directories, visit_symlinks,
                           follow_directory_symlinks, *ancestors);
}

#scope_file
#import "Basic";
P :: #import "POSIX";
S :: #import "String";
F :: #import "File_Utilities";

File_Visit_Info :: F.File_Visit_Info;

Visit_Identity :: struct {
    device: u64;
    inode: u64;
}

visit_directory :: (path: string, recursive: bool, user_data: $T,
                     proc: (info: *File_Visit_Info, user_data: T), files: bool,
                     directories: bool, symlinks: bool, follow: bool,
                     ancestors: *[..] Visit_Identity) -> bool {
    name := to_c_string(path);

    defer free(name);
    stat: P.stat_t;
    if P.stat(name, *stat) != 0 || !P.S_ISDIR(stat.st_mode) return false;

    identity := Visit_Identity.{ cast(u64) stat.st_dev, cast(u64) stat.st_ino };

    for ancestor: ancestors.* {
        if ancestor.device == identity.device && ancestor.inode == identity.inode return false;
    }

    array_add(ancestors, identity);
    defer ancestors.count -= 1;
    directory := P.opendir(name);
    if directory == null return false;

    success := true;

    while true {
        P.set_errno(cast(P.OS_Error_Code) 0);
        entry := P.readdir(directory);
        if entry == null {
            if P.errno() != 0 success = false;
            break;
        }

        short := to_string(entry.d_name.data);
        if short == "." || short == ".." continue;

        full := S.path_join(path, short);
        cname := to_c_string(full);
        info: File_Visit_Info;
        info.short_name = short;
        info.full_name = full;
        info.descend_into_directory = recursive;

        if P.lstat(cname, *stat) != 0 {
            info.had_error = true;
            success = false;
        } else {
            info.is_symlink = P.S_ISLNK(stat.st_mode);
            if info.is_symlink && P.stat(cname, *stat) != 0 {
                info.had_error = true;
                success = false;
            } else {
                info.is_directory = P.S_ISDIR(stat.st_mode);
                info.size = stat.st_size;
                time, ok := F.file_modtime(full);

                info.modification_time = time;

                if !ok {
                    info.had_error = true;
                    success = false;
                }
            }
        }

        free(cname);
        should_visit := ifx info.is_symlink then symlinks else
                        ifx info.is_directory then directories else files;
        if should_visit proc(*info, user_data);

        if !info.had_error && info.is_directory && info.descend_into_directory &&
           recursive && (!info.is_symlink || follow) {
            if !visit_directory(full, true, user_data, proc, files, directories,
                                symlinks, follow, ancestors) success = false;
        }

        free(full);
    }

    if P.closedir(directory) != 0 success = false;

    return success;
}
