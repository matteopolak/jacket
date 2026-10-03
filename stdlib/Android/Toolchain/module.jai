#module_parameters(SDK_LEVEL := 34, ENABLE_ARM_LSE := true);

ANDROID_SDK_LEVEL :: SDK_LEVEL;

#load "project-types.jai";
#load "project-api.jai";
#load "manifest-types.jai";
#load "manifest-api.jai";
#load "sdk-api.jai";
#load "helper-api.jai";
#load "device-api.jai";
#load "gradle-api.jai";

get_android_project_jni_architecture_name :: Impl.get_android_project_jni_architecture_name;
sha1 :: Digests.sha1;
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
