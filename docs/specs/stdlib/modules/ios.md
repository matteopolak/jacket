# iOS

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### iOS-specific declarations

- `get_ios_target_triple :: (cpu: CPU_Tag) -> string`
- `get_ios_sdk_path :: () -> string`
- `get_ios_simulator_sdk_path :: () -> string`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

This module provides iOS toolchain helpers. `get_ios_target_triple` maps `.ARM64` to `arm64-apple-ios` and every other `CPU_Tag` value to `x86_64-apple-ios`; unsupported CPU values are not rejected. `get_ios_sdk_path` and `get_ios_simulator_sdk_path` invoke `xcrun` for the `iphoneos` and `iphonesimulator` SDK path, respectively, cache the trimmed output after a successful command, and return an empty string when invocation fails or produces no output. Failure is logged; the SDK lookup helpers do not return a separate status or error value. Their cached values persist for the process lifetime, so changing the selected SDK environment after the first call is not reflected.

Importing the module does not perform an SDK lookup; each lookup is lazy on its first call.

The public surface is a direct native binding: declared records, constants, and function signatures preserve the represented external ABI; this wrapper does not itself define a different ownership or error model. Native status/error values and handle lifetimes follow the external API. ABI compatibility across external-library versions is not specified unless a version note appears above.

External libraries/frameworks named in source: no explicit `#library` name identified.


No iOS framework binding surface is declared here; the public surface consists of these three toolchain helpers. The wider iOS SDK lookup/source graph and any additional exports are unspecified.
