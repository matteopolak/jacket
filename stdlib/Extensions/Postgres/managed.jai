// Managed PostgreSQL: downloads pinned server binaries once, keeps a throwaway cluster, starts it
// on a Unix socket (TCP on localhost for Windows) and hands back a `Config`. See
// docs/postgres/managed.md.
//
//     settings: Pg.Managed_Settings;
//     server, error := Pg.start_managed(settings);
//     if error { print("%\n", error); return; }
//     defer Pg.stop_managed(*server);
//     db, pg_error := Pg.connect(server.config); // inside Io.io_run
//
// Nothing here talks to the server, so none of it needs a fiber. The calls block while they
// download, unpack and wait for the server.

// The default release. `Managed_Settings.version` may name any other release the checksum table
// below pins: the latest of each supported major version (15, 16, 17 and 18).
MANAGED_VERSION :: "17.11.1";

Managed_Settings :: struct {
    // Which release to use; it needs an entry in `MANAGED_ARCHIVES`.
    version := MANAGED_VERSION;
    // Where the downloaded binaries live. Empty: `managed_cache_dir()`.
    cache_dir: string;
    // The cluster's folder. Empty: `<cache_dir>/cluster-<major version>`.
    data_dir: string;
    // Where the Unix socket goes (ignored on Windows). Empty: `<data_dir>/sock`, or a short
    // folder under /tmp when that path is too long for a socket.
    socket_dir: string;
    // 0: 5432 for a Unix socket (it only names the socket file), a free-looking port on Windows.
    port: u16;
    // Seconds `pg_ctl` waits for the server to start or stop.
    timeout_seconds := 60;
    // Lines appended to the managed settings file, e.g. "max_connections = 50".
    extra_conf: string;
}

// The settings the compile-time describe service uses: the defaults, with `JACKET_PG_MANAGED_VERSION`
// (a pinned release such as "16.15.1") and `JACKET_PG_MANAGED_CACHE` (the cache folder) applied.
// Managed mode as the project's jacket.toml sets it, under `[postgres]`: `managed = true` turns it
// on (-1 when the file or key is missing) and `managed_version = "16.11.1"` picks a release.
// JACKET_PG_MANAGED and JACKET_PG_MANAGED_VERSION win over both.
project_managed_settings :: (root: string) -> managed: s64, version: string {
    path := join(root, "/jacket.toml",, temp);
    if !File_Utilities.file_exists(path) return -1, "";

    text, ok := read_entire_file(path,, temp);
    if !ok return -1, "";

    managed := -1;
    version := "";
    in_section := false;

    for line: split(text, "\n",, temp) {
        trimmed := trim(line);
        hash := find_index_from_left(trimmed, #char "#");

        if hash >= 0 && !contains(slice(trimmed, 0, hash), "\"") trimmed = trim(slice(trimmed, 0, hash));
        if !trimmed continue;

        if trimmed[0] == #char "[" {
            in_section = trimmed == "[postgres]";
            continue;
        }

        if !in_section continue;

        equals := find_index_from_left(trimmed, #char "=");
        if equals < 0 continue;

        key := trim(slice(trimmed, 0, equals));
        value := trim(advance(trimmed, equals + 1));

        if key == "managed" {
            if value == "true" managed = 1;
            else if value == "false" managed = 0;
        } else if key == "managed_version" && value.count >= 2 && value[0] == #char "\"" && value[value.count - 1] == #char "\"" {
            version = copy_string(slice(value, 1, value.count - 2),, temp);
        }
    }

    return managed, version;
}

// The settings for the project at `root`: the environment, then its jacket.toml.
managed_settings_for :: (root: string) -> Managed_Settings {
    settings := managed_settings_from_environment();

    if !environment_variable("JACKET_PG_MANAGED_VERSION") {
        _, version := project_managed_settings(root);
        if version settings.version = version;
    }

    return settings;
}

managed_settings_from_environment :: () -> Managed_Settings {
    settings: Managed_Settings;

    version := environment_variable("JACKET_PG_MANAGED_VERSION");

    if version settings.version = version;

    settings.cache_dir = environment_variable("JACKET_PG_MANAGED_CACHE");

    return settings;
}

