# Android/Toolchain

## Module parameters

- `SDK_LEVEL` defaults to `34`. It supplies the default target SDK level in project/build configuration and is also used when forming Android target/link inputs. `ANDROID_SDK_LEVEL` is the exported alias for this value.
- `ENABLE_ARM_LSE` defaults to `true`. When the Android compiler plugin is active, this setting tells the compiler backend to treat ARM Large System Extensions as available. Enabling it requires an ARM target that supports the extension; it is not a runtime feature probe.

These are import-time configuration values. A project's `target_sdk_level` and build options' `sdk_level` can be set independently, so callers must keep them compatible when they intend project metadata and native linking to target the same SDK.

## Public declaration inventory

This namespace exports the declarations listed here, including the type members below. ABI binding semantics are governed by the Android NDK or SDK API represented by each declaration; native handles, status codes, and ownership follow that API.

- `is_adb_installed :: () -> bool, string`
- `android_is_app_installed :: (package_name: string) -> bool`
- `is_app_running :: (package_name: string) -> running: bool, success: bool`
- `stop_app :: (package_name: string) -> bool`
- `android_check_for_adb_and_select_device :: (required_reason := "by metaprogram", preferred_model_name := "") #compile_time`
- `android_get_external_data_path :: (app_id: string) -> string`
- `android_copy_file_to_device :: (file_path: string, app_id: string, external_storage_relative_path := "") -> bool`
- `android_copy_directories_to_device_quest :: android_copy_directories_to_device`
- `android_copy_directories_to_device :: (directories: [] string, app_id: string, dest_relative_path := "", only_update_newer_files := false, force_create_directories := false) -> bool`
- `Adb_Version_Info :: struct`
- `adb_version :: () -> Adb_Version_Info, bool`
- `Adb_Device :: struct`
- `adb_devices :: (return_only_ready_devices := false) -> bool, [] Adb_Device`
- `adb_device_serial: string`
- `Adb_Compression_Algorithm :: enum`
- `adb_push :: (local_source: string, remote_destination: string, only_update_newer_files := false, dry_run := false, compression_algorithm := Adb_Compression_Algorithm.UNSPECIFIED, disable_compression := false) -> bool`
- `adb_pull :: (remote_source: string, local_destination := "", preserve_timestamps := true, disable_compression := false, compression_algorithm := Adb_Compression_Algorithm.UNSPECIFIED) -> bool`
- `adb_shell_mkdir :: (path: string) -> bool`
- `adb_shell_pidof :: (package_name: string) -> bool, s32`
- `adb_install :: (apk_path: string, reinstall_keeping_data := false, allow_test_apk := false, allow_version_code_downgrade := false, uninstall_and_retry_on_key_mismatch := true, log_errors := true) -> bool`
- `adb_uninstall :: (package_name: string) -> bool`
- `adb :: (command_args: .. string, device_serial := "", log_adb_command_errors := false, print_captured_output := false, loc := #caller_location, try_and_escape_spaces_in_shell_arguments := true) -> (bool, output_string := "", error_string := "", exit_code := 0)`
- `__adb_path: string`
- `Android_Project_Config :: struct`
- `Android_Screen_Orientation :: enum`
- `Android_Default_Theme :: enum u16`
- `android_theme_to_string :: (theme: Android_Default_Theme) -> string`
- `generate_android_project :: (using config: Android_Project_Config, recreate_existing_files := true) -> bool`
- `get_android_project_jni_architecture_name :: (cpu_target: CPU_Tag) -> string`
- `Android_Signing_Key_Kind :: enum`
- `Android_Signing_Key :: struct`
- `Android_Build_Options :: struct`
- `android_generate_apk :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string`
- `Android_Apk :: struct`
- `apk_load :: (filepath: string) -> Android_Apk`
- `apk_deinit :: (apk: *Android_Apk)`
- `Android_Apk_Entry :: struct`
- `for_expansion :: (apk: Android_Apk, body: Code, flags: For_Flags) #expand`
- `apk_print_sections :: (apk: Android_Apk)`
- `ZIP_CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE : u32`
- `ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE : u32`
- `ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE :: 22`
- `ZIP_LOCAL_HEADER_SIZE :: size_of(Zip_Local_File_Header)`
- `Zip_Compression_Method :: enum u16`
- `Zip_Local_File_Header :: struct`
- `Zip_Central_Directory_Header :: struct`
- `Zip_End_Of_Central_Directory_Record :: struct`
- `ZIP_VERSION_DEFAULT : u16 : 0x0014`
- `Zip_Version :: struct`
- `zip_decode_version :: (version_word: u16) -> Zip_Version`
- `zip_encode_version :: (version: Zip_Version) -> u16`
- `ZIP_EOCD_OFFSET_FIELD_OFFSET :: 16`
- `zip_eocd_get_start_of_central_directory_offset :: (eocd: string) -> u32`
- `zip_eocd_set_start_of_central_directory_offset :: (eocd: string, new_offset: u32)`
- `zip_crc32 :: (s: string) -> u32`
- `Zip_Writer :: struct`
- `zip_init_writer :: (using w: *Zip_Writer, do_crcs := true, max_alignment := 0)`
- `zip_add_file :: (using w: *Zip_Writer, filepath: string, zip_filename: string, compression := Zip_Compression_Method.NONE, alignment := 0) -> bool`
- `zip_add_file_from_memory :: (using w: *Zip_Writer, file_data: string, zip_filename: string, compression := Zip_Compression_Method.NONE, alignment := 0) -> bool`
- `zip_write_central_directory :: (using w: *Zip_Writer)`
- `Kilobytes :: (x: int) -> int #expand`
- `Megabytes :: (x: int) -> int #expand`
- `android_generate_apk_gradle :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string`
- `android_generate_apk :: (project_path: string, release := false) -> bool, string #deprecated "Pass an Android_Build_Options as the first argument instead."`
- `gradle_install_apk :: (project_path: string, release := false) -> bool`
- `run_gradle_task :: (project_path: string, task: string, print_output := true) -> bool`
- `android_generate_apk_command_line_tools :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string`
- `android_generate_apk_handmade :: (options: Android_Build_Options, config: Android_Project_Config) -> bool, string`
- `Android_Signing_Key_Signature_Algorithm :: enum`
- `android_generate_signing_key_direct :: (out_path: string, key_name: string, algorithm := Android_Signing_Key_Signature_Algorithm.EC, key_size := 0, common_name := "Jai Android", organization_name := "Jai", country_code := "US") -> bool, Android_Signing_Key`
- `android_generate_jks_keystore :: (java_bin_path: string, keystore_path: string, key_alias := DEFAULT_KEYSTORE_ALIAS, key_password := DEFAULT_KEYSTORE_PASSWORD, algorithm := Android_Signing_Key_Signature_Algorithm.EC, key_size := 0, common_name := "Jai Android", organization_name := "Jai", country_code := "US") -> bool, Android_Signing_Key`
- `JAVA_ENV_NAME :: "JAVA_HOME"`
- `find_java_bin_path_from_environment :: () -> bool, bin_path: string, exe_path: string`
- `android_find_sdk_path :: () -> bool, string`
- `android_find_ndk_path :: (log_errors := false) -> bool, string`
- `ANDROID_APK_SIGNING_BLOCK_MAGIC :: "APK Sig Block 42"`
- `SIGNING_BLOCK_ID_V2_SIGNATURE : u32`
- `SIGNING_BLOCK_ID_V3_SIGNATURE : u32`
- `SIGNING_BLOCK_ID_V31_SIGNATURE : u32`
- `SIGNING_BLOCK_ID_VERITY_PADDING : u32`
- `Algorithm_Id :: enum u32`
- `Digest_Hash_Algorithm :: enum`
- `Digest :: struct`
- `Signature :: struct`
- `Certificiate :: struct`
- `Apk_Signing_Block_V2 :: struct`
- `apk_sign :: (unsigned_apk_filepath: string, keystore_filepath: string, keystore_password := "password") -> bool, string`
- `apk_sign2 :: (unsigned_apk_filepath: string, private_key_info: string, certificate: string) -> bool, string`
- `apk_read_signing_block_v2 :: (s: *string) -> bool, Apk_Signing_Block_V2`
- `apk_verify :: (filepath: string, verbose := false) -> bool`
- `apk_verify :: (apk: Android_Apk, verbose := false) -> bool`
- `apk_compute_digest_libressl :: (apk: Android_Apk, algorithm: Digest_Hash_Algorithm) -> [SHA256_DIGEST_LENGTH] u8`
- `sign_data :: (private_key_info: string, message: string) -> bool, string`
- `sha256 :: (s: string) -> [SHA256_DIGEST_LENGTH] u8`
- `sha1 :: (s: string) -> [SHA_DIGEST_LENGTH] u8`
- `libressl_get_public_key :: (subject_public_key_info: string) -> *evp_pkey_st`
- `libressl_get_private_key :: (private_key_info: string) -> *evp_pkey_st`
- `CheckSSL :: (result: s32, loc := #caller_location) #expand`
- `libressl_error_callback :: (str: *u8, len: u64, u: *void) -> s32 #c_call`
- `ASN1_Tag_Class :: enum u8`
- `ASN1_Tag_Length_Value :: enum u8`
- `ASN1_Universal_Type :: enum u8`
- `ASN1_Tag :: struct`
- `ASN1_Value :: struct`
- `ASN1_Object_Identifier :: struct`
- `asn1_oid_equals :: (a: ASN1_Object_Identifier, b: ASN1_Object_Identifier) -> bool`
- `asn1_print :: (name: string, value: ASN1_Value)`
- `asn1_print :: (name: string, data: string)`
- `asn1_print :: (data: string, depth := 0, hack_skip_depth := false)`
- `asn1_read_value :: (s: *string, expected: ASN1_Tag) -> ASN1_Value`
- `asn1_read_value :: (s: *string) -> ASN1_Value`
- `asn1_read_tag :: (s: *string) -> ASN1_Tag`
- `asn1_read_length :: (s: *string) -> u64`
- `asn1_decode_integer :: (data: string) -> s64`
- `ASN1_KNOWN_OIDS :: [1] ASN1_Object_Identifier`
- `asn1_decode_object_identifier :: (data: string) -> ASN1_Object_Identifier`
- `asn1_to_string :: (oid: ASN1_Object_Identifier) -> string`
- `x509_get_subject_public_key_info :: (certificate_data: string) -> ASN1_Value`
- `decrypt_private_key :: (encrypted_private_key_info: string, password: string) -> bool, private_key_info: string`
- `JKS_MAGIC : u32 : 0xfeedfeed`
- `JKS_MAGIC_LITTLE_ENDIAN :: #run -> u32` (constant value: `0xfeedfeed`)
- `OID_SUN_JKS_ENCRYPTION :: ASN1_Object_Identifier.{parts_count = 11, parts = .[1, 3, 6, 1, 4, 1, 42, 2, 17, 1, 1, 0]}`
- `jks_read :: (data: string, password: string) -> bool, certificate: string, encrypted_private_key_info: string`
- `jdk_decrypt_private_key :: (encrypted_data: string, password: string) -> bool, string`
- `read_length_prefixed_block :: (s: *string) -> string`
- `read_block :: (s: *string, length: int) -> string`
- `read_silly_alias :: ($Int_Type: Type, s: *string, $BIG_ENDIAN := false) -> Int_Type`
- `read :: read_silly_alias`
- `fh :: #bake_arguments formatInt(base = 16)`
- `fh2 :: #bake_arguments formatInt(minimum_digits = 2, base = 16)`
- `fh4 :: #bake_arguments formatInt(minimum_digits = 4, base = 16)`
- `fh8 :: #bake_arguments formatInt(minimum_digits = 8, base = 16)`
- `to_hex_string :: (s: string) -> string`
- `to_hex_string :: (a: [8] $T) -> string`

