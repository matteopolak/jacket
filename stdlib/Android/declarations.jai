// Generated from docs/specs/stdlib/modules/android.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "POSIX";

lib_glue :: #library,system "android_native_app_glue";
libandroid :: #library,system "android";
libcamera2ndk :: #library,system "camera2ndk";
liblog :: #library,system "log";
libmediandk :: #library,system "mediandk";
#scope_export

// Types and layouts

_jfieldID :: struct {}

_jmethodID :: struct {}

AAsset :: struct {}

AASSET_MODE :: enum u32 {
    UNKNOWN :: 0;
    RANDOM :: 1;
    STREAMING :: 2;
    BUFFER :: 3;
}

AAssetDir :: struct {}

AAssetManager :: struct {}

#if INCLUDE_CAMERA_BINDINGS {
    ACAMERA :: enum u32 {
        TYPE_BYTE :: 0;
        TYPE_INT32 :: 1;
        TYPE_FLOAT :: 2;
        TYPE_INT64 :: 3;
        TYPE_DOUBLE :: 4;
        TYPE_RATIONAL :: 5;
        NUM_TYPES :: 6;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_automotive_lens_facing :: enum u32 {
        EXTERIOR_OTHER :: 0;
        EXTERIOR_FRONT :: 1;
        EXTERIOR_REAR :: 2;
        EXTERIOR_LEFT :: 3;
        EXTERIOR_RIGHT :: 4;
        INTERIOR_OTHER :: 5;
        INTERIOR_SEAT_ROW_1_LEFT :: 6;
        INTERIOR_SEAT_ROW_1_CENTER :: 7;
        INTERIOR_SEAT_ROW_1_RIGHT :: 8;
        INTERIOR_SEAT_ROW_2_LEFT :: 9;
        INTERIOR_SEAT_ROW_2_CENTER :: 10;
        INTERIOR_SEAT_ROW_2_RIGHT :: 11;
        INTERIOR_SEAT_ROW_3_LEFT :: 12;
        INTERIOR_SEAT_ROW_3_CENTER :: 13;
        INTERIOR_SEAT_ROW_3_RIGHT :: 14;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_automotive_location :: enum u32 {
        INTERIOR :: 0;
        EXTERIOR_OTHER :: 1;
        EXTERIOR_FRONT :: 2;
        EXTERIOR_REAR :: 3;
        EXTERIOR_LEFT :: 4;
        EXTERIOR_RIGHT :: 5;
        EXTRA_OTHER :: 6;
        EXTRA_FRONT :: 7;
        EXTRA_REAR :: 8;
        EXTRA_LEFT :: 9;
        EXTRA_RIGHT :: 10;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_black_level_lock :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_color_correction_aberration_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_color_correction_mode :: enum u32 {
        TRANSFORM_MATRIX :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_antibanding_mode :: enum u32 {
        OFF :: 0;
        _50HZ :: 1;
        _60HZ :: 2;
        AUTO :: 3;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_lock :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_lock_available :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
        ON_AUTO_FLASH :: 2;
        ON_ALWAYS_FLASH :: 3;
        ON_AUTO_FLASH_REDEYE :: 4;
        ON_EXTERNAL_FLASH :: 5;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_precapture_trigger :: enum u32 {
        IDLE :: 0;
        START :: 1;
        CANCEL :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_ae_state :: enum u32 {
        INACTIVE :: 0;
        SEARCHING :: 1;
        CONVERGED :: 2;
        LOCKED :: 3;
        FLASH_REQUIRED :: 4;
        PRECAPTURE :: 5;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_af_mode :: enum u32 {
        OFF :: 0;
        AUTO :: 1;
        MACRO :: 2;
        CONTINUOUS_VIDEO :: 3;
        CONTINUOUS_PICTURE :: 4;
        EDOF :: 5;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_af_scene_change :: enum u32 {
        NOT_DETECTED :: 0;
        DETECTED :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_af_state :: enum u32 {
        INACTIVE :: 0;
        PASSIVE_SCAN :: 1;
        PASSIVE_FOCUSED :: 2;
        ACTIVE_SCAN :: 3;
        FOCUSED_LOCKED :: 4;
        NOT_FOCUSED_LOCKED :: 5;
        PASSIVE_UNFOCUSED :: 6;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_af_trigger :: enum u32 {
        IDLE :: 0;
        START :: 1;
        CANCEL :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_awb_lock :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_awb_lock_available :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_awb_mode :: enum u32 {
        OFF :: 0;
        AUTO :: 1;
        INCANDESCENT :: 2;
        FLUORESCENT :: 3;
        WARM_FLUORESCENT :: 4;
        DAYLIGHT :: 5;
        CLOUDY_DAYLIGHT :: 6;
        TWILIGHT :: 7;
        SHADE :: 8;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_awb_state :: enum u32 {
        INACTIVE :: 0;
        SEARCHING :: 1;
        CONVERGED :: 2;
        LOCKED :: 3;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_capture_intent :: enum u32 {
        CUSTOM :: 0;
        PREVIEW :: 1;
        STILL_CAPTURE :: 2;
        VIDEO_RECORD :: 3;
        VIDEO_SNAPSHOT :: 4;
        ZERO_SHUTTER_LAG :: 5;
        MANUAL :: 6;
        MOTION_TRACKING :: 7;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_effect_mode :: enum u32 {
        OFF :: 0;
        MONO :: 1;
        NEGATIVE :: 2;
        SOLARIZE :: 3;
        SEPIA :: 4;
        POSTERIZE :: 5;
        WHITEBOARD :: 6;
        BLACKBOARD :: 7;
        AQUA :: 8;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_enable_zsl :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_extended_scene_mode :: enum u32 {
        DISABLED :: 0;
        BOKEH_STILL_CAPTURE :: 1;
        BOKEH_CONTINUOUS :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_mode :: enum u32 {
        OFF :: 0;
        AUTO :: 1;
        USE_SCENE_MODE :: 2;
        OFF_KEEP_STATE :: 3;
        USE_EXTENDED_SCENE_MODE :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_scene_mode :: enum u32 {
        DISABLED :: 0;
        FACE_PRIORITY :: 1;
        ACTION :: 2;
        PORTRAIT :: 3;
        LANDSCAPE :: 4;
        NIGHT :: 5;
        NIGHT_PORTRAIT :: 6;
        THEATRE :: 7;
        BEACH :: 8;
        SNOW :: 9;
        SUNSET :: 10;
        STEADYPHOTO :: 11;
        FIREWORKS :: 12;
        SPORTS :: 13;
        PARTY :: 14;
        CANDLELIGHT :: 15;
        BARCODE :: 16;
        HDR :: 18;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_control_video_stabilization_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
        PREVIEW_STABILIZATION :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_depth_available_depth_stream_configurations :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_depth_available_depth_stream_configurations_maximum_resolution :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_depth_available_dynamic_depth_stream_configurations :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_depth_available_dynamic_depth_stream_configurations_maximum_resolution :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_depth_depth_is_exclusive :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_distortion_correction_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_edge_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
        ZERO_SHUTTER_LAG :: 3;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_flash_info_available :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_flash_mode :: enum u32 {
        OFF :: 0;
        SINGLE :: 1;
        TORCH :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_flash_state :: enum u32 {
        UNAVAILABLE :: 0;
        CHARGING :: 1;
        READY :: 2;
        FIRED :: 3;
        PARTIAL :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_heic_available_heic_stream_configurations :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_heic_available_heic_stream_configurations_maximum_resolution :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_hot_pixel_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_info_supported_hardware_level :: enum u32 {
        LIMITED :: 0;
        FULL :: 1;
        LEGACY :: 2;
        _3 :: 3;
        EXTERNAL :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_lens_facing :: enum u32 {
        FRONT :: 0;
        BACK :: 1;
        EXTERNAL :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_lens_info_focus_distance_calibration :: enum u32 {
        UNCALIBRATED :: 0;
        APPROXIMATE :: 1;
        CALIBRATED :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_lens_optical_stabilization_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_lens_pose_reference :: enum u32 {
        PRIMARY_CAMERA :: 0;
        GYROSCOPE :: 1;
        UNDEFINED :: 2;
        AUTOMOTIVE :: 3;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_lens_state :: enum u32 {
        STATIONARY :: 0;
        MOVING :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_logical_multi_camera_sensor_sync_type :: enum u32 {
        APPROXIMATE :: 0;
        CALIBRATED :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_noise_reduction_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
        MINIMAL :: 3;
        ZERO_SHUTTER_LAG :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_request_available_capabilities :: enum u32 {
        BACKWARD_COMPATIBLE :: 0;
        MANUAL_SENSOR :: 1;
        MANUAL_POST_PROCESSING :: 2;
        RAW :: 3;
        READ_SENSOR_SETTINGS :: 5;
        BURST_CAPTURE :: 6;
        DEPTH_OUTPUT :: 8;
        MOTION_TRACKING :: 10;
        LOGICAL_MULTI_CAMERA :: 11;
        MONOCHROME :: 12;
        SECURE_IMAGE_DATA :: 13;
        SYSTEM_CAMERA :: 14;
        ULTRA_HIGH_RESOLUTION_SENSOR :: 16;
        STREAM_USE_CASE :: 19;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_request_available_dynamic_range_profiles_map :: enum u32 {
        STANDARD :: 1;
        HLG10 :: 2;
        HDR10 :: 4;
        HDR10_PLUS :: 8;
        DOLBY_VISION_10B_HDR_REF :: 16;
        DOLBY_VISION_10B_HDR_REF_PO :: 32;
        DOLBY_VISION_10B_HDR_OEM :: 64;
        DOLBY_VISION_10B_HDR_OEM_PO :: 128;
        DOLBY_VISION_8B_HDR_REF :: 256;
        DOLBY_VISION_8B_HDR_REF_PO :: 512;
        DOLBY_VISION_8B_HDR_OEM :: 1024;
        DOLBY_VISION_8B_HDR_OEM_PO :: 2048;
        MAX :: 4096;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_available_recommended_stream_configurations :: enum u32 {
        PREVIEW :: 0;
        RECORD :: 1;
        VIDEO_SNAPSHOT :: 2;
        SNAPSHOT :: 3;
        ZSL :: 4;
        RAW :: 5;
        LOW_LATENCY_SNAPSHOT :: 6;
        PUBLIC_END :: 7;
        _10BIT_OUTPUT :: 8;
        PUBLIC_END_3_8 :: 9;
        VENDOR_START :: 24;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_available_stream_configurations :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_available_stream_configurations_maximum_resolution :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_available_stream_use_cases :: enum u32 {
        DEFAULT :: 0;
        PREVIEW :: 1;
        STILL_CAPTURE :: 2;
        VIDEO_RECORD :: 3;
        PREVIEW_VIDEO_STILL :: 4;
        VIDEO_CALL :: 5;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_cropping_type :: enum u32 {
        CENTER_ONLY :: 0;
        FREEFORM :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_multi_resolution_stream_supported :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_physical_camera_multi_resolution_stream_configurations :: enum u32 {
        OUTPUT :: 0;
        INPUT :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_scaler_rotate_and_crop :: enum u32 {
        NONE :: 0;
        _90 :: 1;
        _180 :: 2;
        _270 :: 3;
        AUTO :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_info_color_filter_arrangement :: enum u32 {
        RGGB :: 0;
        GRBG :: 1;
        GBRG :: 2;
        BGGR :: 3;
        RGB :: 4;
        MONO :: 5;
        NIR :: 6;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_info_lens_shading_applied :: enum u32 {
        FALSE :: 0;
        TRUE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_info_timestamp_source :: enum u32 {
        UNKNOWN :: 0;
        REALTIME :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_pixel_mode :: enum u32 {
        DEFAULT :: 0;
        MAXIMUM_RESOLUTION :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_raw_binning_factor_used :: enum u32 {
        TRUE :: 0;
        FALSE :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_reference_illuminant1 :: enum u32 {
        DAYLIGHT :: 1;
        FLUORESCENT :: 2;
        TUNGSTEN :: 3;
        FLASH :: 4;
        FINE_WEATHER :: 9;
        CLOUDY_WEATHER :: 10;
        SHADE :: 11;
        DAYLIGHT_FLUORESCENT :: 12;
        DAY_WHITE_FLUORESCENT :: 13;
        COOL_WHITE_FLUORESCENT :: 14;
        WHITE_FLUORESCENT :: 15;
        STANDARD_A :: 17;
        STANDARD_B :: 18;
        STANDARD_C :: 19;
        D55 :: 20;
        D65 :: 21;
        D75 :: 22;
        D50 :: 23;
        ISO_STUDIO_TUNGSTEN :: 24;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sensor_test_pattern_mode :: enum u32 {
        OFF :: 0;
        SOLID_COLOR :: 1;
        COLOR_BARS :: 2;
        COLOR_BARS_FADE_TO_GRAY :: 3;
        PN9 :: 4;
        CUSTOM1 :: 256;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_shading_mode :: enum u32 {
        OFF :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_statistics_face_detect_mode :: enum u32 {
        OFF :: 0;
        SIMPLE :: 1;
        FULL :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_statistics_hot_pixel_map_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_statistics_lens_shading_map_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_statistics_ois_data_mode :: enum u32 {
        OFF :: 0;
        ON :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_statistics_scene_flicker :: enum u32 {
        NONE :: 0;
        _50HZ :: 1;
        _60HZ :: 2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sync_frame_number :: enum s32 {
        CONVERGING :: -1;
        UNKNOWN :: -2;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_sync_max_latency :: enum s32 {
        PER_FRAME_CONTROL :: 0;
        UNKNOWN :: -1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_tonemap_mode :: enum u32 {
        CONTRAST_CURVE :: 0;
        FAST :: 1;
        HIGH_QUALITY :: 2;
        GAMMA_VALUE :: 3;
        PRESET_CURVE :: 4;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_acamera_tonemap_preset_curve :: enum u32 {
        SRGB :: 0;
        REC709 :: 1;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_section :: enum u32 {
        COLOR_CORRECTION :: 0;
        CONTROL :: 1;
        DEMOSAIC :: 2;
        EDGE :: 3;
        FLASH :: 4;
        FLASH_INFO :: 5;
        HOT_PIXEL :: 6;
        JPEG :: 7;
        LENS :: 8;
        LENS_INFO :: 9;
        NOISE_REDUCTION :: 10;
        QUIRKS :: 11;
        REQUEST :: 12;
        SCALER :: 13;
        SENSOR :: 14;
        SENSOR_INFO :: 15;
        SHADING :: 16;
        STATISTICS :: 17;
        STATISTICS_INFO :: 18;
        TONEMAP :: 19;
        LED :: 20;
        INFO :: 21;
        BLACK_LEVEL :: 22;
        SYNC :: 23;
        REPROCESS :: 24;
        DEPTH :: 25;
        LOGICAL_MULTI_CAMERA :: 26;
        DISTORTION_CORRECTION :: 27;
        HEIC :: 28;
        HEIC_INFO :: 29;
        AUTOMOTIVE :: 30;
        AUTOMOTIVE_LENS :: 31;
        SECTION_COUNT :: 32;
        VENDOR :: 32768;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_section_start :: enum s32 {
        COLOR_CORRECTION_START :: 0;
        CONTROL_START :: 65536;
        DEMOSAIC_START :: 131072;
        EDGE_START :: 196608;
        FLASH_START :: 262144;
        FLASH_INFO_START :: 327680;
        HOT_PIXEL_START :: 393216;
        JPEG_START :: 458752;
        LENS_START :: 524288;
        LENS_INFO_START :: 589824;
        NOISE_REDUCTION_START :: 655360;
        QUIRKS_START :: 720896;
        REQUEST_START :: 786432;
        SCALER_START :: 851968;
        SENSOR_START :: 917504;
        SENSOR_INFO_START :: 983040;
        SHADING_START :: 1048576;
        STATISTICS_START :: 1114112;
        STATISTICS_INFO_START :: 1179648;
        TONEMAP_START :: 1245184;
        LED_START :: 1310720;
        INFO_START :: 1376256;
        BLACK_LEVEL_START :: 1441792;
        SYNC_START :: 1507328;
        REPROCESS_START :: 1572864;
        DEPTH_START :: 1638400;
        LOGICAL_MULTI_CAMERA_START :: 1703936;
        DISTORTION_CORRECTION_START :: 1769472;
        HEIC_START :: 1835008;
        HEIC_INFO_START :: 1900544;
        AUTOMOTIVE_START :: 1966080;
        AUTOMOTIVE_LENS_START :: 2031616;
        VENDOR_START :: -2147483648;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_tag :: enum u32 {
        COLOR_CORRECTION_MODE :: 0;
        COLOR_CORRECTION_TRANSFORM :: 1;
        COLOR_CORRECTION_GAINS :: 2;
        COLOR_CORRECTION_ABERRATION_MODE :: 3;
        COLOR_CORRECTION_AVAILABLE_ABERRATION_MODES :: 4;
        COLOR_CORRECTION_END :: 5;
        CONTROL_AE_ANTIBANDING_MODE :: 65536;
        CONTROL_AE_EXPOSURE_COMPENSATION :: 65537;
        CONTROL_AE_LOCK :: 65538;
        CONTROL_AE_MODE :: 65539;
        CONTROL_AE_REGIONS :: 65540;
        CONTROL_AE_TARGET_FPS_RANGE :: 65541;
        CONTROL_AE_PRECAPTURE_TRIGGER :: 65542;
        CONTROL_AF_MODE :: 65543;
        CONTROL_AF_REGIONS :: 65544;
        CONTROL_AF_TRIGGER :: 65545;
        CONTROL_AWB_LOCK :: 65546;
        CONTROL_AWB_MODE :: 65547;
        CONTROL_AWB_REGIONS :: 65548;
        CONTROL_CAPTURE_INTENT :: 65549;
        CONTROL_EFFECT_MODE :: 65550;
        CONTROL_MODE :: 65551;
        CONTROL_SCENE_MODE :: 65552;
        CONTROL_VIDEO_STABILIZATION_MODE :: 65553;
        CONTROL_AE_AVAILABLE_ANTIBANDING_MODES :: 65554;
        CONTROL_AE_AVAILABLE_MODES :: 65555;
        CONTROL_AE_AVAILABLE_TARGET_FPS_RANGES :: 65556;
        CONTROL_AE_COMPENSATION_RANGE :: 65557;
        CONTROL_AE_COMPENSATION_STEP :: 65558;
        CONTROL_AF_AVAILABLE_MODES :: 65559;
        CONTROL_AVAILABLE_EFFECTS :: 65560;
        CONTROL_AVAILABLE_SCENE_MODES :: 65561;
        CONTROL_AVAILABLE_VIDEO_STABILIZATION_MODES :: 65562;
        CONTROL_AWB_AVAILABLE_MODES :: 65563;
        CONTROL_MAX_REGIONS :: 65564;
        CONTROL_AE_STATE :: 65567;
        CONTROL_AF_STATE :: 65568;
        CONTROL_AWB_STATE :: 65570;
        CONTROL_AE_LOCK_AVAILABLE :: 65572;
        CONTROL_AWB_LOCK_AVAILABLE :: 65573;
        CONTROL_AVAILABLE_MODES :: 65574;
        CONTROL_POST_RAW_SENSITIVITY_BOOST_RANGE :: 65575;
        CONTROL_POST_RAW_SENSITIVITY_BOOST :: 65576;
        CONTROL_ENABLE_ZSL :: 65577;
        CONTROL_AF_SCENE_CHANGE :: 65578;
        CONTROL_AVAILABLE_EXTENDED_SCENE_MODE_MAX_SIZES :: 65579;
        CONTROL_AVAILABLE_EXTENDED_SCENE_MODE_ZOOM_RATIO_RANGES :: 65580;
        CONTROL_EXTENDED_SCENE_MODE :: 65581;
        CONTROL_ZOOM_RATIO_RANGE :: 65582;
        CONTROL_ZOOM_RATIO :: 65583;
        CONTROL_END :: 65584;
        EDGE_MODE :: 196608;
        EDGE_AVAILABLE_EDGE_MODES :: 196610;
        EDGE_END :: 196611;
        FLASH_MODE :: 262146;
        FLASH_STATE :: 262149;
        FLASH_END :: 262150;
        FLASH_INFO_AVAILABLE :: 327680;
        FLASH_INFO_STRENGTH_MAXIMUM_LEVEL :: 327682;
        FLASH_INFO_STRENGTH_DEFAULT_LEVEL :: 327683;
        FLASH_INFO_END :: 327684;
        HOT_PIXEL_MODE :: 393216;
        HOT_PIXEL_AVAILABLE_HOT_PIXEL_MODES :: 393217;
        HOT_PIXEL_END :: 393218;
        JPEG_GPS_COORDINATES :: 458752;
        JPEG_GPS_PROCESSING_METHOD :: 458753;
        JPEG_GPS_TIMESTAMP :: 458754;
        JPEG_ORIENTATION :: 458755;
        JPEG_QUALITY :: 458756;
        JPEG_THUMBNAIL_QUALITY :: 458757;
        JPEG_THUMBNAIL_SIZE :: 458758;
        JPEG_AVAILABLE_THUMBNAIL_SIZES :: 458759;
        JPEG_END :: 458760;
        LENS_APERTURE :: 524288;
        LENS_FILTER_DENSITY :: 524289;
        LENS_FOCAL_LENGTH :: 524290;
        LENS_FOCUS_DISTANCE :: 524291;
        LENS_OPTICAL_STABILIZATION_MODE :: 524292;
        LENS_FACING :: 524293;
        LENS_POSE_ROTATION :: 524294;
        LENS_POSE_TRANSLATION :: 524295;
        LENS_FOCUS_RANGE :: 524296;
        LENS_STATE :: 524297;
        LENS_INTRINSIC_CALIBRATION :: 524298;
        LENS_RADIAL_DISTORTION :: 524299;
        LENS_POSE_REFERENCE :: 524300;
        LENS_DISTORTION :: 524301;
        LENS_DISTORTION_MAXIMUM_RESOLUTION :: 524302;
        LENS_INTRINSIC_CALIBRATION_MAXIMUM_RESOLUTION :: 524303;
        LENS_END :: 524304;
        LENS_INFO_AVAILABLE_APERTURES :: 589824;
        LENS_INFO_AVAILABLE_FILTER_DENSITIES :: 589825;
        LENS_INFO_AVAILABLE_FOCAL_LENGTHS :: 589826;
        LENS_INFO_AVAILABLE_OPTICAL_STABILIZATION :: 589827;
        LENS_INFO_HYPERFOCAL_DISTANCE :: 589828;
        LENS_INFO_MINIMUM_FOCUS_DISTANCE :: 589829;
        LENS_INFO_SHADING_MAP_SIZE :: 589830;
        LENS_INFO_FOCUS_DISTANCE_CALIBRATION :: 589831;
        LENS_INFO_END :: 589832;
        NOISE_REDUCTION_MODE :: 655360;
        NOISE_REDUCTION_AVAILABLE_NOISE_REDUCTION_MODES :: 655362;
        NOISE_REDUCTION_END :: 655363;
        REQUEST_MAX_NUM_OUTPUT_STREAMS :: 786438;
        REQUEST_PIPELINE_DEPTH :: 786441;
        REQUEST_PIPELINE_MAX_DEPTH :: 786442;
        REQUEST_PARTIAL_RESULT_COUNT :: 786443;
        REQUEST_AVAILABLE_CAPABILITIES :: 786444;
        REQUEST_AVAILABLE_REQUEST_KEYS :: 786445;
        REQUEST_AVAILABLE_RESULT_KEYS :: 786446;
        REQUEST_AVAILABLE_CHARACTERISTICS_KEYS :: 786447;
        REQUEST_AVAILABLE_SESSION_KEYS :: 786448;
        REQUEST_AVAILABLE_PHYSICAL_CAMERA_REQUEST_KEYS :: 786449;
        REQUEST_AVAILABLE_DYNAMIC_RANGE_PROFILES_MAP :: 786451;
        REQUEST_END :: 786452;
        SCALER_CROP_REGION :: 851968;
        SCALER_AVAILABLE_MAX_DIGITAL_ZOOM :: 851972;
        SCALER_AVAILABLE_STREAM_CONFIGURATIONS :: 851978;
        SCALER_AVAILABLE_MIN_FRAME_DURATIONS :: 851979;
        SCALER_AVAILABLE_STALL_DURATIONS :: 851980;
        SCALER_CROPPING_TYPE :: 851981;
        SCALER_AVAILABLE_RECOMMENDED_STREAM_CONFIGURATIONS :: 851982;
        SCALER_AVAILABLE_RECOMMENDED_INPUT_OUTPUT_FORMATS_MAP :: 851983;
        SCALER_AVAILABLE_ROTATE_AND_CROP_MODES :: 851984;
        SCALER_ROTATE_AND_CROP :: 851985;
        SCALER_DEFAULT_SECURE_IMAGE_SIZE :: 851986;
        SCALER_PHYSICAL_CAMERA_MULTI_RESOLUTION_STREAM_CONFIGURATIONS :: 851987;
        SCALER_AVAILABLE_STREAM_CONFIGURATIONS_MAXIMUM_RESOLUTION :: 851988;
        SCALER_AVAILABLE_MIN_FRAME_DURATIONS_MAXIMUM_RESOLUTION :: 851989;
        SCALER_AVAILABLE_STALL_DURATIONS_MAXIMUM_RESOLUTION :: 851990;
        SCALER_MULTI_RESOLUTION_STREAM_SUPPORTED :: 851992;
        SCALER_AVAILABLE_STREAM_USE_CASES :: 851993;
        SCALER_END :: 851994;
        SENSOR_EXPOSURE_TIME :: 917504;
        SENSOR_FRAME_DURATION :: 917505;
        SENSOR_SENSITIVITY :: 917506;
        SENSOR_REFERENCE_ILLUMINANT1 :: 917507;
        SENSOR_REFERENCE_ILLUMINANT2 :: 917508;
        SENSOR_CALIBRATION_TRANSFORM1 :: 917509;
        SENSOR_CALIBRATION_TRANSFORM2 :: 917510;
        SENSOR_COLOR_TRANSFORM1 :: 917511;
        SENSOR_COLOR_TRANSFORM2 :: 917512;
        SENSOR_FORWARD_MATRIX1 :: 917513;
        SENSOR_FORWARD_MATRIX2 :: 917514;
        SENSOR_BLACK_LEVEL_PATTERN :: 917516;
        SENSOR_MAX_ANALOG_SENSITIVITY :: 917517;
        SENSOR_ORIENTATION :: 917518;
        SENSOR_TIMESTAMP :: 917520;
        SENSOR_NEUTRAL_COLOR_POINT :: 917522;
        SENSOR_NOISE_PROFILE :: 917523;
        SENSOR_GREEN_SPLIT :: 917526;
        SENSOR_TEST_PATTERN_DATA :: 917527;
        SENSOR_TEST_PATTERN_MODE :: 917528;
        SENSOR_AVAILABLE_TEST_PATTERN_MODES :: 917529;
        SENSOR_ROLLING_SHUTTER_SKEW :: 917530;
        SENSOR_OPTICAL_BLACK_REGIONS :: 917531;
        SENSOR_DYNAMIC_BLACK_LEVEL :: 917532;
        SENSOR_DYNAMIC_WHITE_LEVEL :: 917533;
        SENSOR_PIXEL_MODE :: 917536;
        SENSOR_RAW_BINNING_FACTOR_USED :: 917537;
        SENSOR_END :: 917538;
        SENSOR_INFO_ACTIVE_ARRAY_SIZE :: 983040;
        SENSOR_INFO_SENSITIVITY_RANGE :: 983041;
        SENSOR_INFO_COLOR_FILTER_ARRANGEMENT :: 983042;
        SENSOR_INFO_EXPOSURE_TIME_RANGE :: 983043;
        SENSOR_INFO_MAX_FRAME_DURATION :: 983044;
        SENSOR_INFO_PHYSICAL_SIZE :: 983045;
        SENSOR_INFO_PIXEL_ARRAY_SIZE :: 983046;
        SENSOR_INFO_WHITE_LEVEL :: 983047;
        SENSOR_INFO_TIMESTAMP_SOURCE :: 983048;
        SENSOR_INFO_LENS_SHADING_APPLIED :: 983049;
        SENSOR_INFO_PRE_CORRECTION_ACTIVE_ARRAY_SIZE :: 983050;
        SENSOR_INFO_ACTIVE_ARRAY_SIZE_MAXIMUM_RESOLUTION :: 983051;
        SENSOR_INFO_PIXEL_ARRAY_SIZE_MAXIMUM_RESOLUTION :: 983052;
        SENSOR_INFO_PRE_CORRECTION_ACTIVE_ARRAY_SIZE_MAXIMUM_RESOLUTION :: 983053;
        SENSOR_INFO_BINNING_FACTOR :: 983054;
        SENSOR_INFO_END :: 983055;
        SHADING_MODE :: 1048576;
        SHADING_AVAILABLE_MODES :: 1048578;
        SHADING_END :: 1048579;
        STATISTICS_FACE_DETECT_MODE :: 1114112;
        STATISTICS_HOT_PIXEL_MAP_MODE :: 1114115;
        STATISTICS_FACE_IDS :: 1114116;
        STATISTICS_FACE_LANDMARKS :: 1114117;
        STATISTICS_FACE_RECTANGLES :: 1114118;
        STATISTICS_FACE_SCORES :: 1114119;
        STATISTICS_LENS_SHADING_MAP :: 1114123;
        STATISTICS_SCENE_FLICKER :: 1114126;
        STATISTICS_HOT_PIXEL_MAP :: 1114127;
        STATISTICS_LENS_SHADING_MAP_MODE :: 1114128;
        STATISTICS_OIS_DATA_MODE :: 1114129;
        STATISTICS_OIS_TIMESTAMPS :: 1114130;
        STATISTICS_OIS_X_SHIFTS :: 1114131;
        STATISTICS_OIS_Y_SHIFTS :: 1114132;
        STATISTICS_END :: 1114133;
        STATISTICS_INFO_AVAILABLE_FACE_DETECT_MODES :: 1179648;
        STATISTICS_INFO_MAX_FACE_COUNT :: 1179650;
        STATISTICS_INFO_AVAILABLE_HOT_PIXEL_MAP_MODES :: 1179654;
        STATISTICS_INFO_AVAILABLE_LENS_SHADING_MAP_MODES :: 1179655;
        STATISTICS_INFO_AVAILABLE_OIS_DATA_MODES :: 1179656;
        STATISTICS_INFO_END :: 1179657;
        TONEMAP_CURVE_BLUE :: 1245184;
        TONEMAP_CURVE_GREEN :: 1245185;
        TONEMAP_CURVE_RED :: 1245186;
        TONEMAP_MODE :: 1245187;
        TONEMAP_MAX_CURVE_POINTS :: 1245188;
        TONEMAP_AVAILABLE_TONE_MAP_MODES :: 1245189;
        TONEMAP_GAMMA :: 1245190;
        TONEMAP_PRESET_CURVE :: 1245191;
        TONEMAP_END :: 1245192;
        INFO_SUPPORTED_HARDWARE_LEVEL :: 1376256;
        INFO_VERSION :: 1376257;
        INFO_DEVICE_STATE_ORIENTATIONS :: 1376259;
        INFO_END :: 1376260;
        BLACK_LEVEL_LOCK :: 1441792;
        BLACK_LEVEL_END :: 1441793;
        SYNC_FRAME_NUMBER :: 1507328;
        SYNC_MAX_LATENCY :: 1507329;
        SYNC_END :: 1507330;
        DEPTH_AVAILABLE_DEPTH_STREAM_CONFIGURATIONS :: 1638401;
        DEPTH_AVAILABLE_DEPTH_MIN_FRAME_DURATIONS :: 1638402;
        DEPTH_AVAILABLE_DEPTH_STALL_DURATIONS :: 1638403;
        DEPTH_DEPTH_IS_EXCLUSIVE :: 1638404;
        DEPTH_AVAILABLE_RECOMMENDED_DEPTH_STREAM_CONFIGURATIONS :: 1638405;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_STREAM_CONFIGURATIONS :: 1638406;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_MIN_FRAME_DURATIONS :: 1638407;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_STALL_DURATIONS :: 1638408;
        DEPTH_AVAILABLE_DEPTH_STREAM_CONFIGURATIONS_MAXIMUM_RESOLUTION :: 1638409;
        DEPTH_AVAILABLE_DEPTH_MIN_FRAME_DURATIONS_MAXIMUM_RESOLUTION :: 1638410;
        DEPTH_AVAILABLE_DEPTH_STALL_DURATIONS_MAXIMUM_RESOLUTION :: 1638411;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_STREAM_CONFIGURATIONS_MAXIMUM_RESOLUTION :: 1638412;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_MIN_FRAME_DURATIONS_MAXIMUM_RESOLUTION :: 1638413;
        DEPTH_AVAILABLE_DYNAMIC_DEPTH_STALL_DURATIONS_MAXIMUM_RESOLUTION :: 1638414;
        DEPTH_END :: 1638415;
        LOGICAL_MULTI_CAMERA_PHYSICAL_IDS :: 1703936;
        LOGICAL_MULTI_CAMERA_SENSOR_SYNC_TYPE :: 1703937;
        LOGICAL_MULTI_CAMERA_ACTIVE_PHYSICAL_ID :: 1703938;
        LOGICAL_MULTI_CAMERA_END :: 1703939;
        DISTORTION_CORRECTION_MODE :: 1769472;
        DISTORTION_CORRECTION_AVAILABLE_MODES :: 1769473;
        DISTORTION_CORRECTION_END :: 1769474;
        HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS :: 1835008;
        HEIC_AVAILABLE_HEIC_MIN_FRAME_DURATIONS :: 1835009;
        HEIC_AVAILABLE_HEIC_STALL_DURATIONS :: 1835010;
        HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS_MAXIMUM_RESOLUTION :: 1835011;
        HEIC_AVAILABLE_HEIC_MIN_FRAME_DURATIONS_MAXIMUM_RESOLUTION :: 1835012;
        HEIC_AVAILABLE_HEIC_STALL_DURATIONS_MAXIMUM_RESOLUTION :: 1835013;
        HEIC_END :: 1835014;
        AUTOMOTIVE_LOCATION :: 1966080;
        AUTOMOTIVE_END :: 1966081;
        AUTOMOTIVE_LENS_FACING :: 2031616;
        AUTOMOTIVE_LENS_END :: 2031617;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureFailure :: struct {
        frameNumber: s64;
        reason: s32;
        sequenceId: s32;
        wasImageCaptured: bool;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallbacks :: struct {
        _context: *void;
        onCaptureStarted: ACameraCaptureSession_captureCallback_start;
        onCaptureProgressed: ACameraCaptureSession_captureCallback_result;
        onCaptureCompleted: ACameraCaptureSession_captureCallback_result;
        onCaptureFailed: ACameraCaptureSession_captureCallback_failed;
        onCaptureSequenceCompleted: ACameraCaptureSession_captureCallback_sequenceEnd;
        onCaptureSequenceAborted: ACameraCaptureSession_captureCallback_sequenceAbort;
        onCaptureBufferLost: ACameraCaptureSession_captureCallback_bufferLost;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallbacksV2 :: struct {
        _context: *void;
        onCaptureStarted: ACameraCaptureSession_captureCallback_startV2;
        onCaptureProgressed: ACameraCaptureSession_captureCallback_result;
        onCaptureCompleted: ACameraCaptureSession_captureCallback_result;
        onCaptureFailed: ACameraCaptureSession_captureCallback_failed;
        onCaptureSequenceCompleted: ACameraCaptureSession_captureCallback_sequenceEnd;
        onCaptureSequenceAborted: ACameraCaptureSession_captureCallback_sequenceAbort;
        onCaptureBufferLost: ACameraCaptureSession_captureCallback_bufferLost;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_captureCallbacks :: struct {
        _context: *void;
        onCaptureStarted: ACameraCaptureSession_captureCallback_start;
        onCaptureProgressed: ACameraCaptureSession_captureCallback_result;
        onLogicalCameraCaptureCompleted: ACameraCaptureSession_logicalCamera_captureCallback_result;
        onLogicalCameraCaptureFailed: ACameraCaptureSession_logicalCamera_captureCallback_failed;
        onCaptureSequenceCompleted: ACameraCaptureSession_captureCallback_sequenceEnd;
        onCaptureSequenceAborted: ACameraCaptureSession_captureCallback_sequenceAbort;
        onCaptureBufferLost: ACameraCaptureSession_captureCallback_bufferLost;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_captureCallbacksV2 :: struct {
        _context: *void;
        onCaptureStarted: ACameraCaptureSession_captureCallback_startV2;
        onCaptureProgressed: ACameraCaptureSession_captureCallback_result;
        onLogicalCameraCaptureCompleted: ACameraCaptureSession_logicalCamera_captureCallback_result;
        onLogicalCameraCaptureFailed: ACameraCaptureSession_logicalCamera_captureCallback_failed;
        onCaptureSequenceCompleted: ACameraCaptureSession_captureCallback_sequenceEnd;
        onCaptureSequenceAborted: ACameraCaptureSession_captureCallback_sequenceAbort;
        onCaptureBufferLost: ACameraCaptureSession_captureCallback_bufferLost;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_stateCallbacks :: struct {
        _context: *void;
        onClosed: ACameraCaptureSession_stateCallback;
        onReady: ACameraCaptureSession_stateCallback;
        onActive: ACameraCaptureSession_stateCallback;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_request_template :: enum u32 {
        PREVIEW :: 1;
        STILL_CAPTURE :: 2;
        RECORD :: 3;
        VIDEO_SNAPSHOT :: 4;
        ZERO_SHUTTER_LAG :: 5;
        MANUAL :: 6;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_StateCallbacks :: struct {
        _context: *void;
        onDisconnected: ACameraDevice_StateCallback;
        onError: ACameraDevice_ErrorStateCallback;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraIdList :: struct {
        numCameras: s32;
        cameraIds: **u8;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_AvailabilityListener :: struct {
        _context: *void;
        onCameraAvailable: ACameraManager_AvailabilityCallback;
        onCameraUnavailable: ACameraManager_AvailabilityCallback;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_ExtendedAvailabilityListener :: struct {
        availabilityCallbacks: ACameraManager_AvailabilityCallbacks;
        onCameraAccessPrioritiesChanged: ACameraManager_AccessPrioritiesChangedCallback;
        onPhysicalCameraAvailable: ACameraManager_PhysicalCameraAvailabilityCallback;
        onPhysicalCameraUnavailable: ACameraManager_PhysicalCameraAvailabilityCallback;
        reserved: [4] *void;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_const_entry :: struct {
        tag: u32;
        type: u8;
        count: u32;
        data: struct {
            _u8: *u8;
            #overlay (_u8) i32: *s32;
            #overlay (_u8) f: *float;
            #overlay (_u8) i64: *s64;
            #overlay (_u8) d: *float64;
            #overlay (_u8) r: *ACameraMetadata_rational;
        };
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_rational :: struct {
        numerator: s32;
        denominator: s32;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraOutputTarget :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraOutputTargets :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutput :: struct {}

}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutputContainer :: struct {}

}

AConfiguration :: struct {}

ACONFIGURATION :: enum u32 {
    ORIENTATION_ANY :: 0;
    ORIENTATION_PORT :: 1;
    ORIENTATION_LAND :: 2;
    ORIENTATION_SQUARE :: 3;
    TOUCHSCREEN_ANY :: 0;
    TOUCHSCREEN_NOTOUCH :: 1;
    TOUCHSCREEN_STYLUS :: 2;
    TOUCHSCREEN_FINGER :: 3;
    DENSITY_DEFAULT :: 0;
    DENSITY_LOW :: 120;
    DENSITY_MEDIUM :: 160;
    DENSITY_TV :: 213;
    DENSITY_HIGH :: 240;
    DENSITY_XHIGH :: 320;
    DENSITY_XXHIGH :: 480;
    DENSITY_XXXHIGH :: 640;
    DENSITY_ANY :: 65534;
    DENSITY_NONE :: 65535;
    KEYBOARD_ANY :: 0;
    KEYBOARD_NOKEYS :: 1;
    KEYBOARD_QWERTY :: 2;
    KEYBOARD_12KEY :: 3;
    NAVIGATION_ANY :: 0;
    NAVIGATION_NONAV :: 1;
    NAVIGATION_DPAD :: 2;
    NAVIGATION_TRACKBALL :: 3;
    NAVIGATION_WHEEL :: 4;
    KEYSHIDDEN_ANY :: 0;
    KEYSHIDDEN_NO :: 1;
    KEYSHIDDEN_YES :: 2;
    KEYSHIDDEN_SOFT :: 3;
    NAVHIDDEN_ANY :: 0;
    NAVHIDDEN_NO :: 1;
    NAVHIDDEN_YES :: 2;
    SCREENSIZE_ANY :: 0;
    SCREENSIZE_SMALL :: 1;
    SCREENSIZE_NORMAL :: 2;
    SCREENSIZE_LARGE :: 3;
    SCREENSIZE_XLARGE :: 4;
    SCREENLONG_ANY :: 0;
    SCREENLONG_NO :: 1;
    SCREENLONG_YES :: 2;
    SCREENROUND_ANY :: 0;
    SCREENROUND_NO :: 1;
    SCREENROUND_YES :: 2;
    WIDE_COLOR_GAMUT_ANY :: 0;
    WIDE_COLOR_GAMUT_NO :: 1;
    WIDE_COLOR_GAMUT_YES :: 2;
    HDR_ANY :: 0;
    HDR_NO :: 1;
    HDR_YES :: 2;
    UI_MODE_TYPE_ANY :: 0;
    UI_MODE_TYPE_NORMAL :: 1;
    UI_MODE_TYPE_DESK :: 2;
    UI_MODE_TYPE_CAR :: 3;
    UI_MODE_TYPE_TELEVISION :: 4;
    UI_MODE_TYPE_APPLIANCE :: 5;
    UI_MODE_TYPE_WATCH :: 6;
    UI_MODE_TYPE_VR_HEADSET :: 7;
    UI_MODE_NIGHT_ANY :: 0;
    UI_MODE_NIGHT_NO :: 1;
    UI_MODE_NIGHT_YES :: 2;
    SCREEN_WIDTH_DP_ANY :: 0;
    SCREEN_HEIGHT_DP_ANY :: 0;
    SMALLEST_SCREEN_WIDTH_DP_ANY :: 0;
    LAYOUTDIR_ANY :: 0;
    LAYOUTDIR_LTR :: 1;
    LAYOUTDIR_RTL :: 2;
    MCC :: 1;
    MNC :: 2;
    LOCALE :: 4;
    TOUCHSCREEN :: 8;
    KEYBOARD :: 16;
    KEYBOARD_HIDDEN :: 32;
    NAVIGATION :: 64;
    ORIENTATION :: 128;
    DENSITY :: 256;
    SCREEN_SIZE :: 512;
    VERSION :: 1024;
    SCREEN_LAYOUT :: 2048;
    UI_MODE :: 4096;
    SMALLEST_SCREEN_SIZE :: 8192;
    LAYOUTDIR :: 16384;
    SCREEN_ROUND :: 32768;
    COLOR_MODE :: 65536;
    MNC_ZERO :: 65535;
}

AHardwareBuffer :: struct {}

AHardwareBuffer_Desc :: struct {
    width: u32;
    height: u32;
    layers: u32;
    format: u32;
    usage: u64;
    stride: u32;
    rfu0: u32;
    rfu1: u64;
}

AHardwareBuffer_Format :: enum u32 {
    R8G8B8A8_UNORM :: 1;
    R8G8B8X8_UNORM :: 2;
    R8G8B8_UNORM :: 3;
    R5G6B5_UNORM :: 4;
    R16G16B16A16_FLOAT :: 22;
    R10G10B10A2_UNORM :: 43;
    BLOB :: 33;
    D16_UNORM :: 48;
    D24_UNORM :: 49;
    D24_UNORM_S8_UINT :: 50;
    D32_FLOAT :: 51;
    D32_FLOAT_S8_UINT :: 52;
    S8_UINT :: 53;
    Y8Cb8Cr8_420 :: 35;
    YCbCr_P010 :: 54;
    R8_UNORM :: 56;
    R16_UINT :: 57;
    R16G16_UINT :: 58;
    R10G10B10A10_UNORM :: 59;
}

AHardwareBuffer_Plane :: struct {
    data: *void;
    pixelStride: u32;
    rowStride: u32;
}

AHardwareBuffer_Planes :: struct {
    planeCount: u32;
    planes: [4] AHardwareBuffer_Plane;
}

AHardwareBuffer_UsageFlags :: enum u64 {
    CPU_READ_NEVER :: 0;
    CPU_READ_RARELY :: 2;
    CPU_READ_OFTEN :: 3;
    CPU_READ_MASK :: 15;
    CPU_WRITE_NEVER :: 0;
    CPU_WRITE_RARELY :: 32;
    CPU_WRITE_OFTEN :: 48;
    CPU_WRITE_MASK :: 240;
    GPU_SAMPLED_IMAGE :: 256;
    GPU_FRAMEBUFFER :: 512;
    GPU_COLOR_OUTPUT :: 512;
    COMPOSER_OVERLAY :: 2048;
    PROTECTED_CONTENT :: 16384;
    VIDEO_ENCODE :: 65536;
    SENSOR_DIRECT_DATA :: 8388608;
    GPU_DATA_BUFFER :: 16777216;
    GPU_CUBE_MAP :: 33554432;
    GPU_MIPMAP_COMPLETE :: 67108864;
    FRONT_BUFFER :: 4294967296;
    VENDOR_0 :: 268435456;
    VENDOR_1 :: 536870912;
    VENDOR_2 :: 1073741824;
    VENDOR_3 :: 2147483648;
    VENDOR_4 :: 281474976710656;
    VENDOR_5 :: 562949953421312;
    VENDOR_6 :: 1125899906842624;
    VENDOR_7 :: 2251799813685248;
    VENDOR_8 :: 4503599627370496;
    VENDOR_9 :: 9007199254740992;
    VENDOR_10 :: 18014398509481984;
    VENDOR_11 :: 36028797018963968;
    VENDOR_12 :: 72057594037927936;
    VENDOR_13 :: 144115188075855872;
    VENDOR_14 :: 288230376151711744;
    VENDOR_15 :: 576460752303423488;
    VENDOR_16 :: 1152921504606846976;
    VENDOR_17 :: 2305843009213693952;
    VENDOR_18 :: 4611686018427387904;
    VENDOR_19 :: 9223372036854775808;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AIMAGE_FORMAT :: enum u32 {
        RGBA_8888 :: 1;
        RGBX_8888 :: 2;
        RGB_888 :: 3;
        RGB_565 :: 4;
        RGBA_FP16 :: 22;
        YUV_420_888 :: 35;
        JPEG :: 256;
        RAW16 :: 32;
        RAW_PRIVATE :: 36;
        RAW10 :: 37;
        RAW12 :: 38;
        DEPTH16 :: 1144402265;
        DEPTH_POINT_CLOUD :: 257;
        PRIVATE :: 34;
        Y8 :: 538982489;
        HEIC :: 1212500294;
        DEPTH_JPEG :: 1768253795;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AImageCropRect :: struct {
        left: s32;
        top: s32;
        right: s32;
        bottom: s32;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_BufferRemovedListener :: struct {
        _context: *void;
        onBufferRemoved: AImageReader_BufferRemovedCallback;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_ImageListener :: struct {
        _context: *void;
        onImageAvailable: AImageReader_ImageCallback;
    }

}

AINPUT_EVENT_TYPE :: enum u32 {
    KEY :: 1;
    MOTION :: 2;
    FOCUS :: 3;
    CAPTURE :: 4;
    DRAG :: 5;
    TOUCH_MODE :: 6;
}

AINPUT_KEYBOARD_TYPE :: enum u32 {
    NONE :: 0;
    NON_ALPHABETIC :: 1;
    ALPHABETIC :: 2;
}

AINPUT_MOTION_RANGE :: enum u32 {
    X :: 0;
    Y :: 1;
    PRESSURE :: 2;
    SIZE :: 3;
    TOUCH_MAJOR :: 4;
    TOUCH_MINOR :: 5;
    TOOL_MAJOR :: 6;
    TOOL_MINOR :: 7;
    ORIENTATION :: 8;
}

AINPUT_SOURCE :: enum u32 {
    UNKNOWN :: 0;
    KEYBOARD :: 257;
    DPAD :: 513;
    GAMEPAD :: 1025;
    TOUCHSCREEN :: 4098;
    MOUSE :: 8194;
    STYLUS :: 16386;
    BLUETOOTH_STYLUS :: 49154;
    TRACKBALL :: 65540;
    MOUSE_RELATIVE :: 131076;
    TOUCHPAD :: 1048584;
    TOUCH_NAVIGATION :: 2097152;
    JOYSTICK :: 16777232;
    HDMI :: 33554433;
    SENSOR :: 67108864;
    ROTARY_ENCODER :: 4194304;
    ANY :: 4294967040;
}

AINPUT_SOURCE_CLASS :: enum u32 {
    MASK :: 255;
    NONE :: 0;
    BUTTON :: 1;
    POINTER :: 2;
    NAVIGATION :: 4;
    POSITION :: 8;
    JOYSTICK :: 16;
}

AInputEvent :: struct {}

AInputQueue :: struct {}

AKEY_EVENT_ACTION :: enum u32 {
    DOWN :: 0;
    UP :: 1;
    MULTIPLE :: 2;
}

AKEY_EVENT_FLAG :: enum_flags u32 {
    WOKE_HERE :: 0x1;
    SOFT_KEYBOARD :: 0x2;
    KEEP_TOUCH_MODE :: 0x4;
    FROM_SYSTEM :: 0x8;
    EDITOR_ACTION :: 0x10;
    CANCELED :: 0x20;
    VIRTUAL_HARD_KEY :: 0x40;
    LONG_PRESS :: 0x80;
    CANCELED_LONG_PRESS :: 0x100;
    TRACKING :: 0x200;
    FALLBACK :: 0x400;
}

AKEY_STATE :: enum s32 {
    UNKNOWN :: -1;
    UP :: 0;
    DOWN :: 1;
    VIRTUAL :: 2;
}

AKEYCODE :: enum u32 {
    UNKNOWN :: 0;
    SOFT_LEFT :: 1;
    SOFT_RIGHT :: 2;
    HOME :: 3;
    BACK :: 4;
    CALL :: 5;
    ENDCALL :: 6;
    _0 :: 7;
    _1 :: 8;
    _2 :: 9;
    _3 :: 10;
    _4 :: 11;
    _5 :: 12;
    _6 :: 13;
    _7 :: 14;
    _8 :: 15;
    _9 :: 16;
    STAR :: 17;
    POUND :: 18;
    DPAD_UP :: 19;
    DPAD_DOWN :: 20;
    DPAD_LEFT :: 21;
    DPAD_RIGHT :: 22;
    DPAD_CENTER :: 23;
    VOLUME_UP :: 24;
    VOLUME_DOWN :: 25;
    POWER :: 26;
    CAMERA :: 27;
    CLEAR :: 28;
    A :: 29;
    B :: 30;
    C :: 31;
    D :: 32;
    E :: 33;
    F :: 34;
    G :: 35;
    H :: 36;
    I :: 37;
    J :: 38;
    K :: 39;
    L :: 40;
    M :: 41;
    N :: 42;
    O :: 43;
    P :: 44;
    Q :: 45;
    R :: 46;
    S :: 47;
    T :: 48;
    U :: 49;
    V :: 50;
    W :: 51;
    X :: 52;
    Y :: 53;
    Z :: 54;
    COMMA :: 55;
    PERIOD :: 56;
    ALT_LEFT :: 57;
    ALT_RIGHT :: 58;
    SHIFT_LEFT :: 59;
    SHIFT_RIGHT :: 60;
    TAB :: 61;
    SPACE :: 62;
    SYM :: 63;
    EXPLORER :: 64;
    ENVELOPE :: 65;
    ENTER :: 66;
    DEL :: 67;
    GRAVE :: 68;
    MINUS :: 69;
    EQUALS :: 70;
    LEFT_BRACKET :: 71;
    RIGHT_BRACKET :: 72;
    BACKSLASH :: 73;
    SEMICOLON :: 74;
    APOSTROPHE :: 75;
    SLASH :: 76;
    AT :: 77;
    NUM :: 78;
    HEADSETHOOK :: 79;
    FOCUS :: 80;
    PLUS :: 81;
    MENU :: 82;
    NOTIFICATION :: 83;
    SEARCH :: 84;
    MEDIA_PLAY_PAUSE :: 85;
    MEDIA_STOP :: 86;
    MEDIA_NEXT :: 87;
    MEDIA_PREVIOUS :: 88;
    MEDIA_REWIND :: 89;
    MEDIA_FAST_FORWARD :: 90;
    MUTE :: 91;
    PAGE_UP :: 92;
    PAGE_DOWN :: 93;
    PICTSYMBOLS :: 94;
    SWITCH_CHARSET :: 95;
    BUTTON_A :: 96;
    BUTTON_B :: 97;
    BUTTON_C :: 98;
    BUTTON_X :: 99;
    BUTTON_Y :: 100;
    BUTTON_Z :: 101;
    BUTTON_L1 :: 102;
    BUTTON_R1 :: 103;
    BUTTON_L2 :: 104;
    BUTTON_R2 :: 105;
    BUTTON_THUMBL :: 106;
    BUTTON_THUMBR :: 107;
    BUTTON_START :: 108;
    BUTTON_SELECT :: 109;
    BUTTON_MODE :: 110;
    ESCAPE :: 111;
    FORWARD_DEL :: 112;
    CTRL_LEFT :: 113;
    CTRL_RIGHT :: 114;
    CAPS_LOCK :: 115;
    SCROLL_LOCK :: 116;
    META_LEFT :: 117;
    META_RIGHT :: 118;
    FUNCTION :: 119;
    SYSRQ :: 120;
    BREAK :: 121;
    MOVE_HOME :: 122;
    MOVE_END :: 123;
    INSERT :: 124;
    FORWARD :: 125;
    MEDIA_PLAY :: 126;
    MEDIA_PAUSE :: 127;
    MEDIA_CLOSE :: 128;
    MEDIA_EJECT :: 129;
    MEDIA_RECORD :: 130;
    F1 :: 131;
    F2 :: 132;
    F3 :: 133;
    F4 :: 134;
    F5 :: 135;
    F6 :: 136;
    F7 :: 137;
    F8 :: 138;
    F9 :: 139;
    F10 :: 140;
    F11 :: 141;
    F12 :: 142;
    NUM_LOCK :: 143;
    NUMPAD_0 :: 144;
    NUMPAD_1 :: 145;
    NUMPAD_2 :: 146;
    NUMPAD_3 :: 147;
    NUMPAD_4 :: 148;
    NUMPAD_5 :: 149;
    NUMPAD_6 :: 150;
    NUMPAD_7 :: 151;
    NUMPAD_8 :: 152;
    NUMPAD_9 :: 153;
    NUMPAD_DIVIDE :: 154;
    NUMPAD_MULTIPLY :: 155;
    NUMPAD_SUBTRACT :: 156;
    NUMPAD_ADD :: 157;
    NUMPAD_DOT :: 158;
    NUMPAD_COMMA :: 159;
    NUMPAD_ENTER :: 160;
    NUMPAD_EQUALS :: 161;
    NUMPAD_LEFT_PAREN :: 162;
    NUMPAD_RIGHT_PAREN :: 163;
    VOLUME_MUTE :: 164;
    INFO :: 165;
    CHANNEL_UP :: 166;
    CHANNEL_DOWN :: 167;
    ZOOM_IN :: 168;
    ZOOM_OUT :: 169;
    TV :: 170;
    WINDOW :: 171;
    GUIDE :: 172;
    DVR :: 173;
    BOOKMARK :: 174;
    CAPTIONS :: 175;
    SETTINGS :: 176;
    TV_POWER :: 177;
    TV_INPUT :: 178;
    STB_POWER :: 179;
    STB_INPUT :: 180;
    AVR_POWER :: 181;
    AVR_INPUT :: 182;
    PROG_RED :: 183;
    PROG_GREEN :: 184;
    PROG_YELLOW :: 185;
    PROG_BLUE :: 186;
    APP_SWITCH :: 187;
    BUTTON_1 :: 188;
    BUTTON_2 :: 189;
    BUTTON_3 :: 190;
    BUTTON_4 :: 191;
    BUTTON_5 :: 192;
    BUTTON_6 :: 193;
    BUTTON_7 :: 194;
    BUTTON_8 :: 195;
    BUTTON_9 :: 196;
    BUTTON_10 :: 197;
    BUTTON_11 :: 198;
    BUTTON_12 :: 199;
    BUTTON_13 :: 200;
    BUTTON_14 :: 201;
    BUTTON_15 :: 202;
    BUTTON_16 :: 203;
    LANGUAGE_SWITCH :: 204;
    MANNER_MODE :: 205;
    _3D_MODE :: 206;
    CONTACTS :: 207;
    CALENDAR :: 208;
    MUSIC :: 209;
    CALCULATOR :: 210;
    ZENKAKU_HANKAKU :: 211;
    EISU :: 212;
    MUHENKAN :: 213;
    HENKAN :: 214;
    KATAKANA_HIRAGANA :: 215;
    YEN :: 216;
    RO :: 217;
    KANA :: 218;
    ASSIST :: 219;
    BRIGHTNESS_DOWN :: 220;
    BRIGHTNESS_UP :: 221;
    MEDIA_AUDIO_TRACK :: 222;
    SLEEP :: 223;
    WAKEUP :: 224;
    PAIRING :: 225;
    MEDIA_TOP_MENU :: 226;
    _11 :: 227;
    _12 :: 228;
    LAST_CHANNEL :: 229;
    TV_DATA_SERVICE :: 230;
    VOICE_ASSIST :: 231;
    TV_RADIO_SERVICE :: 232;
    TV_TELETEXT :: 233;
    TV_NUMBER_ENTRY :: 234;
    TV_TERRESTRIAL_ANALOG :: 235;
    TV_TERRESTRIAL_DIGITAL :: 236;
    TV_SATELLITE :: 237;
    TV_SATELLITE_BS :: 238;
    TV_SATELLITE_CS :: 239;
    TV_SATELLITE_SERVICE :: 240;
    TV_NETWORK :: 241;
    TV_ANTENNA_CABLE :: 242;
    TV_INPUT_HDMI_1 :: 243;
    TV_INPUT_HDMI_2 :: 244;
    TV_INPUT_HDMI_3 :: 245;
    TV_INPUT_HDMI_4 :: 246;
    TV_INPUT_COMPOSITE_1 :: 247;
    TV_INPUT_COMPOSITE_2 :: 248;
    TV_INPUT_COMPONENT_1 :: 249;
    TV_INPUT_COMPONENT_2 :: 250;
    TV_INPUT_VGA_1 :: 251;
    TV_AUDIO_DESCRIPTION :: 252;
    TV_AUDIO_DESCRIPTION_MIX_UP :: 253;
    TV_AUDIO_DESCRIPTION_MIX_DOWN :: 254;
    TV_ZOOM_MODE :: 255;
    TV_CONTENTS_MENU :: 256;
    TV_MEDIA_CONTEXT_MENU :: 257;
    TV_TIMER_PROGRAMMING :: 258;
    HELP :: 259;
    NAVIGATE_PREVIOUS :: 260;
    NAVIGATE_NEXT :: 261;
    NAVIGATE_IN :: 262;
    NAVIGATE_OUT :: 263;
    STEM_PRIMARY :: 264;
    STEM_1 :: 265;
    STEM_2 :: 266;
    STEM_3 :: 267;
    DPAD_UP_LEFT :: 268;
    DPAD_DOWN_LEFT :: 269;
    DPAD_UP_RIGHT :: 270;
    DPAD_DOWN_RIGHT :: 271;
    MEDIA_SKIP_FORWARD :: 272;
    MEDIA_SKIP_BACKWARD :: 273;
    MEDIA_STEP_FORWARD :: 274;
    MEDIA_STEP_BACKWARD :: 275;
    SOFT_SLEEP :: 276;
    CUT :: 277;
    COPY :: 278;
    PASTE :: 279;
    SYSTEM_NAVIGATION_UP :: 280;
    SYSTEM_NAVIGATION_DOWN :: 281;
    SYSTEM_NAVIGATION_LEFT :: 282;
    SYSTEM_NAVIGATION_RIGHT :: 283;
    ALL_APPS :: 284;
    REFRESH :: 285;
    THUMBS_UP :: 286;
    THUMBS_DOWN :: 287;
    PROFILE_SWITCH :: 288;
    VIDEO_APP_1 :: 289;
    VIDEO_APP_2 :: 290;
    VIDEO_APP_3 :: 291;
    VIDEO_APP_4 :: 292;
    VIDEO_APP_5 :: 293;
    VIDEO_APP_6 :: 294;
    VIDEO_APP_7 :: 295;
    VIDEO_APP_8 :: 296;
    FEATURED_APP_1 :: 297;
    FEATURED_APP_2 :: 298;
    FEATURED_APP_3 :: 299;
    FEATURED_APP_4 :: 300;
    DEMO_APP_1 :: 301;
    DEMO_APP_2 :: 302;
    DEMO_APP_3 :: 303;
    DEMO_APP_4 :: 304;
}

#if INCLUDE_CAMERA_BINDINGS {
    ALogicalCameraCaptureFailure :: struct {
        captureFailure: ACameraCaptureFailure;
        physicalCameraId: *u8;
    }

}

ALooper :: struct {}

ALOOPER_EVENT :: enum u32 {
    INPUT :: 1;
    OUTPUT :: 2;
    ERROR :: 4;
    HANGUP :: 8;
    INVALID :: 16;
}

ALOOPER_POLL :: enum s32 {
    WAKE :: -1;
    CALLBACK :: -2;
    TIMEOUT :: -3;
    ERROR :: -4;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC :: enum s32 {
        BUFFER_FLAG_CODEC_CONFIG :: 2;
        BUFFER_FLAG_END_OF_STREAM :: 4;
        BUFFER_FLAG_PARTIAL_FRAME :: 8;
        CONFIGURE_FLAG_ENCODE :: 1;
        INFO_OUTPUT_BUFFERS_CHANGED :: -3;
        INFO_OUTPUT_FORMAT_CHANGED :: -2;
        INFO_TRY_AGAIN_LATER :: -1;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecBufferInfo :: struct {
        offset: s32;
        size: s32;
        presentationTimeUs: s64;
        flags: u32;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnAsyncNotifyCallback :: struct {
        onAsyncInputAvailable: AMediaCodecOnAsyncInputAvailable;
        onAsyncOutputAvailable: AMediaCodecOnAsyncOutputAvailable;
        onAsyncFormatChanged: AMediaCodecOnAsyncFormatChanged;
        onAsyncError: AMediaCodecOnAsyncError;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCrypto :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmByteArray :: struct {
        ptr: *u8;
        length: u64;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmEventType :: enum u32 {
        PROVISION_REQUIRED :: 1;
        KEY_REQUIRED :: 2;
        KEY_EXPIRED :: 3;
        VENDOR_DEFINED :: 4;
        SESSION_RECLAIMED :: 5;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyRequestType :: enum s32 {
        INITIAL :: 0;
        RENEWAL :: 1;
        RELEASE :: 2;
        NONE :: 3;
        UPDATE :: 4;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyStatus :: struct {
        keyId: AMediaDrmKeyId;
        keyType: AMediaDrmKeyStatusType;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyType :: enum u32 {
        STREAMING :: 1;
        OFFLINE :: 2;
        RELEASE :: 3;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyValuePair :: struct {
        mKey: *u8;
        mValue: *u8;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAEXTRACTOR_SAMPLE_FLAG :: enum u32 {
        SYNC :: 1;
        ENCRYPTED :: 2;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat :: struct {}

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaKeyStatusType :: enum u32 {
        USABLE :: 0;
        EXPIRED :: 1;
        OUTPUTNOTALLOWED :: 2;
        STATUSPENDING :: 3;
        INTERNALERROR :: 4;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer :: struct {}

}

AMETA :: enum_flags u32 {
    NONE :: 0x0;
    ALT_ON :: 0x2;
    ALT_LEFT_ON :: 0x10;
    ALT_RIGHT_ON :: 0x20;
    SHIFT_ON :: 0x1;
    SHIFT_LEFT_ON :: 0x40;
    SHIFT_RIGHT_ON :: 0x80;
    SYM_ON :: 0x4;
    FUNCTION_ON :: 0x8;
    CTRL_ON :: 0x1000;
    CTRL_LEFT_ON :: 0x2000;
    CTRL_RIGHT_ON :: 0x4000;
    META_ON :: 0x10000;
    META_LEFT_ON :: 0x20000;
    META_RIGHT_ON :: 0x40000;
    CAPS_LOCK_ON :: 0x100000;
    NUM_LOCK_ON :: 0x200000;
    SCROLL_LOCK_ON :: 0x400000;
}

AMOTION_EVENT_ACTION :: enum u32 {
    MASK :: 255;
    POINTER_INDEX_MASK :: 65280;
    DOWN :: 0;
    UP :: 1;
    MOVE :: 2;
    CANCEL :: 3;
    OUTSIDE :: 4;
    POINTER_DOWN :: 5;
    POINTER_UP :: 6;
    HOVER_MOVE :: 7;
    SCROLL :: 8;
    HOVER_ENTER :: 9;
    HOVER_EXIT :: 10;
    BUTTON_PRESS :: 11;
    BUTTON_RELEASE :: 12;
}

AMOTION_EVENT_AXIS :: enum u32 {
    X :: 0;
    Y :: 1;
    PRESSURE :: 2;
    SIZE :: 3;
    TOUCH_MAJOR :: 4;
    TOUCH_MINOR :: 5;
    TOOL_MAJOR :: 6;
    TOOL_MINOR :: 7;
    ORIENTATION :: 8;
    VSCROLL :: 9;
    HSCROLL :: 10;
    Z :: 11;
    RX :: 12;
    RY :: 13;
    RZ :: 14;
    HAT_X :: 15;
    HAT_Y :: 16;
    LTRIGGER :: 17;
    RTRIGGER :: 18;
    THROTTLE :: 19;
    RUDDER :: 20;
    WHEEL :: 21;
    GAS :: 22;
    BRAKE :: 23;
    DISTANCE :: 24;
    TILT :: 25;
    SCROLL :: 26;
    RELATIVE_X :: 27;
    RELATIVE_Y :: 28;
    GENERIC_1 :: 32;
    GENERIC_2 :: 33;
    GENERIC_3 :: 34;
    GENERIC_4 :: 35;
    GENERIC_5 :: 36;
    GENERIC_6 :: 37;
    GENERIC_7 :: 38;
    GENERIC_8 :: 39;
    GENERIC_9 :: 40;
    GENERIC_10 :: 41;
    GENERIC_11 :: 42;
    GENERIC_12 :: 43;
    GENERIC_13 :: 44;
    GENERIC_14 :: 45;
    GENERIC_15 :: 46;
    GENERIC_16 :: 47;
}

AMOTION_EVENT_BUTTON :: enum u32 {
    PRIMARY :: 1;
    SECONDARY :: 2;
    TERTIARY :: 4;
    BACK :: 8;
    FORWARD :: 16;
    STYLUS_PRIMARY :: 32;
    STYLUS_SECONDARY :: 64;
}

AMOTION_EVENT_EDGE_FLAG :: enum_flags u32 {
    NONE :: 0x0;
    TOP :: 0x1;
    BOTTOM :: 0x2;
    LEFT :: 0x4;
    RIGHT :: 0x8;
}

AMOTION_EVENT_TOOL_TYPE :: enum u32 {
    UNKNOWN :: 0;
    FINGER :: 1;
    STYLUS :: 2;
    MOUSE :: 3;
    ERASER :: 4;
    PALM :: 5;
}

AMotionClassification :: enum u32 {
    NONE :: 0;
    AMBIGUOUS_GESTURE :: 1;
    DEEP_PRESS :: 2;
}

ANativeActivity :: struct {
    callbacks: *ANativeActivityCallbacks;
    vm: *JavaVM;
    env: *JNIEnv;
    clazz: jobject;
    internalDataPath: *u8;
    externalDataPath: *u8;
    sdkVersion: s32;
    instance: *void;
    assetManager: *AAssetManager;
    obbPath: *u8;
}

ANATIVEACTIVITY_HIDE_SOFT_INPUT :: enum u32 {
    IMPLICIT_ONLY :: 1;
    NOT_ALWAYS :: 2;
}

ANATIVEACTIVITY_SHOW_SOFT_INPUT :: enum u32 {
    IMPLICIT :: 1;
    FORCED :: 2;
}

ANativeActivityCallbacks :: struct {
    onStart: #type (activity: *ANativeActivity) -> void #c_call;
    onResume: #type (activity: *ANativeActivity) -> void #c_call;
    onSaveInstanceState: #type (activity: *ANativeActivity, outSize: *u64) -> *void #c_call;
    onPause: #type (activity: *ANativeActivity) -> void #c_call;
    onStop: #type (activity: *ANativeActivity) -> void #c_call;
    onDestroy: #type (activity: *ANativeActivity) -> void #c_call;
    onWindowFocusChanged: #type (activity: *ANativeActivity, hasFocus: s32) -> void #c_call;
    onNativeWindowCreated: #type (activity: *ANativeActivity, window: *ANativeWindow) -> void #c_call;
    onNativeWindowResized: #type (activity: *ANativeActivity, window: *ANativeWindow) -> void #c_call;
    onNativeWindowRedrawNeeded: #type (activity: *ANativeActivity, window: *ANativeWindow) -> void #c_call;
    onNativeWindowDestroyed: #type (activity: *ANativeActivity, window: *ANativeWindow) -> void #c_call;
    onInputQueueCreated: #type (activity: *ANativeActivity, queue: *AInputQueue) -> void #c_call;
    onInputQueueDestroyed: #type (activity: *ANativeActivity, queue: *AInputQueue) -> void #c_call;
    onContentRectChanged: #type (activity: *ANativeActivity, rect: *ARect) -> void #c_call;
    onConfigurationChanged: #type (activity: *ANativeActivity) -> void #c_call;
    onLowMemory: #type (activity: *ANativeActivity) -> void #c_call;
}

ANativeWindow :: struct {}

ANativeWindow_Buffer :: struct {
    width: s32;
    height: s32;
    stride: s32;
    format: s32;
    bits: *void;
    reserved: [6] u32;
}

ANativeWindow_ChangeFrameRateStrategy :: enum u32 {
    ONLY_IF_SEAMLESS :: 0;
    ALWAYS :: 1;
}

ANativeWindow_FrameRateCompatibility :: enum u32 {
    DEFAULT :: 0;
    FIXED_SOURCE :: 1;
}

ANativeWindow_LegacyFormat :: enum u32 {
    RGBA_8888 :: 1;
    RGBX_8888 :: 2;
    RGB_565 :: 4;
}

ANativeWindowTransform :: enum u32 {
    IDENTITY :: 0;
    MIRROR_HORIZONTAL :: 1;
    MIRROR_VERTICAL :: 2;
    ROTATE_90 :: 4;
    ROTATE_180 :: 3;
    ROTATE_270 :: 7;
}

#if USE_GLUE_NATIVE_PORT {
    Android_App :: struct {
        userData: *void;
        onAppCmd: #type (app: *Android_App, cmd: s32) -> void #c_call;
        onInputEvent: #type (app: *Android_App, event: *AInputEvent) -> s32 #c_call;
        activity: *ANativeActivity;
        config: *AConfiguration;
        savedState: *void;
        savedStateSize: u64;
        looper: *ALooper;
        inputQueue: *AInputQueue;
        window: *ANativeWindow;
        contentRect: ARect;
        activityState: s32;
        destroyRequested: s32;
        mutex: pthread_mutex_t;
        cond: pthread_cond_t;
        msgread: s32;
        msgwrite: s32;
        thread: pthread_t;
        cmdPollSource: Android_Poll_Source;
        inputPollSource: Android_Poll_Source;
        running: s32;
        stateSaved: s32;
        destroyed: s32;
        redrawNeeded: s32;
        pendingInputQueue: *AInputQueue;
        pendingWindow: *ANativeWindow;
        pendingContentRect: ARect;
    }
}

#if !USE_GLUE_NATIVE_PORT {
    Android_App :: struct {
        userData: *void;
        onAppCmd: #type (app: *Android_App, cmd: s32) -> void #c_call;
        onInputEvent: #type (app: *Android_App, event: *AInputEvent) -> s32 #c_call;
        activity: *ANativeActivity;
        config: *AConfiguration;
        savedState: *void;
        savedStateSize: u64;
        looper: *ALooper;
        inputQueue: *AInputQueue;
        window: *ANativeWindow;
        contentRect: ARect;
        activityState: s32;
        destroyRequested: s32;
    }
}

Android_Poll_Source :: struct {
    id: s32;
    app: *Android_App;
    process: #type (app: *Android_App, source: *Android_Poll_Source) -> void #c_call;
}

anon_enum :: enum u32 {
    ALOOPER_PREPARE_ALLOW_NON_CALLBACKS :: 1;
}

anon_enum_1 :: enum u32 {
    AMOTION_EVENT_FLAG_WINDOW_IS_OBSCURED :: 1;
}

APP_CMD :: enum u32 {
    INPUT_CHANGED :: 0;
    INIT_WINDOW :: 1;
    TERM_WINDOW :: 2;
    WINDOW_RESIZED :: 3;
    WINDOW_REDRAW_NEEDED :: 4;
    CONTENT_RECT_CHANGED :: 5;
    GAINED_FOCUS :: 6;
    LOST_FOCUS :: 7;
    CONFIG_CHANGED :: 8;
    LOW_MEMORY :: 9;
    START :: 10;
    RESUME :: 11;
    SAVE_STATE :: 12;
    PAUSE :: 13;
    STOP :: 14;
    DESTROY :: 15;
}

#if INCLUDE_MEDIA_BINDINGS {
    AppendMode :: enum u32 {
        IGNORE_LAST_VIDEO_GOP :: 0;
        TO_EXISTING_DATA :: 1;
    }

}

ARect :: struct {
    left: s32;
    top: s32;
    right: s32;
    bottom: s32;
}

AWINDOW_FLAG :: enum u32 {
    ALLOW_LOCK_WHILE_SCREEN_ON :: 1;
    DIM_BEHIND :: 2;
    BLUR_BEHIND :: 4;
    NOT_FOCUSABLE :: 8;
    NOT_TOUCHABLE :: 16;
    NOT_TOUCH_MODAL :: 32;
    TOUCHABLE_WHEN_WAKING :: 64;
    KEEP_SCREEN_ON :: 128;
    LAYOUT_IN_SCREEN :: 256;
    LAYOUT_NO_LIMITS :: 512;
    FULLSCREEN :: 1024;
    FORCE_NOT_FULLSCREEN :: 2048;
    DITHER :: 4096;
    SECURE :: 8192;
    SCALED :: 16384;
    IGNORE_CHEEK_PRESSES :: 32768;
    LAYOUT_INSET_DECOR :: 65536;
    ALT_FOCUSABLE_IM :: 131072;
    WATCH_OUTSIDE_TOUCH :: 262144;
    SHOW_WHEN_LOCKED :: 524288;
    SHOW_WALLPAPER :: 1048576;
    TURN_SCREEN_ON :: 2097152;
    DISMISS_KEYGUARD :: 4194304;
}

#if INCLUDE_CAMERA_BINDINGS {
    camera_status_t :: enum s32 {
        OK :: 0;
        ERROR_BASE :: -10000;
        ERROR_UNKNOWN :: -10000;
        ERROR_INVALID_PARAMETER :: -10001;
        ERROR_CAMERA_DISCONNECTED :: -10002;
        ERROR_NOT_ENOUGH_MEMORY :: -10003;
        ERROR_METADATA_NOT_FOUND :: -10004;
        ERROR_CAMERA_DEVICE :: -10005;
        ERROR_CAMERA_SERVICE :: -10006;
        ERROR_SESSION_CLOSED :: -10007;
        ERROR_INVALID_OPERATION :: -10008;
        ERROR_STREAM_CONFIGURE_FAIL :: -10009;
        ERROR_CAMERA_IN_USE :: -10010;
        ERROR_MAX_CAMERA_IN_USE :: -10011;
        ERROR_CAMERA_DISABLED :: -10012;
        ERROR_PERMISSION_DENIED :: -10013;
        ERROR_UNSUPPORTED_OPERATION :: -10014;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    CAPTURE_FAILURE_REASON :: enum u32 {
        FLUSHED :: 0;
        ERROR :: 1;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    cryptoinfo_mode_t :: enum u32 {
        CLEAR :: 0;
        AES_CTR :: 1;
        AES_WV :: 2;
        AES_CBC :: 3;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    cryptoinfo_pattern_t :: struct {
        encryptBlocks: s32;
        skipBlocks: s32;
    }

}

#if INCLUDE_CAMERA_BINDINGS {
    ERROR :: enum u32 {
        CAMERA_IN_USE :: 1;
        MAX_CAMERAS_IN_USE :: 2;
        CAMERA_DISABLED :: 3;
        CAMERA_DEVICE :: 4;
        CAMERA_SERVICE :: 5;
    }

}

JavaVMAttachArgs :: struct {
    version: jint;
    name: *u8;
    group: jobject;
}

JavaVMInitArgs :: struct {
    version: jint;
    nOptions: jint;
    options: *JavaVMOption;
    ignoreUnrecognized: jboolean;
}

JavaVMOption :: struct {
    optionString: *u8;
    extraInfo: *void;
}

JNIInvokeInterface :: struct {
    reserved0: *void;
    reserved1: *void;
    reserved2: *void;
    DestroyJavaVM: #type (unknown0: *JavaVM) -> jint #c_call;
    AttachCurrentThread: #type (unknown0: *JavaVM, unknown1: **JNIEnv, unknown2: *void) -> jint #c_call;
    DetachCurrentThread: #type (unknown0: *JavaVM) -> jint #c_call;
    GetEnv: #type (unknown0: *JavaVM, unknown1: **void, unknown2: jint) -> jint #c_call;
    AttachCurrentThreadAsDaemon: #type (unknown0: *JavaVM, unknown1: **JNIEnv, unknown2: *void) -> jint #c_call;
}

JNINativeInterface :: struct #type_info_none {
    reserved0: *void;
    reserved1: *void;
    reserved2: *void;
    reserved3: *void;
    GetVersion: #type (unknown0: *JNIEnv) -> jint #c_call;
    DefineClass: #type (unknown0: *JNIEnv, unknown1: *u8, unknown2: jobject, unknown3: *jbyte, unknown4: jsize) -> jclass #c_call;
    FindClass: #type (unknown0: *JNIEnv, unknown1: *u8) -> jclass #c_call;
    FromReflectedMethod: #type (unknown0: *JNIEnv, unknown1: jobject) -> jmethodID #c_call;
    FromReflectedField: #type (unknown0: *JNIEnv, unknown1: jobject) -> jfieldID #c_call;
    ToReflectedMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: jboolean) -> jobject #c_call;
    GetSuperclass: #type (unknown0: *JNIEnv, unknown1: jclass) -> jclass #c_call;
    IsAssignableFrom: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jclass) -> jboolean #c_call;
    ToReflectedField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jboolean) -> jobject #c_call;
    Throw: #type (unknown0: *JNIEnv, unknown1: jthrowable) -> jint #c_call;
    ThrowNew: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *u8) -> jint #c_call;
    ExceptionOccurred: #type (unknown0: *JNIEnv) -> jthrowable #c_call;
    ExceptionDescribe: #type (unknown0: *JNIEnv) -> void #c_call;
    ExceptionClear: #type (unknown0: *JNIEnv) -> void #c_call;
    FatalError: #type (unknown0: *JNIEnv, unknown1: *u8) -> void #c_call;
    PushLocalFrame: #type (unknown0: *JNIEnv, unknown1: jint) -> jint #c_call;
    PopLocalFrame: #type (unknown0: *JNIEnv, unknown1: jobject) -> jobject #c_call;
    NewGlobalRef: #type (unknown0: *JNIEnv, unknown1: jobject) -> jobject #c_call;
    DeleteGlobalRef: #type (unknown0: *JNIEnv, unknown1: jobject) -> void #c_call;
    DeleteLocalRef: #type (unknown0: *JNIEnv, unknown1: jobject) -> void #c_call;
    IsSameObject: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jobject) -> jboolean #c_call;
    NewLocalRef: #type (unknown0: *JNIEnv, unknown1: jobject) -> jobject #c_call;
    EnsureLocalCapacity: #type (unknown0: *JNIEnv, unknown1: jint) -> jint #c_call;
    AllocObject: #type (unknown0: *JNIEnv, unknown1: jclass) -> jobject #c_call;
    NewObject: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jobject #c_call;
    NewObjectV: *void;
    NewObjectA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jobject #c_call;
    GetObjectClass: #type (unknown0: *JNIEnv, unknown1: jobject) -> jclass #c_call;
    IsInstanceOf: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass) -> jboolean #c_call;
    GetMethodID: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *u8, unknown3: *u8) -> jmethodID #c_call;
    CallObjectMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jobject #c_call;
    CallObjectMethodV: *void;
    CallObjectMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jobject #c_call;
    CallBooleanMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jboolean #c_call;
    CallBooleanMethodV: *void;
    CallBooleanMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jboolean #c_call;
    CallByteMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jbyte #c_call;
    CallByteMethodV: *void;
    CallByteMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jbyte #c_call;
    CallCharMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jchar #c_call;
    CallCharMethodV: *void;
    CallCharMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jchar #c_call;
    CallShortMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jshort #c_call;
    CallShortMethodV: *void;
    CallShortMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jshort #c_call;
    CallIntMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jint #c_call;
    CallIntMethodV: *void;
    CallIntMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jint #c_call;
    CallLongMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jlong #c_call;
    CallLongMethodV: *void;
    CallLongMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jlong #c_call;
    CallFloatMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jfloat #c_call;
    CallFloatMethodV: *void;
    CallFloatMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jfloat #c_call;
    CallDoubleMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> jdouble #c_call;
    CallDoubleMethodV: *void;
    CallDoubleMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> jdouble #c_call;
    CallVoidMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, __args: ..Any) -> void #c_call;
    CallVoidMethodV: *void;
    CallVoidMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jmethodID, unknown3: *jvalue) -> void #c_call;
    CallNonvirtualObjectMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jobject #c_call;
    CallNonvirtualObjectMethodV: *void;
    CallNonvirtualObjectMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jobject #c_call;
    CallNonvirtualBooleanMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jboolean #c_call;
    CallNonvirtualBooleanMethodV: *void;
    CallNonvirtualBooleanMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jboolean #c_call;
    CallNonvirtualByteMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jbyte #c_call;
    CallNonvirtualByteMethodV: *void;
    CallNonvirtualByteMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jbyte #c_call;
    CallNonvirtualCharMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jchar #c_call;
    CallNonvirtualCharMethodV: *void;
    CallNonvirtualCharMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jchar #c_call;
    CallNonvirtualShortMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jshort #c_call;
    CallNonvirtualShortMethodV: *void;
    CallNonvirtualShortMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jshort #c_call;
    CallNonvirtualIntMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jint #c_call;
    CallNonvirtualIntMethodV: *void;
    CallNonvirtualIntMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jint #c_call;
    CallNonvirtualLongMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jlong #c_call;
    CallNonvirtualLongMethodV: *void;
    CallNonvirtualLongMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jlong #c_call;
    CallNonvirtualFloatMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jfloat #c_call;
    CallNonvirtualFloatMethodV: *void;
    CallNonvirtualFloatMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jfloat #c_call;
    CallNonvirtualDoubleMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> jdouble #c_call;
    CallNonvirtualDoubleMethodV: *void;
    CallNonvirtualDoubleMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> jdouble #c_call;
    CallNonvirtualVoidMethod: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, __args: ..Any) -> void #c_call;
    CallNonvirtualVoidMethodV: *void;
    CallNonvirtualVoidMethodA: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jclass, unknown3: jmethodID, unknown4: *jvalue) -> void #c_call;
    GetFieldID: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *u8, unknown3: *u8) -> jfieldID #c_call;
    GetObjectField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jobject #c_call;
    GetBooleanField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jboolean #c_call;
    GetByteField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jbyte #c_call;
    GetCharField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jchar #c_call;
    GetShortField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jshort #c_call;
    GetIntField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jint #c_call;
    GetLongField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jlong #c_call;
    GetFloatField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jfloat #c_call;
    GetDoubleField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID) -> jdouble #c_call;
    SetObjectField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jobject) -> void #c_call;
    SetBooleanField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jboolean) -> void #c_call;
    SetByteField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jbyte) -> void #c_call;
    SetCharField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jchar) -> void #c_call;
    SetShortField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jshort) -> void #c_call;
    SetIntField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jint) -> void #c_call;
    SetLongField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jlong) -> void #c_call;
    SetFloatField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jfloat) -> void #c_call;
    SetDoubleField: #type (unknown0: *JNIEnv, unknown1: jobject, unknown2: jfieldID, unknown3: jdouble) -> void #c_call;
    GetStaticMethodID: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *u8, unknown3: *u8) -> jmethodID #c_call;
    CallStaticObjectMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jobject #c_call;
    CallStaticObjectMethodV: *void;
    CallStaticObjectMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jobject #c_call;
    CallStaticBooleanMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jboolean #c_call;
    CallStaticBooleanMethodV: *void;
    CallStaticBooleanMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jboolean #c_call;
    CallStaticByteMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jbyte #c_call;
    CallStaticByteMethodV: *void;
    CallStaticByteMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jbyte #c_call;
    CallStaticCharMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jchar #c_call;
    CallStaticCharMethodV: *void;
    CallStaticCharMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jchar #c_call;
    CallStaticShortMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jshort #c_call;
    CallStaticShortMethodV: *void;
    CallStaticShortMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jshort #c_call;
    CallStaticIntMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jint #c_call;
    CallStaticIntMethodV: *void;
    CallStaticIntMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jint #c_call;
    CallStaticLongMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jlong #c_call;
    CallStaticLongMethodV: *void;
    CallStaticLongMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jlong #c_call;
    CallStaticFloatMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jfloat #c_call;
    CallStaticFloatMethodV: *void;
    CallStaticFloatMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jfloat #c_call;
    CallStaticDoubleMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> jdouble #c_call;
    CallStaticDoubleMethodV: *void;
    CallStaticDoubleMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> jdouble #c_call;
    CallStaticVoidMethod: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, __args: ..Any) -> void #c_call;
    CallStaticVoidMethodV: *void;
    CallStaticVoidMethodA: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jmethodID, unknown3: *jvalue) -> void #c_call;
    GetStaticFieldID: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *u8, unknown3: *u8) -> jfieldID #c_call;
    GetStaticObjectField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jobject #c_call;
    GetStaticBooleanField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jboolean #c_call;
    GetStaticByteField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jbyte #c_call;
    GetStaticCharField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jchar #c_call;
    GetStaticShortField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jshort #c_call;
    GetStaticIntField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jint #c_call;
    GetStaticLongField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jlong #c_call;
    GetStaticFloatField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jfloat #c_call;
    GetStaticDoubleField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID) -> jdouble #c_call;
    SetStaticObjectField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jobject) -> void #c_call;
    SetStaticBooleanField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jboolean) -> void #c_call;
    SetStaticByteField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jbyte) -> void #c_call;
    SetStaticCharField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jchar) -> void #c_call;
    SetStaticShortField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jshort) -> void #c_call;
    SetStaticIntField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jint) -> void #c_call;
    SetStaticLongField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jlong) -> void #c_call;
    SetStaticFloatField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jfloat) -> void #c_call;
    SetStaticDoubleField: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: jfieldID, unknown3: jdouble) -> void #c_call;
    NewString: #type (unknown0: *JNIEnv, unknown1: *jchar, unknown2: jsize) -> jstring #c_call;
    GetStringLength: #type (unknown0: *JNIEnv, unknown1: jstring) -> jsize #c_call;
    GetStringChars: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *jboolean) -> *jchar #c_call;
    ReleaseStringChars: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *jchar) -> void #c_call;
    NewStringUTF: #type (unknown0: *JNIEnv, unknown1: *u8) -> jstring #c_call;
    GetStringUTFLength: #type (unknown0: *JNIEnv, unknown1: jstring) -> jsize #c_call;
    GetStringUTFChars: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *jboolean) -> *u8 #c_call;
    ReleaseStringUTFChars: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *u8) -> void #c_call;
    GetArrayLength: #type (unknown0: *JNIEnv, unknown1: jarray) -> jsize #c_call;
    NewObjectArray: #type (unknown0: *JNIEnv, unknown1: jsize, unknown2: jclass, unknown3: jobject) -> jobjectArray #c_call;
    GetObjectArrayElement: #type (unknown0: *JNIEnv, unknown1: jobjectArray, unknown2: jsize) -> jobject #c_call;
    SetObjectArrayElement: #type (unknown0: *JNIEnv, unknown1: jobjectArray, unknown2: jsize, unknown3: jobject) -> void #c_call;
    NewBooleanArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jbooleanArray #c_call;
    NewByteArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jbyteArray #c_call;
    NewCharArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jcharArray #c_call;
    NewShortArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jshortArray #c_call;
    NewIntArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jintArray #c_call;
    NewLongArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jlongArray #c_call;
    NewFloatArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jfloatArray #c_call;
    NewDoubleArray: #type (unknown0: *JNIEnv, unknown1: jsize) -> jdoubleArray #c_call;
    GetBooleanArrayElements: #type (unknown0: *JNIEnv, unknown1: jbooleanArray, unknown2: *jboolean) -> *jboolean #c_call;
    GetByteArrayElements: #type (unknown0: *JNIEnv, unknown1: jbyteArray, unknown2: *jboolean) -> *jbyte #c_call;
    GetCharArrayElements: #type (unknown0: *JNIEnv, unknown1: jcharArray, unknown2: *jboolean) -> *jchar #c_call;
    GetShortArrayElements: #type (unknown0: *JNIEnv, unknown1: jshortArray, unknown2: *jboolean) -> *jshort #c_call;
    GetIntArrayElements: #type (unknown0: *JNIEnv, unknown1: jintArray, unknown2: *jboolean) -> *jint #c_call;
    GetLongArrayElements: #type (unknown0: *JNIEnv, unknown1: jlongArray, unknown2: *jboolean) -> *jlong #c_call;
    GetFloatArrayElements: #type (unknown0: *JNIEnv, unknown1: jfloatArray, unknown2: *jboolean) -> *jfloat #c_call;
    GetDoubleArrayElements: #type (unknown0: *JNIEnv, unknown1: jdoubleArray, unknown2: *jboolean) -> *jdouble #c_call;
    ReleaseBooleanArrayElements: #type (unknown0: *JNIEnv, unknown1: jbooleanArray, unknown2: *jboolean, unknown3: jint) -> void #c_call;
    ReleaseByteArrayElements: #type (unknown0: *JNIEnv, unknown1: jbyteArray, unknown2: *jbyte, unknown3: jint) -> void #c_call;
    ReleaseCharArrayElements: #type (unknown0: *JNIEnv, unknown1: jcharArray, unknown2: *jchar, unknown3: jint) -> void #c_call;
    ReleaseShortArrayElements: #type (unknown0: *JNIEnv, unknown1: jshortArray, unknown2: *jshort, unknown3: jint) -> void #c_call;
    ReleaseIntArrayElements: #type (unknown0: *JNIEnv, unknown1: jintArray, unknown2: *jint, unknown3: jint) -> void #c_call;
    ReleaseLongArrayElements: #type (unknown0: *JNIEnv, unknown1: jlongArray, unknown2: *jlong, unknown3: jint) -> void #c_call;
    ReleaseFloatArrayElements: #type (unknown0: *JNIEnv, unknown1: jfloatArray, unknown2: *jfloat, unknown3: jint) -> void #c_call;
    ReleaseDoubleArrayElements: #type (unknown0: *JNIEnv, unknown1: jdoubleArray, unknown2: *jdouble, unknown3: jint) -> void #c_call;
    GetBooleanArrayRegion: #type (unknown0: *JNIEnv, unknown1: jbooleanArray, unknown2: jsize, unknown3: jsize, unknown4: *jboolean) -> void #c_call;
    GetByteArrayRegion: #type (unknown0: *JNIEnv, unknown1: jbyteArray, unknown2: jsize, unknown3: jsize, unknown4: *jbyte) -> void #c_call;
    GetCharArrayRegion: #type (unknown0: *JNIEnv, unknown1: jcharArray, unknown2: jsize, unknown3: jsize, unknown4: *jchar) -> void #c_call;
    GetShortArrayRegion: #type (unknown0: *JNIEnv, unknown1: jshortArray, unknown2: jsize, unknown3: jsize, unknown4: *jshort) -> void #c_call;
    GetIntArrayRegion: #type (unknown0: *JNIEnv, unknown1: jintArray, unknown2: jsize, unknown3: jsize, unknown4: *jint) -> void #c_call;
    GetLongArrayRegion: #type (unknown0: *JNIEnv, unknown1: jlongArray, unknown2: jsize, unknown3: jsize, unknown4: *jlong) -> void #c_call;
    GetFloatArrayRegion: #type (unknown0: *JNIEnv, unknown1: jfloatArray, unknown2: jsize, unknown3: jsize, unknown4: *jfloat) -> void #c_call;
    GetDoubleArrayRegion: #type (unknown0: *JNIEnv, unknown1: jdoubleArray, unknown2: jsize, unknown3: jsize, unknown4: *jdouble) -> void #c_call;
    SetBooleanArrayRegion: #type (unknown0: *JNIEnv, unknown1: jbooleanArray, unknown2: jsize, unknown3: jsize, unknown4: *jboolean) -> void #c_call;
    SetByteArrayRegion: #type (unknown0: *JNIEnv, unknown1: jbyteArray, unknown2: jsize, unknown3: jsize, unknown4: *jbyte) -> void #c_call;
    SetCharArrayRegion: #type (unknown0: *JNIEnv, unknown1: jcharArray, unknown2: jsize, unknown3: jsize, unknown4: *jchar) -> void #c_call;
    SetShortArrayRegion: #type (unknown0: *JNIEnv, unknown1: jshortArray, unknown2: jsize, unknown3: jsize, unknown4: *jshort) -> void #c_call;
    SetIntArrayRegion: #type (unknown0: *JNIEnv, unknown1: jintArray, unknown2: jsize, unknown3: jsize, unknown4: *jint) -> void #c_call;
    SetLongArrayRegion: #type (unknown0: *JNIEnv, unknown1: jlongArray, unknown2: jsize, unknown3: jsize, unknown4: *jlong) -> void #c_call;
    SetFloatArrayRegion: #type (unknown0: *JNIEnv, unknown1: jfloatArray, unknown2: jsize, unknown3: jsize, unknown4: *jfloat) -> void #c_call;
    SetDoubleArrayRegion: #type (unknown0: *JNIEnv, unknown1: jdoubleArray, unknown2: jsize, unknown3: jsize, unknown4: *jdouble) -> void #c_call;
    RegisterNatives: #type (unknown0: *JNIEnv, unknown1: jclass, unknown2: *JNINativeMethod, unknown3: jint) -> jint #c_call;
    UnregisterNatives: #type (unknown0: *JNIEnv, unknown1: jclass) -> jint #c_call;
    MonitorEnter: #type (unknown0: *JNIEnv, unknown1: jobject) -> jint #c_call;
    MonitorExit: #type (unknown0: *JNIEnv, unknown1: jobject) -> jint #c_call;
    GetJavaVM: #type (unknown0: *JNIEnv, unknown1: **JavaVM) -> jint #c_call;
    GetStringRegion: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: jsize, unknown3: jsize, unknown4: *jchar) -> void #c_call;
    GetStringUTFRegion: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: jsize, unknown3: jsize, unknown4: *u8) -> void #c_call;
    GetPrimitiveArrayCritical: #type (unknown0: *JNIEnv, unknown1: jarray, unknown2: *jboolean) -> *void #c_call;
    ReleasePrimitiveArrayCritical: #type (unknown0: *JNIEnv, unknown1: jarray, unknown2: *void, unknown3: jint) -> void #c_call;
    GetStringCritical: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *jboolean) -> *jchar #c_call;
    ReleaseStringCritical: #type (unknown0: *JNIEnv, unknown1: jstring, unknown2: *jchar) -> void #c_call;
    NewWeakGlobalRef: #type (unknown0: *JNIEnv, unknown1: jobject) -> jweak #c_call;
    DeleteWeakGlobalRef: #type (unknown0: *JNIEnv, unknown1: jweak) -> void #c_call;
    ExceptionCheck: #type (unknown0: *JNIEnv) -> jboolean #c_call;
    NewDirectByteBuffer: #type (unknown0: *JNIEnv, unknown1: *void, unknown2: jlong) -> jobject #c_call;
    GetDirectBufferAddress: #type (unknown0: *JNIEnv, unknown1: jobject) -> *void #c_call;
    GetDirectBufferCapacity: #type (unknown0: *JNIEnv, unknown1: jobject) -> jlong #c_call;
    GetObjectRefType: #type (unknown0: *JNIEnv, unknown1: jobject) -> jobjectRefType #c_call;
}

JNINativeMethod :: struct {
    name: *u8;
    signature: *u8;
    fnPtr: *void;
}

jobjectRefType :: enum u32 {
    InvalidRefType :: 0;
    LocalRefType :: 1;
    GlobalRefType :: 2;
    WeakGlobalRefType :: 3;
}

jvalue :: struct {
    j: jlong;
    #overlay (j) z: jboolean;
    #overlay (j) b: jbyte;
    #overlay (j) c: jchar;
    #overlay (j) s: jshort;
    #overlay (j) i: jint;
    #overlay (j) f: jfloat;
    #overlay (j) d: jdouble;
    #overlay (j) l: jobject;
}

Log_Id :: enum u32 {
    MIN :: 0;
    MAIN :: 0;
    RADIO :: 1;
    EVENTS :: 2;
    SYSTEM :: 3;
    CRASH :: 4;
    STATS :: 5;
    SECURITY :: 6;
    KERNEL :: 7;
    MAX :: 8;
    DEFAULT :: 2147483647;
}

Log_Message :: struct {
    struct_size: u64;
    buffer_id: s32;
    priority: s32;
    tag: *u8;
    file: *u8;
    line: u32;
    message: *u8;
}

Log_Priority :: enum u32 {
    UNKNOWN :: 0;
    DEFAULT :: 1;
    VERBOSE :: 2;
    DEBUG :: 3;
    INFO :: 4;
    WARN :: 5;
    ERROR :: 6;
    FATAL :: 7;
    SILENT :: 8;
}

LOOPER_ID :: enum u32 {
    MAIN :: 1;
    INPUT :: 2;
    USER :: 3;
}

#if INCLUDE_MEDIA_BINDINGS {
    media_status_t :: enum s32 {
        OK :: 0;
        CODEC_ERROR_INSUFFICIENT_RESOURCE :: 1100;
        CODEC_ERROR_RECLAIMED :: 1101;
        ERROR_BASE :: -10000;
        ERROR_UNKNOWN :: -10000;
        ERROR_MALFORMED :: -10001;
        ERROR_UNSUPPORTED :: -10002;
        ERROR_INVALID_OBJECT :: -10003;
        ERROR_INVALID_PARAMETER :: -10004;
        ERROR_INVALID_OPERATION :: -10005;
        ERROR_END_OF_STREAM :: -10006;
        ERROR_IO :: -10007;
        ERROR_WOULD_BLOCK :: -10008;
        DRM_ERROR_BASE :: -20000;
        DRM_NOT_PROVISIONED :: -20001;
        DRM_RESOURCE_BUSY :: -20002;
        DRM_DEVICE_REVOKED :: -20003;
        DRM_SHORT_BUFFER :: -20004;
        DRM_SESSION_NOT_OPENED :: -20005;
        DRM_TAMPER_DETECTED :: -20006;
        DRM_VERIFY_FAILED :: -20007;
        DRM_NEED_KEY :: -20008;
        DRM_LICENSE_EXPIRED :: -20009;
        IMGREADER_ERROR_BASE :: -30000;
        IMGREADER_NO_BUFFER_AVAILABLE :: -30001;
        IMGREADER_MAX_IMAGES_ACQUIRED :: -30002;
        IMGREADER_CANNOT_LOCK_IMAGE :: -30003;
        IMGREADER_CANNOT_UNLOCK_IMAGE :: -30004;
        IMGREADER_IMAGE_NOT_LOCKED :: -30005;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    OutputFormat :: enum u32 {
        MPEG_4 :: 0;
        WEBM :: 1;
        THREE_GPP :: 2;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    PsshEntry :: struct {
        uuid: AMediaUUID;
        datalen: u64;
        data: *void;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    PsshInfo :: struct {
        numentries: u64;
        entries: [0] PsshEntry;
    }

}

#if INCLUDE_MEDIA_BINDINGS {
    SeekMode :: enum u32 {
        PREVIOUS_SYNC :: 0;
        NEXT_SYNC :: 1;
        CLOSEST_SYNC :: 2;
    }

}

// Callback and opaque types

__android_aborter_function :: #type (abort_message: *u8) -> void #c_call;

__android_logger_function :: #type (log_message: *Log_Message) -> void #c_call;

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_bufferLost :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, window: *ACameraWindowType, frameNumber: s64) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_failed :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, failure: *ACameraCaptureFailure) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_result :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, result: *ACameraMetadata) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_sequenceAbort :: #type (_context: *void, session: *ACameraCaptureSession, sequenceId: s32) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_sequenceEnd :: #type (_context: *void, session: *ACameraCaptureSession, sequenceId: s32, frameNumber: s64) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_start :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, timestamp: s64) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureCallback_startV2 :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, timestamp: s64, frameNumber: s64) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_captureCallback_failed :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, failure: *ALogicalCameraCaptureFailure) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_captureCallback_result :: #type (_context: *void, session: *ACameraCaptureSession, request: *ACaptureRequest, result: *ACameraMetadata, physicalResultCount: u64, physicalCameraIds: **u8, physicalResults: **ACameraMetadata) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_stateCallback :: #type (_context: *void, session: *ACameraCaptureSession) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_ErrorStateCallback :: #type (_context: *void, device: *ACameraDevice, error: s32) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_StateCallback :: #type (_context: *void, device: *ACameraDevice) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_AccessPrioritiesChangedCallback :: #type (_context: *void) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_AvailabilityCallback :: #type (_context: *void, cameraId: *u8) -> void #c_call;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_PhysicalCameraAvailabilityCallback :: #type (_context: *void, cameraId: *u8, physicalCameraId: *u8) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_BufferRemovedCallback :: #type (_context: *void, reader: *AImageReader, buffer: *AHardwareBuffer) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_ImageCallback :: #type (_context: *void, reader: *AImageReader) -> void #c_call;
}

ALooper_callbackFunc :: #type (fd: s32, events: s32, data: *void) -> s32 #c_call;

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnAsyncError :: #type (codec: *AMediaCodec, userdata: *void, error: media_status_t, actionCode: s32, detail: *u8) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnAsyncFormatChanged :: #type (codec: *AMediaCodec, userdata: *void, format: *AMediaFormat) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnAsyncInputAvailable :: #type (codec: *AMediaCodec, userdata: *void, index: s32) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnAsyncOutputAvailable :: #type (codec: *AMediaCodec, userdata: *void, index: s32, bufferInfo: *AMediaCodecBufferInfo) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecOnFrameRendered :: #type (codec: *AMediaCodec, userdata: *void, mediaTimeUs: s64, systemNano: s64) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSourceClose :: #type (userdata: *void) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSourceGetAvailableSize :: #type (userdata: *void, offset: s64) -> s64 #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSourceGetSize :: #type (userdata: *void) -> s64 #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSourceReadAt :: #type (userdata: *void, offset: s64, buffer: *void, size: u64) -> s64 #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmEventListener :: #type (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, eventType: AMediaDrmEventType, extra: s32, data: *u8, dataSize: u64) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmExpirationUpdateListener :: #type (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, expiryTimeInMS: s64) -> void #c_call;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeysChangeListener :: #type (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, keyStatus: *AMediaDrmKeyStatus, numKeys: u64, hasNewUsableKey: bool) -> void #c_call;
}

ANativeActivity_createFunc :: #type (activity: *ANativeActivity, savedState: *void, savedStateSize: u64) -> void #c_call;

Android_Command_Proc :: #type (app: *Android_App, cmd: s32);

Android_Input_Proc :: #type (event: *AInputEvent) -> handled: bool;

// Aliases and constants

#if INCLUDE_MEDIA_BINDINGS {
    _off_t_compat :: s64;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_automotive_lens_facing_t :: acamera_metadata_enum_acamera_automotive_lens_facing;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_automotive_location_t :: acamera_metadata_enum_acamera_automotive_location;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_black_level_lock_t :: acamera_metadata_enum_acamera_black_level_lock;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_color_correction_aberration_mode_t :: acamera_metadata_enum_acamera_color_correction_aberration_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_color_correction_mode_t :: acamera_metadata_enum_acamera_color_correction_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_antibanding_mode_t :: acamera_metadata_enum_acamera_control_ae_antibanding_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_lock_available_t :: acamera_metadata_enum_acamera_control_ae_lock_available;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_lock_t :: acamera_metadata_enum_acamera_control_ae_lock;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_mode_t :: acamera_metadata_enum_acamera_control_ae_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_precapture_trigger_t :: acamera_metadata_enum_acamera_control_ae_precapture_trigger;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_ae_state_t :: acamera_metadata_enum_acamera_control_ae_state;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_af_mode_t :: acamera_metadata_enum_acamera_control_af_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_af_scene_change_t :: acamera_metadata_enum_acamera_control_af_scene_change;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_af_state_t :: acamera_metadata_enum_acamera_control_af_state;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_af_trigger_t :: acamera_metadata_enum_acamera_control_af_trigger;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_awb_lock_available_t :: acamera_metadata_enum_acamera_control_awb_lock_available;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_awb_lock_t :: acamera_metadata_enum_acamera_control_awb_lock;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_awb_mode_t :: acamera_metadata_enum_acamera_control_awb_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_awb_state_t :: acamera_metadata_enum_acamera_control_awb_state;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_capture_intent_t :: acamera_metadata_enum_acamera_control_capture_intent;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_effect_mode_t :: acamera_metadata_enum_acamera_control_effect_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_enable_zsl_t :: acamera_metadata_enum_acamera_control_enable_zsl;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_extended_scene_mode_t :: acamera_metadata_enum_acamera_control_extended_scene_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_mode_t :: acamera_metadata_enum_acamera_control_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_scene_mode_t :: acamera_metadata_enum_acamera_control_scene_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_control_video_stabilization_mode_t :: acamera_metadata_enum_acamera_control_video_stabilization_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_depth_available_depth_stream_configurations_maximum_resolution_t :: acamera_metadata_enum_acamera_depth_available_depth_stream_configurations_maximum_resolution;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_depth_available_depth_stream_configurations_t :: acamera_metadata_enum_acamera_depth_available_depth_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_depth_available_dynamic_depth_stream_configurations_maximum_resolution_t :: acamera_metadata_enum_acamera_depth_available_dynamic_depth_stream_configurations_maximum_resolution;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_depth_available_dynamic_depth_stream_configurations_t :: acamera_metadata_enum_acamera_depth_available_dynamic_depth_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_depth_depth_is_exclusive_t :: acamera_metadata_enum_acamera_depth_depth_is_exclusive;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_distortion_correction_mode_t :: acamera_metadata_enum_acamera_distortion_correction_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_edge_mode_t :: acamera_metadata_enum_acamera_edge_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_flash_info_available_t :: acamera_metadata_enum_acamera_flash_info_available;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_flash_mode_t :: acamera_metadata_enum_acamera_flash_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_flash_state_t :: acamera_metadata_enum_acamera_flash_state;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_heic_available_heic_stream_configurations_maximum_resolution_t :: acamera_metadata_enum_acamera_heic_available_heic_stream_configurations_maximum_resolution;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_heic_available_heic_stream_configurations_t :: acamera_metadata_enum_acamera_heic_available_heic_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_hot_pixel_mode_t :: acamera_metadata_enum_acamera_hot_pixel_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_info_supported_hardware_level_t :: acamera_metadata_enum_acamera_info_supported_hardware_level;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_lens_facing_t :: acamera_metadata_enum_acamera_lens_facing;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_lens_info_focus_distance_calibration_t :: acamera_metadata_enum_acamera_lens_info_focus_distance_calibration;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_lens_optical_stabilization_mode_t :: acamera_metadata_enum_acamera_lens_optical_stabilization_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_lens_pose_reference_t :: acamera_metadata_enum_acamera_lens_pose_reference;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_lens_state_t :: acamera_metadata_enum_acamera_lens_state;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_logical_multi_camera_sensor_sync_type_t :: acamera_metadata_enum_acamera_logical_multi_camera_sensor_sync_type;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_noise_reduction_mode_t :: acamera_metadata_enum_acamera_noise_reduction_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_request_available_capabilities_t :: acamera_metadata_enum_acamera_request_available_capabilities;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_request_available_dynamic_range_profiles_map_t :: acamera_metadata_enum_acamera_request_available_dynamic_range_profiles_map;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_available_recommended_stream_configurations_t :: acamera_metadata_enum_acamera_scaler_available_recommended_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_available_stream_configurations_maximum_resolution_t :: acamera_metadata_enum_acamera_scaler_available_stream_configurations_maximum_resolution;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_available_stream_configurations_t :: acamera_metadata_enum_acamera_scaler_available_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_available_stream_use_cases_t :: acamera_metadata_enum_acamera_scaler_available_stream_use_cases;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_cropping_type_t :: acamera_metadata_enum_acamera_scaler_cropping_type;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_multi_resolution_stream_supported_t :: acamera_metadata_enum_acamera_scaler_multi_resolution_stream_supported;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_physical_camera_multi_resolution_stream_configurations_t :: acamera_metadata_enum_acamera_scaler_physical_camera_multi_resolution_stream_configurations;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_scaler_rotate_and_crop_t :: acamera_metadata_enum_acamera_scaler_rotate_and_crop;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_info_color_filter_arrangement_t :: acamera_metadata_enum_acamera_sensor_info_color_filter_arrangement;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_info_lens_shading_applied_t :: acamera_metadata_enum_acamera_sensor_info_lens_shading_applied;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_info_timestamp_source_t :: acamera_metadata_enum_acamera_sensor_info_timestamp_source;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_pixel_mode_t :: acamera_metadata_enum_acamera_sensor_pixel_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_raw_binning_factor_used_t :: acamera_metadata_enum_acamera_sensor_raw_binning_factor_used;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_reference_illuminant1_t :: acamera_metadata_enum_acamera_sensor_reference_illuminant1;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sensor_test_pattern_mode_t :: acamera_metadata_enum_acamera_sensor_test_pattern_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_shading_mode_t :: acamera_metadata_enum_acamera_shading_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_statistics_face_detect_mode_t :: acamera_metadata_enum_acamera_statistics_face_detect_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_statistics_hot_pixel_map_mode_t :: acamera_metadata_enum_acamera_statistics_hot_pixel_map_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_statistics_lens_shading_map_mode_t :: acamera_metadata_enum_acamera_statistics_lens_shading_map_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_statistics_ois_data_mode_t :: acamera_metadata_enum_acamera_statistics_ois_data_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_statistics_scene_flicker_t :: acamera_metadata_enum_acamera_statistics_scene_flicker;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sync_frame_number_t :: acamera_metadata_enum_acamera_sync_frame_number;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_sync_max_latency_t :: acamera_metadata_enum_acamera_sync_max_latency;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_tonemap_mode_t :: acamera_metadata_enum_acamera_tonemap_mode;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_enum_android_tonemap_preset_curve_t :: acamera_metadata_enum_acamera_tonemap_preset_curve;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_section_start_t :: acamera_metadata_section_start;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_section_t :: acamera_metadata_section;
}

#if INCLUDE_CAMERA_BINDINGS {
    acamera_metadata_tag_t :: acamera_metadata_tag;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_stateCallbacks :: ACameraDevice_StateCallbacks;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_AvailabilityCallbacks :: ACameraManager_AvailabilityListener;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_ExtendedAvailabilityCallbacks :: ACameraManager_ExtendedAvailabilityListener;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraWindowType :: ANativeWindow;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyId :: AMediaDrmByteArray;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeySetId :: AMediaDrmByteArray;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyStatusType :: AMediaKeyStatusType;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmKeyValue :: AMediaDrmKeyValuePair;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmScope :: AMediaDrmByteArray;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmSecureStop :: AMediaDrmByteArray;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrmSessionId :: AMediaDrmByteArray;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaUUID :: [16] u8;
}

AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT :: 8;

ANDROID_WARN_ABOUT_SHADOWED_ASSETS :: WARN_ABOUT_SHADOWED_ASSETS;

jarray :: jobject;

JavaVM :: *JNIInvokeInterface;

jboolean :: u8;

jbooleanArray :: jarray;

jbyte :: s8;

jbyteArray :: jarray;

jchar :: u16;

jcharArray :: jarray;

jclass :: jobject;

jdouble :: float64;

jdoubleArray :: jarray;

jfieldID :: *_jfieldID;

jfloat :: float;

jfloatArray :: jarray;

jint :: s32;

jintArray :: jarray;

jlong :: s64;

jlongArray :: jarray;

jmethodID :: *_jmethodID;

JNI_ABORT :: 2;

JNI_COMMIT :: 1;

JNI_EDETACHED :: -2;

JNI_EEXIST :: -5;

JNI_EINVAL :: -6;

JNI_ENOMEM :: -4;

JNI_ERR :: -1;

JNI_EVERSION :: -3;

JNI_FALSE :: 0;

JNI_OK :: 0;

JNI_TRUE :: 1;

JNI_VERSION_1_1 :: 0x00010001;

JNI_VERSION_1_2 :: 0x00010002;

JNI_VERSION_1_4 :: 0x00010004;

JNI_VERSION_1_6 :: 0x00010006;

JNIEnv :: *JNINativeInterface;

jobject :: *void;

jobjectArray :: jarray;

jshort :: s16;

jshortArray :: jarray;

jsize :: jint;

jstring :: jobject;

jthrowable :: jobject;

jweak :: jobject;

#if INCLUDE_MEDIA_BINDINGS {
    PROPERTY_ALGORITHMS :: "algorithms";
}

#if INCLUDE_MEDIA_BINDINGS {
    PROPERTY_DESCRIPTION :: "description";
}

#if INCLUDE_MEDIA_BINDINGS {
    PROPERTY_DEVICE_UNIQUE_ID :: "deviceUniqueId";
}

#if INCLUDE_MEDIA_BINDINGS {
    PROPERTY_VENDOR :: "vendor";
}

#if INCLUDE_MEDIA_BINDINGS {
    PROPERTY_VERSION :: "version";
}

// Native storage

#if DEFINE_ANDROID_MAIN_AND_CALL_THIS {
    __android_handle_command: Android_Command_Proc;
}

#if DEFINE_ANDROID_MAIN_AND_CALL_THIS {
    __android_handle_input_event: Android_Input_Proc;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_HDR10_PLUS_INFO: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_LOW_LATENCY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_OFFSET_TIME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_REQUEST_SYNC_FRAME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_SUSPEND: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_SUSPEND_TIME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIACODEC_KEY_VIDEO_BITRATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_DRC_ATTENUATION_FACTOR: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_DRC_BOOST_FACTOR: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_DRC_HEAVY_COMPRESSION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_DRC_TARGET_REFERENCE_LEVEL: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_ENCODED_TARGET_LEVEL: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_MAX_OUTPUT_CHANNEL_COUNT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_PROFILE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AAC_SBR_MODE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ALBUM: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ALBUMART: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ALBUMARTIST: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ARTIST: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AUDIO_PRESENTATION_INFO: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AUDIO_SESSION_ID: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_AUTHOR: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_BIT_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_BITRATE_MODE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_BITS_PER_SAMPLE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CAPTURE_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CDTRACKNUMBER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CHANNEL_COUNT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CHANNEL_MASK: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COLOR_FORMAT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COLOR_RANGE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COLOR_STANDARD: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COLOR_TRANSFER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COMPILATION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COMPLEXITY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_COMPOSER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CREATE_INPUT_SURFACE_SUSPENDED: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_DEFAULT_IV_SIZE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_ENCRYPTED_BYTE_BLOCK: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_ENCRYPTED_SIZES: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_IV: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_KEY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_MODE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_PLAIN_SIZES: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CRYPTO_SKIP_BYTE_BLOCK: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD_0: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD_1: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD_2: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD_AVC: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_CSD_HEVC: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_D263: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DISCNUMBER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DISPLAY_CROP: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DISPLAY_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DISPLAY_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_DURATION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ENCODER_DELAY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ENCODER_PADDING: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ESDS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_EXIF_OFFSET: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_EXIF_SIZE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_FLAC_COMPRESSION_LEVEL: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_FRAME_COUNT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_FRAME_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_GENRE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_GRID_COLUMNS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_GRID_ROWS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_HAPTIC_CHANNEL_COUNT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_HDR10_PLUS_INFO: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_HDR_STATIC_INFO: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_I_FRAME_INTERVAL: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ICC_PROFILE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_INTRA_REFRESH_PERIOD: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_IS_ADTS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_IS_AUTOSELECT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_IS_DEFAULT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_IS_FORCED_SUBTITLE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_IS_SYNC_FRAME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LANGUAGE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LAST_SAMPLE_INDEX_IN_CHUNK: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LATENCY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LEVEL: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LOCATION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LOOP: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LOW_LATENCY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_LYRICIST: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MANUFACTURER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_BIT_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_FPS_TO_ENCODER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_INPUT_SIZE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_PTS_GAP_TO_ENCODER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MAX_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MIME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MPEG2_STREAM_HEADER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MPEG_USER_DATA: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MPEGH_COMPATIBLE_SETS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MPEGH_PROFILE_LEVEL_INDICATION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_MPEGH_REFERENCE_CHANNEL_LAYOUT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_OPERATING_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PCM_BIG_ENDIAN: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PCM_ENCODING: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PRIORITY: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PROFILE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PSSH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_PUSH_BLANK_BUFFERS_ON_STOP: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_REPEAT_PREVIOUS_FRAME_AFTER: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_ROTATION: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SAMPLE_FILE_OFFSET: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SAMPLE_RATE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SAMPLE_TIME_BEFORE_APPEND: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SAR_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SAR_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SEI: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SLICE_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_SLOW_MOTION_MARKERS: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_STRIDE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TARGET_TIME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TEMPORAL_LAYER_COUNT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TEMPORAL_LAYER_ID: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TEMPORAL_LAYERING: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TEXT_FORMAT_DATA: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_THUMBNAIL_CSD_AV1C: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_THUMBNAIL_CSD_HEVC: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_THUMBNAIL_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_THUMBNAIL_TIME: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_THUMBNAIL_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TILE_HEIGHT: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TILE_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TIME_US: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TITLE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TRACK_ID: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_TRACK_INDEX: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_VALID_SAMPLES: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_WIDTH: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_XMP_OFFSET: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_XMP_SIZE: *u8 #elsewhere libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMEDIAFORMAT_KEY_YEAR: *u8 #elsewhere libmediandk;
}

#if DEFINE_ANDROID_MAIN_AND_CALL_THIS {
    android_app: *Android_App;
}

// Native entry points

__android_log_assert :: (cond: *u8, tag: *u8, fmt: *u8, __args: ..Any) -> void #foreign liblog;

__android_log_buf_print :: (bufID: Log_Id, prio: Log_Priority, tag: *u8, fmt: *u8, __args: ..Any) -> s32 #foreign liblog;

__android_log_buf_write :: (bufID: Log_Id, prio: Log_Priority, tag: *u8, text: *u8) -> s32 #foreign liblog;

__android_log_call_aborter :: (abort_message: *u8) -> void #foreign liblog;

__android_log_default_aborter :: (abort_message: *u8) -> void #foreign liblog;

__android_log_get_minimum_priority :: () -> Log_Priority #foreign liblog;

__android_log_is_loggable :: (prio: Log_Priority, tag: *u8, default_prio: s32) -> s32 #foreign liblog;

__android_log_is_loggable_len :: (prio: Log_Priority, tag: *u8, len: u64, default_prio: s32) -> s32 #foreign liblog;

__android_log_logd_logger :: (log_message: *Log_Message) -> void #foreign liblog;

__android_log_print :: (prio: Log_Priority, tag: *u8, fmt: *u8, __args: ..Any) -> s32 #foreign liblog;

__android_log_set_aborter :: (aborter: __android_aborter_function) -> void #foreign liblog;

__android_log_set_default_tag :: (tag: *u8) -> void #foreign liblog;

__android_log_set_logger :: (logger: __android_logger_function) -> void #foreign liblog;

__android_log_set_minimum_priority :: (priority: Log_Priority) -> s32 #foreign liblog;

__android_log_stderr_logger :: (log_message: *Log_Message) -> void #foreign liblog;

__android_log_write :: (prio: Log_Priority, tag: *u8, text: *u8) -> s32 #foreign liblog;

__android_log_write_log_message :: (log_message: *Log_Message) -> void #foreign liblog;

AAsset_close :: (asset: *AAsset) -> void #foreign libandroid;

AAsset_getBuffer :: (asset: *AAsset) -> *void #foreign libandroid;

AAsset_getLength :: (asset: *AAsset) -> s64 #foreign libandroid;

AAsset_getLength64 :: (asset: *AAsset) -> s64 #foreign libandroid;

AAsset_getRemainingLength :: (asset: *AAsset) -> s64 #foreign libandroid;

AAsset_getRemainingLength64 :: (asset: *AAsset) -> s64 #foreign libandroid;

AAsset_isAllocated :: (asset: *AAsset) -> s32 #foreign libandroid;

AAsset_openFileDescriptor :: (asset: *AAsset, outStart: *s64, outLength: *s64) -> s32 #foreign libandroid;

AAsset_openFileDescriptor64 :: (asset: *AAsset, outStart: *s64, outLength: *s64) -> s32 #foreign libandroid;

AAsset_read :: (asset: *AAsset, buf: *void, count: u64) -> s32 #foreign libandroid;

AAsset_seek :: (asset: *AAsset, offset: s64, whence: s32) -> s64 #foreign libandroid;

AAsset_seek64 :: (asset: *AAsset, offset: s64, whence: s32) -> s64 #foreign libandroid;

AAssetDir_close :: (assetDir: *AAssetDir) -> void #foreign libandroid;

AAssetDir_getNextFileName :: (assetDir: *AAssetDir) -> *u8 #foreign libandroid;

AAssetDir_rewind :: (assetDir: *AAssetDir) -> void #foreign libandroid;

AAssetManager_open :: (mgr: *AAssetManager, filename: *u8, mode: AASSET_MODE) -> *AAsset #foreign libandroid;

AAssetManager_openDir :: (mgr: *AAssetManager, dirName: *u8) -> *AAssetDir #foreign libandroid;

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_abortCaptures :: (session: *ACameraCaptureSession) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_capture :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_captureCallbacks, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_captureV2 :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_captureCallbacksV2, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_close :: (session: *ACameraCaptureSession) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_getDevice :: (session: *ACameraCaptureSession, device: **ACameraDevice) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_capture :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_logicalCamera_captureCallbacks, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_captureV2 :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_logicalCamera_captureCallbacksV2, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_setRepeatingRequest :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_logicalCamera_captureCallbacks, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_logicalCamera_setRepeatingRequestV2 :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_logicalCamera_captureCallbacksV2, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_setRepeatingRequest :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_captureCallbacks, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_setRepeatingRequestV2 :: (session: *ACameraCaptureSession, callbacks: *ACameraCaptureSession_captureCallbacksV2, numRequests: s32, requests: **ACaptureRequest, captureSequenceId: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_stopRepeating :: (session: *ACameraCaptureSession) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraCaptureSession_updateSharedOutput :: (session: *ACameraCaptureSession, output: *ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_close :: (device: *ACameraDevice) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_createCaptureRequest :: (device: *ACameraDevice, templateId: ACameraDevice_request_template, request: **ACaptureRequest) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_createCaptureRequest_withPhysicalIds :: (device: *ACameraDevice, templateId: ACameraDevice_request_template, physicalIdList: *ACameraIdList, request: **ACaptureRequest) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_createCaptureSession :: (device: *ACameraDevice, outputs: *ACaptureSessionOutputContainer, callbacks: *ACameraCaptureSession_stateCallbacks, session: **ACameraCaptureSession) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_createCaptureSessionWithSessionParameters :: (device: *ACameraDevice, outputs: *ACaptureSessionOutputContainer, sessionParameters: *ACaptureRequest, callbacks: *ACameraCaptureSession_stateCallbacks, session: **ACameraCaptureSession) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_getId :: (device: *ACameraDevice) -> *u8 #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraDevice_isSessionConfigurationSupported :: (device: *ACameraDevice, sessionOutputContainer: *ACaptureSessionOutputContainer) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_create :: (__args: ..Any) -> *ACameraManager #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_delete :: (manager: *ACameraManager) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_deleteCameraIdList :: (cameraIdList: *ACameraIdList) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_getCameraCharacteristics :: (manager: *ACameraManager, cameraId: *u8, characteristics: **ACameraMetadata) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_getCameraIdList :: (manager: *ACameraManager, cameraIdList: **ACameraIdList) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_openCamera :: (manager: *ACameraManager, cameraId: *u8, callback: *ACameraDevice_stateCallbacks, device: **ACameraDevice) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_registerAvailabilityCallback :: (manager: *ACameraManager, callback: *ACameraManager_AvailabilityCallbacks) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_registerExtendedAvailabilityCallback :: (manager: *ACameraManager, callback: *ACameraManager_ExtendedAvailabilityCallbacks) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_unregisterAvailabilityCallback :: (manager: *ACameraManager, callback: *ACameraManager_AvailabilityCallbacks) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraManager_unregisterExtendedAvailabilityCallback :: (manager: *ACameraManager, callback: *ACameraManager_ExtendedAvailabilityCallbacks) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_copy :: (src: *ACameraMetadata) -> *ACameraMetadata #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_free :: (metadata: *ACameraMetadata) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_fromCameraMetadata :: (env: *JNIEnv, cameraMetadata: jobject) -> *ACameraMetadata #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_getAllTags :: (metadata: *ACameraMetadata, numEntries: *s32, tags: **u32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_getConstEntry :: (metadata: *ACameraMetadata, tag: acamera_metadata_tag_t, entry: *ACameraMetadata_const_entry) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraMetadata_isLogicalMultiCamera :: (staticMetadata: *ACameraMetadata, numPhysicalCameras: *u64, physicalCameraIds: ***u8) -> bool #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraOutputTarget_create :: (window: *ACameraWindowType, output: **ACameraOutputTarget) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACameraOutputTarget_free :: (output: *ACameraOutputTarget) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_addTarget :: (request: *ACaptureRequest, output: *ACameraOutputTarget) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_copy :: (src: *ACaptureRequest) -> *ACaptureRequest #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_free :: (request: *ACaptureRequest) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_getAllTags :: (request: *ACaptureRequest, numTags: *s32, tags: **u32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_getConstEntry :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, entry: *ACameraMetadata_const_entry) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_getConstEntry_physicalCamera :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, entry: *ACameraMetadata_const_entry) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_getUserContext :: (request: *ACaptureRequest, _context: **void) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_removeTarget :: (request: *ACaptureRequest, output: *ACameraOutputTarget) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_double :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *float64) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_float :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *float) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_i32 :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_i64 :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *s64) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_double :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *float64) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_float :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *float) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_i32 :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *s32) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_i64 :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *s64) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_rational :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *ACameraMetadata_rational) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_physicalCamera_u8 :: (request: *ACaptureRequest, physicalId: *u8, tag: acamera_metadata_tag_t, count: u32, data: *u8) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_rational :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *ACameraMetadata_rational) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setEntry_u8 :: (request: *ACaptureRequest, tag: acamera_metadata_tag_t, count: u32, data: *u8) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureRequest_setUserContext :: (request: *ACaptureRequest, _context: *void) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutput_create :: (anw: *ACameraWindowType, output: **ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutput_free :: (output: *ACaptureSessionOutput) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutputContainer_add :: (container: *ACaptureSessionOutputContainer, output: *ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutputContainer_create :: (container: **ACaptureSessionOutputContainer) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutputContainer_free :: (container: *ACaptureSessionOutputContainer) -> void #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionOutputContainer_remove :: (container: *ACaptureSessionOutputContainer, output: *ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionPhysicalOutput_create :: (anw: *ACameraWindowType, physicalId: *u8, output: **ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionSharedOutput_add :: (output: *ACaptureSessionOutput, anw: *ACameraWindowType) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionSharedOutput_create :: (anw: *ACameraWindowType, output: **ACaptureSessionOutput) -> camera_status_t #foreign libcamera2ndk;
}

#if INCLUDE_CAMERA_BINDINGS {
    ACaptureSessionSharedOutput_remove :: (output: *ACaptureSessionOutput, anw: *ACameraWindowType) -> camera_status_t #foreign libcamera2ndk;
}

AConfiguration_copy :: (dest: *AConfiguration, src: *AConfiguration) -> void #foreign libandroid;

AConfiguration_delete :: (config: *AConfiguration) -> void #foreign libandroid;

AConfiguration_diff :: (config1: *AConfiguration, config2: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_fromAssetManager :: (out: *AConfiguration, am: *AAssetManager) -> void #foreign libandroid;

AConfiguration_getCountry :: (config: *AConfiguration, outCountry: *u8) -> void #foreign libandroid;

AConfiguration_getDensity :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getKeyboard :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getKeysHidden :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getLanguage :: (config: *AConfiguration, outLanguage: *u8) -> void #foreign libandroid;

AConfiguration_getLayoutDirection :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getMcc :: (config: *AConfiguration) -> s32 #foreign libandroid;

AConfiguration_getMnc :: (config: *AConfiguration) -> s32 #foreign libandroid;

AConfiguration_getNavHidden :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getNavigation :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getOrientation :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getScreenHeightDp :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getScreenLong :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getScreenRound :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getScreenSize :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getScreenWidthDp :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getSdkVersion :: (config: *AConfiguration) -> s32 #foreign libandroid;

AConfiguration_getSmallestScreenWidthDp :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getTouchscreen :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getUiModeNight :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_getUiModeType :: (config: *AConfiguration) -> ACONFIGURATION #foreign libandroid;

AConfiguration_isBetterThan :: (base: *AConfiguration, test: *AConfiguration, requested: *AConfiguration) -> s32 #foreign libandroid;

AConfiguration_match :: (base: *AConfiguration, requested: *AConfiguration) -> s32 #foreign libandroid;

AConfiguration_new :: (__args: ..Any) -> *AConfiguration #foreign libandroid;

AConfiguration_setCountry :: (config: *AConfiguration, country: *u8) -> void #foreign libandroid;

AConfiguration_setDensity :: (config: *AConfiguration, density: s32) -> void #foreign libandroid;

AConfiguration_setKeyboard :: (config: *AConfiguration, keyboard: s32) -> void #foreign libandroid;

AConfiguration_setKeysHidden :: (config: *AConfiguration, keysHidden: s32) -> void #foreign libandroid;

AConfiguration_setLanguage :: (config: *AConfiguration, language: *u8) -> void #foreign libandroid;

AConfiguration_setLayoutDirection :: (config: *AConfiguration, value: s32) -> void #foreign libandroid;

AConfiguration_setMcc :: (config: *AConfiguration, mcc: s32) -> void #foreign libandroid;

AConfiguration_setMnc :: (config: *AConfiguration, mnc: s32) -> void #foreign libandroid;

AConfiguration_setNavHidden :: (config: *AConfiguration, navHidden: s32) -> void #foreign libandroid;

AConfiguration_setNavigation :: (config: *AConfiguration, navigation: s32) -> void #foreign libandroid;

AConfiguration_setOrientation :: (config: *AConfiguration, orientation: s32) -> void #foreign libandroid;

AConfiguration_setScreenHeightDp :: (config: *AConfiguration, value: s32) -> void #foreign libandroid;

AConfiguration_setScreenLong :: (config: *AConfiguration, screenLong: s32) -> void #foreign libandroid;

AConfiguration_setScreenSize :: (config: *AConfiguration, screenSize: s32) -> void #foreign libandroid;

AConfiguration_setScreenWidthDp :: (config: *AConfiguration, value: s32) -> void #foreign libandroid;

AConfiguration_setSdkVersion :: (config: *AConfiguration, sdkVersion: s32) -> void #foreign libandroid;

AConfiguration_setSmallestScreenWidthDp :: (config: *AConfiguration, value: s32) -> void #foreign libandroid;

AConfiguration_setTouchscreen :: (config: *AConfiguration, touchscreen: s32) -> void #foreign libandroid;

AConfiguration_setUiModeNight :: (config: *AConfiguration, uiModeNight: s32) -> void #foreign libandroid;

AConfiguration_setUiModeType :: (config: *AConfiguration, uiModeType: s32) -> void #foreign libandroid;

AHardwareBuffer_acquire :: (buffer: *AHardwareBuffer) -> void #foreign libandroid;

AHardwareBuffer_allocate :: (desc: *AHardwareBuffer_Desc, outBuffer: **AHardwareBuffer) -> s32 #foreign libandroid;

AHardwareBuffer_describe :: (buffer: *AHardwareBuffer, outDesc: *AHardwareBuffer_Desc) -> void #foreign libandroid;

AHardwareBuffer_getId :: (buffer: *AHardwareBuffer, outId: *u64) -> s32 #foreign libandroid;

AHardwareBuffer_isSupported :: (desc: *AHardwareBuffer_Desc) -> s32 #foreign libandroid;

AHardwareBuffer_lock :: (buffer: *AHardwareBuffer, usage: u64, fence: s32, rect: *ARect, outVirtualAddress: **void) -> s32 #foreign libandroid;

AHardwareBuffer_lockAndGetInfo :: (buffer: *AHardwareBuffer, usage: u64, fence: s32, rect: *ARect, outVirtualAddress: **void, outBytesPerPixel: *s32, outBytesPerStride: *s32) -> s32 #foreign libandroid;

AHardwareBuffer_lockPlanes :: (buffer: *AHardwareBuffer, usage: u64, fence: s32, rect: *ARect, outPlanes: *AHardwareBuffer_Planes) -> s32 #foreign libandroid;

AHardwareBuffer_recvHandleFromUnixSocket :: (socketFd: s32, outBuffer: **AHardwareBuffer) -> s32 #foreign libandroid;

AHardwareBuffer_release :: (buffer: *AHardwareBuffer) -> void #foreign libandroid;

AHardwareBuffer_sendHandleToUnixSocket :: (buffer: *AHardwareBuffer, socketFd: s32) -> s32 #foreign libandroid;

AHardwareBuffer_unlock :: (buffer: *AHardwareBuffer, fence: *s32) -> s32 #foreign libandroid;

#if INCLUDE_MEDIA_BINDINGS {
    AImage_delete :: (image: *AImage) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_deleteAsync :: (image: *AImage, releaseFenceFd: s32) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getCropRect :: (image: *AImage, rect: *AImageCropRect) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getFormat :: (image: *AImage, format: *AIMAGE_FORMAT) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getHardwareBuffer :: (image: *AImage, buffer: **AHardwareBuffer) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getHeight :: (image: *AImage, height: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getNumberOfPlanes :: (image: *AImage, numPlanes: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getPlaneData :: (image: *AImage, planeIdx: s32, data: **u8, dataLength: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getPlanePixelStride :: (image: *AImage, planeIdx: s32, pixelStride: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getPlaneRowStride :: (image: *AImage, planeIdx: s32, rowStride: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getTimestamp :: (image: *AImage, timestampNs: *s64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImage_getWidth :: (image: *AImage, width: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_acquireLatestImage :: (reader: *AImageReader, image: **AImage) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_acquireLatestImageAsync :: (reader: *AImageReader, image: **AImage, acquireFenceFd: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_acquireNextImage :: (reader: *AImageReader, image: **AImage) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_acquireNextImageAsync :: (reader: *AImageReader, image: **AImage, acquireFenceFd: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_delete :: (reader: *AImageReader) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_getFormat :: (reader: *AImageReader, format: *AIMAGE_FORMAT) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_getHeight :: (reader: *AImageReader, height: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_getMaxImages :: (reader: *AImageReader, maxImages: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_getWidth :: (reader: *AImageReader, width: *s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_getWindow :: (reader: *AImageReader, window: **ACameraWindowType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_new :: (width: s32, height: s32, format: AIMAGE_FORMAT, maxImages: s32, reader: **AImageReader) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_newWithUsage :: (width: s32, height: s32, format: AIMAGE_FORMAT, usage: u64, maxImages: s32, reader: **AImageReader) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_setBufferRemovedListener :: (reader: *AImageReader, listener: *AImageReader_BufferRemovedListener) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AImageReader_setImageListener :: (reader: *AImageReader, listener: *AImageReader_ImageListener) -> media_status_t #foreign libmediandk;
}

AInputEvent_getDeviceId :: (event: *AInputEvent) -> s32 #foreign libandroid;

AInputEvent_getSource :: (event: *AInputEvent) -> s32 #foreign libandroid;

AInputEvent_getType :: (event: *AInputEvent) -> AINPUT_EVENT_TYPE #foreign libandroid;

AInputEvent_release :: (event: *AInputEvent) -> void #foreign libandroid;

AInputQueue_attachLooper :: (queue: *AInputQueue, looper: *ALooper, ident: s32, callback: ALooper_callbackFunc, data: *void) -> void #foreign libandroid;

AInputQueue_detachLooper :: (queue: *AInputQueue) -> void #foreign libandroid;

AInputQueue_finishEvent :: (queue: *AInputQueue, event: *AInputEvent, handled: s32) -> void #foreign libandroid;

AInputQueue_fromJava :: (env: *JNIEnv, inputQueue: jobject) -> *AInputQueue #foreign libandroid;

AInputQueue_getEvent :: (queue: *AInputQueue, outEvent: **AInputEvent) -> s32 #foreign libandroid;

AInputQueue_hasEvents :: (queue: *AInputQueue) -> s32 #foreign libandroid;

AInputQueue_preDispatchEvent :: (queue: *AInputQueue, event: *AInputEvent) -> s32 #foreign libandroid;

AKeyEvent_fromJava :: (env: *JNIEnv, keyEvent: jobject) -> *AInputEvent #foreign libandroid;

AKeyEvent_getAction :: (key_event: *AInputEvent) -> AKEY_EVENT_ACTION #foreign libandroid;

AKeyEvent_getDownTime :: (key_event: *AInputEvent) -> s64 #foreign libandroid;

AKeyEvent_getEventTime :: (key_event: *AInputEvent) -> s64 #foreign libandroid;

AKeyEvent_getFlags :: (key_event: *AInputEvent) -> AKEY_EVENT_FLAG #foreign libandroid;

AKeyEvent_getKeyCode :: (key_event: *AInputEvent) -> AKEYCODE #foreign libandroid;

AKeyEvent_getMetaState :: (key_event: *AInputEvent) -> AMETA #foreign libandroid;

AKeyEvent_getRepeatCount :: (key_event: *AInputEvent) -> s32 #foreign libandroid;

AKeyEvent_getScanCode :: (key_event: *AInputEvent) -> s32 #foreign libandroid;

ALooper_acquire :: (looper: *ALooper) -> void #foreign libandroid;

ALooper_addFd :: (looper: *ALooper, fd: s32, ident: s32, events: s32, callback: ALooper_callbackFunc, data: *void) -> s32 #foreign libandroid;

ALooper_forThread :: (__args: ..Any) -> *ALooper #foreign libandroid;

ALooper_pollAll :: (timeoutMillis: s32, outFd: *s32, outEvents: *s32, outData: **void) -> s32 #foreign libandroid;

ALooper_pollOnce :: (timeoutMillis: s32, outFd: *s32, outEvents: *s32, outData: **void) -> s32 #foreign libandroid;

ALooper_prepare :: (opts: s32) -> *ALooper #foreign libandroid;

ALooper_release :: (looper: *ALooper) -> void #foreign libandroid;

ALooper_removeFd :: (looper: *ALooper, fd: s32) -> s32 #foreign libandroid;

ALooper_wake :: (looper: *ALooper) -> void #foreign libandroid;

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_configure :: (unknown0: *AMediaCodec, format: *AMediaFormat, surface: *ACameraWindowType, crypto: *AMediaCrypto, flags: u32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_createCodecByName :: (name: *u8) -> *AMediaCodec #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_createDecoderByType :: (mime_type: *u8) -> *AMediaCodec #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_createEncoderByType :: (mime_type: *u8) -> *AMediaCodec #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_createInputSurface :: (mData: *AMediaCodec, surface: **ACameraWindowType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_createPersistentInputSurface :: (surface: **ACameraWindowType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_delete :: (unknown0: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_dequeueInputBuffer :: (unknown0: *AMediaCodec, timeoutUs: s64) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_dequeueOutputBuffer :: (unknown0: *AMediaCodec, info: *AMediaCodecBufferInfo, timeoutUs: s64) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_flush :: (unknown0: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getBufferFormat :: (unknown0: *AMediaCodec, index: u64) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getInputBuffer :: (unknown0: *AMediaCodec, idx: u64, out_size: *u64) -> *u8 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getInputFormat :: (unknown0: *AMediaCodec) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getName :: (unknown0: *AMediaCodec, out_name: **u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getOutputBuffer :: (unknown0: *AMediaCodec, idx: u64, out_size: *u64) -> *u8 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_getOutputFormat :: (unknown0: *AMediaCodec) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_queueInputBuffer :: (unknown0: *AMediaCodec, idx: u64, offset: s64, size: u64, time: u64, flags: u32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_queueSecureInputBuffer :: (unknown0: *AMediaCodec, idx: u64, offset: s64, unknown1: *AMediaCodecCryptoInfo, time: u64, flags: u32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_releaseCrypto :: (unknown0: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_releaseName :: (unknown0: *AMediaCodec, name: *u8) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_releaseOutputBuffer :: (unknown0: *AMediaCodec, idx: u64, render: bool) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_releaseOutputBufferAtTime :: (mData: *AMediaCodec, idx: u64, timestampNs: s64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_setAsyncNotifyCallback :: (unknown0: *AMediaCodec, callback: AMediaCodecOnAsyncNotifyCallback, userdata: *void) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_setInputSurface :: (mData: *AMediaCodec, surface: *ACameraWindowType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_setOnFrameRenderedCallback :: (unknown0: *AMediaCodec, callback: AMediaCodecOnFrameRendered, userdata: *void) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_setOutputSurface :: (unknown0: *AMediaCodec, surface: *ACameraWindowType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_setParameters :: (mData: *AMediaCodec, params: *AMediaFormat) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_signalEndOfInputStream :: (mData: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_start :: (unknown0: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodec_stop :: (unknown0: *AMediaCodec) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecActionCode_isRecoverable :: (actionCode: s32) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecActionCode_isTransient :: (actionCode: s32) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_delete :: (unknown0: *AMediaCodecCryptoInfo) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getClearBytes :: (unknown0: *AMediaCodecCryptoInfo, dst: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getEncryptedBytes :: (unknown0: *AMediaCodecCryptoInfo, dst: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getIV :: (unknown0: *AMediaCodecCryptoInfo, dst: *u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getKey :: (unknown0: *AMediaCodecCryptoInfo, dst: *u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getMode :: (unknown0: *AMediaCodecCryptoInfo) -> cryptoinfo_mode_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_getNumSubSamples :: (unknown0: *AMediaCodecCryptoInfo) -> u64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_new :: (numsubsamples: s32, key: *[16] u8, iv: *[16] u8, mode: cryptoinfo_mode_t, clearbytes: *u64, encryptedbytes: *u64) -> *AMediaCodecCryptoInfo #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCodecCryptoInfo_setPattern :: (info: *AMediaCodecCryptoInfo, pattern: *cryptoinfo_pattern_t) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCrypto_delete :: (crypto: *AMediaCrypto) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCrypto_isCryptoSchemeSupported :: (uuid: *AMediaUUID) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCrypto_new :: (uuid: *AMediaUUID, initData: *void, initDataSize: u64) -> *AMediaCrypto #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaCrypto_requiresSecureDecoderComponent :: (mime: *u8) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_close :: (unknown0: *AMediaDataSource) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_delete :: (unknown0: *AMediaDataSource) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_new :: (__args: ..Any) -> *AMediaDataSource #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_newUri :: (uri: *u8, numheaders: s32, key_values: **u8) -> *AMediaDataSource #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_setClose :: (unknown0: *AMediaDataSource, unknown1: AMediaDataSourceClose) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_setGetAvailableSize :: (unknown0: *AMediaDataSource, unknown1: AMediaDataSourceGetAvailableSize) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_setGetSize :: (unknown0: *AMediaDataSource, unknown1: AMediaDataSourceGetSize) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_setReadAt :: (unknown0: *AMediaDataSource, unknown1: AMediaDataSourceReadAt) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDataSource_setUserdata :: (unknown0: *AMediaDataSource, userdata: *void) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_closeSession :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_createByUUID :: (uuid: *u8) -> *AMediaDrm #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_decrypt :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, cipherAlgorithm: *u8, keyId: *u8, iv: *u8, input: *u8, output: *u8, dataSize: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_encrypt :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, cipherAlgorithm: *u8, keyId: *u8, iv: *u8, input: *u8, output: *u8, dataSize: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getKeyRequest :: (unknown0: *AMediaDrm, scope: *AMediaDrmScope, init: *u8, initSize: u64, mimeType: *u8, keyType: AMediaDrmKeyType, optionalParameters: *AMediaDrmKeyValue, numOptionalParameters: u64, keyRequest: **u8, keyRequestSize: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getKeyRequestWithDefaultUrlAndType :: (unknown0: *AMediaDrm, scope: *AMediaDrmScope, init: *u8, initSize: u64, mimeType: *u8, keyType: AMediaDrmKeyType, optionalParameters: *AMediaDrmKeyValue, numOptionalParameters: u64, keyRequest: **u8, keyRequestSize: *u64, defaultUrl: **u8, keyRequestType: *AMediaDrmKeyRequestType) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getPropertyByteArray :: (unknown0: *AMediaDrm, propertyName: *u8, propertyValue: *AMediaDrmByteArray) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getPropertyString :: (unknown0: *AMediaDrm, propertyName: *u8, propertyValue: **u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getProvisionRequest :: (unknown0: *AMediaDrm, provisionRequest: **u8, provisionRequestSize: *u64, serverUrl: **u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_getSecureStops :: (unknown0: *AMediaDrm, secureStops: *AMediaDrmSecureStop, numSecureStops: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_isCryptoSchemeSupported :: (uuid: *u8, mimeType: *u8) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_openSession :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_provideKeyResponse :: (unknown0: *AMediaDrm, scope: *AMediaDrmScope, response: *u8, responseSize: u64, keySetId: *AMediaDrmKeySetId) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_provideProvisionResponse :: (unknown0: *AMediaDrm, response: *u8, responseSize: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_queryKeyStatus :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, keyValuePairs: *AMediaDrmKeyValue, numPairs: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_release :: (unknown0: *AMediaDrm) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_releaseSecureStops :: (unknown0: *AMediaDrm, ssRelease: *AMediaDrmSecureStop) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_removeKeys :: (unknown0: *AMediaDrm, keySetId: *AMediaDrmSessionId) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_restoreKeys :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, keySetId: *AMediaDrmKeySetId) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_setOnEventListener :: (unknown0: *AMediaDrm, listener: AMediaDrmEventListener) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_setOnExpirationUpdateListener :: (unknown0: *AMediaDrm, listener: AMediaDrmExpirationUpdateListener) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_setOnKeysChangeListener :: (unknown0: *AMediaDrm, listener: AMediaDrmKeysChangeListener) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_setPropertyByteArray :: (unknown0: *AMediaDrm, propertyName: *u8, value: *u8, valueSize: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_setPropertyString :: (unknown0: *AMediaDrm, propertyName: *u8, value: *u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_sign :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, macAlgorithm: *u8, keyId: *u8, message: *u8, messageSize: u64, signature: *u8, signatureSize: *u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaDrm_verify :: (unknown0: *AMediaDrm, sessionId: *AMediaDrmSessionId, macAlgorithm: *u8, keyId: *u8, message: *u8, messageSize: u64, signature: *u8, signatureSize: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_advance :: (unknown0: *AMediaExtractor) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_delete :: (unknown0: *AMediaExtractor) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getCachedDuration :: (unknown0: *AMediaExtractor) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getFileFormat :: (unknown0: *AMediaExtractor) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getPsshInfo :: (unknown0: *AMediaExtractor) -> *PsshInfo #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleCryptoInfo :: (unknown0: *AMediaExtractor) -> *AMediaCodecCryptoInfo #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleFlags :: (unknown0: *AMediaExtractor) -> u32 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleFormat :: (ex: *AMediaExtractor, fmt: *AMediaFormat) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleSize :: (unknown0: *AMediaExtractor) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleTime :: (unknown0: *AMediaExtractor) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getSampleTrackIndex :: (unknown0: *AMediaExtractor) -> s32 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getTrackCount :: (unknown0: *AMediaExtractor) -> u64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_getTrackFormat :: (unknown0: *AMediaExtractor, idx: u64) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_new :: (__args: ..Any) -> *AMediaExtractor #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_readSampleData :: (unknown0: *AMediaExtractor, buffer: *u8, capacity: u64) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_seekTo :: (unknown0: *AMediaExtractor, seekPosUs: s64, mode: SeekMode) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_selectTrack :: (unknown0: *AMediaExtractor, idx: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_setDataSource :: (unknown0: *AMediaExtractor, location: *u8) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_setDataSourceCustom :: (unknown0: *AMediaExtractor, src: *AMediaDataSource) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_setDataSourceFd :: (unknown0: *AMediaExtractor, fd: s32, offset: s64, length: s64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaExtractor_unselectTrack :: (unknown0: *AMediaExtractor, idx: u64) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_clear :: (unknown0: *AMediaFormat) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_copy :: (to: *AMediaFormat, from: *AMediaFormat) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_delete :: (unknown0: *AMediaFormat) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getBuffer :: (unknown0: *AMediaFormat, name: *u8, data: **void, size: *u64) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getDouble :: (unknown0: *AMediaFormat, name: *u8, out: *float64) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getFloat :: (unknown0: *AMediaFormat, name: *u8, out: *float) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getInt32 :: (unknown0: *AMediaFormat, name: *u8, out: *s32) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getInt64 :: (unknown0: *AMediaFormat, name: *u8, out: *s64) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getRect :: (unknown0: *AMediaFormat, name: *u8, left: *s32, top: *s32, right: *s32, bottom: *s32) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getSize :: (unknown0: *AMediaFormat, name: *u8, out: *u64) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_getString :: (unknown0: *AMediaFormat, name: *u8, out: **u8) -> bool #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_new :: (__args: ..Any) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setBuffer :: (unknown0: *AMediaFormat, name: *u8, data: *void, size: u64) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setDouble :: (unknown0: *AMediaFormat, name: *u8, value: float64) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setFloat :: (unknown0: *AMediaFormat, name: *u8, value: float) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setInt32 :: (unknown0: *AMediaFormat, name: *u8, value: s32) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setInt64 :: (unknown0: *AMediaFormat, name: *u8, value: s64) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setRect :: (unknown0: *AMediaFormat, name: *u8, left: s32, top: s32, right: s32, bottom: s32) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setSize :: (unknown0: *AMediaFormat, name: *u8, value: u64) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_setString :: (unknown0: *AMediaFormat, name: *u8, value: *u8) -> void #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaFormat_toString :: (unknown0: *AMediaFormat) -> *u8 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_addTrack :: (unknown0: *AMediaMuxer, format: *AMediaFormat) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_append :: (fd: s32, mode: AppendMode) -> *AMediaMuxer #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_delete :: (unknown0: *AMediaMuxer) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_getTrackCount :: (unknown0: *AMediaMuxer) -> s64 #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_getTrackFormat :: (muxer: *AMediaMuxer, idx: u64) -> *AMediaFormat #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_new :: (fd: s32, format: OutputFormat) -> *AMediaMuxer #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_setLocation :: (unknown0: *AMediaMuxer, latitude: float, longitude: float) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_setOrientationHint :: (unknown0: *AMediaMuxer, degrees: s32) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_start :: (unknown0: *AMediaMuxer) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_stop :: (unknown0: *AMediaMuxer) -> media_status_t #foreign libmediandk;
}

#if INCLUDE_MEDIA_BINDINGS {
    AMediaMuxer_writeSampleData :: (muxer: *AMediaMuxer, trackIdx: u64, data: *u8, info: *AMediaCodecBufferInfo) -> media_status_t #foreign libmediandk;
}

AMotionEvent_fromJava :: (env: *JNIEnv, motionEvent: jobject) -> *AInputEvent #foreign libandroid;

AMotionEvent_getAction :: (motion_event: *AInputEvent) -> AMOTION_EVENT_ACTION #foreign libandroid;

AMotionEvent_getActionButton :: (motion_event: *AInputEvent) -> AMOTION_EVENT_BUTTON #foreign libandroid;

AMotionEvent_getAxisValue :: (motion_event: *AInputEvent, axis: AMOTION_EVENT_AXIS, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getButtonState :: (motion_event: *AInputEvent) -> s32 #foreign libandroid;

AMotionEvent_getClassification :: (motion_event: *AInputEvent) -> s32 #foreign libandroid;

AMotionEvent_getDownTime :: (motion_event: *AInputEvent) -> s64 #foreign libandroid;

AMotionEvent_getEdgeFlags :: (motion_event: *AInputEvent) -> s32 #foreign libandroid;

AMotionEvent_getEventTime :: (motion_event: *AInputEvent) -> s64 #foreign libandroid;

AMotionEvent_getFlags :: (motion_event: *AInputEvent) -> s32 #foreign libandroid;

AMotionEvent_getHistoricalAxisValue :: (motion_event: *AInputEvent, axis: s32, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalEventTime :: (motion_event: *AInputEvent, history_index: u64) -> s64 #foreign libandroid;

AMotionEvent_getHistoricalOrientation :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalPressure :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalRawX :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalRawY :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalSize :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalToolMajor :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalToolMinor :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalTouchMajor :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalTouchMinor :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalX :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistoricalY :: (motion_event: *AInputEvent, pointer_index: u64, history_index: u64) -> float #foreign libandroid;

AMotionEvent_getHistorySize :: (motion_event: *AInputEvent) -> u64 #foreign libandroid;

AMotionEvent_getMetaState :: (motion_event: *AInputEvent) -> AMETA #foreign libandroid;

AMotionEvent_getOrientation :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getPointerCount :: (motion_event: *AInputEvent) -> u64 #foreign libandroid;

AMotionEvent_getPointerId :: (motion_event: *AInputEvent, pointer_index: u64) -> s32 #foreign libandroid;

AMotionEvent_getPressure :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getRawX :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getRawY :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getSize :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getToolMajor :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getToolMinor :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getToolType :: (motion_event: *AInputEvent, pointer_index: u64) -> AMOTION_EVENT_TOOL_TYPE #foreign libandroid;

AMotionEvent_getTouchMajor :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getTouchMinor :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getX :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getXOffset :: (motion_event: *AInputEvent) -> float #foreign libandroid;

AMotionEvent_getXPrecision :: (motion_event: *AInputEvent) -> float #foreign libandroid;

AMotionEvent_getY :: (motion_event: *AInputEvent, pointer_index: u64) -> float #foreign libandroid;

AMotionEvent_getYOffset :: (motion_event: *AInputEvent) -> float #foreign libandroid;

AMotionEvent_getYPrecision :: (motion_event: *AInputEvent) -> float #foreign libandroid;

ANativeActivity_finish :: (activity: *ANativeActivity) -> void #foreign libandroid;

ANativeActivity_hideSoftInput :: (activity: *ANativeActivity, flags: u32) -> void #foreign libandroid;

ANativeActivity_setWindowFlags :: (activity: *ANativeActivity, addFlags: u32, removeFlags: u32) -> void #foreign libandroid;

ANativeActivity_setWindowFormat :: (activity: *ANativeActivity, format: s32) -> void #foreign libandroid;

ANativeActivity_showSoftInput :: (activity: *ANativeActivity, flags: u32) -> void #foreign libandroid;

ANativeWindow_acquire :: (window: *ACameraWindowType) -> void #foreign libandroid;

ANativeWindow_fromSurface :: (env: *JNIEnv, surface: jobject) -> *ACameraWindowType #foreign libandroid;

ANativeWindow_getFormat :: (window: *ACameraWindowType) -> AHardwareBuffer_Format #foreign libandroid;

ANativeWindow_getHeight :: (window: *ACameraWindowType) -> s32 #foreign libandroid;

ANativeWindow_getWidth :: (window: *ACameraWindowType) -> s32 #foreign libandroid;

ANativeWindow_lock :: (window: *ACameraWindowType, outBuffer: *ANativeWindow_Buffer, inOutDirtyBounds: *ARect) -> s32 #foreign libandroid;

ANativeWindow_release :: (window: *ACameraWindowType) -> void #foreign libandroid;

ANativeWindow_setBuffersGeometry :: (window: *ACameraWindowType, width: s32, height: s32, format: AHardwareBuffer_Format) -> s32 #foreign libandroid;

ANativeWindow_toSurface :: (env: *JNIEnv, window: *ACameraWindowType) -> jobject #foreign libandroid;

ANativeWindow_unlockAndPost :: (window: *ACameraWindowType) -> s32 #foreign libandroid;

#if !USE_GLUE_NATIVE_PORT {
    android_app_post_exec_cmd :: (android_app: *Android_App, cmd: s8) -> void #foreign lib_glue;
}

#if !USE_GLUE_NATIVE_PORT {
    android_app_read_cmd :: (android_app: *Android_App) -> s8 #foreign lib_glue;
}

app_dummy :: (__args: ..Any) -> void #foreign lib_glue;