Managed_Server :: struct {
    config: Config; // connect with this; user `postgres`, database `postgres`, no password
    url: string; // the same as a `postgresql://` URL
    bin_dir: string;
    data_dir: string;
    socket_dir: string; // empty on Windows
    port: u16;
    started_here: bool; // false: a server was already running on `data_dir` and was reused
}

Managed_Os :: enum u8 {
    MACOS;
    LINUX;
    WINDOWS;
}

Managed_Archive :: struct {
    version: string;
    triple: string;
    sha256: string; // of the `.tar.gz`, as committed here
}

// SHA-256 of every release archive the managed mode accepts, taken from the release's `.sha256`
// files and cross-checked against the digests GitHub reports for the assets. A download is
// compared with these, never with a checksum fetched next to it.
MANAGED_ARCHIVES :: Managed_Archive.[
    .{ "15.19.1", "aarch64-apple-darwin", "02214be6486ec1be214eae66664f384be218b0893d29a9c151408afe5646a8ab" },
    .{ "15.19.1", "x86_64-apple-darwin", "4e8149811d40eb09fdbcef532d1fea177b2b267132a188450af0139969bc96f0" },
    .{ "15.19.1", "aarch64-unknown-linux-gnu", "d38b8cfd65e02fd6f08b2368b2d607c70d0d0527b1ca12baeee780cc2a947b07" },
    .{ "15.19.1", "x86_64-unknown-linux-gnu", "f34ebfdbff9029f47011709fe8ef68d4a65ea5caadfd065e68a1161e2a021565" },
    .{ "15.19.1", "aarch64-unknown-linux-musl", "6948dd84286157fbee74c6b4c6980a52a6716ab0461403d4735d96f094fb86ec" },
    .{ "15.19.1", "x86_64-unknown-linux-musl", "b0844ce1e1a6ad00222d586a5c3e8ba22fce8c32d6319f5948f7aaa67a1a08ff" },
    .{ "15.19.1", "x86_64-pc-windows-msvc", "7f2c068b3e7c55b3df8eed6c3baa5891edbcb6121cf73ba3124cf1d6478f8232" },
    .{ "16.15.1", "aarch64-apple-darwin", "0a70b8a3bbb991b5671dc7678ba9a8cf1a389f2fc595ac4834fb1e29e757b89e" },
    .{ "16.15.1", "x86_64-apple-darwin", "1b10aa67f9c111dd288f96d800c995dc424ab1e474fb8dd60ea6d05edb055f4c" },
    .{ "16.15.1", "aarch64-unknown-linux-gnu", "0a6fdf1639ac702fc8a97cd9e4ffeeea6281cf5c9f614beec3e4687525556bd8" },
    .{ "16.15.1", "x86_64-unknown-linux-gnu", "0c42e7c05f0cb596b82ac07990c96fc261ec2e8106055d557c76442d712dbf74" },
    .{ "16.15.1", "aarch64-unknown-linux-musl", "1d417dd2985bbdf24bde03be6868373c86b2792595ec463e060c3296c88608c4" },
    .{ "16.15.1", "x86_64-unknown-linux-musl", "48d00d5ba39e2ef757db9dbaad53f3e87f05636d4ee345fa68bdf618e28778e4" },
    .{ "16.15.1", "aarch64-pc-windows-msvc", "51862a60500d3826394b8a0df5ec5d14840d180cdf97ea239295031c125d1104" },
    .{ "16.15.1", "x86_64-pc-windows-msvc", "9d617660ac12b599b57481b4b7a3b9658d35fe3ebc2fd72c95ea595d775c95ad" },
    .{ "17.11.1", "aarch64-apple-darwin", "f4b2ff0587d80b6f22bf09ec2d133a3c2949b6423318d3fed0f4ddddddf2f004" },
    .{ "17.11.1", "x86_64-apple-darwin", "edb44b25cf8bffec60fa300fbe100fa4238bf2cab4f4bd98d4b7255c61780df3" },
    .{ "17.11.1", "aarch64-unknown-linux-gnu", "edaab3230a014680ebedd836824962628f66ceabd94c36ba3f953bf9e04dc166" },
    .{ "17.11.1", "x86_64-unknown-linux-gnu", "aa8e2e97574bd789119ac01f1dddcd7a038c84e5c308d7b753018618258f0fbd" },
    .{ "17.11.1", "aarch64-unknown-linux-musl", "1791b7126eafafa519996fcca88f5c4066e659879d73fe7bcceacb994de02776" },
    .{ "17.11.1", "x86_64-unknown-linux-musl", "f554c6a5d232da640ba256ea1a26b8f7dd38621295998268aad3be79e7a60fd0" },
    .{ "17.11.1", "aarch64-pc-windows-msvc", "966a689aa98a3488649b6065d15c2fc635b720c4006ca54542814b5c7499a617" },
    .{ "17.11.1", "x86_64-pc-windows-msvc", "9855f1b2a9acf127e148fbdd929ef118bfe2fc095c9358a7da74bdc02cedfc27" },
    .{ "18.6.1", "aarch64-apple-darwin", "89e91cc80650a43d22c018b95ca356a0c9e15838a1182fed2880bfb5d7d9c9a6" },
    .{ "18.6.1", "x86_64-apple-darwin", "4229812543e19c91864ebdf5d8f78883c5816d51d4cc5d065298260c1f64d37e" },
    .{ "18.6.1", "aarch64-unknown-linux-gnu", "1ab47fa2f06ff70454309ad02e5d89c46488300d43977231481fd6b160b9d556" },
    .{ "18.6.1", "x86_64-unknown-linux-gnu", "e84b02dd8d0ce5879fb697330a789231c8b8f8d958dae3ad4d90fecc710b1315" },
    .{ "18.6.1", "aarch64-unknown-linux-musl", "d33d7d3249d7cf7ae72f35aaa20e46d30bdc1c17f4d102b2b3a39d618848ea8b" },
    .{ "18.6.1", "x86_64-unknown-linux-musl", "80287b4596d427cf8456eb946c44b1607b3d2f0a0ad48ffe26dde0068bee5ff3" },
    .{ "18.6.1", "aarch64-pc-windows-msvc", "29669a4e020f252d8ff7a59e5e5c4632901fcff0e5ee7793f8b011a0e2157f68" },
    .{ "18.6.1", "x86_64-pc-windows-msvc", "0c0c845626ed77c605d716051c9e8df3368d07fbbd9f61ea39a4c5c0536fde7e" },
];