`JKS_MAGIC` and `JKS_MAGIC_LITTLE_ENDIAN` both evaluate to `0xfeedfeed`; byte swapping leaves this repeated-byte pattern unchanged. `OID_SUN_JKS_ENCRYPTION` is initialized with `parts_count = 11` and the exact 12-element backing array shown above. The declaration does not explain the extra trailing stored value, so consumers must honor the count field rather than assume every backing-array element is active.
- `USE_FULL_MODULE :: false`
- `Android_Install_Location :: enum u32`
- `LAUNCH_MODE_SINGLE_TASK : u32`
- `Android_Screen_Orientation_ :: enum s32`
- `Android_Config_Changes :: enum_flags s32`
- `android_config_changes_from_string :: (s: string) -> Android_Config_Changes`
- `write_binary_app_manifest_to_file :: (using config: Android_Project_Config, out_filepath: string) -> bool`
- `DUMMY_U32 : u32`
- `write_binary_app_manifest_to_memory :: (config: Android_Project_Config) -> bool, string`
- `write_binary_app_manifest_to_builder :: (b: *String_Builder, config: Android_Project_Config) -> bool`
- `ANDROID_XML_SCHEMA :: "http://schemas.android.com/apk/res/android"`
- `Android_Manifest_Binary :: struct`
- `deinit :: (using manifest: *Android_Manifest_Binary)`
- `read_binary_manifest_from_apk :: (apk_filename: string) -> bool, Android_Manifest_Binary`
- `read_binary_manifest_from_file :: (filename: string) -> bool, Android_Manifest_Binary`
- `read_binary_manifest_from_memory :: (data: string, filename := "<memory>") -> bool, Android_Manifest_Binary`
- `print_binary_manifest :: (using manifest: Android_Manifest_Binary, print_strings := false, print_resource_ids := false)`
- `ANDROID_SDK_LEVEL :: SDK_LEVEL`
- `ANDROID_HELPER_VERSION :: 2`
- `android_get_helper_program_executable_name :: () -> string`
- `android_maybe_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time`
- `android_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time`
- `ANDROID_DYNAMIC_LOADER_VERSION :: 3`
- `ANDROID_LOADER_LIB_NAME :: "loader"`
- `android_build_or_update_dynamic_loader :: (project_config: Android_Project_Config, build_options: Build_Options, android_build_options: Android_Build_Options, force_rebuild_loader := false) -> bool`
- `android_build_and_install_loader_app :: (project_config: Android_Project_Config, build_options: Build_Options, android_build_options: Android_Build_Options)`
- `NDK_Paths :: struct`
- `get_android_target_triple :: (cpu: CPU_Tag) -> triple: string, triple_with_sdk: string`
- `get_ndk_paths :: () -> NDK_Paths`
- `get_ndk_libc_paths :: (cpu: CPU_Tag) -> libc_path: string, libc_platform_path: string`
- `get_ndk_libclang_paths :: () -> [] string`
- `run_android_link_command :: (message: *Message_Phase, options: Build_Options, extra_args: [] string = .[])`
- `copy_android_libcpp :: (cpu_target: CPU_Tag, output_dir: string) -> bool`

