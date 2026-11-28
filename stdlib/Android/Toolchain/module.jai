// Builds Android apps, installs them on a device and inspects or signs APK files.
//
// - Projects: [generate_android_project] writes a Gradle project and [write_binary_app_manifest_to_file]
//   writes the binary manifest, both from an [Android_Project_Config].
// - Builds: [android_generate_apk] makes an APK with Gradle or the SDK command-line tools, as chosen by
//   [Android_Build_Options]. The NDK is located with [android_find_ndk_path].
// - Devices: [adb], [adb_install], [adb_push] and [android_copy_file_to_device] run `adb` from the SDK found by
//   [is_adb_installed]; `adb_device_serial` selects the device.
// - Files: [read_binary_manifest_from_apk] reads a manifest out of an APK.
//
// Module parameters: `SDK_LEVEL` (default 34) is the default target API level and `ENABLE_ARM_LSE` (default
// true) lets the compiler assume the ARM LSE atomics.
//
// Tools are found through `ANDROID_HOME` and its older names, `JAVA_HOME` and the NDK variables, and are run as
// child processes. Strings and arrays that a procedure returns are owned by the caller unless its docs say
// they point into temporary storage. Most malformed-input failures log an error and also fail an assertion.

#module_parameters(
    // The Android API level to build against and target.
    SDK_LEVEL := 34,
    // Targets arm64 CPUs with the LSE atomic instructions.
    ENABLE_ARM_LSE := true
);

// Is the Android API level the module targets by default; the value of the `SDK_LEVEL` module parameter.
ANDROID_SDK_LEVEL :: SDK_LEVEL;

#load "project-types.jai";
#load "project-api.jai";
#load "manifest-types.jai";
#load "manifest-api.jai";
#load "sdk-api.jai";
#load "helper-api.jai";
#load "device-api.jai";
#load "gradle-api.jai";

// Returns the Android ABI folder name for `cpu_target`: `x86_64` or `arm64-v8a`; any other CPU fails an
// assertion.
get_android_project_jni_architecture_name :: Impl.get_android_project_jni_architecture_name;
// Returns the 20-byte SHA-1 digest of a string.
sha1 :: Digests.sha1;
// Returns the 32-byte SHA-256 digest of a string.
sha256 :: Digests.sha256;

#load "../../internal/android-toolchain/signing-model.jai";
#load "../../internal/android-toolchain/direct-key.jai";
#load "../../internal/android-toolchain/jks-generation.jai";
#load "../../internal/android-toolchain/format-helpers.jai";

#load "asn1-extension.jai";
#load "../../internal/android-toolchain/adb-types.jai";
#load "../../internal/android-toolchain/adb-process.jai";
#load "../../internal/android-toolchain/adb-transfer.jai";
#load "../../internal/android-toolchain/adb-apps.jai";
#load "../../internal/android-toolchain/java-discovery.jai";
#load "../../internal/android-toolchain/sdk-discovery.jai";
#load "../../internal/android-toolchain/config-flags.jai";
#load "../../internal/android-toolchain/x509-structure.jai";
#load "../../internal/archive/android-zip-wire.jai";
#load "../../internal/archive/android-zip-records.jai";
#load "../../internal/archive/android-zip-writer.jai";
#load "../../internal/archive/android-apk.jai";

#load "../../internal/android-toolchain/crypto-provider.jai";
#load "../../internal/android-toolchain/jks-reader.jai";
#load "../../internal/android-toolchain/jks-decryption.jai";
#load "../../internal/android-toolchain/apk-signing-model.jai";
#load "../../internal/android-toolchain/apk-signing-framing.jai";
#load "../../internal/android-toolchain/apk-signing-digest.jai";
#load "../../internal/android-toolchain/apk-signing-assembly.jai";
#load "../../internal/android-toolchain/apk-signing-orchestration.jai";

#scope_file
Impl :: #import "internal/android-toolchain/abi-names";
Digests :: #import "internal/crypto/sha";