// The port a Unix socket server uses by default (it only names the socket file), and the range
// Windows picks from (the dynamic ports).
MANAGED_SOCKET_PORT :: 5432;
MANAGED_PORT_BASE :: 49152;
MANAGED_PORT_SPAN :: 16000;

MANAGED_RELEASE_URL :: "https://github.com/theseus-rs/postgresql-binaries/releases/download";

// The release's name for a platform, or "" when it publishes nothing for it. `musl` only matters
// on Linux.
managed_triple_for :: (os: Managed_Os, arm64: bool, musl: bool) -> string {
    if os == {
        case .MACOS;
            if arm64 return "aarch64-apple-darwin";

            return "x86_64-apple-darwin";
        case .WINDOWS;
            if arm64 return "aarch64-pc-windows-msvc";

            return "x86_64-pc-windows-msvc";
        case .LINUX;
            if musl {
                if arm64 return "aarch64-unknown-linux-musl";
                return "x86_64-unknown-linux-musl";
            }

            if arm64 return "aarch64-unknown-linux-gnu";

            return "x86_64-unknown-linux-gnu";
    }

    return "";
}

// The triple for the machine this runs on. "" when there is none (an unsupported OS or CPU).
managed_triple :: () -> string {
    #if OS == .MACOS {
        return managed_triple_for(.MACOS, CPU == .ARM64, false);
    } else #if OS == .WINDOWS {
        return managed_triple_for(.WINDOWS, CPU == .ARM64, false);
    } else #if OS == .LINUX {
        musl := file_exists_managed("/lib/ld-musl-x86_64.so.1") || file_exists_managed("/lib/ld-musl-aarch64.so.1");
        return managed_triple_for(.LINUX, CPU == .ARM64, musl);
    } else {
        return "";
    }
}