- `adb_be_verbose := !true`
- `__no_gradle_build_be_verbose := false`
- `__debug_manifest_reading := false`

## Public type members

#### `Adb_Version_Info`

```jai
Adb_Version_Info :: struct {
    tool_version: string;
    protocol_version: string;
    exe_path: string;
}
```
#### `Adb_Device`

```jai
Adb_Device :: struct {
    Connection_State :: enum {
        CONNECTING;
        AUTHORIZING;
        UNAUTHORIZED;
        NOPERM;
        OFFLINE;
        BOOTLOADER;
        DEVICE;
        HOST;
        RECOVERY;
        SIDELOAD;
        UNKNOWN;
        READY :: DEVICE;
    }
    serial: string;
    connection_state: Connection_State;
    product_name: string;
    model_name:   string;
    device_name:  string;
    transport_id: int;
}
```
#### `Adb_Compression_Algorithm`

```jai
Adb_Compression_Algorithm :: enum {
    UNSPECIFIED;
    ANY;
    NONE;
    BROTLI;
    LZ4;
    ZSTD;
}
```
#### `Android_Project_Config`

```jai
Android_Project_Config :: struct {
    app_name := "MyJaiApp";
    project_path := "AndroidProject";
    app_id := "jai.example";
    version_code := 1;
    version_name := "0.1";
    min_sdk_level    := 29;
    target_sdk_level := SDK_LEVEL;
    lib_name := "main";
    theme := Android_Default_Theme.Theme_Black_NoTitleBar;
    screen_orientation := Android_Screen_Orientation.UNSPECIFIED;
    vr := false;
    quest_enable_passthrough_splash := false;
    android_manifest: string;
    Meta_Data_Item :: struct {
        name:  string;
        value: string;
    }
    application_meta_data: [] Meta_Data_Item;
    permissions: [] string;
    Feature :: struct {
        name: string;
        required: bool;
    }
    features : [] Feature;
    android_gradle_plugin_version := "8.2.0";
    gradle_version := "8.2";
}
```
#### `Android_Screen_Orientation`

