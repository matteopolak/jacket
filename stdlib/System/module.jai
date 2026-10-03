#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;
Processor_Query_Type :: Impl.Processor_Query_Type;
read_cpu_counter :: Impl.read_cpu_counter;

#if CPU == .X64 M64 :: #import "Machine_X64";

get_username :: Impl.get_username;
get_home_directory :: Impl.get_home_directory;
get_machinename :: Impl.get_machinename;
get_path_of_running_executable :: Impl.get_path_of_running_executable;
get_number_of_processors :: Impl.get_number_of_processors;

#if OS == .WINDOWS {
    Windows :: #import "Windows";

    get_error_value :: Windows.GetLastError;
    read_cpu_counter :: Impl.read_cpu_counter;
} else get_error_value :: P.errno;

get_error_string :: Impl.get_error_string;
get_error_value_and_string :: Impl.get_error_value_and_string;
#if OS == .LINUX || OS == .ANDROID get_passwd_for_user :: Impl.get_passwd_for_user;

#scope_file
#if OS == .WINDOWS Impl :: #import "internal/system/windows";

else {
    P :: #import "POSIX";
    Impl :: #import "internal/system/unix";
}