// The pinned SHA-256 for a release and triple, or "".
managed_checksum :: (version: string, triple: string) -> string {
    for MANAGED_ARCHIVES if it.version == version && it.triple == triple return it.sha256;
    return "";
}

// The per-user folder for downloaded binaries, from the platform's conventions. The arguments are
// the environment variables it reads, so the choice can be checked without them.
managed_cache_dir_for :: (os: Managed_Os, home: string, xdg_cache_home: string, local_app_data: string) -> string {
    if os == {
        case .WINDOWS;
            if local_app_data return sprint("%\\jacket\\postgres", local_app_data);
            if home return sprint("%\\AppData\\Local\\jacket\\postgres", home);
        case .MACOS;
            if home return sprint("%/Library/Caches/jacket/postgres", home);
        case .LINUX;
            if xdg_cache_home return sprint("%/jacket/postgres", xdg_cache_home);
            if home return sprint("%/.cache/jacket/postgres", home);
    }

    return "";
}

managed_cache_dir :: () -> string {
    #if OS == .WINDOWS {
        return managed_cache_dir_for(.WINDOWS, environment_variable("USERPROFILE"), "", environment_variable("LOCALAPPDATA"));
    } else #if OS == .MACOS {
        return managed_cache_dir_for(.MACOS, environment_variable("HOME"), "", "");
    } else {
        return managed_cache_dir_for(.LINUX, environment_variable("HOME"), environment_variable("XDG_CACHE_HOME"), "");
    }
}

// Downloads the release for this machine into the cache unless it is there, and returns its
// `bin` folder. Safe to run in several processes at once: each unpacks into its own folder and
// the first to rename it into place wins.
ensure_binaries :: (settings: Managed_Settings) -> bin_dir: string, error: string {
    triple := managed_triple();
    if !triple return "", sprint("no PostgreSQL binaries are published for this platform");

    expected := managed_checksum(settings.version, triple);
    if !expected return "", sprint("no pinned checksum for PostgreSQL % on %; add it to MANAGED_ARCHIVES", settings.version, triple);

    cache := settings.cache_dir;
    if !cache cache = managed_cache_dir();
    if !cache return "", sprint("cannot find a cache folder: set Managed_Settings.cache_dir");

    root := sprint("%/%-%", cache, settings.version, triple);
    bin := sprint("%/bin", root);

    if managed_binaries_ready(bin) return bin, "";

    if !make_directory_if_it_does_not_exist(cache, true) return "", sprint("cannot create %", cache);

    work := sprint("%/.tmp-%-%", cache, managed_process_id(), to_microseconds(current_time_monotonic()));
    if !make_directory_if_it_does_not_exist(work) return "", sprint("cannot create %", work);

    defer managed_remove_tree(work);

    archive := sprint("%/archive.tar.gz", work);
    url := sprint("%/%/postgresql-%-%.tar.gz", MANAGED_RELEASE_URL, settings.version, settings.version, triple);
    ok, text := managed_run("curl", "-fsSL", "--retry", "3", "-o", archive, url);

    if !ok return "", sprint("downloading % failed (managed mode needs `curl` on the PATH): %", url, text);

    data, read_ok := read_entire_file(archive);
    if !read_ok return "", sprint("cannot read the downloaded %", archive);

    actual := managed_sha256_hex(data);

    free(data.data);
    if actual != expected return "", sprint("the download of % does not match the pinned checksum (got %, want %)", url, actual, expected);

    unpacked := sprint("%/unpacked", work);
    if !make_directory_if_it_does_not_exist(unpacked) return "", sprint("cannot create %", unpacked);

    // Windows ships bsdtar in System32; Git's GNU tar, often earlier on PATH, reads `C:` as a host.
    tar := "tar";

    #if OS == .WINDOWS {
        system_root := environment_variable("SystemRoot");
        if system_root tar = tprint("%\\System32\\tar.exe", system_root);
    }

    ok, text = managed_run(tar, "-xzf", archive, "-C", unpacked, "--strip-components=1");
    if !ok return "", sprint("unpacking the archive failed (managed mode needs `tar` on the PATH): %", text);
    if !managed_binaries_ready(sprint("%/bin", unpacked)) return "", sprint("the archive has no bin/pg_ctl: %", url);

    if !file_move(unpacked, root) {
        // Another process finished first.
        if managed_binaries_ready(bin) return bin, "";
        return "", sprint("cannot move the unpacked binaries to %", root);
    }

    return bin, "";
}