```jai
Android_Screen_Orientation :: enum {
    UNSPECIFIED;
    LANDSCAPE;
    PORTRAIT;
    USER;
    BEHIND;
    SENSOR;
    NOSENSOR;
    SENSOR_LANDSCAPE;
    SENSOR_PORTRAIT;
    REVERSE_LANDSCAPE;
    REVERSE_PORTRAIT;
    FULL_SENSOR;
    USER_LANDSCAPE;
    USER_PORTRAIT;
    FULL_USER;
    LOCKED;
}
```
#### `Android_Default_Theme`

```jai
Android_Default_Theme :: enum u16 {
    Theme                                   :: 0x0005;
    Theme_NoTitleBar                        :: 0x0006;
    Theme_NoTitleBar_Fullscreen             :: 0x0007;
    Theme_Black                             :: 0x0008;
    Theme_Black_NoTitleBar                  :: 0x0009;
    Theme_Black_NoTitleBar_Fullscreen       :: 0x000a;
    Theme_Dialog                            :: 0x000b;
    Theme_Light                             :: 0x000c;
    Theme_Light_NoTitleBar                  :: 0x000d;
    Theme_Light_NoTitleBar_Fullscreen       :: 0x000e;
    Theme_Translucent                       :: 0x000f;
    Theme_Translucent_NoTitleBar            :: 0x0010;
    Theme_Translucent_NoTitleBar_Fullscreen :: 0x0011;
}
```
#### `Android_Signing_Key_Kind`

```jai
Android_Signing_Key_Kind :: enum {
    JKS;
    ASN1_CERTIFICATE_AND_PRIVATE_KEY;
}
```
#### `Android_Signing_Key`

```jai
Android_Signing_Key :: struct {
    kind: Android_Signing_Key_Kind;
    union {
        using jks: struct {
            keystore_path: string;
            alias: string;
            password: string;
        }
        direct: struct {
            certificate_path: string;
            private_key_path: string;
        }
        static: struct {
            certificate: string;
            private_key: string;
        }
    }
}
```
#### `Android_Build_Options`

```jai
Android_Build_Options :: struct {
    mode: enum {
        GRADLE;
        COMMAND_LINE_TOOLS;
        ILL_WRITE_IT_AND_WELL_DO_IT_LIVE_FUCKING_THING_SUCKS;
    } = .GRADLE;
    sdk_path: string;
    java_bin_path: string;
    sdk_level := SDK_LEVEL;
    build_type: enum {
        DEBUG;
        RELEASE;
    } = .DEBUG;
    DEFAULT_INTERMEDIATE_PATH :: ".build_android";
    intermediate_path := DEFAULT_INTERMEDIATE_PATH;
    cpu_target := CPU_Tag.ARM64;
    output_path: string;
    output_apk_name := "what_is_in_a_name";
    assets_path_type: enum {
        CONTENTS_OF_PATH;
        SINGLE_DIRECTORY;
    } = .SINGLE_DIRECTORY;
    assets_path: string;
    shared_library_path: string;
    use_jks_keystore_in_handmade_mode := false;
    signing_key_source: enum {
        AUTO_GENERATE;
        CUSTOM;
        SDK_DEFAULT_DEBUG;
    } = .AUTO_GENERATE;
    custom_signing_key: Android_Signing_Key;
}
```
#### `Android_Apk`

```jai
Android_Apk :: struct {
    filepath: string;
    zip_dir: Zip_Directory;
    entire_apk_is_loaded := true;
    data: string;
    content:                  string;
    signing_block:            string;
    central_directory:        string;
    end_of_central_directory: string;
    is_signed := false;
}
```
#### `Android_Apk_Entry`

```jai
Android_Apk_Entry :: struct {
    filename: string;
    compression_method: Zip_Compression_Method;
    compressed_size  :  u32;
    uncompressed_size:  u32;
    offset: u32;
}
```
`Android_Apk.filepath` borrows the argument string supplied to `apk_load`. `data` owns the complete loaded file; `content`, `signing_block`, `central_directory`, and `end_of_central_directory` are views into it. `zip_dir` owns a separate central-directory buffer and entry array; each entry filename is a view into that buffer. `apk_deinit` releases the directory storage and `data`, then zeroes the record. The caller must keep `filepath` storage alive for as long as it uses that field. `entire_apk_is_loaded` defaults to true and the loader always reads the whole APK. `is_signed` means a signing-block magic marker was found immediately before the central directory; it does not guarantee block validity or a valid signature.

`apk_load` asserts if ZIP-directory loading, whole-file reading, or the expected central-directory signature fails. Malformed files have no recoverable result. When the signing marker is present, the loader uses the adjacent size fields to form views without fully validating their bounds or internal records. Implementations may reject malformed/truncated input safely rather than reproduce unchecked reads. `for_expansion` walks entries in the directory and yields an `Android_Apk_Entry` value as `it`, plus its index as `it_index`; `.REVERSE` reverses iteration and `.POINTER` is rejected. The entry's `offset` is the file-data payload position after the local header, local filename and local extra field, not the local-header offset. Its filename is borrowed from `apk.zip_dir` and remains valid until `apk_deinit`; the offset and size fields come from the local file header. The iterator requires the full APK bytes already loaded.

`apk_verify` is distinct from the `is_signed` marker. It parses the supported v2 signing-block form, attempts signature, APK digest and certificate-key matching checks, and reports mismatches; it logs some cryptographic mismatches but does not make all such mismatches a false return. It is therefore not a reliable general-purpose signature-validity predicate. The v2 reader requires one signer and expects the supported digest, certificate, signature and public-key entries; unhandled attributes and other signing-block versions are unsupported. Its returned strings are views into the input block; its result arrays use the active context allocation. No guarantee is made for arbitrary or malformed signing-block input.

