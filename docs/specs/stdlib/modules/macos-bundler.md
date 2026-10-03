# MacOS_Bundler

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `create_app_bundle :: (app_name: string, app_exe_path: string, icon_file_path: string, resource_directory_to_copy: string, only_copy_subdirectories_and_files: bool, write_plist: bool) -> app_bundle_dir: string, resources_path: string`
- `make_app_bundle_subdirectories :: (app_bundle_name: string) -> macos_folder_path: string, resources_path: string`
- `convert_image_to_icns :: (input_path: string, output_path: string) -> bool`
- `plist_start :: (builder: *String_Builder)`
- `plist_end :: (builder: *String_Builder)`
- `plist_write_key_value :: (builder: *String_Builder, key: string, value: Any) -> success: bool`
- `plist_write_key :: (builder: *String_Builder, key: string)`
- `plist_write_string :: (builder: *String_Builder, str: string)`
- `plist_write_bool :: (builder: *String_Builder, value: bool)`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

The bundler creates a macOS `.app` bundle from an executable and resource directory. Its copy option controls whether only selected subdirectories/files or broader resource contents are copied. File-system failures are reported by the bundle operation; bundle signing/notarization and plist completeness are not guaranteed by this helper.