// Makes sure the binaries are there, creates the cluster if needed, and starts a server on it,
// or reuses one already running on the same data folder (found through `postmaster.pid`). The
// returned `config` connects as `postgres`.
//
// The cluster uses `fsync = off`, `synchronous_commit = off` and `full_page_writes = off`: it is
// for throwaway data and loses it on a crash. Two processes starting the same data folder take
// turns on a lock folder next to it (`<data_dir>.lock`): the second waits for the first, then
// finds the server running and reuses it. A lock whose owner has died is taken over.
start_managed :: (settings: Managed_Settings) -> Managed_Server, error: string {
    data := managed_data_dir(settings);
    if !data return .{}, "no cache folder: set HOME (or Managed_Settings.cache_dir or data_dir)";

    lock, lock_error := acquire_managed_lock(data);
    if lock_error return .{}, lock_error;

    defer release_managed_lock(lock);

    return start_managed_locked(settings);
}

// The cluster's folder: the setting, else `<cache>/cluster-<major version>`.
managed_data_dir :: (settings: Managed_Settings) -> string {
    if settings.data_dir return settings.data_dir;

    cache := settings.cache_dir;
    if !cache cache = managed_cache_dir();
    if !cache return "";

    return sprint("%/cluster-%", cache, managed_major(settings.version));
}

// Everything `start_managed` does before starting the server: the download, and the cluster.
// These are the slow parts (seconds to tens of seconds, the first time only), so code with a
// time limit on its requests calls this first, somewhere without one, and `start_managed` is
// then quick. Safe to call again and from several processes; it returns an error text.
prepare_managed :: (settings: Managed_Settings) -> error: string {
    data := managed_data_dir(settings);
    if !data return "no cache folder: set HOME (or Managed_Settings.cache_dir or data_dir)";

    lock, lock_error := acquire_managed_lock(data);
    if lock_error return lock_error;

    defer release_managed_lock(lock);

    _, error := prepare_managed_locked(settings);

    return error;
}

prepare_managed_locked :: (settings: Managed_Settings) -> bin: string, error: string {
    bin, error := ensure_binaries(settings);
    if error return "", error;

    data := managed_data_dir(settings);
    if managed_has_quote(data) return "", sprint("the data folder must not contain a single quote: %", data);

    marker := sprint("%/.jacket-managed", data);
    if !file_exists_managed(marker) {
        if file_exists_managed(sprint("%/PG_VERSION", data)) {
            return "", sprint("% holds a PostgreSQL cluster that managed mode did not create (or an initdb that did not finish); remove it or pick another data_dir", data);
        }

        if !make_directory_if_it_does_not_exist(data, true) return "", sprint("cannot create %", data);

        ok, text := managed_run(managed_tool(bin, "initdb"), "-D", data, "-U", "postgres", "--auth=trust", "-E", "UTF8", "--locale=C", "--no-sync");
        if !ok return "", sprint("initdb failed: %", text);

        conf := sprint("%/postgresql.conf", data);
        existing, read_ok := read_entire_file(conf);

        if !read_ok return "", sprint("cannot read %", conf);

        if !write_entire_file(conf, sprint("%\n\ninclude_if_exists = 'jacket.conf'\n", existing)) return "", sprint("cannot write %", conf);
        if !write_entire_file(marker, "managed by Jacket: initdb finished\n") return "", sprint("cannot write %", marker);
    }

    return bin, "";
}