`generate_android_project` creates or refreshes the Gradle project files under `project_path`. Its `recreate_existing_files` default is true: when an existing app build file is detected, the generated project files are refreshed; with false, it reports failure instead of replacing them. The IDE workspace file is preserved when already present. The result is false if required directory or file operations fail. `app_name` supplies the generated project's display name, `app_id` the package identity, `lib_name` the shared-library name, and the manifest-related arrays provide application metadata, permissions, and feature declarations. A nonempty `android_manifest` is used as a caller-provided manifest override; an empty value selects generated manifest content. Permission SDK requirements are not automatically derived or enforced.

The binary-manifest writer overloads return a success flag. The memory overload returns serialized bytes on success; callers should treat the returned string as an allocated result associated with the active allocation context. The builder overload appends the representation to the supplied builder. The file overload writes to the requested path. The API does not expose a separate manifest-output destructor.

Signing-key generation has two distinct output forms. `android_generate_signing_key_direct` writes a certificate and private-key file below `out_path`, naming them from `key_name`, and returns a key record containing those paths. Its default is EC/P-256. The `key_size` argument does not change its generated key size; RSA is fixed at 2048 bits, and DSA is unsupported by this direct generator. `android_generate_jks_keystore` delegates JKS creation to the Java key tool at the supplied Java binary path and returns a JKS key record containing the path, alias, and password. Its default alias is `jaiiskewl`, its default password is `password`, and its default algorithm is EC with the `secp256r1` curve. For RSA, a zero key size selects 1024 bits and supported sizes are 1024, 2048, 4096, 8192, and 16384; EC accepts only the default size; DSA defaults to 1024 bits and accepts 1024, 2048, or 3072. Unsupported size/algorithm combinations return false; process or output failures also return false. The exact cryptographic encoding beyond the documented key kind/algorithm and generated files is not a caller-facing format guarantee.

The SDK-provided debug keystore is a separate signing-key source: it uses alias `androiddebugkey` and password `android`, and is looked up from the standard user Android configuration location or SDK location. These credentials identify that built-in debug key source; they do not describe the defaults for a newly generated JKS keystore.
#### `Zip_Compression_Method`

```jai
Zip_Compression_Method :: enum u16 {
    NONE      ::  0;
    SHRUNK    ::  1;
    REDUCED_1 ::  2;
    REDUCED_2 ::  3;
    REDUCED_3 ::  4;
    REDUCED_4 ::  5;
    IMPLODED  ::  6;
    RESERVED  ::  7;
    DEFLATE   ::  8;
    DEFLATE64 ::  9;
    BZIP2 :: 12;
    LZMA :: 14;
    ZSTD     :: 93;
    MP3      :: 94;
    XZ       :: 95;
    JPEG     :: 96;
    WAV_PACK :: 97;
    PPMD     :: 98;
}
```
#### `Zip_Local_File_Header`

```jai
Zip_Local_File_Header :: struct {
    SIGNATURE : u32 : 0x04034b50;
    signature: u32 = SIGNATURE;
    version_needed_to_extract: u16;
    general_purpose_bit_flag:  u16;
    compression_method:        Zip_Compression_Method;
    last_mod_file_time:        u16;
    last_mod_file_date:        u16;
    crc_32:                    u32 #align 1;
    compressed_size:           u32 #align 1;
    uncompressed_size:         u32 #align 1;
    filename_length:           u16 #align 1;
    extra_field_length:        u16 #align 1;
} #no_padding
```
#### `Zip_Central_Directory_Header`

```jai
Zip_Central_Directory_Header :: struct {
    SIGNATURE : u32 : 0x02014b50;
    signature: u32 = SIGNATURE;
    version_made_by:                 u16;
    version_needed_to_extract:       u16;
    general_purpose_bit_flag:        u16;
    compression_method:              Zip_Compression_Method;
    last_mod_file_time:              u16;
    last_mod_file_date:              u16;
    crc_32:                          u32;
    compressed_size:                 u32;
    uncompressed_size:               u32;
    filename_length:                 u16;
    extra_field_length:              u16;
    file_comment_length:             u16;
    disk_number_start:               u16;
    internal_file_attributes:        u16;
    external_file_attributes:        u32 #align 1;
    relative_offset_of_local_header: u32 #align 1;
} #no_padding
```
#### `Zip_End_Of_Central_Directory_Record`

```jai
Zip_End_Of_Central_Directory_Record :: struct {
    SIGNATURE :: 0x06054b50;
    signature : u32 = SIGNATURE;
    disk_number_this:         u16;
    disk_number_start:        u16;
    total_entries_this_disk:  u16;
    total_entries:            u16;
    central_directory_size:   u32;
    central_directory_offset: u32;
    file_comment_length:      u16;
} #no_padding
```

These are in-memory Jai aggregate layouts that are also appended as ZIP record bytes. Their directives are part of the public layout contract: `Zip_Local_File_Header` applies `#align 1` to `crc_32` and every following field and ends with `#no_padding`; the central-directory record applies `#align 1` to `external_file_attributes` and `relative_offset_of_local_header` and ends with `#no_padding`; the end record ends with `#no_padding`. The corresponding byte offsets are: local header fields `signature=0`, `version_needed_to_extract=4`, `general_purpose_bit_flag=6`, `compression_method=8`, `last_mod_file_time=10`, `last_mod_file_date=12`, `crc_32=14`, `compressed_size=18`, `uncompressed_size=22`, `filename_length=26`, `extra_field_length=28`; central fields `signature=0`, `version_made_by=4`, `version_needed_to_extract=6`, `general_purpose_bit_flag=8`, `compression_method=10`, `last_mod_file_time=12`, `last_mod_file_date=14`, `crc_32=16`, `compressed_size=20`, `uncompressed_size=24`, `filename_length=28`, `extra_field_length=30`, `file_comment_length=32`, `disk_number_start=34`, `internal_file_attributes=36`, `external_file_attributes=38`, `relative_offset_of_local_header=42`; end-record fields `signature=0`, `disk_number_this=4`, `disk_number_start=6`, `total_entries_this_disk=8`, `total_entries=10`, `central_directory_size=12`, `central_directory_offset=16`, `file_comment_length=20`. The local record is 30 bytes (`ZIP_LOCAL_HEADER_SIZE` is `size_of` it and is asserted equal to 30); the central-directory record is 46 bytes; the end record is 22 bytes (`ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE`). This declares these target record layouts; a field-only transcription that omits the alignment directives does not capture the public ABI. This does not imply an arbitrary packing mode for unrelated structs or a general compiler layout rule.

