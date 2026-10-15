#import "Basic";
A :: #import "Android/Toolchain"(SDK_LEVEL = 31, ENABLE_ARM_LSE = false);

main :: () {
    config: A.Android_Project_Config;

    assert(A.ANDROID_SDK_LEVEL == 31);
    assert(config.target_sdk_level == 31);
    assert(!A.ENABLE_ARM_LSE);
    config.target_sdk_level = 34;
    assert(config.target_sdk_level == 34 && A.ANDROID_SDK_LEVEL == 31);
}