start_managed_locked :: (settings: Managed_Settings) -> Managed_Server, error: string {
    bin, error := prepare_managed_locked(settings);
    if error return .{}, error;

    server: Managed_Server;
    server.bin_dir = bin;
    server.data_dir = managed_data_dir(settings);

    data := server.data_dir;

    running, _ := managed_run(managed_tool(bin, "pg_ctl"), "status", "-D", data);

    if running {
        found := managed_describe_running(*server);
        if !found return .{}, sprint("the server on % is running, but its postmaster.pid is unreadable", data);

        return server, "";
    }

    port := settings.port;
    socket_dir := "";

    #if OS == .WINDOWS {
        if port == 0 port = cast(u16) (MANAGED_PORT_BASE + (managed_process_id() * 31 + to_microseconds(current_time_monotonic())) % MANAGED_PORT_SPAN);
    } else {
        if port == 0 port = MANAGED_SOCKET_PORT;

        socket_dir = settings.socket_dir;
        if !socket_dir socket_dir = managed_default_socket_dir(data);
        if managed_has_quote(socket_dir) return .{}, sprint("the socket folder must not contain a single quote: %", socket_dir);
        if socket_dir.count + 16 > 100 return .{}, sprint("the socket folder % is too long for a Unix socket; set Managed_Settings.socket_dir to a shorter path", socket_dir);
        if !make_directory_if_it_does_not_exist(socket_dir, true) return .{}, sprint("cannot create %", socket_dir);
    }

    if !write_entire_file(sprint("%/jacket.conf", data), managed_conf(port, socket_dir, settings.extra_conf)) {
        return .{}, sprint("cannot write %/jacket.conf", data);
    }

    start_ok, text := managed_run_start(managed_tool(bin, "pg_ctl"), "start", "-D", data, "-l", sprint("%/server.log", data), "-w", "-t", sprint("%", settings.timeout_seconds));
    if !start_ok return .{}, sprint("pg_ctl start failed: % (see %/server.log)", text, data);

    server.started_here = true;
    server.port = port;
    server.socket_dir = socket_dir;
    managed_fill_config(*server);

    return server, "";
}

// Stops a server `start_managed` started. A server it found already running is left alone unless
// `force` is set. Returns an error text, empty when it stopped (or was not asked to).
stop_managed :: (server: *Managed_Server, force := false) -> string {
    if !server.bin_dir return "";
    if !server.started_here && !force return "";

    ok, text := managed_run(managed_tool(server.bin_dir, "pg_ctl"), "stop", "-D", server.data_dir, "-m", "fast", "-w", "-t", "30");
    if !ok return sprint("pg_ctl stop failed: %", text);

    server.started_here = false;

    return "";
}

#scope_file

Proc :: #import "Process";
File_Utilities :: #import "File_Utilities";

#if OS == .WINDOWS {
    MANAGED_EXE :: ".exe";

    kernel32_managed :: #system_library "kernel32"; @native(external)
    GetCurrentProcessId :: () -> u32 #foreign kernel32_managed;

    managed_process_id :: () -> s64 {
        return GetCurrentProcessId();
    }

    CreateDirectoryA :: (path: *u8, attributes: *void) -> s32 #foreign kernel32_managed;

    // True when this call created the folder: false when it exists already (or cannot be made).
    managed_create_exclusive :: (path: string) -> bool {
        return CreateDirectoryA(temp_c_string(path), null) != 0;
    }

    // Windows has no cheap test for a dead owner: a lock there is only given up on after the wait.
    managed_process_alive :: (pid: s64) -> bool {
        return true;
    }
} else {
    MANAGED_EXE :: "";

    libc_managed :: #system_library "libc"; @native(external)
    getpid_managed :: () -> s32 #foreign libc_managed "getpid";

    managed_process_id :: () -> s64 {
        return getpid_managed();
    }

    mkdir_managed :: (path: *u8, mode: u32) -> s32 #foreign libc_managed "mkdir";
    kill_managed :: (pid: s32, signal: s32) -> s32 #foreign libc_managed "kill";

    // True when this call created the folder (mode 0755): `mkdir` fails when it exists, which makes it a lock.
    managed_create_exclusive :: (path: string) -> bool {
        return mkdir_managed(temp_c_string(path), 493) == 0;
    }

    managed_process_alive :: (pid: s64) -> bool {
        return kill_managed(cast(s32) pid, 0) == 0;
    }
}