`ZIP_VERSION_DEFAULT` is typed `u16` with value `0x0014` (decimal 20), the encoded version 2.0 word. `zip_decode_version`/`zip_encode_version` split or combine the mapping byte and decimal major/minor byte; encoding requires `minor <= 9`.

## ZIP writer and APK behavior

`Zip_Writer` writes ZIP32 archives by appending local records, file data and, when requested, a central directory plus end record to its public `builder`. `zip_init_writer` defaults to CRC calculation enabled and `max_alignment=0`; it configures a temporary file-info array and a zeroed padding buffer of `max_alignment` bytes when nonzero. Negative capacity is rejected. Keep the temporary allocation context alive through central-directory emission. Initialization is optional for ordinary unaligned writes; no deinitialization/reset procedure or repeat-init/reuse guarantee is provided. Add all files before calling `zip_write_central_directory` once; that call appends the directory/end records and has no return value or finalized-state guard.

The add procedures support `NONE` (stored) and `DEFLATE`; other enum methods return false after logging unsupported compression. File read failure in `zip_add_file` asserts; compression failure returns false. `zip_add_file_from_memory` copies the file bytes into the writer output (compressed for deflate), and `zip_add_file` reads the complete source file before forwarding. The writer's `File_Info.filename` retains the supplied `zip_filename` string view until central-directory emission, so keep that storage alive through finalization. The output buffer then contains its own filename bytes. With `do_crcs=false`, both records receive `DUMMY_CRC` (`0x12345678`) rather than the computed checksum.

Alignment is encoded as a zero-filled local extra field so the file-data payload begins at the requested boundary. The only supported nonzero alignment is 4096 bytes; 16384 is explicitly rejected despite an earlier validation branch accepting it. `max_alignment` is the available zero-padding capacity and must be large enough for the computed padding (at most 4095 bytes for 4096-byte alignment). The central-directory extra length is not populated with this local padding. Record sizes, offsets and entry counts use ZIP32/16-bit fields; ZIP64 and values outside those field widths are not a supported guarantee. The writer serializes the target record layouts above directly; there is no big-endian output contract.
#### `Zip_Version`

```jai
Zip_Version :: struct {
    mapping: u8;
    major: u8;
    minor: u8;
}
```
#### `Zip_Writer`

```jai
Zip_Writer :: struct {
    File_Info :: struct {
        filename: string;
        offset:             u32;
        compression_method: Zip_Compression_Method;
        compressed_size:    u32;
        uncompressed_size:  u32;
        crc:                u32;
    }
    zip_file_infos: [..] File_Info;
    builder: String_Builder;
    dumb_padding_memory: [] u8;
    DUMMY_CRC : u32 : 0x12345678;
    write_crcs := true;
}
```
#### `Android_Signing_Key_Signature_Algorithm`

```jai
Android_Signing_Key_Signature_Algorithm :: enum {
    RSA;
    EC;
    DSA;
}
```
#### `Algorithm_Id`

```jai
Algorithm_Id :: enum u32 {
    RSASSA_PSS_SHA256        :: 0x0101;
    RSASSA_PSS_SHA512        :: 0x0102;
    RSASSA_PKCS1_v1_5_SHA256 :: 0x0103;
    RSASSA_PKCS1_v1_5_SHA512 :: 0x0104;
    ECDSA_SHA256             :: 0x0201;
    ECDSA_SHA512             :: 0x0202;
    DSA_SHA256               :: 0x0301;
}
```
#### `Digest_Hash_Algorithm`

```jai
Digest_Hash_Algorithm :: enum {
    SHA256;
    SHA512;
}
```
#### `Digest`

```jai
Digest :: struct {
    signature_algorithm_id: Algorithm_Id;
    data: string;
}
```
#### `Signature`

```jai
Signature :: struct {
    algorithm_id: Algorithm_Id;
    data: string;
}
```
#### `Certificiate`

```jai
Certificiate :: struct {
    data: string;
}
```
#### `Apk_Signing_Block_V2`

```jai
Apk_Signing_Block_V2 :: struct {
    signers_count :: 1;
    digests:      [..] Digest;
    signatures:   [..] Signature;
    certificates: [..] Certificiate;
    signed_data: string;
    subject_public_key_info: string;
}
```
#### `ASN1_Tag_Class`

```jai
ASN1_Tag_Class :: enum u8 {
    UNIVERSAL;
    APPLICATION;
    CONTEXT_SPECIFIC;
    PRIVATE;
}
```
#### `ASN1_Tag_Length_Value`

```jai
ASN1_Tag_Length_Value :: enum u8 {
    PRIMITIVE;
    CONSTRUCTED;
}
```
#### `ASN1_Universal_Type`