// How long to wait for another process that holds the lock: it may be downloading the archive.
MANAGED_LOCK_WAIT_SECONDS :: 600;
// How long a lock folder may stay without a `pid` file before it is taken to be abandoned.
MANAGED_LOCK_PID_GRACE_MS :: 5_000;

// Takes the lock folder `<data>.lock`, waiting for its holder to finish. Returns the folder.
acquire_managed_lock :: (data: string) -> lock: string, error: string {
    lock := sprint("%.lock", data);
    parent_end := find_index_from_right(lock, #char "/");

    if parent_end > 0 {
        parent := slice(lock, 0, parent_end);
        if !make_directory_if_it_does_not_exist(parent, true) return "", sprint("cannot create %", parent);
    }

    waited := 0;
    pid_missing := 0;

    while waited <= MANAGED_LOCK_WAIT_SECONDS * 1000 {
        if managed_create_exclusive(lock) {
            if !write_entire_file(sprint("%/pid", lock), sprint("%\n", managed_process_id())) {
                managed_remove_tree(lock);
                return "", sprint("cannot write %/pid", lock);
            }

            return lock, "";
        }

        // Is the holder gone? Then the lock is abandoned; remove it and try again at once.
        text, ok := read_entire_file(sprint("%/pid", lock));
        stale := false;

        if ok {
            pid, valid := string_to_int(trim(text));
            stale = valid && !managed_process_alive(pid);
        } else {
            // The holder may be between making the folder and writing its pid.
            pid_missing += 100;
            stale = pid_missing >= MANAGED_LOCK_PID_GRACE_MS;
        }

        if stale {
            managed_remove_tree(lock);
            pid_missing = 0;
            continue;
        }

        sleep_milliseconds(100);
        waited += 100;
    }

    return "", sprint("another process has been starting the managed server for %s; if none is, remove %", MANAGED_LOCK_WAIT_SECONDS, lock);
}

release_managed_lock :: (lock: string) {
    managed_remove_tree(lock);
}

file_exists_managed :: (path: string) -> bool {
    return File_Utilities.file_exists(path);
}

managed_tool :: (bin: string, name: string) -> string {
    return sprint("%/%%", bin, name, MANAGED_EXE);
}

managed_binaries_ready :: (bin: string) -> bool {
    return file_exists_managed(managed_tool(bin, "pg_ctl")) && file_exists_managed(managed_tool(bin, "postgres"));
}

managed_major :: (version: string) -> string {
    dot := find_index_from_left(version, #char ".");
    if dot < 0 return version;

    return slice(version, 0, dot);
}

managed_has_quote :: (path: string) -> bool {
    for path if it == #char "'" || it == 10 return true;
    return false;
}

// Runs a program, capturing its output. The text is what it printed on failure.
managed_run :: (args: ..string) -> bool, string {
    result, output, errors, _ := Proc.run_command(..args, capture_and_return_output = true);
    if result.type == .FAILED_TO_LAUNCH return false, sprint("cannot run %", args[0]);

    if result.type != .EXITED || result.exit_code != 0 {
        return false, sprint("% exited with %: % %", args[0], result.exit_code, errors, output);
    }

    return true, output;
}

// Runs `pg_ctl start`. On Windows the server it launches inherits pg_ctl's output handles and
// keeps them open while it runs, so captured output would never end: the output is not captured
// there, and the server log says what went wrong. The launch also leaves out the job object that
// ends a child with its parent, which would take the server down as soon as pg_ctl returned.
managed_run_start :: (args: ..string) -> bool, string {
    #if OS != .WINDOWS return managed_run(..args);

    process: Proc.Process;
    if !Proc.create_process(*process, ..args, kill_process_if_parent_exits = false) return false, sprint("cannot run %", args[0]);

    defer Proc.deinit(*process);

    waited, result := Proc.get_process_result(*process);
    if !waited || result.type != .EXITED || result.exit_code != 0 return false, sprint("% exited with %", args[0], result.exit_code);

    return true, "";
}

managed_remove_tree :: (path: string) {
    #if OS == .WINDOWS {
        native := copy_string(path,, temp);
        for *native if it.* == #char "/" it.* = #char "\\";
        managed_run("cmd", "/c", "rmdir", "/s", "/q", native);
    } else {
        managed_run("rm", "-rf", path);
    }
}

managed_sha256_hex :: (data: string) -> string {
    digest := Sha.sha256(data);
    builder: String_Builder;
    builder.allocator = temp;
    for digest print_to_builder(*builder, "%", formatInt(it, base = 16, minimum_digits = 2));

    return builder_to_string(*builder,, temp);
}

// A short folder for the socket when the data folder's own path would not fit in a socket address.
managed_default_socket_dir :: (data: string) -> string {
    inside := sprint("%/sock", data);
    if inside.count + 16 <= 100 return inside;

    return sprint("/tmp/jacket-pg-%", slice(managed_sha256_hex(data), 0, 16));
}

managed_conf :: (port: u16, socket_dir: string, extra: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "# Written by Jacket's managed mode on every start; edit Managed_Settings instead.\n");
    print_to_builder(*builder, "port = %\n", port);

    if socket_dir {
        print_to_builder(*builder, "listen_addresses = ''\nunix_socket_directories = '%'\n", socket_dir);
    } else {
        print_to_builder(*builder, "listen_addresses = '127.0.0.1'\n");
    }

    print_to_builder(*builder, "fsync = off\nsynchronous_commit = off\nfull_page_writes = off\n");
    print_to_builder(*builder, "max_connections = 40\nshared_buffers = 32MB\nlog_min_messages = warning\n");
    if extra print_to_builder(*builder, "%\n", extra);

    return builder_to_string(*builder,, temp);
}

managed_fill_config :: (server: *Managed_Server) {
    config: Config;
    config.user = "postgres";
    config.database = "postgres";
    config.port = server.port;
    config.ssl_mode = .DISABLE;
    config.application_name = "jacket-managed";

    if server.socket_dir {
        config.host = server.socket_dir;
        server.url = sprint("postgresql://postgres@/postgres?host=%&port=%&sslmode=disable", managed_percent_encode(server.socket_dir), server.port);
    } else {
        config.host = "127.0.0.1";
        server.url = sprint("postgresql://postgres@127.0.0.1:%/postgres?sslmode=disable", server.port);
    }

    server.config = config;
}

managed_percent_encode :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for text {
        keep := (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || (it >= #char "0" && it <= #char "9") || it == #char "-" || it == #char "_" || it == #char "." || it == #char "~";
        if keep append(*builder, it);
        else {
            append(*builder, "%");
            print_to_builder(*builder, "%", formatInt(it, base = 16, minimum_digits = 2));
        }
    }

    return builder_to_string(*builder,, temp);
}

// Fills the port and socket folder of an already running server from its `postmaster.pid`
// (lines: pid, data folder, start time, port, socket folder, listen address).
PID_LINE_PORT :: 3;
PID_LINE_SOCKET :: 4;

managed_describe_running :: (server: *Managed_Server) -> bool {
    text, ok := read_entire_file(sprint("%/postmaster.pid", server.data_dir));
    if !ok return false;

    defer free(text.data);
    lines := split(text, "\n");
    if lines.count <= PID_LINE_SOCKET return false;

    port, valid := string_to_int(trim(lines[PID_LINE_PORT]));
    if !valid return false;

    server.port = cast(u16) port;

    #if OS != .WINDOWS {
        server.socket_dir = copy_string(trim(lines[PID_LINE_SOCKET]));
    }

    managed_fill_config(server);

    return true;
}