```jai
ASN1_Universal_Type :: enum u8 {
    RERSERVED_FOR_ENCODING_RULES :: 0;
    BOOLEAN           :: 1;
    INTEGER           :: 2;
    BIT_STRING        :: 3;
    OCTET_STRING      :: 4;
    NULL              :: 5;
    OBJECT_IDENTIFIER :: 6;
    OBJECT_DESCRIPTOR :: 7;
    REAL              :: 9;
    ENUMERATED        :: 10;
    EMBEDDED_PDV      :: 11;
    UTF8_STRING       :: 12;
    RELATIVE_OBJECT_IDENTIFER :: 13;
    TIME              :: 14;
    SEQUENCE          :: 16;
    SET               :: 17;
    NUMERIC_STRING    :: 18;
    PRINTABLE_STRING  :: 19;
    TELETEX_STRING    :: 20;
    VIDEOTEX_STRING   :: 21;
    IA5_STRING        :: 22;
    UTC_TIME          :: 23;
    GENERALIZED_TIME  :: 24;
    GRAPHIC_STRING    :: 25;
    VISIBLE_STRING    :: 26;
    GENERAL_STRING    :: 27;
    UNIVERSAL_STRING  :: 28;
    BMP_STRING        :: 30;
}
```
#### `ASN1_Tag`

```jai
ASN1_Tag :: struct {
    union {
        number: u8;
        universal_type: ASN1_Universal_Type;
    }
    length_value: ASN1_Tag_Length_Value;
    class: ASN1_Tag_Class;
}
```
#### `ASN1_Value`

```jai
ASN1_Value :: struct {
    using tag: ASN1_Tag;
    content: string;
    value_data: string;
}
```
#### `ASN1_Object_Identifier`

```jai
ASN1_Object_Identifier :: struct {
    MAX_SUBIDENTIFIERS :: 12;
    parts: [MAX_SUBIDENTIFIERS] u64;
    parts_count := 0;
}
```
#### `Android_Install_Location`

```jai
Android_Install_Location :: enum u32 {
    AUTO            :: 0;
    INTERNAL_ONLY   :: 1;
    PREFER_EXTERNAL :: 2;
}
```
#### `Android_Screen_Orientation_`

```jai
Android_Screen_Orientation_ :: enum s32 {
    UNSET             :: -2;
    UNSPECIFIED       :: -1;
    LANDSCAPE         :: 0;
    PORTRAIT          :: 1;
    USER              :: 2;
    BEHIND            :: 3;
    SENSOR            :: 4;
    NOSENSOR          :: 5;
    SENSOR_LANDSCAPE  :: 6;
    SENSOR_PORTRAIT   :: 7;
    REVERSE_LANDSCAPE :: 8;
    REVERSE_PORTRAIT  :: 9;
    FULL_SENSOR       :: 10;
    USER_LANDSCAPE    :: 11;
    USER_PORTRAIT     :: 12;
    FULL_USER         :: 13;
    LOCKED            :: 14;
}
```
#### `Android_Config_Changes`

```jai
Android_Config_Changes :: enum_flags s32 {
    mcc                :: 0x0001;
    mnc                :: 0x0002;
    locale             :: 0x0004;
    touchscreen        :: 0x0008;
    keyboard           :: 0x0010;
    keyboardHidden     :: 0x0020;
    navigation         :: 0x0040;
    orientation        :: 0x0080;
    screenLayout       :: 0x0100;
    uiMode             :: 0x0200;
    screenSize         :: 0x0400;
    smallestScreenSize :: 0x0800;
    density            :: 0x1000;
    layoutDirection    :: 0x2000;
    colorMode          :: 0x4000;
    fontScale          :: 0x40000000;
}
```
#### `Android_Manifest_Binary`

```jai
Android_Manifest_Binary :: struct {
    raw_data: string;
    free_data_on_deinit := false;
    string_pool:  [] string;
    resource_ids: [] u32;
    Xml_Namespace :: struct {
        prefix: string;
        uri: string;
    }
    android_namespace: Xml_Namespace;
    Xml_Element :: struct {
        kind: enum {
            START;
            END;
        }
        name: string;
        attributes: [] ResXMLTree_attribute;
    }
    elements: [..] Xml_Element;
}
```
#### `NDK_Paths`

```jai
NDK_Paths :: struct {
    root: string;
    toolchain: string;
    sysroot: string;
}
```

## ASN.1 and binary helper behavior

`ASN1_Value` embeds its `ASN1_Tag` as a promoted `using` field. `content` is a borrowed view of only the value bytes; `value_data` is a borrowed view of the complete encoded value, including identifier and length bytes. Both views point into the caller's input and remain valid only while that input remains valid. `asn1_read_value` consumes a complete tag, length and content from the cursor on well-formed input. The overload with `expected` compares tag class and then the universal type or non-universal tag number; it does not compare primitive/constructed form. A mismatch asserts after the value has been consumed. The source has no rollback guarantee after malformed input: tag/length parsing may already have consumed part of the input. Malformed or truncated input is outside the supported domain; an implementation may reject it safely without reproducing out-of-bounds source behavior.

The reader consumes a single identifier octet; high-tag-number form (low five bits equal to 31) is unsupported. Identifier octet zero is rejected. Universal tags must use a declared `ASN1_Universal_Type`; context-specific tags are accepted with numbers 0 through 30. Application and private tag classes are rejected. Definite lengths use short form for values through 127, or long form with one through eight following big-endian length octets; the resulting length is `u64`. Long form is not required to be the shortest representation. Indefinite length (`0x80`), the reserved `0xff` length octet, and lengths requiring more than eight octets are unsupported. The supported contract is well-formed, in-bounds values; source cursor position after malformed encodings is not a recovery contract.

`asn1_decode_integer` consumes only the integer content bytes supplied by the caller, interprets them as big-endian, and returns `s64`. Its supported values have one through eight content bytes, a clear high bit in the first byte, and therefore fit the nonnegative `s64` range. It does not enforce DER minimal-length encoding or reject redundant leading zero bytes. Empty, negative, or larger-than-eight-byte inputs are not supported; safe implementations may diagnose them rather than reproduce the source's assertions or invalid indexing.

`ASN1_Object_Identifier` stores at most 12 `u64` arcs. The supported input is a nonempty, valid base-128 OID content sequence: the first combined subidentifier is split into the first two arcs using the ASN.1 first-arc rules, and each remaining base-128 subidentifier becomes one arc. Every stored arc must fit `u64`. OIDs with more than 12 arcs or an arc exceeding that range are unsupported. Malformed/truncated base-128 input has no reliable source error policy and may be rejected safely. `asn1_to_string` returns a caller-released allocation containing decimal arcs separated by periods, including a trailing period (for example, arcs `1, 3, 6` render as `1.3.6.`). `asn1_oid_equals` compares the complete record bytes, including inactive `parts` entries; initialize inactive arcs consistently when relying on equality. `ASN1_KNOWN_OIDS` is a one-element array containing `OID_SUN_JKS_ENCRYPTION`.

`asn1_print(data, depth=0, hack_skip_depth=false)` reads a sequence of encoded values and writes diagnostic text to the normal print stream. Each item is indented by four spaces per `depth`; `hack_skip_depth=true` suppresses indentation for that invocation. SEQUENCE and SET print their type name and recurse into `content` at the next depth. OBJECT IDENTIFIER prints its type and dotted form. NULL prints its type and a trailing space, then requires empty content. INTEGER requires primitive form; up to eight bytes are decoded as the supported nonnegative integer, while longer content is printed as hex bytes. PRINTABLE_STRING and UTF8_STRING print their content directly. UTC_TIME requires exactly `YYMMDDhhmmssZ` and prints a converted calendar value. BIT_STRING requires the first content byte to be zero unused bits, then prints the remaining bytes as hex. OCTET_STRING prints its byte count and hex contents. Other universal types assert as unsupported. A constructed context-specific item prints `[tag_number] ` and recursively prints its content at the next depth with indentation skipped for that recursive call; a primitive context-specific item prints `[tag_number] IMPLICT type (0x<count> bytes)` and does not decode its content. Other tag classes fail through the reader.

`asn1_print(name, data)` wraps that diagnostic output in two yellow name banners. `asn1_print(name, value)` forwards only `value.content` to the data overload; it does not print or validate the value's own tag, so it is meaningful when the content itself is an encoded child sequence. The banners and diagnostic formatting are human-readable output, not a serialization format. Input must be well-formed and within the supported type subset.

`read_length_prefixed_block` reads a four-byte little-endian `u32` length, returns a borrowed view of the following bytes, and advances the input by the prefix and block length. `read_silly_alias` (also exported as the same procedure under the name `read`) reads `size_of(Int_Type)` bytes by value and advances by that amount; `BIG_ENDIAN=false` interprets the bytes in host-native order, while `true` byte-swaps the result. The source has no compile-time constraint that limits `Int_Type` to integral types, but behavior for non-integer, nonstandard-width, or aggregate types is unspecified; the supported use is built-in signed or unsigned integer widths of 1, 2, 4, or 8 bytes. The length-prefixed block helper and integer reader return/consume views only; they do not allocate or copy payload data. Inputs must contain the full prefix and payload; malformed-length handling is not specified by the source and may be safely rejected.

## ADB command and SDK path behavior

`android_find_sdk_path` checks the platform's conventional per-user SDK directory first (Windows uses `AppData/Local/Android/Sdk`; other targets use `Android/Sdk` under the home directory). If that directory exists it wins even when environment variables name another SDK. Otherwise it checks `ANDROID_HOME`, `ANDROID_SDK_ROOT`, then `ANDROID_SDK_HOME`, accepting the first value that names an existing directory and warning for set values that are not directories. Failure returns `false` and an empty string. Successful paths are returned as strings created in the active context; no independent ownership transfer is documented. `is_adb_installed` constructs the platform-tools executable path under that SDK, returns it as a temporary string when the file exists, and returns `false, ""` otherwise.

`adb` returns the process success flag, captured stdout and stderr, and the process exit code. Success requires a launched process that exits with code zero; launch failure returns false with empty captures and the default exit code, while signaled/nonzero results are failures. Captures are produced in the active context; callers using a temporary context must consume them before that context ends. With no command arguments it logs an error and returns failure. The `adb_device_serial` global is the fallback device selector: a nonempty per-call `device_serial` wins, otherwise a nonempty global value is used, otherwise no `-s` selector is passed. An empty per-call value therefore does not disable a configured global selector. The executable path is cached after first discovery in `__adb_path`; changing SDK environment or files after that does not refresh the cached path. ADB server startup is attempted as part of each command wrapper call and an unsuccessful startup is asserted.

For a command whose first argument is `shell`, `try_and_escape_spaces_in_shell_arguments=true` wraps each argument containing a literal space in double quotes before launching the host process. It does not escape embedded quotes, backslashes, shell metacharacters, or other whitespace, and it also wraps already quoted strings that contain spaces. This flag is not a general POSIX shell-escaping guarantee. Higher-level wrappers may apply their own quoting. `adb_push` and `adb_pull` reject `disable_compression=true` together with any non-`UNSPECIFIED` compression algorithm by assertion; the two modes cannot be combined. When disabling compression alone they pass `-Z`.

`adb_version` parses exactly three output lines with expected prefixes. A wrong line count or prefix returns `false` and a zero-initialized result; its returned field strings are slices into the temporary captured output and are temporary. `adb_devices` expects the standard heading and asserts when the heading or required row fields are malformed. Its result array uses temporary storage. Device string fields are views into captured process output, not separately allocated strings; the API does not document an independent ownership/release guarantee for that backing storage. When `return_only_ready_devices` is true, only records in `READY` state are included. `adb_shell_pidof` returns the first parsed PID when output contains multiple PIDs and warns that later values were ignored; exit code 1 means “not running” and returns `false, -1`, while other failures are logged and also return `false, -1`.
