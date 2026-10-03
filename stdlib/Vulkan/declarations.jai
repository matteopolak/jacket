// Generated from docs/specs/stdlib/modules/vulkan.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
Binding_Arrays :: #import "internal/bindings/arrays";

binding_abort :: () #foreign libc "abort";

#if OS == .WINDOWS {
    #import "Windows";
}

#if OS == .WINDOWS {
    binding_vkEnumeratePhysicalDeviceGroupsKHR :: (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #foreign libvulkan "vkEnumeratePhysicalDeviceGroupsKHR";
}

#if OS == .LINUX {
    binding_vkEnumeratePhysicalDeviceGroupsKHR :: (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #foreign libvulkan "vkEnumeratePhysicalDeviceGroupsKHR";
}

#if OS == .WINDOWS {
    binding_vkGetImageSparseMemoryRequirements2KHR :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #foreign libvulkan "vkGetImageSparseMemoryRequirements2KHR";
}

#if OS == .LINUX {
    binding_vkGetImageSparseMemoryRequirements2KHR :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #foreign libvulkan "vkGetImageSparseMemoryRequirements2KHR";
}

#if OS == .WINDOWS {
    binding_vkGetPastPresentationTimingGOOGLE :: (device: VkDevice, swapchain: VkSwapchainKHR, pPresentationTimingCount: *u32, pPresentationTimings: *VkPastPresentationTimingGOOGLE) -> VkResult #foreign libvulkan "vkGetPastPresentationTimingGOOGLE";
}

#if OS == .LINUX {
    binding_vkGetPastPresentationTimingGOOGLE :: (device: VkDevice, swapchain: VkSwapchainKHR, pPresentationTimingCount: *u32, pPresentationTimings: *VkPastPresentationTimingGOOGLE) -> VkResult #foreign libvulkan "vkGetPastPresentationTimingGOOGLE";
}

#if OS == .WINDOWS {
    binding_vkGetPhysicalDeviceQueueFamilyProperties2KHR :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #foreign libvulkan "vkGetPhysicalDeviceQueueFamilyProperties2KHR";
}

#if OS == .LINUX {
    binding_vkGetPhysicalDeviceQueueFamilyProperties2KHR :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #foreign libvulkan "vkGetPhysicalDeviceQueueFamilyProperties2KHR";
}

#if OS == .WINDOWS {
    binding_vkGetPhysicalDeviceSparseImageFormatProperties2KHR :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #foreign libvulkan "vkGetPhysicalDeviceSparseImageFormatProperties2KHR";
}

#if OS == .LINUX {
    binding_vkGetPhysicalDeviceSparseImageFormatProperties2KHR :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #foreign libvulkan "vkGetPhysicalDeviceSparseImageFormatProperties2KHR";
}

#if OS == .WINDOWS {
    binding_vkGetQueueCheckpointDataNV :: (queue: VkQueue, pCheckpointDataCount: *u32, pCheckpointData: *VkCheckpointDataNV) -> void #foreign libvulkan "vkGetQueueCheckpointDataNV";
}

#if OS == .LINUX {
    binding_vkGetQueueCheckpointDataNV :: (queue: VkQueue, pCheckpointDataCount: *u32, pCheckpointData: *VkCheckpointDataNV) -> void #foreign libvulkan "vkGetQueueCheckpointDataNV";
}

libc :: #library,system "c";
libvulkan :: #library,system "vulkan";
#scope_export

// Types and layouts

VK_Version :: struct {
    major, minor, patch: VkVersion;
}

#if OS == .WINDOWS {
    VkAccelerationStructureCreateInfoNVX :: struct {
        sType: VkStructureType = .ACCELERATION_STRUCTURE_CREATE_INFO_NVX;
        pNext: *void;
        type: VkAccelerationStructureTypeNVX;
        flags: VkBuildAccelerationStructureFlagsNVX;
        compactedSize: VkDeviceSize;
        instanceCount: u32;
        geometryCount: u32;
        pGeometries: *VkGeometryNVX;
    }

}

#if OS == .LINUX {
    VkAccelerationStructureCreateInfoNVX :: struct {
        sType: VkStructureType = .ACCELERATION_STRUCTURE_CREATE_INFO_NVX;
        pNext: *void;
        type: VkAccelerationStructureTypeNVX;
        flags: VkBuildAccelerationStructureFlagsNVX;
        compactedSize: VkDeviceSize;
        instanceCount: u32;
        geometryCount: u32;
        pGeometries: *VkGeometryNVX;
    }

}

#if OS == .WINDOWS {
    VkAccelerationStructureMemoryRequirementsInfoNVX :: struct {
        sType: VkStructureType = .ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX;
        pNext: *void;
        accelerationStructure: VkAccelerationStructureNVX;
    }

}

#if OS == .LINUX {
    VkAccelerationStructureMemoryRequirementsInfoNVX :: struct {
        sType: VkStructureType = .ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX;
        pNext: *void;
        accelerationStructure: VkAccelerationStructureNVX;
    }

}

#if OS == .WINDOWS {
    VkAccelerationStructureNVX_T :: struct {}

}

#if OS == .LINUX {
    VkAccelerationStructureNVX_T :: struct {}

}

#if OS == .WINDOWS {
    VkAccelerationStructureTypeNVX :: enum u32 {
        TOP_LEVEL_NVX :: 0;
        BOTTOM_LEVEL_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_ACCELERATION_STRUCTURE_TYPE_TOP_LEVEL_NVX :: TOP_LEVEL_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_BOTTOM_LEVEL_NVX :: BOTTOM_LEVEL_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkAccelerationStructureTypeNVX :: enum u32 {
        TOP_LEVEL_NVX :: 0;
        BOTTOM_LEVEL_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_ACCELERATION_STRUCTURE_TYPE_TOP_LEVEL_NVX :: TOP_LEVEL_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_BOTTOM_LEVEL_NVX :: BOTTOM_LEVEL_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_ACCELERATION_STRUCTURE_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkAccessFlagBits :: enum_flags u32 {
        INDIRECT_COMMAND_READ_BIT :: 0x1;
        INDEX_READ_BIT :: 0x2;
        VERTEX_ATTRIBUTE_READ_BIT :: 0x4;
        UNIFORM_READ_BIT :: 0x8;
        INPUT_ATTACHMENT_READ_BIT :: 0x10;
        SHADER_READ_BIT :: 0x20;
        SHADER_WRITE_BIT :: 0x40;
        COLOR_ATTACHMENT_READ_BIT :: 0x80;
        COLOR_ATTACHMENT_WRITE_BIT :: 0x100;
        DEPTH_STENCIL_ATTACHMENT_READ_BIT :: 0x200;
        DEPTH_STENCIL_ATTACHMENT_WRITE_BIT :: 0x400;
        TRANSFER_READ_BIT :: 0x800;
        TRANSFER_WRITE_BIT :: 0x1000;
        HOST_READ_BIT :: 0x2000;
        HOST_WRITE_BIT :: 0x4000;
        MEMORY_READ_BIT :: 0x8000;
        MEMORY_WRITE_BIT :: 0x10000;
        CONDITIONAL_RENDERING_READ_BIT_EXT :: 0x100000;
        COMMAND_PROCESS_READ_BIT_NVX :: 0x20000;
        COMMAND_PROCESS_WRITE_BIT_NVX :: 0x40000;
        COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT :: 0x80000;
        SHADING_RATE_IMAGE_READ_BIT_NV :: 0x800000;
        ACCELERATION_STRUCTURE_READ_BIT_NVX :: 0x200000;
        ACCELERATION_STRUCTURE_WRITE_BIT_NVX :: 0x400000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_ACCESS_INDIRECT_COMMAND_READ_BIT :: INDIRECT_COMMAND_READ_BIT;
        VK_ACCESS_INDEX_READ_BIT :: INDEX_READ_BIT;
        VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT :: VERTEX_ATTRIBUTE_READ_BIT;
        VK_ACCESS_UNIFORM_READ_BIT :: UNIFORM_READ_BIT;
        VK_ACCESS_INPUT_ATTACHMENT_READ_BIT :: INPUT_ATTACHMENT_READ_BIT;
        VK_ACCESS_SHADER_READ_BIT :: SHADER_READ_BIT;
        VK_ACCESS_SHADER_WRITE_BIT :: SHADER_WRITE_BIT;
        VK_ACCESS_COLOR_ATTACHMENT_READ_BIT :: COLOR_ATTACHMENT_READ_BIT;
        VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT :: COLOR_ATTACHMENT_WRITE_BIT;
        VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT :: DEPTH_STENCIL_ATTACHMENT_READ_BIT;
        VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT :: DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
        VK_ACCESS_TRANSFER_READ_BIT :: TRANSFER_READ_BIT;
        VK_ACCESS_TRANSFER_WRITE_BIT :: TRANSFER_WRITE_BIT;
        VK_ACCESS_HOST_READ_BIT :: HOST_READ_BIT;
        VK_ACCESS_HOST_WRITE_BIT :: HOST_WRITE_BIT;
        VK_ACCESS_MEMORY_READ_BIT :: MEMORY_READ_BIT;
        VK_ACCESS_MEMORY_WRITE_BIT :: MEMORY_WRITE_BIT;
        VK_ACCESS_CONDITIONAL_RENDERING_READ_BIT_EXT :: CONDITIONAL_RENDERING_READ_BIT_EXT;
        VK_ACCESS_COMMAND_PROCESS_READ_BIT_NVX :: COMMAND_PROCESS_READ_BIT_NVX;
        VK_ACCESS_COMMAND_PROCESS_WRITE_BIT_NVX :: COMMAND_PROCESS_WRITE_BIT_NVX;
        VK_ACCESS_COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT :: COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT;
        VK_ACCESS_SHADING_RATE_IMAGE_READ_BIT_NV :: SHADING_RATE_IMAGE_READ_BIT_NV;
        VK_ACCESS_ACCELERATION_STRUCTURE_READ_BIT_NVX :: ACCELERATION_STRUCTURE_READ_BIT_NVX;
        VK_ACCESS_ACCELERATION_STRUCTURE_WRITE_BIT_NVX :: ACCELERATION_STRUCTURE_WRITE_BIT_NVX;
        VK_ACCESS_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkAccessFlagBits :: enum_flags u32 {
        INDIRECT_COMMAND_READ_BIT :: 0x1;
        INDEX_READ_BIT :: 0x2;
        VERTEX_ATTRIBUTE_READ_BIT :: 0x4;
        UNIFORM_READ_BIT :: 0x8;
        INPUT_ATTACHMENT_READ_BIT :: 0x10;
        SHADER_READ_BIT :: 0x20;
        SHADER_WRITE_BIT :: 0x40;
        COLOR_ATTACHMENT_READ_BIT :: 0x80;
        COLOR_ATTACHMENT_WRITE_BIT :: 0x100;
        DEPTH_STENCIL_ATTACHMENT_READ_BIT :: 0x200;
        DEPTH_STENCIL_ATTACHMENT_WRITE_BIT :: 0x400;
        TRANSFER_READ_BIT :: 0x800;
        TRANSFER_WRITE_BIT :: 0x1000;
        HOST_READ_BIT :: 0x2000;
        HOST_WRITE_BIT :: 0x4000;
        MEMORY_READ_BIT :: 0x8000;
        MEMORY_WRITE_BIT :: 0x10000;
        CONDITIONAL_RENDERING_READ_BIT_EXT :: 0x100000;
        COMMAND_PROCESS_READ_BIT_NVX :: 0x20000;
        COMMAND_PROCESS_WRITE_BIT_NVX :: 0x40000;
        COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT :: 0x80000;
        SHADING_RATE_IMAGE_READ_BIT_NV :: 0x800000;
        ACCELERATION_STRUCTURE_READ_BIT_NVX :: 0x200000;
        ACCELERATION_STRUCTURE_WRITE_BIT_NVX :: 0x400000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_ACCESS_INDIRECT_COMMAND_READ_BIT :: INDIRECT_COMMAND_READ_BIT;
        VK_ACCESS_INDEX_READ_BIT :: INDEX_READ_BIT;
        VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT :: VERTEX_ATTRIBUTE_READ_BIT;
        VK_ACCESS_UNIFORM_READ_BIT :: UNIFORM_READ_BIT;
        VK_ACCESS_INPUT_ATTACHMENT_READ_BIT :: INPUT_ATTACHMENT_READ_BIT;
        VK_ACCESS_SHADER_READ_BIT :: SHADER_READ_BIT;
        VK_ACCESS_SHADER_WRITE_BIT :: SHADER_WRITE_BIT;
        VK_ACCESS_COLOR_ATTACHMENT_READ_BIT :: COLOR_ATTACHMENT_READ_BIT;
        VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT :: COLOR_ATTACHMENT_WRITE_BIT;
        VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT :: DEPTH_STENCIL_ATTACHMENT_READ_BIT;
        VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT :: DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
        VK_ACCESS_TRANSFER_READ_BIT :: TRANSFER_READ_BIT;
        VK_ACCESS_TRANSFER_WRITE_BIT :: TRANSFER_WRITE_BIT;
        VK_ACCESS_HOST_READ_BIT :: HOST_READ_BIT;
        VK_ACCESS_HOST_WRITE_BIT :: HOST_WRITE_BIT;
        VK_ACCESS_MEMORY_READ_BIT :: MEMORY_READ_BIT;
        VK_ACCESS_MEMORY_WRITE_BIT :: MEMORY_WRITE_BIT;
        VK_ACCESS_CONDITIONAL_RENDERING_READ_BIT_EXT :: CONDITIONAL_RENDERING_READ_BIT_EXT;
        VK_ACCESS_COMMAND_PROCESS_READ_BIT_NVX :: COMMAND_PROCESS_READ_BIT_NVX;
        VK_ACCESS_COMMAND_PROCESS_WRITE_BIT_NVX :: COMMAND_PROCESS_WRITE_BIT_NVX;
        VK_ACCESS_COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT :: COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT;
        VK_ACCESS_SHADING_RATE_IMAGE_READ_BIT_NV :: SHADING_RATE_IMAGE_READ_BIT_NV;
        VK_ACCESS_ACCELERATION_STRUCTURE_READ_BIT_NVX :: ACCELERATION_STRUCTURE_READ_BIT_NVX;
        VK_ACCESS_ACCELERATION_STRUCTURE_WRITE_BIT_NVX :: ACCELERATION_STRUCTURE_WRITE_BIT_NVX;
        VK_ACCESS_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkAcquireNextImageInfoKHR :: struct {
        sType: VkStructureType = .ACQUIRE_NEXT_IMAGE_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
        timeout: u64;
        semaphore: VkSemaphore;
        fence: VkFence;
        deviceMask: u32;
    }

}

#if OS == .LINUX {
    VkAcquireNextImageInfoKHR :: struct {
        sType: VkStructureType = .ACQUIRE_NEXT_IMAGE_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
        timeout: u64;
        semaphore: VkSemaphore;
        fence: VkFence;
        deviceMask: u32;
    }

}

#if OS == .WINDOWS {
    VkAllocationCallbacks :: struct {
        pUserData: *void;
        pfnAllocation: PFN_vkAllocationFunction;
        pfnReallocation: PFN_vkReallocationFunction;
        pfnFree: PFN_vkFreeFunction;
        pfnInternalAllocation: PFN_vkInternalAllocationNotification;
        pfnInternalFree: PFN_vkInternalFreeNotification;
    }

}

#if OS == .LINUX {
    VkAllocationCallbacks :: struct {
        pUserData: *void;
        pfnAllocation: PFN_vkAllocationFunction;
        pfnReallocation: PFN_vkReallocationFunction;
        pfnFree: PFN_vkFreeFunction;
        pfnInternalAllocation: PFN_vkInternalAllocationNotification;
        pfnInternalFree: PFN_vkInternalFreeNotification;
    }

}

#if OS == .WINDOWS {
    VkApplicationInfo :: struct {
        sType: VkStructureType = .APPLICATION_INFO;
        pNext: *void;
        pApplicationName: *u8;
        applicationVersion: u32;
        pEngineName: *u8;
        engineVersion: u32;
        apiVersion: u32;
    }

}

#if OS == .LINUX {
    VkApplicationInfo :: struct {
        sType: VkStructureType = .APPLICATION_INFO;
        pNext: *void;
        pApplicationName: *u8;
        applicationVersion: u32;
        pEngineName: *u8;
        engineVersion: u32;
        apiVersion: u32;
    }

}

#if OS == .WINDOWS {
    VkAttachmentDescription :: struct {
        flags: VkAttachmentDescriptionFlags;
        format: VkFormat;
        samples: VkSampleCountFlagBits;
        loadOp: VkAttachmentLoadOp;
        storeOp: VkAttachmentStoreOp;
        stencilLoadOp: VkAttachmentLoadOp;
        stencilStoreOp: VkAttachmentStoreOp;
        initialLayout: VkImageLayout;
        finalLayout: VkImageLayout;
    }

}

#if OS == .LINUX {
    VkAttachmentDescription :: struct {
        flags: VkAttachmentDescriptionFlags;
        format: VkFormat;
        samples: VkSampleCountFlagBits;
        loadOp: VkAttachmentLoadOp;
        storeOp: VkAttachmentStoreOp;
        stencilLoadOp: VkAttachmentLoadOp;
        stencilStoreOp: VkAttachmentStoreOp;
        initialLayout: VkImageLayout;
        finalLayout: VkImageLayout;
    }

}

#if OS == .WINDOWS {
    VkAttachmentDescription2KHR :: struct {
        sType: VkStructureType = .ATTACHMENT_DESCRIPTION_2_KHR;
        pNext: *void;
        flags: VkAttachmentDescriptionFlags;
        format: VkFormat;
        samples: VkSampleCountFlagBits;
        loadOp: VkAttachmentLoadOp;
        storeOp: VkAttachmentStoreOp;
        stencilLoadOp: VkAttachmentLoadOp;
        stencilStoreOp: VkAttachmentStoreOp;
        initialLayout: VkImageLayout;
        finalLayout: VkImageLayout;
    }

}

#if OS == .LINUX {
    VkAttachmentDescription2KHR :: struct {
        sType: VkStructureType = .ATTACHMENT_DESCRIPTION_2_KHR;
        pNext: *void;
        flags: VkAttachmentDescriptionFlags;
        format: VkFormat;
        samples: VkSampleCountFlagBits;
        loadOp: VkAttachmentLoadOp;
        storeOp: VkAttachmentStoreOp;
        stencilLoadOp: VkAttachmentLoadOp;
        stencilStoreOp: VkAttachmentStoreOp;
        initialLayout: VkImageLayout;
        finalLayout: VkImageLayout;
    }

}

#if OS == .WINDOWS {
    VkAttachmentDescriptionFlagBits :: enum_flags u32 {
        MAY_ALIAS_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_ATTACHMENT_DESCRIPTION_MAY_ALIAS_BIT :: MAY_ALIAS_BIT;
        VK_ATTACHMENT_DESCRIPTION_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkAttachmentDescriptionFlagBits :: enum_flags u32 {
        MAY_ALIAS_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_ATTACHMENT_DESCRIPTION_MAY_ALIAS_BIT :: MAY_ALIAS_BIT;
        VK_ATTACHMENT_DESCRIPTION_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkAttachmentLoadOp :: enum u32 {
        LOAD :: 0;
        CLEAR :: 1;
        DONT_CARE :: 2;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_ATTACHMENT_LOAD_OP_LOAD :: LOAD;
        VK_ATTACHMENT_LOAD_OP_CLEAR :: CLEAR;
        VK_ATTACHMENT_LOAD_OP_DONT_CARE :: DONT_CARE;
        VK_ATTACHMENT_LOAD_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_ATTACHMENT_LOAD_OP_END_RANGE :: END_RANGE;
        VK_ATTACHMENT_LOAD_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_ATTACHMENT_LOAD_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkAttachmentLoadOp :: enum u32 {
        LOAD :: 0;
        CLEAR :: 1;
        DONT_CARE :: 2;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_ATTACHMENT_LOAD_OP_LOAD :: LOAD;
        VK_ATTACHMENT_LOAD_OP_CLEAR :: CLEAR;
        VK_ATTACHMENT_LOAD_OP_DONT_CARE :: DONT_CARE;
        VK_ATTACHMENT_LOAD_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_ATTACHMENT_LOAD_OP_END_RANGE :: END_RANGE;
        VK_ATTACHMENT_LOAD_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_ATTACHMENT_LOAD_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkAttachmentReference :: struct {
        attachment: u32;
        layout: VkImageLayout;
    }

}

#if OS == .LINUX {
    VkAttachmentReference :: struct {
        attachment: u32;
        layout: VkImageLayout;
    }

}

#if OS == .WINDOWS {
    VkAttachmentReference2KHR :: struct {
        sType: VkStructureType = .ATTACHMENT_REFERENCE_2_KHR;
        pNext: *void;
        attachment: u32;
        layout: VkImageLayout;
        aspectMask: VkImageAspectFlags;
    }

}

#if OS == .LINUX {
    VkAttachmentReference2KHR :: struct {
        sType: VkStructureType = .ATTACHMENT_REFERENCE_2_KHR;
        pNext: *void;
        attachment: u32;
        layout: VkImageLayout;
        aspectMask: VkImageAspectFlags;
    }

}

#if OS == .WINDOWS {
    VkAttachmentSampleLocationsEXT :: struct {
        attachmentIndex: u32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .LINUX {
    VkAttachmentSampleLocationsEXT :: struct {
        attachmentIndex: u32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .WINDOWS {
    VkAttachmentStoreOp :: enum u32 {
        STORE :: 0;
        DONT_CARE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_ATTACHMENT_STORE_OP_STORE :: STORE;
        VK_ATTACHMENT_STORE_OP_DONT_CARE :: DONT_CARE;
        VK_ATTACHMENT_STORE_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_ATTACHMENT_STORE_OP_END_RANGE :: END_RANGE;
        VK_ATTACHMENT_STORE_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_ATTACHMENT_STORE_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkAttachmentStoreOp :: enum u32 {
        STORE :: 0;
        DONT_CARE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_ATTACHMENT_STORE_OP_STORE :: STORE;
        VK_ATTACHMENT_STORE_OP_DONT_CARE :: DONT_CARE;
        VK_ATTACHMENT_STORE_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_ATTACHMENT_STORE_OP_END_RANGE :: END_RANGE;
        VK_ATTACHMENT_STORE_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_ATTACHMENT_STORE_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBaseInStructure :: struct {
        sType: VkStructureType;
        pNext: *VkBaseInStructure;
    }

}

#if OS == .LINUX {
    VkBaseInStructure :: struct {
        sType: VkStructureType;
        pNext: *VkBaseInStructure;
    }

}

#if OS == .WINDOWS {
    VkBaseOutStructure :: struct {
        sType: VkStructureType;
        pNext: *VkBaseOutStructure;
    }

}

#if OS == .LINUX {
    VkBaseOutStructure :: struct {
        sType: VkStructureType;
        pNext: *VkBaseOutStructure;
    }

}

#if OS == .WINDOWS {
    VkBindAccelerationStructureMemoryInfoNVX :: struct {
        sType: VkStructureType = .BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX;
        pNext: *void;
        accelerationStructure: VkAccelerationStructureNVX;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
    }

}

#if OS == .LINUX {
    VkBindAccelerationStructureMemoryInfoNVX :: struct {
        sType: VkStructureType = .BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX;
        pNext: *void;
        accelerationStructure: VkAccelerationStructureNVX;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
    }

}

#if OS == .WINDOWS {
    VkBindBufferMemoryDeviceGroupInfo :: struct {
        sType: VkStructureType = .BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO;
        pNext: *void;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
    }

}

#if OS == .LINUX {
    VkBindBufferMemoryDeviceGroupInfo :: struct {
        sType: VkStructureType = .BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO;
        pNext: *void;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
    }

}

#if OS == .WINDOWS {
    VkBindBufferMemoryInfo :: struct {
        sType: VkStructureType = .BIND_BUFFER_MEMORY_INFO;
        pNext: *void;
        buffer: VkBuffer;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkBindBufferMemoryInfo :: struct {
        sType: VkStructureType = .BIND_BUFFER_MEMORY_INFO;
        pNext: *void;
        buffer: VkBuffer;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkBindImageMemoryDeviceGroupInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO;
        pNext: *void;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
        splitInstanceBindRegionCount: u32;
        pSplitInstanceBindRegions: *VkRect2D;
    }

}

#if OS == .LINUX {
    VkBindImageMemoryDeviceGroupInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO;
        pNext: *void;
        deviceIndexCount: u32;
        pDeviceIndices: *u32;
        splitInstanceBindRegionCount: u32;
        pSplitInstanceBindRegions: *VkRect2D;
    }

}

#if OS == .WINDOWS {
    VkBindImageMemoryInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_INFO;
        pNext: *void;
        image: VkImage;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkBindImageMemoryInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_INFO;
        pNext: *void;
        image: VkImage;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkBindImageMemorySwapchainInfoKHR :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
        imageIndex: u32;
    }

}

#if OS == .LINUX {
    VkBindImageMemorySwapchainInfoKHR :: struct {
        sType: VkStructureType = .BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
        imageIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkBindImagePlaneMemoryInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_PLANE_MEMORY_INFO;
        pNext: *void;
        planeAspect: VkImageAspectFlagBits;
    }

}

#if OS == .LINUX {
    VkBindImagePlaneMemoryInfo :: struct {
        sType: VkStructureType = .BIND_IMAGE_PLANE_MEMORY_INFO;
        pNext: *void;
        planeAspect: VkImageAspectFlagBits;
    }

}

#if OS == .WINDOWS {
    VkBindSparseInfo :: struct {
        sType: VkStructureType = .BIND_SPARSE_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        bufferBindCount: u32;
        pBufferBinds: *VkSparseBufferMemoryBindInfo;
        imageOpaqueBindCount: u32;
        pImageOpaqueBinds: *VkSparseImageOpaqueMemoryBindInfo;
        imageBindCount: u32;
        pImageBinds: *VkSparseImageMemoryBindInfo;
        signalSemaphoreCount: u32;
        pSignalSemaphores: *VkSemaphore;
    }

}

#if OS == .LINUX {
    VkBindSparseInfo :: struct {
        sType: VkStructureType = .BIND_SPARSE_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        bufferBindCount: u32;
        pBufferBinds: *VkSparseBufferMemoryBindInfo;
        imageOpaqueBindCount: u32;
        pImageOpaqueBinds: *VkSparseImageOpaqueMemoryBindInfo;
        imageBindCount: u32;
        pImageBinds: *VkSparseImageMemoryBindInfo;
        signalSemaphoreCount: u32;
        pSignalSemaphores: *VkSemaphore;
    }

}

#if OS == .WINDOWS {
    VkBlendFactor :: enum u32 {
        ZERO :: 0;
        ONE :: 1;
        SRC_COLOR :: 2;
        ONE_MINUS_SRC_COLOR :: 3;
        DST_COLOR :: 4;
        ONE_MINUS_DST_COLOR :: 5;
        SRC_ALPHA :: 6;
        ONE_MINUS_SRC_ALPHA :: 7;
        DST_ALPHA :: 8;
        ONE_MINUS_DST_ALPHA :: 9;
        CONSTANT_COLOR :: 10;
        ONE_MINUS_CONSTANT_COLOR :: 11;
        CONSTANT_ALPHA :: 12;
        ONE_MINUS_CONSTANT_ALPHA :: 13;
        SRC_ALPHA_SATURATE :: 14;
        SRC1_COLOR :: 15;
        ONE_MINUS_SRC1_COLOR :: 16;
        SRC1_ALPHA :: 17;
        ONE_MINUS_SRC1_ALPHA :: 18;
        BEGIN_RANGE :: 0;
        END_RANGE :: 18;
        RANGE_SIZE :: 19;
        MAX_ENUM :: 2147483647;

        VK_BLEND_FACTOR_ZERO :: ZERO;
        VK_BLEND_FACTOR_ONE :: ONE;
        VK_BLEND_FACTOR_SRC_COLOR :: SRC_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR :: ONE_MINUS_SRC_COLOR;
        VK_BLEND_FACTOR_DST_COLOR :: DST_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR :: ONE_MINUS_DST_COLOR;
        VK_BLEND_FACTOR_SRC_ALPHA :: SRC_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA :: ONE_MINUS_SRC_ALPHA;
        VK_BLEND_FACTOR_DST_ALPHA :: DST_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA :: ONE_MINUS_DST_ALPHA;
        VK_BLEND_FACTOR_CONSTANT_COLOR :: CONSTANT_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR :: ONE_MINUS_CONSTANT_COLOR;
        VK_BLEND_FACTOR_CONSTANT_ALPHA :: CONSTANT_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA :: ONE_MINUS_CONSTANT_ALPHA;
        VK_BLEND_FACTOR_SRC_ALPHA_SATURATE :: SRC_ALPHA_SATURATE;
        VK_BLEND_FACTOR_SRC1_COLOR :: SRC1_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR :: ONE_MINUS_SRC1_COLOR;
        VK_BLEND_FACTOR_SRC1_ALPHA :: SRC1_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA :: ONE_MINUS_SRC1_ALPHA;
        VK_BLEND_FACTOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BLEND_FACTOR_END_RANGE :: END_RANGE;
        VK_BLEND_FACTOR_RANGE_SIZE :: RANGE_SIZE;
        VK_BLEND_FACTOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkBlendFactor :: enum u32 {
        ZERO :: 0;
        ONE :: 1;
        SRC_COLOR :: 2;
        ONE_MINUS_SRC_COLOR :: 3;
        DST_COLOR :: 4;
        ONE_MINUS_DST_COLOR :: 5;
        SRC_ALPHA :: 6;
        ONE_MINUS_SRC_ALPHA :: 7;
        DST_ALPHA :: 8;
        ONE_MINUS_DST_ALPHA :: 9;
        CONSTANT_COLOR :: 10;
        ONE_MINUS_CONSTANT_COLOR :: 11;
        CONSTANT_ALPHA :: 12;
        ONE_MINUS_CONSTANT_ALPHA :: 13;
        SRC_ALPHA_SATURATE :: 14;
        SRC1_COLOR :: 15;
        ONE_MINUS_SRC1_COLOR :: 16;
        SRC1_ALPHA :: 17;
        ONE_MINUS_SRC1_ALPHA :: 18;
        BEGIN_RANGE :: 0;
        END_RANGE :: 18;
        RANGE_SIZE :: 19;
        MAX_ENUM :: 2147483647;

        VK_BLEND_FACTOR_ZERO :: ZERO;
        VK_BLEND_FACTOR_ONE :: ONE;
        VK_BLEND_FACTOR_SRC_COLOR :: SRC_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR :: ONE_MINUS_SRC_COLOR;
        VK_BLEND_FACTOR_DST_COLOR :: DST_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR :: ONE_MINUS_DST_COLOR;
        VK_BLEND_FACTOR_SRC_ALPHA :: SRC_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA :: ONE_MINUS_SRC_ALPHA;
        VK_BLEND_FACTOR_DST_ALPHA :: DST_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA :: ONE_MINUS_DST_ALPHA;
        VK_BLEND_FACTOR_CONSTANT_COLOR :: CONSTANT_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR :: ONE_MINUS_CONSTANT_COLOR;
        VK_BLEND_FACTOR_CONSTANT_ALPHA :: CONSTANT_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA :: ONE_MINUS_CONSTANT_ALPHA;
        VK_BLEND_FACTOR_SRC_ALPHA_SATURATE :: SRC_ALPHA_SATURATE;
        VK_BLEND_FACTOR_SRC1_COLOR :: SRC1_COLOR;
        VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR :: ONE_MINUS_SRC1_COLOR;
        VK_BLEND_FACTOR_SRC1_ALPHA :: SRC1_ALPHA;
        VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA :: ONE_MINUS_SRC1_ALPHA;
        VK_BLEND_FACTOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BLEND_FACTOR_END_RANGE :: END_RANGE;
        VK_BLEND_FACTOR_RANGE_SIZE :: RANGE_SIZE;
        VK_BLEND_FACTOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBlendOp :: enum u32 {
        ADD :: 0;
        SUBTRACT :: 1;
        REVERSE_SUBTRACT :: 2;
        MIN :: 3;
        MAX :: 4;
        ZERO_EXT :: 1000148000;
        SRC_EXT :: 1000148001;
        DST_EXT :: 1000148002;
        SRC_OVER_EXT :: 1000148003;
        DST_OVER_EXT :: 1000148004;
        SRC_IN_EXT :: 1000148005;
        DST_IN_EXT :: 1000148006;
        SRC_OUT_EXT :: 1000148007;
        DST_OUT_EXT :: 1000148008;
        SRC_ATOP_EXT :: 1000148009;
        DST_ATOP_EXT :: 1000148010;
        XOR_EXT :: 1000148011;
        MULTIPLY_EXT :: 1000148012;
        SCREEN_EXT :: 1000148013;
        OVERLAY_EXT :: 1000148014;
        DARKEN_EXT :: 1000148015;
        LIGHTEN_EXT :: 1000148016;
        COLORDODGE_EXT :: 1000148017;
        COLORBURN_EXT :: 1000148018;
        HARDLIGHT_EXT :: 1000148019;
        SOFTLIGHT_EXT :: 1000148020;
        DIFFERENCE_EXT :: 1000148021;
        EXCLUSION_EXT :: 1000148022;
        INVERT_EXT :: 1000148023;
        INVERT_RGB_EXT :: 1000148024;
        LINEARDODGE_EXT :: 1000148025;
        LINEARBURN_EXT :: 1000148026;
        VIVIDLIGHT_EXT :: 1000148027;
        LINEARLIGHT_EXT :: 1000148028;
        PINLIGHT_EXT :: 1000148029;
        HARDMIX_EXT :: 1000148030;
        HSL_HUE_EXT :: 1000148031;
        HSL_SATURATION_EXT :: 1000148032;
        HSL_COLOR_EXT :: 1000148033;
        HSL_LUMINOSITY_EXT :: 1000148034;
        PLUS_EXT :: 1000148035;
        PLUS_CLAMPED_EXT :: 1000148036;
        PLUS_CLAMPED_ALPHA_EXT :: 1000148037;
        PLUS_DARKER_EXT :: 1000148038;
        MINUS_EXT :: 1000148039;
        MINUS_CLAMPED_EXT :: 1000148040;
        CONTRAST_EXT :: 1000148041;
        INVERT_OVG_EXT :: 1000148042;
        RED_EXT :: 1000148043;
        GREEN_EXT :: 1000148044;
        BLUE_EXT :: 1000148045;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_BLEND_OP_ADD :: ADD;
        VK_BLEND_OP_SUBTRACT :: SUBTRACT;
        VK_BLEND_OP_REVERSE_SUBTRACT :: REVERSE_SUBTRACT;
        VK_BLEND_OP_MIN :: MIN;
        VK_BLEND_OP_MAX :: MAX;
        VK_BLEND_OP_ZERO_EXT :: ZERO_EXT;
        VK_BLEND_OP_SRC_EXT :: SRC_EXT;
        VK_BLEND_OP_DST_EXT :: DST_EXT;
        VK_BLEND_OP_SRC_OVER_EXT :: SRC_OVER_EXT;
        VK_BLEND_OP_DST_OVER_EXT :: DST_OVER_EXT;
        VK_BLEND_OP_SRC_IN_EXT :: SRC_IN_EXT;
        VK_BLEND_OP_DST_IN_EXT :: DST_IN_EXT;
        VK_BLEND_OP_SRC_OUT_EXT :: SRC_OUT_EXT;
        VK_BLEND_OP_DST_OUT_EXT :: DST_OUT_EXT;
        VK_BLEND_OP_SRC_ATOP_EXT :: SRC_ATOP_EXT;
        VK_BLEND_OP_DST_ATOP_EXT :: DST_ATOP_EXT;
        VK_BLEND_OP_XOR_EXT :: XOR_EXT;
        VK_BLEND_OP_MULTIPLY_EXT :: MULTIPLY_EXT;
        VK_BLEND_OP_SCREEN_EXT :: SCREEN_EXT;
        VK_BLEND_OP_OVERLAY_EXT :: OVERLAY_EXT;
        VK_BLEND_OP_DARKEN_EXT :: DARKEN_EXT;
        VK_BLEND_OP_LIGHTEN_EXT :: LIGHTEN_EXT;
        VK_BLEND_OP_COLORDODGE_EXT :: COLORDODGE_EXT;
        VK_BLEND_OP_COLORBURN_EXT :: COLORBURN_EXT;
        VK_BLEND_OP_HARDLIGHT_EXT :: HARDLIGHT_EXT;
        VK_BLEND_OP_SOFTLIGHT_EXT :: SOFTLIGHT_EXT;
        VK_BLEND_OP_DIFFERENCE_EXT :: DIFFERENCE_EXT;
        VK_BLEND_OP_EXCLUSION_EXT :: EXCLUSION_EXT;
        VK_BLEND_OP_INVERT_EXT :: INVERT_EXT;
        VK_BLEND_OP_INVERT_RGB_EXT :: INVERT_RGB_EXT;
        VK_BLEND_OP_LINEARDODGE_EXT :: LINEARDODGE_EXT;
        VK_BLEND_OP_LINEARBURN_EXT :: LINEARBURN_EXT;
        VK_BLEND_OP_VIVIDLIGHT_EXT :: VIVIDLIGHT_EXT;
        VK_BLEND_OP_LINEARLIGHT_EXT :: LINEARLIGHT_EXT;
        VK_BLEND_OP_PINLIGHT_EXT :: PINLIGHT_EXT;
        VK_BLEND_OP_HARDMIX_EXT :: HARDMIX_EXT;
        VK_BLEND_OP_HSL_HUE_EXT :: HSL_HUE_EXT;
        VK_BLEND_OP_HSL_SATURATION_EXT :: HSL_SATURATION_EXT;
        VK_BLEND_OP_HSL_COLOR_EXT :: HSL_COLOR_EXT;
        VK_BLEND_OP_HSL_LUMINOSITY_EXT :: HSL_LUMINOSITY_EXT;
        VK_BLEND_OP_PLUS_EXT :: PLUS_EXT;
        VK_BLEND_OP_PLUS_CLAMPED_EXT :: PLUS_CLAMPED_EXT;
        VK_BLEND_OP_PLUS_CLAMPED_ALPHA_EXT :: PLUS_CLAMPED_ALPHA_EXT;
        VK_BLEND_OP_PLUS_DARKER_EXT :: PLUS_DARKER_EXT;
        VK_BLEND_OP_MINUS_EXT :: MINUS_EXT;
        VK_BLEND_OP_MINUS_CLAMPED_EXT :: MINUS_CLAMPED_EXT;
        VK_BLEND_OP_CONTRAST_EXT :: CONTRAST_EXT;
        VK_BLEND_OP_INVERT_OVG_EXT :: INVERT_OVG_EXT;
        VK_BLEND_OP_RED_EXT :: RED_EXT;
        VK_BLEND_OP_GREEN_EXT :: GREEN_EXT;
        VK_BLEND_OP_BLUE_EXT :: BLUE_EXT;
        VK_BLEND_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BLEND_OP_END_RANGE :: END_RANGE;
        VK_BLEND_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_BLEND_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkBlendOp :: enum u32 {
        ADD :: 0;
        SUBTRACT :: 1;
        REVERSE_SUBTRACT :: 2;
        MIN :: 3;
        MAX :: 4;
        ZERO_EXT :: 1000148000;
        SRC_EXT :: 1000148001;
        DST_EXT :: 1000148002;
        SRC_OVER_EXT :: 1000148003;
        DST_OVER_EXT :: 1000148004;
        SRC_IN_EXT :: 1000148005;
        DST_IN_EXT :: 1000148006;
        SRC_OUT_EXT :: 1000148007;
        DST_OUT_EXT :: 1000148008;
        SRC_ATOP_EXT :: 1000148009;
        DST_ATOP_EXT :: 1000148010;
        XOR_EXT :: 1000148011;
        MULTIPLY_EXT :: 1000148012;
        SCREEN_EXT :: 1000148013;
        OVERLAY_EXT :: 1000148014;
        DARKEN_EXT :: 1000148015;
        LIGHTEN_EXT :: 1000148016;
        COLORDODGE_EXT :: 1000148017;
        COLORBURN_EXT :: 1000148018;
        HARDLIGHT_EXT :: 1000148019;
        SOFTLIGHT_EXT :: 1000148020;
        DIFFERENCE_EXT :: 1000148021;
        EXCLUSION_EXT :: 1000148022;
        INVERT_EXT :: 1000148023;
        INVERT_RGB_EXT :: 1000148024;
        LINEARDODGE_EXT :: 1000148025;
        LINEARBURN_EXT :: 1000148026;
        VIVIDLIGHT_EXT :: 1000148027;
        LINEARLIGHT_EXT :: 1000148028;
        PINLIGHT_EXT :: 1000148029;
        HARDMIX_EXT :: 1000148030;
        HSL_HUE_EXT :: 1000148031;
        HSL_SATURATION_EXT :: 1000148032;
        HSL_COLOR_EXT :: 1000148033;
        HSL_LUMINOSITY_EXT :: 1000148034;
        PLUS_EXT :: 1000148035;
        PLUS_CLAMPED_EXT :: 1000148036;
        PLUS_CLAMPED_ALPHA_EXT :: 1000148037;
        PLUS_DARKER_EXT :: 1000148038;
        MINUS_EXT :: 1000148039;
        MINUS_CLAMPED_EXT :: 1000148040;
        CONTRAST_EXT :: 1000148041;
        INVERT_OVG_EXT :: 1000148042;
        RED_EXT :: 1000148043;
        GREEN_EXT :: 1000148044;
        BLUE_EXT :: 1000148045;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_BLEND_OP_ADD :: ADD;
        VK_BLEND_OP_SUBTRACT :: SUBTRACT;
        VK_BLEND_OP_REVERSE_SUBTRACT :: REVERSE_SUBTRACT;
        VK_BLEND_OP_MIN :: MIN;
        VK_BLEND_OP_MAX :: MAX;
        VK_BLEND_OP_ZERO_EXT :: ZERO_EXT;
        VK_BLEND_OP_SRC_EXT :: SRC_EXT;
        VK_BLEND_OP_DST_EXT :: DST_EXT;
        VK_BLEND_OP_SRC_OVER_EXT :: SRC_OVER_EXT;
        VK_BLEND_OP_DST_OVER_EXT :: DST_OVER_EXT;
        VK_BLEND_OP_SRC_IN_EXT :: SRC_IN_EXT;
        VK_BLEND_OP_DST_IN_EXT :: DST_IN_EXT;
        VK_BLEND_OP_SRC_OUT_EXT :: SRC_OUT_EXT;
        VK_BLEND_OP_DST_OUT_EXT :: DST_OUT_EXT;
        VK_BLEND_OP_SRC_ATOP_EXT :: SRC_ATOP_EXT;
        VK_BLEND_OP_DST_ATOP_EXT :: DST_ATOP_EXT;
        VK_BLEND_OP_XOR_EXT :: XOR_EXT;
        VK_BLEND_OP_MULTIPLY_EXT :: MULTIPLY_EXT;
        VK_BLEND_OP_SCREEN_EXT :: SCREEN_EXT;
        VK_BLEND_OP_OVERLAY_EXT :: OVERLAY_EXT;
        VK_BLEND_OP_DARKEN_EXT :: DARKEN_EXT;
        VK_BLEND_OP_LIGHTEN_EXT :: LIGHTEN_EXT;
        VK_BLEND_OP_COLORDODGE_EXT :: COLORDODGE_EXT;
        VK_BLEND_OP_COLORBURN_EXT :: COLORBURN_EXT;
        VK_BLEND_OP_HARDLIGHT_EXT :: HARDLIGHT_EXT;
        VK_BLEND_OP_SOFTLIGHT_EXT :: SOFTLIGHT_EXT;
        VK_BLEND_OP_DIFFERENCE_EXT :: DIFFERENCE_EXT;
        VK_BLEND_OP_EXCLUSION_EXT :: EXCLUSION_EXT;
        VK_BLEND_OP_INVERT_EXT :: INVERT_EXT;
        VK_BLEND_OP_INVERT_RGB_EXT :: INVERT_RGB_EXT;
        VK_BLEND_OP_LINEARDODGE_EXT :: LINEARDODGE_EXT;
        VK_BLEND_OP_LINEARBURN_EXT :: LINEARBURN_EXT;
        VK_BLEND_OP_VIVIDLIGHT_EXT :: VIVIDLIGHT_EXT;
        VK_BLEND_OP_LINEARLIGHT_EXT :: LINEARLIGHT_EXT;
        VK_BLEND_OP_PINLIGHT_EXT :: PINLIGHT_EXT;
        VK_BLEND_OP_HARDMIX_EXT :: HARDMIX_EXT;
        VK_BLEND_OP_HSL_HUE_EXT :: HSL_HUE_EXT;
        VK_BLEND_OP_HSL_SATURATION_EXT :: HSL_SATURATION_EXT;
        VK_BLEND_OP_HSL_COLOR_EXT :: HSL_COLOR_EXT;
        VK_BLEND_OP_HSL_LUMINOSITY_EXT :: HSL_LUMINOSITY_EXT;
        VK_BLEND_OP_PLUS_EXT :: PLUS_EXT;
        VK_BLEND_OP_PLUS_CLAMPED_EXT :: PLUS_CLAMPED_EXT;
        VK_BLEND_OP_PLUS_CLAMPED_ALPHA_EXT :: PLUS_CLAMPED_ALPHA_EXT;
        VK_BLEND_OP_PLUS_DARKER_EXT :: PLUS_DARKER_EXT;
        VK_BLEND_OP_MINUS_EXT :: MINUS_EXT;
        VK_BLEND_OP_MINUS_CLAMPED_EXT :: MINUS_CLAMPED_EXT;
        VK_BLEND_OP_CONTRAST_EXT :: CONTRAST_EXT;
        VK_BLEND_OP_INVERT_OVG_EXT :: INVERT_OVG_EXT;
        VK_BLEND_OP_RED_EXT :: RED_EXT;
        VK_BLEND_OP_GREEN_EXT :: GREEN_EXT;
        VK_BLEND_OP_BLUE_EXT :: BLUE_EXT;
        VK_BLEND_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BLEND_OP_END_RANGE :: END_RANGE;
        VK_BLEND_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_BLEND_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBlendOverlapEXT :: enum u32 {
        UNCORRELATED_EXT :: 0;
        DISJOINT_EXT :: 1;
        CONJOINT_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_BLEND_OVERLAP_UNCORRELATED_EXT :: UNCORRELATED_EXT;
        VK_BLEND_OVERLAP_DISJOINT_EXT :: DISJOINT_EXT;
        VK_BLEND_OVERLAP_CONJOINT_EXT :: CONJOINT_EXT;
        VK_BLEND_OVERLAP_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_BLEND_OVERLAP_END_RANGE_EXT :: END_RANGE_EXT;
        VK_BLEND_OVERLAP_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_BLEND_OVERLAP_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkBlendOverlapEXT :: enum u32 {
        UNCORRELATED_EXT :: 0;
        DISJOINT_EXT :: 1;
        CONJOINT_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_BLEND_OVERLAP_UNCORRELATED_EXT :: UNCORRELATED_EXT;
        VK_BLEND_OVERLAP_DISJOINT_EXT :: DISJOINT_EXT;
        VK_BLEND_OVERLAP_CONJOINT_EXT :: CONJOINT_EXT;
        VK_BLEND_OVERLAP_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_BLEND_OVERLAP_END_RANGE_EXT :: END_RANGE_EXT;
        VK_BLEND_OVERLAP_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_BLEND_OVERLAP_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkBorderColor :: enum u32 {
        FLOAT_TRANSPARENT_BLACK :: 0;
        INT_TRANSPARENT_BLACK :: 1;
        FLOAT_OPAQUE_BLACK :: 2;
        INT_OPAQUE_BLACK :: 3;
        FLOAT_OPAQUE_WHITE :: 4;
        INT_OPAQUE_WHITE :: 5;
        BEGIN_RANGE :: 0;
        END_RANGE :: 5;
        RANGE_SIZE :: 6;
        MAX_ENUM :: 2147483647;

        VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK :: FLOAT_TRANSPARENT_BLACK;
        VK_BORDER_COLOR_INT_TRANSPARENT_BLACK :: INT_TRANSPARENT_BLACK;
        VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK :: FLOAT_OPAQUE_BLACK;
        VK_BORDER_COLOR_INT_OPAQUE_BLACK :: INT_OPAQUE_BLACK;
        VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE :: FLOAT_OPAQUE_WHITE;
        VK_BORDER_COLOR_INT_OPAQUE_WHITE :: INT_OPAQUE_WHITE;
        VK_BORDER_COLOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BORDER_COLOR_END_RANGE :: END_RANGE;
        VK_BORDER_COLOR_RANGE_SIZE :: RANGE_SIZE;
        VK_BORDER_COLOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkBorderColor :: enum u32 {
        FLOAT_TRANSPARENT_BLACK :: 0;
        INT_TRANSPARENT_BLACK :: 1;
        FLOAT_OPAQUE_BLACK :: 2;
        INT_OPAQUE_BLACK :: 3;
        FLOAT_OPAQUE_WHITE :: 4;
        INT_OPAQUE_WHITE :: 5;
        BEGIN_RANGE :: 0;
        END_RANGE :: 5;
        RANGE_SIZE :: 6;
        MAX_ENUM :: 2147483647;

        VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK :: FLOAT_TRANSPARENT_BLACK;
        VK_BORDER_COLOR_INT_TRANSPARENT_BLACK :: INT_TRANSPARENT_BLACK;
        VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK :: FLOAT_OPAQUE_BLACK;
        VK_BORDER_COLOR_INT_OPAQUE_BLACK :: INT_OPAQUE_BLACK;
        VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE :: FLOAT_OPAQUE_WHITE;
        VK_BORDER_COLOR_INT_OPAQUE_WHITE :: INT_OPAQUE_WHITE;
        VK_BORDER_COLOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_BORDER_COLOR_END_RANGE :: END_RANGE;
        VK_BORDER_COLOR_RANGE_SIZE :: RANGE_SIZE;
        VK_BORDER_COLOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBuffer_T :: struct {}

}

#if OS == .LINUX {
    VkBuffer_T :: struct {}

}

#if OS == .WINDOWS {
    VkBufferCopy :: struct {
        srcOffset: VkDeviceSize;
        dstOffset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkBufferCopy :: struct {
        srcOffset: VkDeviceSize;
        dstOffset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkBufferCreateFlagBits :: enum_flags u32 {
        SPARSE_BINDING_BIT :: 0x1;
        SPARSE_RESIDENCY_BIT :: 0x2;
        SPARSE_ALIASED_BIT :: 0x4;
        PROTECTED_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_BUFFER_CREATE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_BUFFER_CREATE_SPARSE_RESIDENCY_BIT :: SPARSE_RESIDENCY_BIT;
        VK_BUFFER_CREATE_SPARSE_ALIASED_BIT :: SPARSE_ALIASED_BIT;
        VK_BUFFER_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_BUFFER_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkBufferCreateFlagBits :: enum_flags u32 {
        SPARSE_BINDING_BIT :: 0x1;
        SPARSE_RESIDENCY_BIT :: 0x2;
        SPARSE_ALIASED_BIT :: 0x4;
        PROTECTED_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_BUFFER_CREATE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_BUFFER_CREATE_SPARSE_RESIDENCY_BIT :: SPARSE_RESIDENCY_BIT;
        VK_BUFFER_CREATE_SPARSE_ALIASED_BIT :: SPARSE_ALIASED_BIT;
        VK_BUFFER_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_BUFFER_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBufferCreateInfo :: struct {
        sType: VkStructureType = .BUFFER_CREATE_INFO;
        pNext: *void;
        flags: VkBufferCreateFlags;
        size: VkDeviceSize;
        usage: VkBufferUsageFlags;
        sharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
    }

}

#if OS == .LINUX {
    VkBufferCreateInfo :: struct {
        sType: VkStructureType = .BUFFER_CREATE_INFO;
        pNext: *void;
        flags: VkBufferCreateFlags;
        size: VkDeviceSize;
        usage: VkBufferUsageFlags;
        sharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
    }

}

#if OS == .WINDOWS {
    VkBufferImageCopy :: struct {
        bufferOffset: VkDeviceSize;
        bufferRowLength: u32;
        bufferImageHeight: u32;
        imageSubresource: VkImageSubresourceLayers;
        imageOffset: VkOffset3D;
        imageExtent: VkExtent3D;
    }

}

#if OS == .LINUX {
    VkBufferImageCopy :: struct {
        bufferOffset: VkDeviceSize;
        bufferRowLength: u32;
        bufferImageHeight: u32;
        imageSubresource: VkImageSubresourceLayers;
        imageOffset: VkOffset3D;
        imageExtent: VkExtent3D;
    }

}

#if OS == .WINDOWS {
    VkBufferMemoryBarrier :: struct {
        sType: VkStructureType = .BUFFER_MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        srcQueueFamilyIndex: u32;
        dstQueueFamilyIndex: u32;
        buffer: VkBuffer;
        offset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkBufferMemoryBarrier :: struct {
        sType: VkStructureType = .BUFFER_MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        srcQueueFamilyIndex: u32;
        dstQueueFamilyIndex: u32;
        buffer: VkBuffer;
        offset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkBufferMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .BUFFER_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        buffer: VkBuffer;
    }

}

#if OS == .LINUX {
    VkBufferMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .BUFFER_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        buffer: VkBuffer;
    }

}

#if OS == .WINDOWS {
    VkBufferUsageFlagBits :: enum_flags u32 {
        TRANSFER_SRC_BIT :: 0x1;
        TRANSFER_DST_BIT :: 0x2;
        UNIFORM_TEXEL_BUFFER_BIT :: 0x4;
        STORAGE_TEXEL_BUFFER_BIT :: 0x8;
        UNIFORM_BUFFER_BIT :: 0x10;
        STORAGE_BUFFER_BIT :: 0x20;
        INDEX_BUFFER_BIT :: 0x40;
        VERTEX_BUFFER_BIT :: 0x80;
        INDIRECT_BUFFER_BIT :: 0x100;
        CONDITIONAL_RENDERING_BIT_EXT :: 0x200;
        RAYTRACING_BIT_NVX :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_BUFFER_USAGE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_BUFFER_USAGE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT :: UNIFORM_TEXEL_BUFFER_BIT;
        VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT :: STORAGE_TEXEL_BUFFER_BIT;
        VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT :: UNIFORM_BUFFER_BIT;
        VK_BUFFER_USAGE_STORAGE_BUFFER_BIT :: STORAGE_BUFFER_BIT;
        VK_BUFFER_USAGE_INDEX_BUFFER_BIT :: INDEX_BUFFER_BIT;
        VK_BUFFER_USAGE_VERTEX_BUFFER_BIT :: VERTEX_BUFFER_BIT;
        VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT :: INDIRECT_BUFFER_BIT;
        VK_BUFFER_USAGE_CONDITIONAL_RENDERING_BIT_EXT :: CONDITIONAL_RENDERING_BIT_EXT;
        VK_BUFFER_USAGE_RAYTRACING_BIT_NVX :: RAYTRACING_BIT_NVX;
        VK_BUFFER_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkBufferUsageFlagBits :: enum_flags u32 {
        TRANSFER_SRC_BIT :: 0x1;
        TRANSFER_DST_BIT :: 0x2;
        UNIFORM_TEXEL_BUFFER_BIT :: 0x4;
        STORAGE_TEXEL_BUFFER_BIT :: 0x8;
        UNIFORM_BUFFER_BIT :: 0x10;
        STORAGE_BUFFER_BIT :: 0x20;
        INDEX_BUFFER_BIT :: 0x40;
        VERTEX_BUFFER_BIT :: 0x80;
        INDIRECT_BUFFER_BIT :: 0x100;
        CONDITIONAL_RENDERING_BIT_EXT :: 0x200;
        RAYTRACING_BIT_NVX :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_BUFFER_USAGE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_BUFFER_USAGE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT :: UNIFORM_TEXEL_BUFFER_BIT;
        VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT :: STORAGE_TEXEL_BUFFER_BIT;
        VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT :: UNIFORM_BUFFER_BIT;
        VK_BUFFER_USAGE_STORAGE_BUFFER_BIT :: STORAGE_BUFFER_BIT;
        VK_BUFFER_USAGE_INDEX_BUFFER_BIT :: INDEX_BUFFER_BIT;
        VK_BUFFER_USAGE_VERTEX_BUFFER_BIT :: VERTEX_BUFFER_BIT;
        VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT :: INDIRECT_BUFFER_BIT;
        VK_BUFFER_USAGE_CONDITIONAL_RENDERING_BIT_EXT :: CONDITIONAL_RENDERING_BIT_EXT;
        VK_BUFFER_USAGE_RAYTRACING_BIT_NVX :: RAYTRACING_BIT_NVX;
        VK_BUFFER_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkBufferView_T :: struct {}

}

#if OS == .LINUX {
    VkBufferView_T :: struct {}

}

#if OS == .WINDOWS {
    VkBufferViewCreateInfo :: struct {
        sType: VkStructureType = .BUFFER_VIEW_CREATE_INFO;
        pNext: *void;
        flags: VkBufferViewCreateFlags;
        buffer: VkBuffer;
        format: VkFormat;
        offset: VkDeviceSize;
        range: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkBufferViewCreateInfo :: struct {
        sType: VkStructureType = .BUFFER_VIEW_CREATE_INFO;
        pNext: *void;
        flags: VkBufferViewCreateFlags;
        buffer: VkBuffer;
        format: VkFormat;
        offset: VkDeviceSize;
        range: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkBuildAccelerationStructureFlagBitsNVX :: enum_flags u32 {
        ALLOW_UPDATE_BIT_NVX :: 0x1;
        ALLOW_COMPACTION_BIT_NVX :: 0x2;
        PREFER_FAST_TRACE_BIT_NVX :: 0x4;
        PREFER_FAST_BUILD_BIT_NVX :: 0x8;
        LOW_MEMORY_BIT_NVX :: 0x10;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_BUILD_ACCELERATION_STRUCTURE_ALLOW_UPDATE_BIT_NVX :: ALLOW_UPDATE_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_ALLOW_COMPACTION_BIT_NVX :: ALLOW_COMPACTION_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_PREFER_FAST_TRACE_BIT_NVX :: PREFER_FAST_TRACE_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_PREFER_FAST_BUILD_BIT_NVX :: PREFER_FAST_BUILD_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_LOW_MEMORY_BIT_NVX :: LOW_MEMORY_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkBuildAccelerationStructureFlagBitsNVX :: enum_flags u32 {
        ALLOW_UPDATE_BIT_NVX :: 0x1;
        ALLOW_COMPACTION_BIT_NVX :: 0x2;
        PREFER_FAST_TRACE_BIT_NVX :: 0x4;
        PREFER_FAST_BUILD_BIT_NVX :: 0x8;
        LOW_MEMORY_BIT_NVX :: 0x10;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_BUILD_ACCELERATION_STRUCTURE_ALLOW_UPDATE_BIT_NVX :: ALLOW_UPDATE_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_ALLOW_COMPACTION_BIT_NVX :: ALLOW_COMPACTION_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_PREFER_FAST_TRACE_BIT_NVX :: PREFER_FAST_TRACE_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_PREFER_FAST_BUILD_BIT_NVX :: PREFER_FAST_BUILD_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_LOW_MEMORY_BIT_NVX :: LOW_MEMORY_BIT_NVX;
        VK_BUILD_ACCELERATION_STRUCTURE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkCheckpointDataNV :: struct {
        sType: VkStructureType = .CHECKPOINT_DATA_NV;
        pNext: *void;
        stage: VkPipelineStageFlagBits;
        pCheckpointMarker: *void;
    }

}

#if OS == .LINUX {
    VkCheckpointDataNV :: struct {
        sType: VkStructureType = .CHECKPOINT_DATA_NV;
        pNext: *void;
        stage: VkPipelineStageFlagBits;
        pCheckpointMarker: *void;
    }

}

#if OS == .WINDOWS {
    VkChromaLocation :: enum u32 {
        COSITED_EVEN :: 0;
        MIDPOINT :: 1;
        COSITED_EVEN_KHR :: 0;
        MIDPOINT_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_CHROMA_LOCATION_COSITED_EVEN :: COSITED_EVEN;
        VK_CHROMA_LOCATION_MIDPOINT :: MIDPOINT;
        VK_CHROMA_LOCATION_COSITED_EVEN_KHR :: COSITED_EVEN_KHR;
        VK_CHROMA_LOCATION_MIDPOINT_KHR :: MIDPOINT_KHR;
        VK_CHROMA_LOCATION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_CHROMA_LOCATION_END_RANGE :: END_RANGE;
        VK_CHROMA_LOCATION_RANGE_SIZE :: RANGE_SIZE;
        VK_CHROMA_LOCATION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkChromaLocation :: enum u32 {
        COSITED_EVEN :: 0;
        MIDPOINT :: 1;
        COSITED_EVEN_KHR :: 0;
        MIDPOINT_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_CHROMA_LOCATION_COSITED_EVEN :: COSITED_EVEN;
        VK_CHROMA_LOCATION_MIDPOINT :: MIDPOINT;
        VK_CHROMA_LOCATION_COSITED_EVEN_KHR :: COSITED_EVEN_KHR;
        VK_CHROMA_LOCATION_MIDPOINT_KHR :: MIDPOINT_KHR;
        VK_CHROMA_LOCATION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_CHROMA_LOCATION_END_RANGE :: END_RANGE;
        VK_CHROMA_LOCATION_RANGE_SIZE :: RANGE_SIZE;
        VK_CHROMA_LOCATION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkClearAttachment :: struct {
        aspectMask: VkImageAspectFlags;
        colorAttachment: u32;
        clearValue: VkClearValue;
    }

}

#if OS == .LINUX {
    VkClearAttachment :: struct {
        aspectMask: VkImageAspectFlags;
        colorAttachment: u32;
        clearValue: VkClearValue;
    }

}

#if OS == .WINDOWS {
    VkClearColorValue :: struct {
        _float32: [4] float;

        #overlay (_float32) int32: [4] s32;
        #overlay (_float32) uint32: [4] u32;
    }

}

#if OS == .LINUX {
    VkClearColorValue :: struct {
        _float32: [4] float;

        #overlay (_float32) int32: [4] s32;
        #overlay (_float32) uint32: [4] u32;
    }

}

#if OS == .WINDOWS {
    VkClearDepthStencilValue :: struct {
        depth: float;
        stencil: u32;
    }

}

#if OS == .LINUX {
    VkClearDepthStencilValue :: struct {
        depth: float;
        stencil: u32;
    }

}

#if OS == .WINDOWS {
    VkClearRect :: struct {
        rect: VkRect2D;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .LINUX {
    VkClearRect :: struct {
        rect: VkRect2D;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .WINDOWS {
    VkClearValue :: struct {
        color: VkClearColorValue;

        #overlay (color) depthStencil: VkClearDepthStencilValue;
    }

}

#if OS == .LINUX {
    VkClearValue :: struct {
        color: VkClearColorValue;

        #overlay (color) depthStencil: VkClearDepthStencilValue;
    }

}

#if OS == .WINDOWS {
    VkCmdProcessCommandsInfoNVX :: struct {
        sType: VkStructureType = .CMD_PROCESS_COMMANDS_INFO_NVX;
        pNext: *void;
        objectTable: VkObjectTableNVX;
        indirectCommandsLayout: VkIndirectCommandsLayoutNVX;
        indirectCommandsTokenCount: u32;
        pIndirectCommandsTokens: *VkIndirectCommandsTokenNVX;
        maxSequencesCount: u32;
        targetCommandBuffer: VkCommandBuffer;
        sequencesCountBuffer: VkBuffer;
        sequencesCountOffset: VkDeviceSize;
        sequencesIndexBuffer: VkBuffer;
        sequencesIndexOffset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkCmdProcessCommandsInfoNVX :: struct {
        sType: VkStructureType = .CMD_PROCESS_COMMANDS_INFO_NVX;
        pNext: *void;
        objectTable: VkObjectTableNVX;
        indirectCommandsLayout: VkIndirectCommandsLayoutNVX;
        indirectCommandsTokenCount: u32;
        pIndirectCommandsTokens: *VkIndirectCommandsTokenNVX;
        maxSequencesCount: u32;
        targetCommandBuffer: VkCommandBuffer;
        sequencesCountBuffer: VkBuffer;
        sequencesCountOffset: VkDeviceSize;
        sequencesIndexBuffer: VkBuffer;
        sequencesIndexOffset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkCmdReserveSpaceForCommandsInfoNVX :: struct {
        sType: VkStructureType = .CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX;
        pNext: *void;
        objectTable: VkObjectTableNVX;
        indirectCommandsLayout: VkIndirectCommandsLayoutNVX;
        maxSequencesCount: u32;
    }

}

#if OS == .LINUX {
    VkCmdReserveSpaceForCommandsInfoNVX :: struct {
        sType: VkStructureType = .CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX;
        pNext: *void;
        objectTable: VkObjectTableNVX;
        indirectCommandsLayout: VkIndirectCommandsLayoutNVX;
        maxSequencesCount: u32;
    }

}

#if OS == .WINDOWS {
    VkCoarseSampleLocationNV :: struct {
        pixelX: u32;
        pixelY: u32;
        sample: u32;
    }

}

#if OS == .LINUX {
    VkCoarseSampleLocationNV :: struct {
        pixelX: u32;
        pixelY: u32;
        sample: u32;
    }

}

#if OS == .WINDOWS {
    VkCoarseSampleOrderCustomNV :: struct {
        shadingRate: VkShadingRatePaletteEntryNV;
        sampleCount: u32;
        sampleLocationCount: u32;
        pSampleLocations: *VkCoarseSampleLocationNV;
    }

}

#if OS == .LINUX {
    VkCoarseSampleOrderCustomNV :: struct {
        shadingRate: VkShadingRatePaletteEntryNV;
        sampleCount: u32;
        sampleLocationCount: u32;
        pSampleLocations: *VkCoarseSampleLocationNV;
    }

}

#if OS == .WINDOWS {
    VkCoarseSampleOrderTypeNV :: enum u32 {
        DEFAULT_NV :: 0;
        CUSTOM_NV :: 1;
        PIXEL_MAJOR_NV :: 2;
        SAMPLE_MAJOR_NV :: 3;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 3;
        RANGE_SIZE_NV :: 4;
        MAX_ENUM_NV :: 2147483647;

        VK_COARSE_SAMPLE_ORDER_TYPE_DEFAULT_NV :: DEFAULT_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_CUSTOM_NV :: CUSTOM_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_PIXEL_MAJOR_NV :: PIXEL_MAJOR_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_SAMPLE_MAJOR_NV :: SAMPLE_MAJOR_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_END_RANGE_NV :: END_RANGE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkCoarseSampleOrderTypeNV :: enum u32 {
        DEFAULT_NV :: 0;
        CUSTOM_NV :: 1;
        PIXEL_MAJOR_NV :: 2;
        SAMPLE_MAJOR_NV :: 3;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 3;
        RANGE_SIZE_NV :: 4;
        MAX_ENUM_NV :: 2147483647;

        VK_COARSE_SAMPLE_ORDER_TYPE_DEFAULT_NV :: DEFAULT_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_CUSTOM_NV :: CUSTOM_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_PIXEL_MAJOR_NV :: PIXEL_MAJOR_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_SAMPLE_MAJOR_NV :: SAMPLE_MAJOR_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_END_RANGE_NV :: END_RANGE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_COARSE_SAMPLE_ORDER_TYPE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkColorComponentFlagBits :: enum_flags u32 {
        R_BIT :: 0x1;
        G_BIT :: 0x2;
        B_BIT :: 0x4;
        A_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COLOR_COMPONENT_R_BIT :: R_BIT;
        VK_COLOR_COMPONENT_G_BIT :: G_BIT;
        VK_COLOR_COMPONENT_B_BIT :: B_BIT;
        VK_COLOR_COMPONENT_A_BIT :: A_BIT;
        VK_COLOR_COMPONENT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkColorComponentFlagBits :: enum_flags u32 {
        R_BIT :: 0x1;
        G_BIT :: 0x2;
        B_BIT :: 0x4;
        A_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COLOR_COMPONENT_R_BIT :: R_BIT;
        VK_COLOR_COMPONENT_G_BIT :: G_BIT;
        VK_COLOR_COMPONENT_B_BIT :: B_BIT;
        VK_COLOR_COMPONENT_A_BIT :: A_BIT;
        VK_COLOR_COMPONENT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkColorSpaceKHR :: enum u32 {
        COLOR_SPACE_SRGB_NONLINEAR_KHR :: 0;
        COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT :: 1000104001;
        COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT :: 1000104002;
        COLOR_SPACE_DCI_P3_LINEAR_EXT :: 1000104003;
        COLOR_SPACE_DCI_P3_NONLINEAR_EXT :: 1000104004;
        COLOR_SPACE_BT709_LINEAR_EXT :: 1000104005;
        COLOR_SPACE_BT709_NONLINEAR_EXT :: 1000104006;
        COLOR_SPACE_BT2020_LINEAR_EXT :: 1000104007;
        COLOR_SPACE_HDR10_ST2084_EXT :: 1000104008;
        COLOR_SPACE_DOLBYVISION_EXT :: 1000104009;
        COLOR_SPACE_HDR10_HLG_EXT :: 1000104010;
        COLOR_SPACE_ADOBERGB_LINEAR_EXT :: 1000104011;
        COLOR_SPACE_ADOBERGB_NONLINEAR_EXT :: 1000104012;
        COLOR_SPACE_PASS_THROUGH_EXT :: 1000104013;
        COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT :: 1000104014;
        COLORSPACE_SRGB_NONLINEAR_KHR :: 0;
        COLOR_SPACE_BEGIN_RANGE_KHR :: 0;
        COLOR_SPACE_END_RANGE_KHR :: 0;
        COLOR_SPACE_RANGE_SIZE_KHR :: 1;
        COLOR_SPACE_MAX_ENUM_KHR :: 2147483647;

        VK_COLOR_SPACE_SRGB_NONLINEAR_KHR :: COLOR_SPACE_SRGB_NONLINEAR_KHR;
        VK_COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT :: COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT;
        VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT :: COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT;
        VK_COLOR_SPACE_DCI_P3_LINEAR_EXT :: COLOR_SPACE_DCI_P3_LINEAR_EXT;
        VK_COLOR_SPACE_DCI_P3_NONLINEAR_EXT :: COLOR_SPACE_DCI_P3_NONLINEAR_EXT;
        VK_COLOR_SPACE_BT709_LINEAR_EXT :: COLOR_SPACE_BT709_LINEAR_EXT;
        VK_COLOR_SPACE_BT709_NONLINEAR_EXT :: COLOR_SPACE_BT709_NONLINEAR_EXT;
        VK_COLOR_SPACE_BT2020_LINEAR_EXT :: COLOR_SPACE_BT2020_LINEAR_EXT;
        VK_COLOR_SPACE_HDR10_ST2084_EXT :: COLOR_SPACE_HDR10_ST2084_EXT;
        VK_COLOR_SPACE_DOLBYVISION_EXT :: COLOR_SPACE_DOLBYVISION_EXT;
        VK_COLOR_SPACE_HDR10_HLG_EXT :: COLOR_SPACE_HDR10_HLG_EXT;
        VK_COLOR_SPACE_ADOBERGB_LINEAR_EXT :: COLOR_SPACE_ADOBERGB_LINEAR_EXT;
        VK_COLOR_SPACE_ADOBERGB_NONLINEAR_EXT :: COLOR_SPACE_ADOBERGB_NONLINEAR_EXT;
        VK_COLOR_SPACE_PASS_THROUGH_EXT :: COLOR_SPACE_PASS_THROUGH_EXT;
        VK_COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT :: COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT;
        VK_COLORSPACE_SRGB_NONLINEAR_KHR :: COLORSPACE_SRGB_NONLINEAR_KHR;
        VK_COLOR_SPACE_BEGIN_RANGE_KHR :: COLOR_SPACE_BEGIN_RANGE_KHR;
        VK_COLOR_SPACE_END_RANGE_KHR :: COLOR_SPACE_END_RANGE_KHR;
        VK_COLOR_SPACE_RANGE_SIZE_KHR :: COLOR_SPACE_RANGE_SIZE_KHR;
        VK_COLOR_SPACE_MAX_ENUM_KHR :: COLOR_SPACE_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkColorSpaceKHR :: enum u32 {
        COLOR_SPACE_SRGB_NONLINEAR_KHR :: 0;
        COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT :: 1000104001;
        COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT :: 1000104002;
        COLOR_SPACE_DCI_P3_LINEAR_EXT :: 1000104003;
        COLOR_SPACE_DCI_P3_NONLINEAR_EXT :: 1000104004;
        COLOR_SPACE_BT709_LINEAR_EXT :: 1000104005;
        COLOR_SPACE_BT709_NONLINEAR_EXT :: 1000104006;
        COLOR_SPACE_BT2020_LINEAR_EXT :: 1000104007;
        COLOR_SPACE_HDR10_ST2084_EXT :: 1000104008;
        COLOR_SPACE_DOLBYVISION_EXT :: 1000104009;
        COLOR_SPACE_HDR10_HLG_EXT :: 1000104010;
        COLOR_SPACE_ADOBERGB_LINEAR_EXT :: 1000104011;
        COLOR_SPACE_ADOBERGB_NONLINEAR_EXT :: 1000104012;
        COLOR_SPACE_PASS_THROUGH_EXT :: 1000104013;
        COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT :: 1000104014;
        COLORSPACE_SRGB_NONLINEAR_KHR :: 0;
        COLOR_SPACE_BEGIN_RANGE_KHR :: 0;
        COLOR_SPACE_END_RANGE_KHR :: 0;
        COLOR_SPACE_RANGE_SIZE_KHR :: 1;
        COLOR_SPACE_MAX_ENUM_KHR :: 2147483647;

        VK_COLOR_SPACE_SRGB_NONLINEAR_KHR :: COLOR_SPACE_SRGB_NONLINEAR_KHR;
        VK_COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT :: COLOR_SPACE_DISPLAY_P3_NONLINEAR_EXT;
        VK_COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT :: COLOR_SPACE_EXTENDED_SRGB_LINEAR_EXT;
        VK_COLOR_SPACE_DCI_P3_LINEAR_EXT :: COLOR_SPACE_DCI_P3_LINEAR_EXT;
        VK_COLOR_SPACE_DCI_P3_NONLINEAR_EXT :: COLOR_SPACE_DCI_P3_NONLINEAR_EXT;
        VK_COLOR_SPACE_BT709_LINEAR_EXT :: COLOR_SPACE_BT709_LINEAR_EXT;
        VK_COLOR_SPACE_BT709_NONLINEAR_EXT :: COLOR_SPACE_BT709_NONLINEAR_EXT;
        VK_COLOR_SPACE_BT2020_LINEAR_EXT :: COLOR_SPACE_BT2020_LINEAR_EXT;
        VK_COLOR_SPACE_HDR10_ST2084_EXT :: COLOR_SPACE_HDR10_ST2084_EXT;
        VK_COLOR_SPACE_DOLBYVISION_EXT :: COLOR_SPACE_DOLBYVISION_EXT;
        VK_COLOR_SPACE_HDR10_HLG_EXT :: COLOR_SPACE_HDR10_HLG_EXT;
        VK_COLOR_SPACE_ADOBERGB_LINEAR_EXT :: COLOR_SPACE_ADOBERGB_LINEAR_EXT;
        VK_COLOR_SPACE_ADOBERGB_NONLINEAR_EXT :: COLOR_SPACE_ADOBERGB_NONLINEAR_EXT;
        VK_COLOR_SPACE_PASS_THROUGH_EXT :: COLOR_SPACE_PASS_THROUGH_EXT;
        VK_COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT :: COLOR_SPACE_EXTENDED_SRGB_NONLINEAR_EXT;
        VK_COLORSPACE_SRGB_NONLINEAR_KHR :: COLORSPACE_SRGB_NONLINEAR_KHR;
        VK_COLOR_SPACE_BEGIN_RANGE_KHR :: COLOR_SPACE_BEGIN_RANGE_KHR;
        VK_COLOR_SPACE_END_RANGE_KHR :: COLOR_SPACE_END_RANGE_KHR;
        VK_COLOR_SPACE_RANGE_SIZE_KHR :: COLOR_SPACE_RANGE_SIZE_KHR;
        VK_COLOR_SPACE_MAX_ENUM_KHR :: COLOR_SPACE_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkCommandBuffer_T :: struct {}

}

#if OS == .LINUX {
    VkCommandBuffer_T :: struct {}

}

#if OS == .WINDOWS {
    VkCommandBufferAllocateInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_ALLOCATE_INFO;
        pNext: *void;
        commandPool: VkCommandPool;
        level: VkCommandBufferLevel;
        commandBufferCount: u32;
    }

}

#if OS == .LINUX {
    VkCommandBufferAllocateInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_ALLOCATE_INFO;
        pNext: *void;
        commandPool: VkCommandPool;
        level: VkCommandBufferLevel;
        commandBufferCount: u32;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferBeginInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_BEGIN_INFO;
        pNext: *void;
        flags: VkCommandBufferUsageFlags;
        pInheritanceInfo: *VkCommandBufferInheritanceInfo;
    }

}

#if OS == .LINUX {
    VkCommandBufferBeginInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_BEGIN_INFO;
        pNext: *void;
        flags: VkCommandBufferUsageFlags;
        pInheritanceInfo: *VkCommandBufferInheritanceInfo;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferInheritanceConditionalRenderingInfoEXT :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT;
        pNext: *void;
        conditionalRenderingEnable: VkBool32;
    }

}

#if OS == .LINUX {
    VkCommandBufferInheritanceConditionalRenderingInfoEXT :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT;
        pNext: *void;
        conditionalRenderingEnable: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferInheritanceInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_INHERITANCE_INFO;
        pNext: *void;
        renderPass: VkRenderPass;
        subpass: u32;
        framebuffer: VkFramebuffer;
        occlusionQueryEnable: VkBool32;
        queryFlags: VkQueryControlFlags;
        pipelineStatistics: VkQueryPipelineStatisticFlags;
    }

}

#if OS == .LINUX {
    VkCommandBufferInheritanceInfo :: struct {
        sType: VkStructureType = .COMMAND_BUFFER_INHERITANCE_INFO;
        pNext: *void;
        renderPass: VkRenderPass;
        subpass: u32;
        framebuffer: VkFramebuffer;
        occlusionQueryEnable: VkBool32;
        queryFlags: VkQueryControlFlags;
        pipelineStatistics: VkQueryPipelineStatisticFlags;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferLevel :: enum u32 {
        PRIMARY :: 0;
        SECONDARY :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_COMMAND_BUFFER_LEVEL_PRIMARY :: PRIMARY;
        VK_COMMAND_BUFFER_LEVEL_SECONDARY :: SECONDARY;
        VK_COMMAND_BUFFER_LEVEL_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMMAND_BUFFER_LEVEL_END_RANGE :: END_RANGE;
        VK_COMMAND_BUFFER_LEVEL_RANGE_SIZE :: RANGE_SIZE;
        VK_COMMAND_BUFFER_LEVEL_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCommandBufferLevel :: enum u32 {
        PRIMARY :: 0;
        SECONDARY :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_COMMAND_BUFFER_LEVEL_PRIMARY :: PRIMARY;
        VK_COMMAND_BUFFER_LEVEL_SECONDARY :: SECONDARY;
        VK_COMMAND_BUFFER_LEVEL_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMMAND_BUFFER_LEVEL_END_RANGE :: END_RANGE;
        VK_COMMAND_BUFFER_LEVEL_RANGE_SIZE :: RANGE_SIZE;
        VK_COMMAND_BUFFER_LEVEL_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferResetFlagBits :: enum_flags u32 {
        RELEASE_RESOURCES_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT :: RELEASE_RESOURCES_BIT;
        VK_COMMAND_BUFFER_RESET_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCommandBufferResetFlagBits :: enum_flags u32 {
        RELEASE_RESOURCES_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT :: RELEASE_RESOURCES_BIT;
        VK_COMMAND_BUFFER_RESET_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCommandBufferUsageFlagBits :: enum_flags u32 {
        ONE_TIME_SUBMIT_BIT :: 0x1;
        RENDER_PASS_CONTINUE_BIT :: 0x2;
        SIMULTANEOUS_USE_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT :: ONE_TIME_SUBMIT_BIT;
        VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT :: RENDER_PASS_CONTINUE_BIT;
        VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT :: SIMULTANEOUS_USE_BIT;
        VK_COMMAND_BUFFER_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCommandBufferUsageFlagBits :: enum_flags u32 {
        ONE_TIME_SUBMIT_BIT :: 0x1;
        RENDER_PASS_CONTINUE_BIT :: 0x2;
        SIMULTANEOUS_USE_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT :: ONE_TIME_SUBMIT_BIT;
        VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT :: RENDER_PASS_CONTINUE_BIT;
        VK_COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT :: SIMULTANEOUS_USE_BIT;
        VK_COMMAND_BUFFER_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCommandPool_T :: struct {}

}

#if OS == .LINUX {
    VkCommandPool_T :: struct {}

}

#if OS == .WINDOWS {
    VkCommandPoolCreateFlagBits :: enum_flags u32 {
        TRANSIENT_BIT :: 0x1;
        RESET_COMMAND_BUFFER_BIT :: 0x2;
        PROTECTED_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_POOL_CREATE_TRANSIENT_BIT :: TRANSIENT_BIT;
        VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT :: RESET_COMMAND_BUFFER_BIT;
        VK_COMMAND_POOL_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_COMMAND_POOL_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCommandPoolCreateFlagBits :: enum_flags u32 {
        TRANSIENT_BIT :: 0x1;
        RESET_COMMAND_BUFFER_BIT :: 0x2;
        PROTECTED_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_POOL_CREATE_TRANSIENT_BIT :: TRANSIENT_BIT;
        VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT :: RESET_COMMAND_BUFFER_BIT;
        VK_COMMAND_POOL_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_COMMAND_POOL_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCommandPoolCreateInfo :: struct {
        sType: VkStructureType = .COMMAND_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkCommandPoolCreateFlags;
        queueFamilyIndex: u32;
    }

}

#if OS == .LINUX {
    VkCommandPoolCreateInfo :: struct {
        sType: VkStructureType = .COMMAND_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkCommandPoolCreateFlags;
        queueFamilyIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkCommandPoolResetFlagBits :: enum_flags u32 {
        RELEASE_RESOURCES_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT :: RELEASE_RESOURCES_BIT;
        VK_COMMAND_POOL_RESET_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCommandPoolResetFlagBits :: enum_flags u32 {
        RELEASE_RESOURCES_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_COMMAND_POOL_RESET_RELEASE_RESOURCES_BIT :: RELEASE_RESOURCES_BIT;
        VK_COMMAND_POOL_RESET_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCompareOp :: enum u32 {
        NEVER :: 0;
        LESS :: 1;
        EQUAL :: 2;
        LESS_OR_EQUAL :: 3;
        GREATER :: 4;
        NOT_EQUAL :: 5;
        GREATER_OR_EQUAL :: 6;
        ALWAYS :: 7;
        BEGIN_RANGE :: 0;
        END_RANGE :: 7;
        RANGE_SIZE :: 8;
        MAX_ENUM :: 2147483647;

        VK_COMPARE_OP_NEVER :: NEVER;
        VK_COMPARE_OP_LESS :: LESS;
        VK_COMPARE_OP_EQUAL :: EQUAL;
        VK_COMPARE_OP_LESS_OR_EQUAL :: LESS_OR_EQUAL;
        VK_COMPARE_OP_GREATER :: GREATER;
        VK_COMPARE_OP_NOT_EQUAL :: NOT_EQUAL;
        VK_COMPARE_OP_GREATER_OR_EQUAL :: GREATER_OR_EQUAL;
        VK_COMPARE_OP_ALWAYS :: ALWAYS;
        VK_COMPARE_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMPARE_OP_END_RANGE :: END_RANGE;
        VK_COMPARE_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_COMPARE_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCompareOp :: enum u32 {
        NEVER :: 0;
        LESS :: 1;
        EQUAL :: 2;
        LESS_OR_EQUAL :: 3;
        GREATER :: 4;
        NOT_EQUAL :: 5;
        GREATER_OR_EQUAL :: 6;
        ALWAYS :: 7;
        BEGIN_RANGE :: 0;
        END_RANGE :: 7;
        RANGE_SIZE :: 8;
        MAX_ENUM :: 2147483647;

        VK_COMPARE_OP_NEVER :: NEVER;
        VK_COMPARE_OP_LESS :: LESS;
        VK_COMPARE_OP_EQUAL :: EQUAL;
        VK_COMPARE_OP_LESS_OR_EQUAL :: LESS_OR_EQUAL;
        VK_COMPARE_OP_GREATER :: GREATER;
        VK_COMPARE_OP_NOT_EQUAL :: NOT_EQUAL;
        VK_COMPARE_OP_GREATER_OR_EQUAL :: GREATER_OR_EQUAL;
        VK_COMPARE_OP_ALWAYS :: ALWAYS;
        VK_COMPARE_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMPARE_OP_END_RANGE :: END_RANGE;
        VK_COMPARE_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_COMPARE_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkComponentMapping :: struct {
        r: VkComponentSwizzle;
        g: VkComponentSwizzle;
        b: VkComponentSwizzle;
        a: VkComponentSwizzle;
    }

}

#if OS == .LINUX {
    VkComponentMapping :: struct {
        r: VkComponentSwizzle;
        g: VkComponentSwizzle;
        b: VkComponentSwizzle;
        a: VkComponentSwizzle;
    }

}

#if OS == .WINDOWS {
    VkComponentSwizzle :: enum u32 {
        IDENTITY :: 0;
        ZERO :: 1;
        ONE :: 2;
        R :: 3;
        G :: 4;
        B :: 5;
        A :: 6;
        BEGIN_RANGE :: 0;
        END_RANGE :: 6;
        RANGE_SIZE :: 7;
        MAX_ENUM :: 2147483647;

        VK_COMPONENT_SWIZZLE_IDENTITY :: IDENTITY;
        VK_COMPONENT_SWIZZLE_ZERO :: ZERO;
        VK_COMPONENT_SWIZZLE_ONE :: ONE;
        VK_COMPONENT_SWIZZLE_R :: R;
        VK_COMPONENT_SWIZZLE_G :: G;
        VK_COMPONENT_SWIZZLE_B :: B;
        VK_COMPONENT_SWIZZLE_A :: A;
        VK_COMPONENT_SWIZZLE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMPONENT_SWIZZLE_END_RANGE :: END_RANGE;
        VK_COMPONENT_SWIZZLE_RANGE_SIZE :: RANGE_SIZE;
        VK_COMPONENT_SWIZZLE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkComponentSwizzle :: enum u32 {
        IDENTITY :: 0;
        ZERO :: 1;
        ONE :: 2;
        R :: 3;
        G :: 4;
        B :: 5;
        A :: 6;
        BEGIN_RANGE :: 0;
        END_RANGE :: 6;
        RANGE_SIZE :: 7;
        MAX_ENUM :: 2147483647;

        VK_COMPONENT_SWIZZLE_IDENTITY :: IDENTITY;
        VK_COMPONENT_SWIZZLE_ZERO :: ZERO;
        VK_COMPONENT_SWIZZLE_ONE :: ONE;
        VK_COMPONENT_SWIZZLE_R :: R;
        VK_COMPONENT_SWIZZLE_G :: G;
        VK_COMPONENT_SWIZZLE_B :: B;
        VK_COMPONENT_SWIZZLE_A :: A;
        VK_COMPONENT_SWIZZLE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_COMPONENT_SWIZZLE_END_RANGE :: END_RANGE;
        VK_COMPONENT_SWIZZLE_RANGE_SIZE :: RANGE_SIZE;
        VK_COMPONENT_SWIZZLE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkCompositeAlphaFlagBitsKHR :: enum_flags u32 {
        OPAQUE_BIT_KHR :: 0x1;
        PRE_MULTIPLIED_BIT_KHR :: 0x2;
        POST_MULTIPLIED_BIT_KHR :: 0x4;
        INHERIT_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR :: OPAQUE_BIT_KHR;
        VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR :: PRE_MULTIPLIED_BIT_KHR;
        VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR :: POST_MULTIPLIED_BIT_KHR;
        VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR :: INHERIT_BIT_KHR;
        VK_COMPOSITE_ALPHA_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkCompositeAlphaFlagBitsKHR :: enum_flags u32 {
        OPAQUE_BIT_KHR :: 0x1;
        PRE_MULTIPLIED_BIT_KHR :: 0x2;
        POST_MULTIPLIED_BIT_KHR :: 0x4;
        INHERIT_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR :: OPAQUE_BIT_KHR;
        VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR :: PRE_MULTIPLIED_BIT_KHR;
        VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR :: POST_MULTIPLIED_BIT_KHR;
        VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR :: INHERIT_BIT_KHR;
        VK_COMPOSITE_ALPHA_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkComputePipelineCreateInfo :: struct {
        sType: VkStructureType = .COMPUTE_PIPELINE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stage: VkPipelineShaderStageCreateInfo;
        layout: VkPipelineLayout;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .LINUX {
    VkComputePipelineCreateInfo :: struct {
        sType: VkStructureType = .COMPUTE_PIPELINE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stage: VkPipelineShaderStageCreateInfo;
        layout: VkPipelineLayout;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .WINDOWS {
    VkConditionalRenderingBeginInfoEXT :: struct {
        sType: VkStructureType = .CONDITIONAL_RENDERING_BEGIN_INFO_EXT;
        pNext: *void;
        buffer: VkBuffer;
        offset: VkDeviceSize;
        flags: VkConditionalRenderingFlagsEXT;
    }

}

#if OS == .LINUX {
    VkConditionalRenderingBeginInfoEXT :: struct {
        sType: VkStructureType = .CONDITIONAL_RENDERING_BEGIN_INFO_EXT;
        pNext: *void;
        buffer: VkBuffer;
        offset: VkDeviceSize;
        flags: VkConditionalRenderingFlagsEXT;
    }

}

#if OS == .WINDOWS {
    VkConditionalRenderingFlagBitsEXT :: enum_flags u32 {
        INVERTED_BIT_EXT :: 0x1;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_CONDITIONAL_RENDERING_INVERTED_BIT_EXT :: INVERTED_BIT_EXT;
        VK_CONDITIONAL_RENDERING_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkConditionalRenderingFlagBitsEXT :: enum_flags u32 {
        INVERTED_BIT_EXT :: 0x1;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_CONDITIONAL_RENDERING_INVERTED_BIT_EXT :: INVERTED_BIT_EXT;
        VK_CONDITIONAL_RENDERING_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkConservativeRasterizationModeEXT :: enum u32 {
        DISABLED_EXT :: 0;
        OVERESTIMATE_EXT :: 1;
        UNDERESTIMATE_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_CONSERVATIVE_RASTERIZATION_MODE_DISABLED_EXT :: DISABLED_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_OVERESTIMATE_EXT :: OVERESTIMATE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_UNDERESTIMATE_EXT :: UNDERESTIMATE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkConservativeRasterizationModeEXT :: enum u32 {
        DISABLED_EXT :: 0;
        OVERESTIMATE_EXT :: 1;
        UNDERESTIMATE_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_CONSERVATIVE_RASTERIZATION_MODE_DISABLED_EXT :: DISABLED_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_OVERESTIMATE_EXT :: OVERESTIMATE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_UNDERESTIMATE_EXT :: UNDERESTIMATE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_CONSERVATIVE_RASTERIZATION_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkCopyAccelerationStructureModeNVX :: enum u32 {
        CLONE_NVX :: 0;
        COMPACT_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_COPY_ACCELERATION_STRUCTURE_MODE_CLONE_NVX :: CLONE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_COMPACT_NVX :: COMPACT_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkCopyAccelerationStructureModeNVX :: enum u32 {
        CLONE_NVX :: 0;
        COMPACT_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_COPY_ACCELERATION_STRUCTURE_MODE_CLONE_NVX :: CLONE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_COMPACT_NVX :: COMPACT_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_COPY_ACCELERATION_STRUCTURE_MODE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkCopyDescriptorSet :: struct {
        sType: VkStructureType = .COPY_DESCRIPTOR_SET;
        pNext: *void;
        srcSet: VkDescriptorSet;
        srcBinding: u32;
        srcArrayElement: u32;
        dstSet: VkDescriptorSet;
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
    }

}

#if OS == .LINUX {
    VkCopyDescriptorSet :: struct {
        sType: VkStructureType = .COPY_DESCRIPTOR_SET;
        pNext: *void;
        srcSet: VkDescriptorSet;
        srcBinding: u32;
        srcArrayElement: u32;
        dstSet: VkDescriptorSet;
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
    }

}

#if OS == .WINDOWS {
    VkCoverageModulationModeNV :: enum u32 {
        NONE_NV :: 0;
        RGB_NV :: 1;
        ALPHA_NV :: 2;
        RGBA_NV :: 3;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 3;
        RANGE_SIZE_NV :: 4;
        MAX_ENUM_NV :: 2147483647;

        VK_COVERAGE_MODULATION_MODE_NONE_NV :: NONE_NV;
        VK_COVERAGE_MODULATION_MODE_RGB_NV :: RGB_NV;
        VK_COVERAGE_MODULATION_MODE_ALPHA_NV :: ALPHA_NV;
        VK_COVERAGE_MODULATION_MODE_RGBA_NV :: RGBA_NV;
        VK_COVERAGE_MODULATION_MODE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_COVERAGE_MODULATION_MODE_END_RANGE_NV :: END_RANGE_NV;
        VK_COVERAGE_MODULATION_MODE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_COVERAGE_MODULATION_MODE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkCoverageModulationModeNV :: enum u32 {
        NONE_NV :: 0;
        RGB_NV :: 1;
        ALPHA_NV :: 2;
        RGBA_NV :: 3;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 3;
        RANGE_SIZE_NV :: 4;
        MAX_ENUM_NV :: 2147483647;

        VK_COVERAGE_MODULATION_MODE_NONE_NV :: NONE_NV;
        VK_COVERAGE_MODULATION_MODE_RGB_NV :: RGB_NV;
        VK_COVERAGE_MODULATION_MODE_ALPHA_NV :: ALPHA_NV;
        VK_COVERAGE_MODULATION_MODE_RGBA_NV :: RGBA_NV;
        VK_COVERAGE_MODULATION_MODE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_COVERAGE_MODULATION_MODE_END_RANGE_NV :: END_RANGE_NV;
        VK_COVERAGE_MODULATION_MODE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_COVERAGE_MODULATION_MODE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkCullModeFlagBits :: enum_flags u32 {
        NONE :: 0x0;
        FRONT_BIT :: 0x1;
        BACK_BIT :: 0x2;
        FRONT_AND_BACK :: 0x3;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_CULL_MODE_NONE :: NONE;
        VK_CULL_MODE_FRONT_BIT :: FRONT_BIT;
        VK_CULL_MODE_BACK_BIT :: BACK_BIT;
        VK_CULL_MODE_FRONT_AND_BACK :: FRONT_AND_BACK;
        VK_CULL_MODE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkCullModeFlagBits :: enum_flags u32 {
        NONE :: 0x0;
        FRONT_BIT :: 0x1;
        BACK_BIT :: 0x2;
        FRONT_AND_BACK :: 0x3;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_CULL_MODE_NONE :: NONE;
        VK_CULL_MODE_FRONT_BIT :: FRONT_BIT;
        VK_CULL_MODE_BACK_BIT :: BACK_BIT;
        VK_CULL_MODE_FRONT_AND_BACK :: FRONT_AND_BACK;
        VK_CULL_MODE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkD3D12FenceSubmitInfoKHR :: struct {
        sType: VkStructureType = .D3D12_FENCE_SUBMIT_INFO_KHR;
        pNext: *void;
        waitSemaphoreValuesCount: u32;
        pWaitSemaphoreValues: *u64;
        signalSemaphoreValuesCount: u32;
        pSignalSemaphoreValues: *u64;
    }

}

#if OS == .WINDOWS {
    VkDebugMarkerMarkerInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_MARKER_INFO_EXT;
        pNext: *void;
        pMarkerName: *u8;
        color: [4] float;
    }

}

#if OS == .LINUX {
    VkDebugMarkerMarkerInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_MARKER_INFO_EXT;
        pNext: *void;
        pMarkerName: *u8;
        color: [4] float;
    }

}

#if OS == .WINDOWS {
    VkDebugMarkerObjectNameInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_OBJECT_NAME_INFO_EXT;
        pNext: *void;
        objectType: VkDebugReportObjectTypeEXT;
        object: u64;
        pObjectName: *u8;
    }

}

#if OS == .LINUX {
    VkDebugMarkerObjectNameInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_OBJECT_NAME_INFO_EXT;
        pNext: *void;
        objectType: VkDebugReportObjectTypeEXT;
        object: u64;
        pObjectName: *u8;
    }

}

#if OS == .WINDOWS {
    VkDebugMarkerObjectTagInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_OBJECT_TAG_INFO_EXT;
        pNext: *void;
        objectType: VkDebugReportObjectTypeEXT;
        object: u64;
        tagName: u64;
        tagSize: u64;
        pTag: *void;
    }

}

#if OS == .LINUX {
    VkDebugMarkerObjectTagInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_MARKER_OBJECT_TAG_INFO_EXT;
        pNext: *void;
        objectType: VkDebugReportObjectTypeEXT;
        object: u64;
        tagName: u64;
        tagSize: u64;
        pTag: *void;
    }

}

#if OS == .WINDOWS {
    VkDebugReportCallbackCreateInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkDebugReportFlagsEXT;
        pfnCallback: PFN_vkDebugReportCallbackEXT;
        pUserData: *void;
    }

}

#if OS == .LINUX {
    VkDebugReportCallbackCreateInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkDebugReportFlagsEXT;
        pfnCallback: PFN_vkDebugReportCallbackEXT;
        pUserData: *void;
    }

}

#if OS == .WINDOWS {
    VkDebugReportCallbackEXT_T :: struct {}

}

#if OS == .LINUX {
    VkDebugReportCallbackEXT_T :: struct {}

}

#if OS == .WINDOWS {
    VkDebugReportFlagBitsEXT :: enum_flags u32 {
        INFORMATION_BIT_EXT :: 0x1;
        WARNING_BIT_EXT :: 0x2;
        PERFORMANCE_WARNING_BIT_EXT :: 0x4;
        ERROR_BIT_EXT :: 0x8;
        DEBUG_BIT_EXT :: 0x10;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_REPORT_INFORMATION_BIT_EXT :: INFORMATION_BIT_EXT;
        VK_DEBUG_REPORT_WARNING_BIT_EXT :: WARNING_BIT_EXT;
        VK_DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT :: PERFORMANCE_WARNING_BIT_EXT;
        VK_DEBUG_REPORT_ERROR_BIT_EXT :: ERROR_BIT_EXT;
        VK_DEBUG_REPORT_DEBUG_BIT_EXT :: DEBUG_BIT_EXT;
        VK_DEBUG_REPORT_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDebugReportFlagBitsEXT :: enum_flags u32 {
        INFORMATION_BIT_EXT :: 0x1;
        WARNING_BIT_EXT :: 0x2;
        PERFORMANCE_WARNING_BIT_EXT :: 0x4;
        ERROR_BIT_EXT :: 0x8;
        DEBUG_BIT_EXT :: 0x10;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_REPORT_INFORMATION_BIT_EXT :: INFORMATION_BIT_EXT;
        VK_DEBUG_REPORT_WARNING_BIT_EXT :: WARNING_BIT_EXT;
        VK_DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT :: PERFORMANCE_WARNING_BIT_EXT;
        VK_DEBUG_REPORT_ERROR_BIT_EXT :: ERROR_BIT_EXT;
        VK_DEBUG_REPORT_DEBUG_BIT_EXT :: DEBUG_BIT_EXT;
        VK_DEBUG_REPORT_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDebugReportObjectTypeEXT :: enum u32 {
        UNKNOWN_EXT :: 0;
        INSTANCE_EXT :: 1;
        PHYSICAL_DEVICE_EXT :: 2;
        DEVICE_EXT :: 3;
        QUEUE_EXT :: 4;
        SEMAPHORE_EXT :: 5;
        COMMAND_BUFFER_EXT :: 6;
        FENCE_EXT :: 7;
        DEVICE_MEMORY_EXT :: 8;
        BUFFER_EXT :: 9;
        IMAGE_EXT :: 10;
        EVENT_EXT :: 11;
        QUERY_POOL_EXT :: 12;
        BUFFER_VIEW_EXT :: 13;
        IMAGE_VIEW_EXT :: 14;
        SHADER_MODULE_EXT :: 15;
        PIPELINE_CACHE_EXT :: 16;
        PIPELINE_LAYOUT_EXT :: 17;
        RENDER_PASS_EXT :: 18;
        PIPELINE_EXT :: 19;
        DESCRIPTOR_SET_LAYOUT_EXT :: 20;
        SAMPLER_EXT :: 21;
        DESCRIPTOR_POOL_EXT :: 22;
        DESCRIPTOR_SET_EXT :: 23;
        FRAMEBUFFER_EXT :: 24;
        COMMAND_POOL_EXT :: 25;
        SURFACE_KHR_EXT :: 26;
        SWAPCHAIN_KHR_EXT :: 27;
        DEBUG_REPORT_CALLBACK_EXT_EXT :: 28;
        DISPLAY_KHR_EXT :: 29;
        DISPLAY_MODE_KHR_EXT :: 30;
        OBJECT_TABLE_NVX_EXT :: 31;
        INDIRECT_COMMANDS_LAYOUT_NVX_EXT :: 32;
        VALIDATION_CACHE_EXT_EXT :: 33;
        SAMPLER_YCBCR_CONVERSION_EXT :: 1000156000;
        DESCRIPTOR_UPDATE_TEMPLATE_EXT :: 1000085000;
        ACCELERATION_STRUCTURE_NVX_EXT :: 1000165000;
        DEBUG_REPORT_EXT :: 28;
        VALIDATION_CACHE_EXT :: 33;
        DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT :: 1000085000;
        SAMPLER_YCBCR_CONVERSION_KHR_EXT :: 1000156000;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 33;
        RANGE_SIZE_EXT :: 34;
        MAX_ENUM_EXT :: 2147483647;

        VK_DEBUG_REPORT_OBJECT_TYPE_UNKNOWN_EXT :: UNKNOWN_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_INSTANCE_EXT :: INSTANCE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PHYSICAL_DEVICE_EXT :: PHYSICAL_DEVICE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_EXT :: DEVICE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_QUEUE_EXT :: QUEUE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SEMAPHORE_EXT :: SEMAPHORE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_BUFFER_EXT :: COMMAND_BUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_FENCE_EXT :: FENCE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_MEMORY_EXT :: DEVICE_MEMORY_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_EXT :: BUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_EXT :: IMAGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_EVENT_EXT :: EVENT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_QUERY_POOL_EXT :: QUERY_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_VIEW_EXT :: BUFFER_VIEW_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_VIEW_EXT :: IMAGE_VIEW_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SHADER_MODULE_EXT :: SHADER_MODULE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_CACHE_EXT :: PIPELINE_CACHE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_LAYOUT_EXT :: PIPELINE_LAYOUT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_RENDER_PASS_EXT :: RENDER_PASS_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_EXT :: PIPELINE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT_EXT :: DESCRIPTOR_SET_LAYOUT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_EXT :: SAMPLER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_POOL_EXT :: DESCRIPTOR_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_EXT :: DESCRIPTOR_SET_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_FRAMEBUFFER_EXT :: FRAMEBUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_POOL_EXT :: COMMAND_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SURFACE_KHR_EXT :: SURFACE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SWAPCHAIN_KHR_EXT :: SWAPCHAIN_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEBUG_REPORT_CALLBACK_EXT_EXT :: DEBUG_REPORT_CALLBACK_EXT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_KHR_EXT :: DISPLAY_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_MODE_KHR_EXT :: DISPLAY_MODE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_OBJECT_TABLE_NVX_EXT :: OBJECT_TABLE_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX_EXT :: INDIRECT_COMMANDS_LAYOUT_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_VALIDATION_CACHE_EXT_EXT :: VALIDATION_CACHE_EXT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_EXT :: SAMPLER_YCBCR_CONVERSION_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_EXT :: DESCRIPTOR_UPDATE_TEMPLATE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_ACCELERATION_STRUCTURE_NVX_EXT :: ACCELERATION_STRUCTURE_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEBUG_REPORT_EXT :: DEBUG_REPORT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_VALIDATION_CACHE_EXT :: VALIDATION_CACHE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT :: DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_KHR_EXT :: SAMPLER_YCBCR_CONVERSION_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDebugReportObjectTypeEXT :: enum u32 {
        UNKNOWN_EXT :: 0;
        INSTANCE_EXT :: 1;
        PHYSICAL_DEVICE_EXT :: 2;
        DEVICE_EXT :: 3;
        QUEUE_EXT :: 4;
        SEMAPHORE_EXT :: 5;
        COMMAND_BUFFER_EXT :: 6;
        FENCE_EXT :: 7;
        DEVICE_MEMORY_EXT :: 8;
        BUFFER_EXT :: 9;
        IMAGE_EXT :: 10;
        EVENT_EXT :: 11;
        QUERY_POOL_EXT :: 12;
        BUFFER_VIEW_EXT :: 13;
        IMAGE_VIEW_EXT :: 14;
        SHADER_MODULE_EXT :: 15;
        PIPELINE_CACHE_EXT :: 16;
        PIPELINE_LAYOUT_EXT :: 17;
        RENDER_PASS_EXT :: 18;
        PIPELINE_EXT :: 19;
        DESCRIPTOR_SET_LAYOUT_EXT :: 20;
        SAMPLER_EXT :: 21;
        DESCRIPTOR_POOL_EXT :: 22;
        DESCRIPTOR_SET_EXT :: 23;
        FRAMEBUFFER_EXT :: 24;
        COMMAND_POOL_EXT :: 25;
        SURFACE_KHR_EXT :: 26;
        SWAPCHAIN_KHR_EXT :: 27;
        DEBUG_REPORT_CALLBACK_EXT_EXT :: 28;
        DISPLAY_KHR_EXT :: 29;
        DISPLAY_MODE_KHR_EXT :: 30;
        OBJECT_TABLE_NVX_EXT :: 31;
        INDIRECT_COMMANDS_LAYOUT_NVX_EXT :: 32;
        VALIDATION_CACHE_EXT_EXT :: 33;
        SAMPLER_YCBCR_CONVERSION_EXT :: 1000156000;
        DESCRIPTOR_UPDATE_TEMPLATE_EXT :: 1000085000;
        ACCELERATION_STRUCTURE_NVX_EXT :: 1000165000;
        DEBUG_REPORT_EXT :: 28;
        VALIDATION_CACHE_EXT :: 33;
        DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT :: 1000085000;
        SAMPLER_YCBCR_CONVERSION_KHR_EXT :: 1000156000;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 33;
        RANGE_SIZE_EXT :: 34;
        MAX_ENUM_EXT :: 2147483647;

        VK_DEBUG_REPORT_OBJECT_TYPE_UNKNOWN_EXT :: UNKNOWN_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_INSTANCE_EXT :: INSTANCE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PHYSICAL_DEVICE_EXT :: PHYSICAL_DEVICE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_EXT :: DEVICE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_QUEUE_EXT :: QUEUE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SEMAPHORE_EXT :: SEMAPHORE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_BUFFER_EXT :: COMMAND_BUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_FENCE_EXT :: FENCE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEVICE_MEMORY_EXT :: DEVICE_MEMORY_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_EXT :: BUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_EXT :: IMAGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_EVENT_EXT :: EVENT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_QUERY_POOL_EXT :: QUERY_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BUFFER_VIEW_EXT :: BUFFER_VIEW_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_IMAGE_VIEW_EXT :: IMAGE_VIEW_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SHADER_MODULE_EXT :: SHADER_MODULE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_CACHE_EXT :: PIPELINE_CACHE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_LAYOUT_EXT :: PIPELINE_LAYOUT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_RENDER_PASS_EXT :: RENDER_PASS_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_PIPELINE_EXT :: PIPELINE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT_EXT :: DESCRIPTOR_SET_LAYOUT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_EXT :: SAMPLER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_POOL_EXT :: DESCRIPTOR_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_SET_EXT :: DESCRIPTOR_SET_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_FRAMEBUFFER_EXT :: FRAMEBUFFER_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_COMMAND_POOL_EXT :: COMMAND_POOL_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SURFACE_KHR_EXT :: SURFACE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SWAPCHAIN_KHR_EXT :: SWAPCHAIN_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEBUG_REPORT_CALLBACK_EXT_EXT :: DEBUG_REPORT_CALLBACK_EXT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_KHR_EXT :: DISPLAY_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DISPLAY_MODE_KHR_EXT :: DISPLAY_MODE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_OBJECT_TABLE_NVX_EXT :: OBJECT_TABLE_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX_EXT :: INDIRECT_COMMANDS_LAYOUT_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_VALIDATION_CACHE_EXT_EXT :: VALIDATION_CACHE_EXT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_EXT :: SAMPLER_YCBCR_CONVERSION_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_EXT :: DESCRIPTOR_UPDATE_TEMPLATE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_ACCELERATION_STRUCTURE_NVX_EXT :: ACCELERATION_STRUCTURE_NVX_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DEBUG_REPORT_EXT :: DEBUG_REPORT_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_VALIDATION_CACHE_EXT :: VALIDATION_CACHE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT :: DESCRIPTOR_UPDATE_TEMPLATE_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_KHR_EXT :: SAMPLER_YCBCR_CONVERSION_KHR_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DEBUG_REPORT_OBJECT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsLabelEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_LABEL_EXT;
        pNext: *void;
        pLabelName: *u8;
        color: [4] float;
    }

}

#if OS == .LINUX {
    VkDebugUtilsLabelEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_LABEL_EXT;
        pNext: *void;
        pLabelName: *u8;
        color: [4] float;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsMessageSeverityFlagBitsEXT :: enum_flags u32 {
        VERBOSE_BIT_EXT :: 0x1;
        INFO_BIT_EXT :: 0x10;
        WARNING_BIT_EXT :: 0x100;
        ERROR_BIT_EXT :: 0x1000;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT :: VERBOSE_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT :: INFO_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT :: WARNING_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT :: ERROR_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDebugUtilsMessageSeverityFlagBitsEXT :: enum_flags u32 {
        VERBOSE_BIT_EXT :: 0x1;
        INFO_BIT_EXT :: 0x10;
        WARNING_BIT_EXT :: 0x100;
        ERROR_BIT_EXT :: 0x1000;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT :: VERBOSE_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT :: INFO_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT :: WARNING_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT :: ERROR_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_SEVERITY_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsMessageTypeFlagBitsEXT :: enum_flags u32 {
        GENERAL_BIT_EXT :: 0x1;
        VALIDATION_BIT_EXT :: 0x2;
        PERFORMANCE_BIT_EXT :: 0x4;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT :: GENERAL_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT :: VALIDATION_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT :: PERFORMANCE_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDebugUtilsMessageTypeFlagBitsEXT :: enum_flags u32 {
        GENERAL_BIT_EXT :: 0x1;
        VALIDATION_BIT_EXT :: 0x2;
        PERFORMANCE_BIT_EXT :: 0x4;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT :: GENERAL_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT :: VALIDATION_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT :: PERFORMANCE_BIT_EXT;
        VK_DEBUG_UTILS_MESSAGE_TYPE_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerCallbackDataEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT;
        pNext: *void;
        flags: VkDebugUtilsMessengerCallbackDataFlagsEXT;
        pMessageIdName: *u8;
        messageIdNumber: s32;
        pMessage: *u8;
        queueLabelCount: u32;
        pQueueLabels: *VkDebugUtilsLabelEXT;
        cmdBufLabelCount: u32;
        pCmdBufLabels: *VkDebugUtilsLabelEXT;
        objectCount: u32;
        pObjects: *VkDebugUtilsObjectNameInfoEXT;
    }

}

#if OS == .LINUX {
    VkDebugUtilsMessengerCallbackDataEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT;
        pNext: *void;
        flags: VkDebugUtilsMessengerCallbackDataFlagsEXT;
        pMessageIdName: *u8;
        messageIdNumber: s32;
        pMessage: *u8;
        queueLabelCount: u32;
        pQueueLabels: *VkDebugUtilsLabelEXT;
        cmdBufLabelCount: u32;
        pCmdBufLabels: *VkDebugUtilsLabelEXT;
        objectCount: u32;
        pObjects: *VkDebugUtilsObjectNameInfoEXT;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerCreateInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkDebugUtilsMessengerCreateFlagsEXT;
        messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT;
        messageType: VkDebugUtilsMessageTypeFlagsEXT;
        pfnUserCallback: PFN_vkDebugUtilsMessengerCallbackEXT;
        pUserData: *void;
    }

}

#if OS == .LINUX {
    VkDebugUtilsMessengerCreateInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkDebugUtilsMessengerCreateFlagsEXT;
        messageSeverity: VkDebugUtilsMessageSeverityFlagsEXT;
        messageType: VkDebugUtilsMessageTypeFlagsEXT;
        pfnUserCallback: PFN_vkDebugUtilsMessengerCallbackEXT;
        pUserData: *void;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerEXT_T :: struct {}

}

#if OS == .LINUX {
    VkDebugUtilsMessengerEXT_T :: struct {}

}

#if OS == .WINDOWS {
    VkDebugUtilsObjectNameInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_OBJECT_NAME_INFO_EXT;
        pNext: *void;
        objectType: VkObjectType;
        objectHandle: u64;
        pObjectName: *u8;
    }

}

#if OS == .LINUX {
    VkDebugUtilsObjectNameInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_OBJECT_NAME_INFO_EXT;
        pNext: *void;
        objectType: VkObjectType;
        objectHandle: u64;
        pObjectName: *u8;
    }

}

#if OS == .WINDOWS {
    VkDebugUtilsObjectTagInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_OBJECT_TAG_INFO_EXT;
        pNext: *void;
        objectType: VkObjectType;
        objectHandle: u64;
        tagName: u64;
        tagSize: u64;
        pTag: *void;
    }

}

#if OS == .LINUX {
    VkDebugUtilsObjectTagInfoEXT :: struct {
        sType: VkStructureType = .DEBUG_UTILS_OBJECT_TAG_INFO_EXT;
        pNext: *void;
        objectType: VkObjectType;
        objectHandle: u64;
        tagName: u64;
        tagSize: u64;
        pTag: *void;
    }

}

#if OS == .WINDOWS {
    VkDedicatedAllocationBufferCreateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV;
        pNext: *void;
        dedicatedAllocation: VkBool32;
    }

}

#if OS == .LINUX {
    VkDedicatedAllocationBufferCreateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV;
        pNext: *void;
        dedicatedAllocation: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDedicatedAllocationImageCreateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV;
        pNext: *void;
        dedicatedAllocation: VkBool32;
    }

}

#if OS == .LINUX {
    VkDedicatedAllocationImageCreateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV;
        pNext: *void;
        dedicatedAllocation: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDedicatedAllocationMemoryAllocateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV;
        pNext: *void;
        image: VkImage;
        buffer: VkBuffer;
    }

}

#if OS == .LINUX {
    VkDedicatedAllocationMemoryAllocateInfoNV :: struct {
        sType: VkStructureType = .DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV;
        pNext: *void;
        image: VkImage;
        buffer: VkBuffer;
    }

}

#if OS == .WINDOWS {
    VkDependencyFlagBits :: enum_flags u32 {
        BY_REGION_BIT :: 0x1;
        DEVICE_GROUP_BIT :: 0x4;
        VIEW_LOCAL_BIT :: 0x2;
        VIEW_LOCAL_BIT_KHR :: 0x2;
        DEVICE_GROUP_BIT_KHR :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DEPENDENCY_BY_REGION_BIT :: BY_REGION_BIT;
        VK_DEPENDENCY_DEVICE_GROUP_BIT :: DEVICE_GROUP_BIT;
        VK_DEPENDENCY_VIEW_LOCAL_BIT :: VIEW_LOCAL_BIT;
        VK_DEPENDENCY_VIEW_LOCAL_BIT_KHR :: VIEW_LOCAL_BIT_KHR;
        VK_DEPENDENCY_DEVICE_GROUP_BIT_KHR :: DEVICE_GROUP_BIT_KHR;
        VK_DEPENDENCY_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDependencyFlagBits :: enum_flags u32 {
        BY_REGION_BIT :: 0x1;
        DEVICE_GROUP_BIT :: 0x4;
        VIEW_LOCAL_BIT :: 0x2;
        VIEW_LOCAL_BIT_KHR :: 0x2;
        DEVICE_GROUP_BIT_KHR :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DEPENDENCY_BY_REGION_BIT :: BY_REGION_BIT;
        VK_DEPENDENCY_DEVICE_GROUP_BIT :: DEVICE_GROUP_BIT;
        VK_DEPENDENCY_VIEW_LOCAL_BIT :: VIEW_LOCAL_BIT;
        VK_DEPENDENCY_VIEW_LOCAL_BIT_KHR :: VIEW_LOCAL_BIT_KHR;
        VK_DEPENDENCY_DEVICE_GROUP_BIT_KHR :: DEVICE_GROUP_BIT_KHR;
        VK_DEPENDENCY_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDescriptorAccelerationStructureInfoNVX :: struct {
        sType: VkStructureType = .DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX;
        pNext: *void;
        accelerationStructureCount: u32;
        pAccelerationStructures: *VkAccelerationStructureNVX;
    }

}

#if OS == .LINUX {
    VkDescriptorAccelerationStructureInfoNVX :: struct {
        sType: VkStructureType = .DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX;
        pNext: *void;
        accelerationStructureCount: u32;
        pAccelerationStructures: *VkAccelerationStructureNVX;
    }

}

#if OS == .WINDOWS {
    VkDescriptorBindingFlagBitsEXT :: enum_flags u32 {
        UPDATE_AFTER_BIND_BIT_EXT :: 0x1;
        UPDATE_UNUSED_WHILE_PENDING_BIT_EXT :: 0x2;
        PARTIALLY_BOUND_BIT_EXT :: 0x4;
        VARIABLE_DESCRIPTOR_COUNT_BIT_EXT :: 0x8;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT_EXT :: UPDATE_AFTER_BIND_BIT_EXT;
        VK_DESCRIPTOR_BINDING_UPDATE_UNUSED_WHILE_PENDING_BIT_EXT :: UPDATE_UNUSED_WHILE_PENDING_BIT_EXT;
        VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT_EXT :: PARTIALLY_BOUND_BIT_EXT;
        VK_DESCRIPTOR_BINDING_VARIABLE_DESCRIPTOR_COUNT_BIT_EXT :: VARIABLE_DESCRIPTOR_COUNT_BIT_EXT;
        VK_DESCRIPTOR_BINDING_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDescriptorBindingFlagBitsEXT :: enum_flags u32 {
        UPDATE_AFTER_BIND_BIT_EXT :: 0x1;
        UPDATE_UNUSED_WHILE_PENDING_BIT_EXT :: 0x2;
        PARTIALLY_BOUND_BIT_EXT :: 0x4;
        VARIABLE_DESCRIPTOR_COUNT_BIT_EXT :: 0x8;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT_EXT :: UPDATE_AFTER_BIND_BIT_EXT;
        VK_DESCRIPTOR_BINDING_UPDATE_UNUSED_WHILE_PENDING_BIT_EXT :: UPDATE_UNUSED_WHILE_PENDING_BIT_EXT;
        VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT_EXT :: PARTIALLY_BOUND_BIT_EXT;
        VK_DESCRIPTOR_BINDING_VARIABLE_DESCRIPTOR_COUNT_BIT_EXT :: VARIABLE_DESCRIPTOR_COUNT_BIT_EXT;
        VK_DESCRIPTOR_BINDING_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDescriptorBufferInfo :: struct {
        buffer: VkBuffer;
        offset: VkDeviceSize;
        range: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkDescriptorBufferInfo :: struct {
        buffer: VkBuffer;
        offset: VkDeviceSize;
        range: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkDescriptorImageInfo :: struct {
        sampler: VkSampler;
        imageView: VkImageView;
        imageLayout: VkImageLayout;
    }

}

#if OS == .LINUX {
    VkDescriptorImageInfo :: struct {
        sampler: VkSampler;
        imageView: VkImageView;
        imageLayout: VkImageLayout;
    }

}

#if OS == .WINDOWS {
    VkDescriptorPool_T :: struct {}

}

#if OS == .LINUX {
    VkDescriptorPool_T :: struct {}

}

#if OS == .WINDOWS {
    VkDescriptorPoolCreateFlagBits :: enum_flags u32 {
        FREE_DESCRIPTOR_SET_BIT :: 0x1;
        UPDATE_AFTER_BIND_BIT_EXT :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT :: FREE_DESCRIPTOR_SET_BIT;
        VK_DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT_EXT :: UPDATE_AFTER_BIND_BIT_EXT;
        VK_DESCRIPTOR_POOL_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDescriptorPoolCreateFlagBits :: enum_flags u32 {
        FREE_DESCRIPTOR_SET_BIT :: 0x1;
        UPDATE_AFTER_BIND_BIT_EXT :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT :: FREE_DESCRIPTOR_SET_BIT;
        VK_DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT_EXT :: UPDATE_AFTER_BIND_BIT_EXT;
        VK_DESCRIPTOR_POOL_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDescriptorPoolCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorPoolCreateFlags;
        maxSets: u32;
        poolSizeCount: u32;
        pPoolSizes: *VkDescriptorPoolSize;
    }

}

#if OS == .LINUX {
    VkDescriptorPoolCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorPoolCreateFlags;
        maxSets: u32;
        poolSizeCount: u32;
        pPoolSizes: *VkDescriptorPoolSize;
    }

}

#if OS == .WINDOWS {
    VkDescriptorPoolInlineUniformBlockCreateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT;
        pNext: *void;
        maxInlineUniformBlockBindings: u32;
    }

}

#if OS == .LINUX {
    VkDescriptorPoolInlineUniformBlockCreateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT;
        pNext: *void;
        maxInlineUniformBlockBindings: u32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorPoolSize :: struct {
        type: VkDescriptorType;
        descriptorCount: u32;
    }

}

#if OS == .LINUX {
    VkDescriptorPoolSize :: struct {
        type: VkDescriptorType;
        descriptorCount: u32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSet_T :: struct {}

}

#if OS == .LINUX {
    VkDescriptorSet_T :: struct {}

}

#if OS == .WINDOWS {
    VkDescriptorSetAllocateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_ALLOCATE_INFO;
        pNext: *void;
        descriptorPool: VkDescriptorPool;
        descriptorSetCount: u32;
        pSetLayouts: *VkDescriptorSetLayout;
    }

}

#if OS == .LINUX {
    VkDescriptorSetAllocateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_ALLOCATE_INFO;
        pNext: *void;
        descriptorPool: VkDescriptorPool;
        descriptorSetCount: u32;
        pSetLayouts: *VkDescriptorSetLayout;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetLayout_T :: struct {}

}

#if OS == .LINUX {
    VkDescriptorSetLayout_T :: struct {}

}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutBinding :: struct {
        binding: u32;
        descriptorType: VkDescriptorType;
        descriptorCount: u32;
        stageFlags: VkShaderStageFlags;
        pImmutableSamplers: *VkSampler;
    }

}

#if OS == .LINUX {
    VkDescriptorSetLayoutBinding :: struct {
        binding: u32;
        descriptorType: VkDescriptorType;
        descriptorCount: u32;
        stageFlags: VkShaderStageFlags;
        pImmutableSamplers: *VkSampler;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutBindingFlagsCreateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT;
        pNext: *void;
        bindingCount: u32;
        pBindingFlags: *VkDescriptorBindingFlagsEXT;
    }

}

#if OS == .LINUX {
    VkDescriptorSetLayoutBindingFlagsCreateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT;
        pNext: *void;
        bindingCount: u32;
        pBindingFlags: *VkDescriptorBindingFlagsEXT;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutCreateFlagBits :: enum_flags u32 {
        PUSH_DESCRIPTOR_BIT_KHR :: 0x1;
        UPDATE_AFTER_BIND_POOL_BIT_EXT :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DESCRIPTOR_SET_LAYOUT_CREATE_PUSH_DESCRIPTOR_BIT_KHR :: PUSH_DESCRIPTOR_BIT_KHR;
        VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT_EXT :: UPDATE_AFTER_BIND_POOL_BIT_EXT;
        VK_DESCRIPTOR_SET_LAYOUT_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDescriptorSetLayoutCreateFlagBits :: enum_flags u32 {
        PUSH_DESCRIPTOR_BIT_KHR :: 0x1;
        UPDATE_AFTER_BIND_POOL_BIT_EXT :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DESCRIPTOR_SET_LAYOUT_CREATE_PUSH_DESCRIPTOR_BIT_KHR :: PUSH_DESCRIPTOR_BIT_KHR;
        VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT_EXT :: UPDATE_AFTER_BIND_POOL_BIT_EXT;
        VK_DESCRIPTOR_SET_LAYOUT_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorSetLayoutCreateFlags;
        bindingCount: u32;
        pBindings: *VkDescriptorSetLayoutBinding;
    }

}

#if OS == .LINUX {
    VkDescriptorSetLayoutCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorSetLayoutCreateFlags;
        bindingCount: u32;
        pBindings: *VkDescriptorSetLayoutBinding;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutSupport :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_SUPPORT;
        pNext: *void;
        supported: VkBool32;
    }

}

#if OS == .LINUX {
    VkDescriptorSetLayoutSupport :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_LAYOUT_SUPPORT;
        pNext: *void;
        supported: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetVariableDescriptorCountAllocateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT;
        pNext: *void;
        descriptorSetCount: u32;
        pDescriptorCounts: *u32;
    }

}

#if OS == .LINUX {
    VkDescriptorSetVariableDescriptorCountAllocateInfoEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT;
        pNext: *void;
        descriptorSetCount: u32;
        pDescriptorCounts: *u32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorSetVariableDescriptorCountLayoutSupportEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT;
        pNext: *void;
        maxVariableDescriptorCount: u32;
    }

}

#if OS == .LINUX {
    VkDescriptorSetVariableDescriptorCountLayoutSupportEXT :: struct {
        sType: VkStructureType = .DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT;
        pNext: *void;
        maxVariableDescriptorCount: u32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorType :: enum u32 {
        SAMPLER :: 0;
        COMBINED_IMAGE_SAMPLER :: 1;
        SAMPLED_IMAGE :: 2;
        STORAGE_IMAGE :: 3;
        UNIFORM_TEXEL_BUFFER :: 4;
        STORAGE_TEXEL_BUFFER :: 5;
        UNIFORM_BUFFER :: 6;
        STORAGE_BUFFER :: 7;
        UNIFORM_BUFFER_DYNAMIC :: 8;
        STORAGE_BUFFER_DYNAMIC :: 9;
        INPUT_ATTACHMENT :: 10;
        INLINE_UNIFORM_BLOCK_EXT :: 1000138000;
        ACCELERATION_STRUCTURE_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 10;
        RANGE_SIZE :: 11;
        MAX_ENUM :: 2147483647;

        VK_DESCRIPTOR_TYPE_SAMPLER :: SAMPLER;
        VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER :: COMBINED_IMAGE_SAMPLER;
        VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE :: SAMPLED_IMAGE;
        VK_DESCRIPTOR_TYPE_STORAGE_IMAGE :: STORAGE_IMAGE;
        VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER :: UNIFORM_TEXEL_BUFFER;
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER :: STORAGE_TEXEL_BUFFER;
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER :: UNIFORM_BUFFER;
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER :: STORAGE_BUFFER;
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC :: UNIFORM_BUFFER_DYNAMIC;
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC :: STORAGE_BUFFER_DYNAMIC;
        VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT :: INPUT_ATTACHMENT;
        VK_DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK_EXT :: INLINE_UNIFORM_BLOCK_EXT;
        VK_DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NVX :: ACCELERATION_STRUCTURE_NVX;
        VK_DESCRIPTOR_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DESCRIPTOR_TYPE_END_RANGE :: END_RANGE;
        VK_DESCRIPTOR_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_DESCRIPTOR_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDescriptorType :: enum u32 {
        SAMPLER :: 0;
        COMBINED_IMAGE_SAMPLER :: 1;
        SAMPLED_IMAGE :: 2;
        STORAGE_IMAGE :: 3;
        UNIFORM_TEXEL_BUFFER :: 4;
        STORAGE_TEXEL_BUFFER :: 5;
        UNIFORM_BUFFER :: 6;
        STORAGE_BUFFER :: 7;
        UNIFORM_BUFFER_DYNAMIC :: 8;
        STORAGE_BUFFER_DYNAMIC :: 9;
        INPUT_ATTACHMENT :: 10;
        INLINE_UNIFORM_BLOCK_EXT :: 1000138000;
        ACCELERATION_STRUCTURE_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 10;
        RANGE_SIZE :: 11;
        MAX_ENUM :: 2147483647;

        VK_DESCRIPTOR_TYPE_SAMPLER :: SAMPLER;
        VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER :: COMBINED_IMAGE_SAMPLER;
        VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE :: SAMPLED_IMAGE;
        VK_DESCRIPTOR_TYPE_STORAGE_IMAGE :: STORAGE_IMAGE;
        VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER :: UNIFORM_TEXEL_BUFFER;
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER :: STORAGE_TEXEL_BUFFER;
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER :: UNIFORM_BUFFER;
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER :: STORAGE_BUFFER;
        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC :: UNIFORM_BUFFER_DYNAMIC;
        VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC :: STORAGE_BUFFER_DYNAMIC;
        VK_DESCRIPTOR_TYPE_INPUT_ATTACHMENT :: INPUT_ATTACHMENT;
        VK_DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK_EXT :: INLINE_UNIFORM_BLOCK_EXT;
        VK_DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NVX :: ACCELERATION_STRUCTURE_NVX;
        VK_DESCRIPTOR_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DESCRIPTOR_TYPE_END_RANGE :: END_RANGE;
        VK_DESCRIPTOR_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_DESCRIPTOR_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplate_T :: struct {}

}

#if OS == .LINUX {
    VkDescriptorUpdateTemplate_T :: struct {}

}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorUpdateTemplateCreateFlags;
        descriptorUpdateEntryCount: u32;
        pDescriptorUpdateEntries: *VkDescriptorUpdateTemplateEntry;
        templateType: VkDescriptorUpdateTemplateType;
        descriptorSetLayout: VkDescriptorSetLayout;
        pipelineBindPoint: VkPipelineBindPoint;
        pipelineLayout: VkPipelineLayout;
        set: u32;
    }

}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateCreateInfo :: struct {
        sType: VkStructureType = .DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO;
        pNext: *void;
        flags: VkDescriptorUpdateTemplateCreateFlags;
        descriptorUpdateEntryCount: u32;
        pDescriptorUpdateEntries: *VkDescriptorUpdateTemplateEntry;
        templateType: VkDescriptorUpdateTemplateType;
        descriptorSetLayout: VkDescriptorSetLayout;
        pipelineBindPoint: VkPipelineBindPoint;
        pipelineLayout: VkPipelineLayout;
        set: u32;
    }

}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateEntry :: struct {
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
        descriptorType: VkDescriptorType;
        offset: u64;
        stride: u64;
    }

}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateEntry :: struct {
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
        descriptorType: VkDescriptorType;
        offset: u64;
        stride: u64;
    }

}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateType :: enum u32 {
        DESCRIPTOR_SET :: 0;
        PUSH_DESCRIPTORS_KHR :: 1;
        DESCRIPTOR_SET_KHR :: 0;
        BEGIN_RANGE :: 0;
        END_RANGE :: 0;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET :: DESCRIPTOR_SET;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_PUSH_DESCRIPTORS_KHR :: PUSH_DESCRIPTORS_KHR;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET_KHR :: DESCRIPTOR_SET_KHR;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_END_RANGE :: END_RANGE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateType :: enum u32 {
        DESCRIPTOR_SET :: 0;
        PUSH_DESCRIPTORS_KHR :: 1;
        DESCRIPTOR_SET_KHR :: 0;
        BEGIN_RANGE :: 0;
        END_RANGE :: 0;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET :: DESCRIPTOR_SET;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_PUSH_DESCRIPTORS_KHR :: PUSH_DESCRIPTORS_KHR;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_DESCRIPTOR_SET_KHR :: DESCRIPTOR_SET_KHR;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_END_RANGE :: END_RANGE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_DESCRIPTOR_UPDATE_TEMPLATE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDevice_T :: struct {}

}

#if OS == .LINUX {
    VkDevice_T :: struct {}

}

#if OS == .WINDOWS {
    VkDeviceCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_CREATE_INFO;
        pNext: *void;
        flags: VkDeviceCreateFlags;
        queueCreateInfoCount: u32;
        pQueueCreateInfos: *VkDeviceQueueCreateInfo;
        enabledLayerCount: u32;
        ppEnabledLayerNames: **u8;
        enabledExtensionCount: u32;
        ppEnabledExtensionNames: **u8;
        pEnabledFeatures: *VkPhysicalDeviceFeatures;
    }

}

#if OS == .LINUX {
    VkDeviceCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_CREATE_INFO;
        pNext: *void;
        flags: VkDeviceCreateFlags;
        queueCreateInfoCount: u32;
        pQueueCreateInfos: *VkDeviceQueueCreateInfo;
        enabledLayerCount: u32;
        ppEnabledLayerNames: **u8;
        enabledExtensionCount: u32;
        ppEnabledExtensionNames: **u8;
        pEnabledFeatures: *VkPhysicalDeviceFeatures;
    }

}

#if OS == .WINDOWS {
    VkDeviceEventInfoEXT :: struct {
        sType: VkStructureType = .DEVICE_EVENT_INFO_EXT;
        pNext: *void;
        deviceEvent: VkDeviceEventTypeEXT;
    }

}

#if OS == .LINUX {
    VkDeviceEventInfoEXT :: struct {
        sType: VkStructureType = .DEVICE_EVENT_INFO_EXT;
        pNext: *void;
        deviceEvent: VkDeviceEventTypeEXT;
    }

}

#if OS == .WINDOWS {
    VkDeviceEventTypeEXT :: enum u32 {
        DISPLAY_HOTPLUG_EXT :: 0;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 0;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_DEVICE_EVENT_TYPE_DISPLAY_HOTPLUG_EXT :: DISPLAY_HOTPLUG_EXT;
        VK_DEVICE_EVENT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DEVICE_EVENT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DEVICE_EVENT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DEVICE_EVENT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDeviceEventTypeEXT :: enum u32 {
        DISPLAY_HOTPLUG_EXT :: 0;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 0;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_DEVICE_EVENT_TYPE_DISPLAY_HOTPLUG_EXT :: DISPLAY_HOTPLUG_EXT;
        VK_DEVICE_EVENT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DEVICE_EVENT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DEVICE_EVENT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DEVICE_EVENT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDeviceGeneratedCommandsFeaturesNVX :: struct {
        sType: VkStructureType = .DEVICE_GENERATED_COMMANDS_FEATURES_NVX;
        pNext: *void;
        computeBindingPointSupport: VkBool32;
    }

}

#if OS == .LINUX {
    VkDeviceGeneratedCommandsFeaturesNVX :: struct {
        sType: VkStructureType = .DEVICE_GENERATED_COMMANDS_FEATURES_NVX;
        pNext: *void;
        computeBindingPointSupport: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDeviceGeneratedCommandsLimitsNVX :: struct {
        sType: VkStructureType = .DEVICE_GENERATED_COMMANDS_LIMITS_NVX;
        pNext: *void;
        maxIndirectCommandsLayoutTokenCount: u32;
        maxObjectEntryCounts: u32;
        minSequenceCountBufferOffsetAlignment: u32;
        minSequenceIndexBufferOffsetAlignment: u32;
        minCommandsTokenBufferOffsetAlignment: u32;
    }

}

#if OS == .LINUX {
    VkDeviceGeneratedCommandsLimitsNVX :: struct {
        sType: VkStructureType = .DEVICE_GENERATED_COMMANDS_LIMITS_NVX;
        pNext: *void;
        maxIndirectCommandsLayoutTokenCount: u32;
        maxObjectEntryCounts: u32;
        minSequenceCountBufferOffsetAlignment: u32;
        minSequenceIndexBufferOffsetAlignment: u32;
        minCommandsTokenBufferOffsetAlignment: u32;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupBindSparseInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_BIND_SPARSE_INFO;
        pNext: *void;
        resourceDeviceIndex: u32;
        memoryDeviceIndex: u32;
    }

}

#if OS == .LINUX {
    VkDeviceGroupBindSparseInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_BIND_SPARSE_INFO;
        pNext: *void;
        resourceDeviceIndex: u32;
        memoryDeviceIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupCommandBufferBeginInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO;
        pNext: *void;
        deviceMask: u32;
    }

}

#if OS == .LINUX {
    VkDeviceGroupCommandBufferBeginInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO;
        pNext: *void;
        deviceMask: u32;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupDeviceCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_DEVICE_CREATE_INFO;
        pNext: *void;
        physicalDeviceCount: u32;
        pPhysicalDevices: *VkPhysicalDevice;
    }

}

#if OS == .LINUX {
    VkDeviceGroupDeviceCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_DEVICE_CREATE_INFO;
        pNext: *void;
        physicalDeviceCount: u32;
        pPhysicalDevices: *VkPhysicalDevice;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupPresentCapabilitiesKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_PRESENT_CAPABILITIES_KHR;
        pNext: *void;
        presentMask: [32] u32;
        modes: VkDeviceGroupPresentModeFlagsKHR;
    }

}

#if OS == .LINUX {
    VkDeviceGroupPresentCapabilitiesKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_PRESENT_CAPABILITIES_KHR;
        pNext: *void;
        presentMask: [32] u32;
        modes: VkDeviceGroupPresentModeFlagsKHR;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupPresentInfoKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_PRESENT_INFO_KHR;
        pNext: *void;
        swapchainCount: u32;
        pDeviceMasks: *u32;
        mode: VkDeviceGroupPresentModeFlagBitsKHR;
    }

}

#if OS == .LINUX {
    VkDeviceGroupPresentInfoKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_PRESENT_INFO_KHR;
        pNext: *void;
        swapchainCount: u32;
        pDeviceMasks: *u32;
        mode: VkDeviceGroupPresentModeFlagBitsKHR;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupPresentModeFlagBitsKHR :: enum_flags u32 {
        LOCAL_BIT_KHR :: 0x1;
        REMOTE_BIT_KHR :: 0x2;
        SUM_BIT_KHR :: 0x4;
        LOCAL_MULTI_DEVICE_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_BIT_KHR :: LOCAL_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_REMOTE_BIT_KHR :: REMOTE_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_SUM_BIT_KHR :: SUM_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_MULTI_DEVICE_BIT_KHR :: LOCAL_MULTI_DEVICE_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkDeviceGroupPresentModeFlagBitsKHR :: enum_flags u32 {
        LOCAL_BIT_KHR :: 0x1;
        REMOTE_BIT_KHR :: 0x2;
        SUM_BIT_KHR :: 0x4;
        LOCAL_MULTI_DEVICE_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_BIT_KHR :: LOCAL_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_REMOTE_BIT_KHR :: REMOTE_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_SUM_BIT_KHR :: SUM_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_LOCAL_MULTI_DEVICE_BIT_KHR :: LOCAL_MULTI_DEVICE_BIT_KHR;
        VK_DEVICE_GROUP_PRESENT_MODE_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupRenderPassBeginInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_RENDER_PASS_BEGIN_INFO;
        pNext: *void;
        deviceMask: u32;
        deviceRenderAreaCount: u32;
        pDeviceRenderAreas: *VkRect2D;
    }

}

#if OS == .LINUX {
    VkDeviceGroupRenderPassBeginInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_RENDER_PASS_BEGIN_INFO;
        pNext: *void;
        deviceMask: u32;
        deviceRenderAreaCount: u32;
        pDeviceRenderAreas: *VkRect2D;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupSubmitInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_SUBMIT_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphoreDeviceIndices: *u32;
        commandBufferCount: u32;
        pCommandBufferDeviceMasks: *u32;
        signalSemaphoreCount: u32;
        pSignalSemaphoreDeviceIndices: *u32;
    }

}

#if OS == .LINUX {
    VkDeviceGroupSubmitInfo :: struct {
        sType: VkStructureType = .DEVICE_GROUP_SUBMIT_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphoreDeviceIndices: *u32;
        commandBufferCount: u32;
        pCommandBufferDeviceMasks: *u32;
        signalSemaphoreCount: u32;
        pSignalSemaphoreDeviceIndices: *u32;
    }

}

#if OS == .WINDOWS {
    VkDeviceGroupSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        modes: VkDeviceGroupPresentModeFlagsKHR;
    }

}

#if OS == .LINUX {
    VkDeviceGroupSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        modes: VkDeviceGroupPresentModeFlagsKHR;
    }

}

#if OS == .WINDOWS {
    VkDeviceMemory_T :: struct {}

}

#if OS == .LINUX {
    VkDeviceMemory_T :: struct {}

}

#if OS == .WINDOWS {
    VkDeviceQueueCreateFlagBits :: enum_flags u32 {
        PROTECTED_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_DEVICE_QUEUE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDeviceQueueCreateFlagBits :: enum_flags u32 {
        PROTECTED_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_DEVICE_QUEUE_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_DEVICE_QUEUE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkDeviceQueueCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_CREATE_INFO;
        pNext: *void;
        flags: VkDeviceQueueCreateFlags;
        queueFamilyIndex: u32;
        queueCount: u32;
        pQueuePriorities: *float;
    }

}

#if OS == .LINUX {
    VkDeviceQueueCreateInfo :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_CREATE_INFO;
        pNext: *void;
        flags: VkDeviceQueueCreateFlags;
        queueFamilyIndex: u32;
        queueCount: u32;
        pQueuePriorities: *float;
    }

}

#if OS == .WINDOWS {
    VkDeviceQueueGlobalPriorityCreateInfoEXT :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT;
        pNext: *void;
        globalPriority: VkQueueGlobalPriorityEXT;
    }

}

#if OS == .LINUX {
    VkDeviceQueueGlobalPriorityCreateInfoEXT :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT;
        pNext: *void;
        globalPriority: VkQueueGlobalPriorityEXT;
    }

}

#if OS == .WINDOWS {
    VkDeviceQueueInfo2 :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_INFO_2;
        pNext: *void;
        flags: VkDeviceQueueCreateFlags;
        queueFamilyIndex: u32;
        queueIndex: u32;
    }

}

#if OS == .LINUX {
    VkDeviceQueueInfo2 :: struct {
        sType: VkStructureType = .DEVICE_QUEUE_INFO_2;
        pNext: *void;
        flags: VkDeviceQueueCreateFlags;
        queueFamilyIndex: u32;
        queueIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkDiscardRectangleModeEXT :: enum u32 {
        INCLUSIVE_EXT :: 0;
        EXCLUSIVE_EXT :: 1;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 2;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISCARD_RECTANGLE_MODE_INCLUSIVE_EXT :: INCLUSIVE_EXT;
        VK_DISCARD_RECTANGLE_MODE_EXCLUSIVE_EXT :: EXCLUSIVE_EXT;
        VK_DISCARD_RECTANGLE_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISCARD_RECTANGLE_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISCARD_RECTANGLE_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISCARD_RECTANGLE_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDiscardRectangleModeEXT :: enum u32 {
        INCLUSIVE_EXT :: 0;
        EXCLUSIVE_EXT :: 1;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 2;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISCARD_RECTANGLE_MODE_INCLUSIVE_EXT :: INCLUSIVE_EXT;
        VK_DISCARD_RECTANGLE_MODE_EXCLUSIVE_EXT :: EXCLUSIVE_EXT;
        VK_DISCARD_RECTANGLE_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISCARD_RECTANGLE_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISCARD_RECTANGLE_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISCARD_RECTANGLE_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDispatchIndirectCommand :: struct {
        x: u32;
        y: u32;
        z: u32;
    }

}

#if OS == .LINUX {
    VkDispatchIndirectCommand :: struct {
        x: u32;
        y: u32;
        z: u32;
    }

}

#if OS == .WINDOWS {
    VkDisplayEventInfoEXT :: struct {
        sType: VkStructureType = .DISPLAY_EVENT_INFO_EXT;
        pNext: *void;
        displayEvent: VkDisplayEventTypeEXT;
    }

}

#if OS == .LINUX {
    VkDisplayEventInfoEXT :: struct {
        sType: VkStructureType = .DISPLAY_EVENT_INFO_EXT;
        pNext: *void;
        displayEvent: VkDisplayEventTypeEXT;
    }

}

#if OS == .WINDOWS {
    VkDisplayEventTypeEXT :: enum u32 {
        FIRST_PIXEL_OUT_EXT :: 0;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 0;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISPLAY_EVENT_TYPE_FIRST_PIXEL_OUT_EXT :: FIRST_PIXEL_OUT_EXT;
        VK_DISPLAY_EVENT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISPLAY_EVENT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISPLAY_EVENT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISPLAY_EVENT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDisplayEventTypeEXT :: enum u32 {
        FIRST_PIXEL_OUT_EXT :: 0;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 0;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISPLAY_EVENT_TYPE_FIRST_PIXEL_OUT_EXT :: FIRST_PIXEL_OUT_EXT;
        VK_DISPLAY_EVENT_TYPE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISPLAY_EVENT_TYPE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISPLAY_EVENT_TYPE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISPLAY_EVENT_TYPE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDisplayKHR_T :: struct {}

}

#if OS == .LINUX {
    VkDisplayKHR_T :: struct {}

}

#if OS == .WINDOWS {
    VkDisplayModeCreateInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_MODE_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkDisplayModeCreateFlagsKHR;
        parameters: VkDisplayModeParametersKHR;
    }

}

#if OS == .LINUX {
    VkDisplayModeCreateInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_MODE_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkDisplayModeCreateFlagsKHR;
        parameters: VkDisplayModeParametersKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayModeKHR_T :: struct {}

}

#if OS == .LINUX {
    VkDisplayModeKHR_T :: struct {}

}

#if OS == .WINDOWS {
    VkDisplayModeParametersKHR :: struct {
        visibleRegion: VkExtent2D;
        refreshRate: u32;
    }

}

#if OS == .LINUX {
    VkDisplayModeParametersKHR :: struct {
        visibleRegion: VkExtent2D;
        refreshRate: u32;
    }

}

#if OS == .WINDOWS {
    VkDisplayModeProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_MODE_PROPERTIES_2_KHR;
        pNext: *void;
        displayModeProperties: VkDisplayModePropertiesKHR;
    }

}

#if OS == .LINUX {
    VkDisplayModeProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_MODE_PROPERTIES_2_KHR;
        pNext: *void;
        displayModeProperties: VkDisplayModePropertiesKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayModePropertiesKHR :: struct {
        displayMode: VkDisplayModeKHR;
        parameters: VkDisplayModeParametersKHR;
    }

}

#if OS == .LINUX {
    VkDisplayModePropertiesKHR :: struct {
        displayMode: VkDisplayModeKHR;
        parameters: VkDisplayModeParametersKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlaneAlphaFlagBitsKHR :: enum_flags u32 {
        OPAQUE_BIT_KHR :: 0x1;
        GLOBAL_BIT_KHR :: 0x2;
        PER_PIXEL_BIT_KHR :: 0x4;
        PER_PIXEL_PREMULTIPLIED_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR :: OPAQUE_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_GLOBAL_BIT_KHR :: GLOBAL_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_BIT_KHR :: PER_PIXEL_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_PREMULTIPLIED_BIT_KHR :: PER_PIXEL_PREMULTIPLIED_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkDisplayPlaneAlphaFlagBitsKHR :: enum_flags u32 {
        OPAQUE_BIT_KHR :: 0x1;
        GLOBAL_BIT_KHR :: 0x2;
        PER_PIXEL_BIT_KHR :: 0x4;
        PER_PIXEL_PREMULTIPLIED_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR :: OPAQUE_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_GLOBAL_BIT_KHR :: GLOBAL_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_BIT_KHR :: PER_PIXEL_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_PREMULTIPLIED_BIT_KHR :: PER_PIXEL_PREMULTIPLIED_BIT_KHR;
        VK_DISPLAY_PLANE_ALPHA_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlaneCapabilities2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_CAPABILITIES_2_KHR;
        pNext: *void;
        capabilities: VkDisplayPlaneCapabilitiesKHR;
    }

}

#if OS == .LINUX {
    VkDisplayPlaneCapabilities2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_CAPABILITIES_2_KHR;
        pNext: *void;
        capabilities: VkDisplayPlaneCapabilitiesKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlaneCapabilitiesKHR :: struct {
        supportedAlpha: VkDisplayPlaneAlphaFlagsKHR;
        minSrcPosition: VkOffset2D;
        maxSrcPosition: VkOffset2D;
        minSrcExtent: VkExtent2D;
        maxSrcExtent: VkExtent2D;
        minDstPosition: VkOffset2D;
        maxDstPosition: VkOffset2D;
        minDstExtent: VkExtent2D;
        maxDstExtent: VkExtent2D;
    }

}

#if OS == .LINUX {
    VkDisplayPlaneCapabilitiesKHR :: struct {
        supportedAlpha: VkDisplayPlaneAlphaFlagsKHR;
        minSrcPosition: VkOffset2D;
        maxSrcPosition: VkOffset2D;
        minSrcExtent: VkExtent2D;
        maxSrcExtent: VkExtent2D;
        minDstPosition: VkOffset2D;
        maxDstPosition: VkOffset2D;
        minDstExtent: VkExtent2D;
        maxDstExtent: VkExtent2D;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlaneInfo2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_INFO_2_KHR;
        pNext: *void;
        mode: VkDisplayModeKHR;
        planeIndex: u32;
    }

}

#if OS == .LINUX {
    VkDisplayPlaneInfo2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_INFO_2_KHR;
        pNext: *void;
        mode: VkDisplayModeKHR;
        planeIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlaneProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_PROPERTIES_2_KHR;
        pNext: *void;
        displayPlaneProperties: VkDisplayPlanePropertiesKHR;
    }

}

#if OS == .LINUX {
    VkDisplayPlaneProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PLANE_PROPERTIES_2_KHR;
        pNext: *void;
        displayPlaneProperties: VkDisplayPlanePropertiesKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayPlanePropertiesKHR :: struct {
        currentDisplay: VkDisplayKHR;
        currentStackIndex: u32;
    }

}

#if OS == .LINUX {
    VkDisplayPlanePropertiesKHR :: struct {
        currentDisplay: VkDisplayKHR;
        currentStackIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkDisplayPowerInfoEXT :: struct {
        sType: VkStructureType = .DISPLAY_POWER_INFO_EXT;
        pNext: *void;
        powerState: VkDisplayPowerStateEXT;
    }

}

#if OS == .LINUX {
    VkDisplayPowerInfoEXT :: struct {
        sType: VkStructureType = .DISPLAY_POWER_INFO_EXT;
        pNext: *void;
        powerState: VkDisplayPowerStateEXT;
    }

}

#if OS == .WINDOWS {
    VkDisplayPowerStateEXT :: enum u32 {
        OFF_EXT :: 0;
        SUSPEND_EXT :: 1;
        ON_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISPLAY_POWER_STATE_OFF_EXT :: OFF_EXT;
        VK_DISPLAY_POWER_STATE_SUSPEND_EXT :: SUSPEND_EXT;
        VK_DISPLAY_POWER_STATE_ON_EXT :: ON_EXT;
        VK_DISPLAY_POWER_STATE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISPLAY_POWER_STATE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISPLAY_POWER_STATE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISPLAY_POWER_STATE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkDisplayPowerStateEXT :: enum u32 {
        OFF_EXT :: 0;
        SUSPEND_EXT :: 1;
        ON_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_DISPLAY_POWER_STATE_OFF_EXT :: OFF_EXT;
        VK_DISPLAY_POWER_STATE_SUSPEND_EXT :: SUSPEND_EXT;
        VK_DISPLAY_POWER_STATE_ON_EXT :: ON_EXT;
        VK_DISPLAY_POWER_STATE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_DISPLAY_POWER_STATE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_DISPLAY_POWER_STATE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_DISPLAY_POWER_STATE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkDisplayPresentInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_PRESENT_INFO_KHR;
        pNext: *void;
        srcRect: VkRect2D;
        dstRect: VkRect2D;
        persistent: VkBool32;
    }

}

#if OS == .LINUX {
    VkDisplayPresentInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_PRESENT_INFO_KHR;
        pNext: *void;
        srcRect: VkRect2D;
        dstRect: VkRect2D;
        persistent: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDisplayProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PROPERTIES_2_KHR;
        pNext: *void;
        displayProperties: VkDisplayPropertiesKHR;
    }

}

#if OS == .LINUX {
    VkDisplayProperties2KHR :: struct {
        sType: VkStructureType = .DISPLAY_PROPERTIES_2_KHR;
        pNext: *void;
        displayProperties: VkDisplayPropertiesKHR;
    }

}

#if OS == .WINDOWS {
    VkDisplayPropertiesKHR :: struct {
        display: VkDisplayKHR;
        displayName: *u8;
        physicalDimensions: VkExtent2D;
        physicalResolution: VkExtent2D;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        planeReorderPossible: VkBool32;
        persistentContent: VkBool32;
    }

}

#if OS == .LINUX {
    VkDisplayPropertiesKHR :: struct {
        display: VkDisplayKHR;
        displayName: *u8;
        physicalDimensions: VkExtent2D;
        physicalResolution: VkExtent2D;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        planeReorderPossible: VkBool32;
        persistentContent: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkDisplaySurfaceCreateInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_SURFACE_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkDisplaySurfaceCreateFlagsKHR;
        displayMode: VkDisplayModeKHR;
        planeIndex: u32;
        planeStackIndex: u32;
        transform: VkSurfaceTransformFlagBitsKHR;
        globalAlpha: float;
        alphaMode: VkDisplayPlaneAlphaFlagBitsKHR;
        imageExtent: VkExtent2D;
    }

}

#if OS == .LINUX {
    VkDisplaySurfaceCreateInfoKHR :: struct {
        sType: VkStructureType = .DISPLAY_SURFACE_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkDisplaySurfaceCreateFlagsKHR;
        displayMode: VkDisplayModeKHR;
        planeIndex: u32;
        planeStackIndex: u32;
        transform: VkSurfaceTransformFlagBitsKHR;
        globalAlpha: float;
        alphaMode: VkDisplayPlaneAlphaFlagBitsKHR;
        imageExtent: VkExtent2D;
    }

}

#if OS == .WINDOWS {
    VkDrawIndexedIndirectCommand :: struct {
        indexCount: u32;
        instanceCount: u32;
        firstIndex: u32;
        vertexOffset: s32;
        firstInstance: u32;
    }

}

#if OS == .LINUX {
    VkDrawIndexedIndirectCommand :: struct {
        indexCount: u32;
        instanceCount: u32;
        firstIndex: u32;
        vertexOffset: s32;
        firstInstance: u32;
    }

}

#if OS == .WINDOWS {
    VkDrawIndirectCommand :: struct {
        vertexCount: u32;
        instanceCount: u32;
        firstVertex: u32;
        firstInstance: u32;
    }

}

#if OS == .LINUX {
    VkDrawIndirectCommand :: struct {
        vertexCount: u32;
        instanceCount: u32;
        firstVertex: u32;
        firstInstance: u32;
    }

}

#if OS == .WINDOWS {
    VkDrawMeshTasksIndirectCommandNV :: struct {
        taskCount: u32;
        firstTask: u32;
    }

}

#if OS == .LINUX {
    VkDrawMeshTasksIndirectCommandNV :: struct {
        taskCount: u32;
        firstTask: u32;
    }

}

#if OS == .WINDOWS {
    VkDynamicState :: enum u32 {
        VIEWPORT :: 0;
        SCISSOR :: 1;
        LINE_WIDTH :: 2;
        DEPTH_BIAS :: 3;
        BLEND_CONSTANTS :: 4;
        DEPTH_BOUNDS :: 5;
        STENCIL_COMPARE_MASK :: 6;
        STENCIL_WRITE_MASK :: 7;
        STENCIL_REFERENCE :: 8;
        VIEWPORT_W_SCALING_NV :: 1000087000;
        DISCARD_RECTANGLE_EXT :: 1000099000;
        SAMPLE_LOCATIONS_EXT :: 1000143000;
        VIEWPORT_SHADING_RATE_PALETTE_NV :: 1000164004;
        VIEWPORT_COARSE_SAMPLE_ORDER_NV :: 1000164006;
        EXCLUSIVE_SCISSOR_NV :: 1000205001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 8;
        RANGE_SIZE :: 9;
        MAX_ENUM :: 2147483647;

        VK_DYNAMIC_STATE_VIEWPORT :: VIEWPORT;
        VK_DYNAMIC_STATE_SCISSOR :: SCISSOR;
        VK_DYNAMIC_STATE_LINE_WIDTH :: LINE_WIDTH;
        VK_DYNAMIC_STATE_DEPTH_BIAS :: DEPTH_BIAS;
        VK_DYNAMIC_STATE_BLEND_CONSTANTS :: BLEND_CONSTANTS;
        VK_DYNAMIC_STATE_DEPTH_BOUNDS :: DEPTH_BOUNDS;
        VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK :: STENCIL_COMPARE_MASK;
        VK_DYNAMIC_STATE_STENCIL_WRITE_MASK :: STENCIL_WRITE_MASK;
        VK_DYNAMIC_STATE_STENCIL_REFERENCE :: STENCIL_REFERENCE;
        VK_DYNAMIC_STATE_VIEWPORT_W_SCALING_NV :: VIEWPORT_W_SCALING_NV;
        VK_DYNAMIC_STATE_DISCARD_RECTANGLE_EXT :: DISCARD_RECTANGLE_EXT;
        VK_DYNAMIC_STATE_SAMPLE_LOCATIONS_EXT :: SAMPLE_LOCATIONS_EXT;
        VK_DYNAMIC_STATE_VIEWPORT_SHADING_RATE_PALETTE_NV :: VIEWPORT_SHADING_RATE_PALETTE_NV;
        VK_DYNAMIC_STATE_VIEWPORT_COARSE_SAMPLE_ORDER_NV :: VIEWPORT_COARSE_SAMPLE_ORDER_NV;
        VK_DYNAMIC_STATE_EXCLUSIVE_SCISSOR_NV :: EXCLUSIVE_SCISSOR_NV;
        VK_DYNAMIC_STATE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DYNAMIC_STATE_END_RANGE :: END_RANGE;
        VK_DYNAMIC_STATE_RANGE_SIZE :: RANGE_SIZE;
        VK_DYNAMIC_STATE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkDynamicState :: enum u32 {
        VIEWPORT :: 0;
        SCISSOR :: 1;
        LINE_WIDTH :: 2;
        DEPTH_BIAS :: 3;
        BLEND_CONSTANTS :: 4;
        DEPTH_BOUNDS :: 5;
        STENCIL_COMPARE_MASK :: 6;
        STENCIL_WRITE_MASK :: 7;
        STENCIL_REFERENCE :: 8;
        VIEWPORT_W_SCALING_NV :: 1000087000;
        DISCARD_RECTANGLE_EXT :: 1000099000;
        SAMPLE_LOCATIONS_EXT :: 1000143000;
        VIEWPORT_SHADING_RATE_PALETTE_NV :: 1000164004;
        VIEWPORT_COARSE_SAMPLE_ORDER_NV :: 1000164006;
        EXCLUSIVE_SCISSOR_NV :: 1000205001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 8;
        RANGE_SIZE :: 9;
        MAX_ENUM :: 2147483647;

        VK_DYNAMIC_STATE_VIEWPORT :: VIEWPORT;
        VK_DYNAMIC_STATE_SCISSOR :: SCISSOR;
        VK_DYNAMIC_STATE_LINE_WIDTH :: LINE_WIDTH;
        VK_DYNAMIC_STATE_DEPTH_BIAS :: DEPTH_BIAS;
        VK_DYNAMIC_STATE_BLEND_CONSTANTS :: BLEND_CONSTANTS;
        VK_DYNAMIC_STATE_DEPTH_BOUNDS :: DEPTH_BOUNDS;
        VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK :: STENCIL_COMPARE_MASK;
        VK_DYNAMIC_STATE_STENCIL_WRITE_MASK :: STENCIL_WRITE_MASK;
        VK_DYNAMIC_STATE_STENCIL_REFERENCE :: STENCIL_REFERENCE;
        VK_DYNAMIC_STATE_VIEWPORT_W_SCALING_NV :: VIEWPORT_W_SCALING_NV;
        VK_DYNAMIC_STATE_DISCARD_RECTANGLE_EXT :: DISCARD_RECTANGLE_EXT;
        VK_DYNAMIC_STATE_SAMPLE_LOCATIONS_EXT :: SAMPLE_LOCATIONS_EXT;
        VK_DYNAMIC_STATE_VIEWPORT_SHADING_RATE_PALETTE_NV :: VIEWPORT_SHADING_RATE_PALETTE_NV;
        VK_DYNAMIC_STATE_VIEWPORT_COARSE_SAMPLE_ORDER_NV :: VIEWPORT_COARSE_SAMPLE_ORDER_NV;
        VK_DYNAMIC_STATE_EXCLUSIVE_SCISSOR_NV :: EXCLUSIVE_SCISSOR_NV;
        VK_DYNAMIC_STATE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_DYNAMIC_STATE_END_RANGE :: END_RANGE;
        VK_DYNAMIC_STATE_RANGE_SIZE :: RANGE_SIZE;
        VK_DYNAMIC_STATE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkEvent_T :: struct {}

}

#if OS == .LINUX {
    VkEvent_T :: struct {}

}

#if OS == .WINDOWS {
    VkEventCreateInfo :: struct {
        sType: VkStructureType = .EVENT_CREATE_INFO;
        pNext: *void;
        flags: VkEventCreateFlags;
    }

}

#if OS == .LINUX {
    VkEventCreateInfo :: struct {
        sType: VkStructureType = .EVENT_CREATE_INFO;
        pNext: *void;
        flags: VkEventCreateFlags;
    }

}

#if OS == .WINDOWS {
    VkExportFenceCreateInfo :: struct {
        sType: VkStructureType = .EXPORT_FENCE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalFenceHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExportFenceCreateInfo :: struct {
        sType: VkStructureType = .EXPORT_FENCE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalFenceHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExportFenceWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .EXPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        pAttributes: *SECURITY_ATTRIBUTES;
        dwAccess: u32;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkExportMemoryAllocateInfo :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_ALLOCATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExportMemoryAllocateInfo :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_ALLOCATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExportMemoryAllocateInfoNV :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_ALLOCATE_INFO_NV;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .LINUX {
    VkExportMemoryAllocateInfoNV :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_ALLOCATE_INFO_NV;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .WINDOWS {
    VkExportMemoryWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        pAttributes: *SECURITY_ATTRIBUTES;
        dwAccess: u32;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkExportMemoryWin32HandleInfoNV :: struct {
        sType: VkStructureType = .EXPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        pNext: *void;
        pAttributes: *SECURITY_ATTRIBUTES;
        dwAccess: u32;
    }

}

#if OS == .WINDOWS {
    VkExportSemaphoreCreateInfo :: struct {
        sType: VkStructureType = .EXPORT_SEMAPHORE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalSemaphoreHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExportSemaphoreCreateInfo :: struct {
        sType: VkStructureType = .EXPORT_SEMAPHORE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalSemaphoreHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExportSemaphoreWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        pAttributes: *SECURITY_ATTRIBUTES;
        dwAccess: u32;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkExtensionProperties :: struct {
        extensionName: [256] u8;
        specVersion: u32;
    }

}

#if OS == .LINUX {
    VkExtensionProperties :: struct {
        extensionName: [256] u8;
        specVersion: u32;
    }

}

#if OS == .WINDOWS {
    VkExtent2D :: struct {
        width: u32;
        height: u32;
    }

}

#if OS == .LINUX {
    VkExtent2D :: struct {
        width: u32;
        height: u32;
    }

}

#if OS == .WINDOWS {
    VkExtent3D :: struct {
        width: u32;
        height: u32;
        depth: u32;
    }

}

#if OS == .LINUX {
    VkExtent3D :: struct {
        width: u32;
        height: u32;
        depth: u32;
    }

}

#if OS == .WINDOWS {
    VkExternalBufferProperties :: struct {
        sType: VkStructureType = .EXTERNAL_BUFFER_PROPERTIES;
        pNext: *void;
        externalMemoryProperties: VkExternalMemoryProperties;
    }

}

#if OS == .LINUX {
    VkExternalBufferProperties :: struct {
        sType: VkStructureType = .EXTERNAL_BUFFER_PROPERTIES;
        pNext: *void;
        externalMemoryProperties: VkExternalMemoryProperties;
    }

}

#if OS == .WINDOWS {
    VkExternalFenceFeatureFlagBits :: enum_flags u32 {
        EXPORTABLE_BIT :: 0x1;
        IMPORTABLE_BIT :: 0x2;
        EXPORTABLE_BIT_KHR :: 0x1;
        IMPORTABLE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_FENCE_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_FENCE_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_FENCE_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_FENCE_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_FENCE_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalFenceFeatureFlagBits :: enum_flags u32 {
        EXPORTABLE_BIT :: 0x1;
        IMPORTABLE_BIT :: 0x2;
        EXPORTABLE_BIT_KHR :: 0x1;
        IMPORTABLE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_FENCE_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_FENCE_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_FENCE_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_FENCE_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_FENCE_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalFenceHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        SYNC_FD_BIT :: 0x8;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        SYNC_FD_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_SYNC_FD_BIT :: SYNC_FD_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_SYNC_FD_BIT_KHR :: SYNC_FD_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalFenceHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        SYNC_FD_BIT :: 0x8;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        SYNC_FD_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_SYNC_FD_BIT :: SYNC_FD_BIT;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_SYNC_FD_BIT_KHR :: SYNC_FD_BIT_KHR;
        VK_EXTERNAL_FENCE_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalFenceProperties :: struct {
        sType: VkStructureType = .EXTERNAL_FENCE_PROPERTIES;
        pNext: *void;
        exportFromImportedHandleTypes: VkExternalFenceHandleTypeFlags;
        compatibleHandleTypes: VkExternalFenceHandleTypeFlags;
        externalFenceFeatures: VkExternalFenceFeatureFlags;
    }

}

#if OS == .LINUX {
    VkExternalFenceProperties :: struct {
        sType: VkStructureType = .EXTERNAL_FENCE_PROPERTIES;
        pNext: *void;
        exportFromImportedHandleTypes: VkExternalFenceHandleTypeFlags;
        compatibleHandleTypes: VkExternalFenceHandleTypeFlags;
        externalFenceFeatures: VkExternalFenceFeatureFlags;
    }

}

#if OS == .WINDOWS {
    VkExternalImageFormatProperties :: struct {
        sType: VkStructureType = .EXTERNAL_IMAGE_FORMAT_PROPERTIES;
        pNext: *void;
        externalMemoryProperties: VkExternalMemoryProperties;
    }

}

#if OS == .LINUX {
    VkExternalImageFormatProperties :: struct {
        sType: VkStructureType = .EXTERNAL_IMAGE_FORMAT_PROPERTIES;
        pNext: *void;
        externalMemoryProperties: VkExternalMemoryProperties;
    }

}

#if OS == .WINDOWS {
    VkExternalImageFormatPropertiesNV :: struct {
        imageFormatProperties: VkImageFormatProperties;
        externalMemoryFeatures: VkExternalMemoryFeatureFlagsNV;
        exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlagsNV;
        compatibleHandleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .LINUX {
    VkExternalImageFormatPropertiesNV :: struct {
        imageFormatProperties: VkImageFormatProperties;
        externalMemoryFeatures: VkExternalMemoryFeatureFlagsNV;
        exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlagsNV;
        compatibleHandleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryBufferCreateInfo :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_BUFFER_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExternalMemoryBufferCreateInfo :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_BUFFER_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlagBits :: enum_flags u32 {
        DEDICATED_ONLY_BIT :: 0x1;
        EXPORTABLE_BIT :: 0x2;
        IMPORTABLE_BIT :: 0x4;
        DEDICATED_ONLY_BIT_KHR :: 0x1;
        EXPORTABLE_BIT_KHR :: 0x2;
        IMPORTABLE_BIT_KHR :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT :: DEDICATED_ONLY_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_KHR :: DEDICATED_ONLY_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlagBits :: enum_flags u32 {
        DEDICATED_ONLY_BIT :: 0x1;
        EXPORTABLE_BIT :: 0x2;
        IMPORTABLE_BIT :: 0x4;
        DEDICATED_ONLY_BIT_KHR :: 0x1;
        EXPORTABLE_BIT_KHR :: 0x2;
        IMPORTABLE_BIT_KHR :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT :: DEDICATED_ONLY_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_KHR :: DEDICATED_ONLY_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlagBitsNV :: enum_flags u32 {
        DEDICATED_ONLY_BIT_NV :: 0x1;
        EXPORTABLE_BIT_NV :: 0x2;
        IMPORTABLE_BIT_NV :: 0x4;
        FLAG_BITS_MAX_ENUM_NV :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_NV :: DEDICATED_ONLY_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_NV :: EXPORTABLE_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_NV :: IMPORTABLE_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM_NV :: FLAG_BITS_MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlagBitsNV :: enum_flags u32 {
        DEDICATED_ONLY_BIT_NV :: 0x1;
        EXPORTABLE_BIT_NV :: 0x2;
        IMPORTABLE_BIT_NV :: 0x4;
        FLAG_BITS_MAX_ENUM_NV :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_FEATURE_DEDICATED_ONLY_BIT_NV :: DEDICATED_ONLY_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_EXPORTABLE_BIT_NV :: EXPORTABLE_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_IMPORTABLE_BIT_NV :: IMPORTABLE_BIT_NV;
        VK_EXTERNAL_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM_NV :: FLAG_BITS_MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        D3D11_TEXTURE_BIT :: 0x8;
        D3D11_TEXTURE_KMT_BIT :: 0x10;
        D3D12_HEAP_BIT :: 0x20;
        D3D12_RESOURCE_BIT :: 0x40;
        DMA_BUF_BIT_EXT :: 0x200;
        ANDROID_HARDWARE_BUFFER_BIT_ANDROID :: 0x400;
        HOST_ALLOCATION_BIT_EXT :: 0x80;
        HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT :: 0x100;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        D3D11_TEXTURE_BIT_KHR :: 0x8;
        D3D11_TEXTURE_KMT_BIT_KHR :: 0x10;
        D3D12_HEAP_BIT_KHR :: 0x20;
        D3D12_RESOURCE_BIT_KHR :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT :: D3D11_TEXTURE_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT :: D3D11_TEXTURE_KMT_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT :: D3D12_HEAP_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT :: D3D12_RESOURCE_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_DMA_BUF_BIT_EXT :: DMA_BUF_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID :: ANDROID_HARDWARE_BUFFER_BIT_ANDROID;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT :: HOST_ALLOCATION_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT :: HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT_KHR :: D3D11_TEXTURE_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT_KHR :: D3D11_TEXTURE_KMT_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT_KHR :: D3D12_HEAP_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT_KHR :: D3D12_RESOURCE_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        D3D11_TEXTURE_BIT :: 0x8;
        D3D11_TEXTURE_KMT_BIT :: 0x10;
        D3D12_HEAP_BIT :: 0x20;
        D3D12_RESOURCE_BIT :: 0x40;
        DMA_BUF_BIT_EXT :: 0x200;
        ANDROID_HARDWARE_BUFFER_BIT_ANDROID :: 0x400;
        HOST_ALLOCATION_BIT_EXT :: 0x80;
        HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT :: 0x100;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        D3D11_TEXTURE_BIT_KHR :: 0x8;
        D3D11_TEXTURE_KMT_BIT_KHR :: 0x10;
        D3D12_HEAP_BIT_KHR :: 0x20;
        D3D12_RESOURCE_BIT_KHR :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT :: D3D11_TEXTURE_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT :: D3D11_TEXTURE_KMT_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT :: D3D12_HEAP_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT :: D3D12_RESOURCE_BIT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_DMA_BUF_BIT_EXT :: DMA_BUF_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_ANDROID_HARDWARE_BUFFER_BIT_ANDROID :: ANDROID_HARDWARE_BUFFER_BIT_ANDROID;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_ALLOCATION_BIT_EXT :: HOST_ALLOCATION_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT :: HOST_MAPPED_FOREIGN_MEMORY_BIT_EXT;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_BIT_KHR :: D3D11_TEXTURE_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_TEXTURE_KMT_BIT_KHR :: D3D11_TEXTURE_KMT_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_HEAP_BIT_KHR :: D3D12_HEAP_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D12_RESOURCE_BIT_KHR :: D3D12_RESOURCE_BIT_KHR;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlagBitsNV :: enum_flags u32 {
        OPAQUE_WIN32_BIT_NV :: 0x1;
        OPAQUE_WIN32_KMT_BIT_NV :: 0x2;
        D3D11_IMAGE_BIT_NV :: 0x4;
        D3D11_IMAGE_KMT_BIT_NV :: 0x8;
        FLAG_BITS_MAX_ENUM_NV :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_NV :: OPAQUE_WIN32_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_NV :: OPAQUE_WIN32_KMT_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_BIT_NV :: D3D11_IMAGE_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_KMT_BIT_NV :: D3D11_IMAGE_KMT_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM_NV :: FLAG_BITS_MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlagBitsNV :: enum_flags u32 {
        OPAQUE_WIN32_BIT_NV :: 0x1;
        OPAQUE_WIN32_KMT_BIT_NV :: 0x2;
        D3D11_IMAGE_BIT_NV :: 0x4;
        D3D11_IMAGE_KMT_BIT_NV :: 0x8;
        FLAG_BITS_MAX_ENUM_NV :: 0x7fffffff;

        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_BIT_NV :: OPAQUE_WIN32_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_NV :: OPAQUE_WIN32_KMT_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_BIT_NV :: D3D11_IMAGE_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_D3D11_IMAGE_KMT_BIT_NV :: D3D11_IMAGE_KMT_BIT_NV;
        VK_EXTERNAL_MEMORY_HANDLE_TYPE_FLAG_BITS_MAX_ENUM_NV :: FLAG_BITS_MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryImageCreateInfo :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_IMAGE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExternalMemoryImageCreateInfo :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_IMAGE_CREATE_INFO;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryImageCreateInfoNV :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .LINUX {
    VkExternalMemoryImageCreateInfoNV :: struct {
        sType: VkStructureType = .EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV;
        pNext: *void;
        handleTypes: VkExternalMemoryHandleTypeFlagsNV;
    }

}

#if OS == .WINDOWS {
    VkExternalMemoryProperties :: struct {
        externalMemoryFeatures: VkExternalMemoryFeatureFlags;
        exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlags;
        compatibleHandleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .LINUX {
    VkExternalMemoryProperties :: struct {
        externalMemoryFeatures: VkExternalMemoryFeatureFlags;
        exportFromImportedHandleTypes: VkExternalMemoryHandleTypeFlags;
        compatibleHandleTypes: VkExternalMemoryHandleTypeFlags;
    }

}

#if OS == .WINDOWS {
    VkExternalSemaphoreFeatureFlagBits :: enum_flags u32 {
        EXPORTABLE_BIT :: 0x1;
        IMPORTABLE_BIT :: 0x2;
        EXPORTABLE_BIT_KHR :: 0x1;
        IMPORTABLE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_SEMAPHORE_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_SEMAPHORE_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_SEMAPHORE_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalSemaphoreFeatureFlagBits :: enum_flags u32 {
        EXPORTABLE_BIT :: 0x1;
        IMPORTABLE_BIT :: 0x2;
        EXPORTABLE_BIT_KHR :: 0x1;
        IMPORTABLE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_SEMAPHORE_FEATURE_EXPORTABLE_BIT :: EXPORTABLE_BIT;
        VK_EXTERNAL_SEMAPHORE_FEATURE_IMPORTABLE_BIT :: IMPORTABLE_BIT;
        VK_EXTERNAL_SEMAPHORE_FEATURE_EXPORTABLE_BIT_KHR :: EXPORTABLE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_FEATURE_IMPORTABLE_BIT_KHR :: IMPORTABLE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalSemaphoreHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        D3D12_FENCE_BIT :: 0x8;
        SYNC_FD_BIT :: 0x10;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        D3D12_FENCE_BIT_KHR :: 0x8;
        SYNC_FD_BIT_KHR :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_D3D12_FENCE_BIT :: D3D12_FENCE_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_SYNC_FD_BIT :: SYNC_FD_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_D3D12_FENCE_BIT_KHR :: D3D12_FENCE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_SYNC_FD_BIT_KHR :: SYNC_FD_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkExternalSemaphoreHandleTypeFlagBits :: enum_flags u32 {
        OPAQUE_FD_BIT :: 0x1;
        OPAQUE_WIN32_BIT :: 0x2;
        OPAQUE_WIN32_KMT_BIT :: 0x4;
        D3D12_FENCE_BIT :: 0x8;
        SYNC_FD_BIT :: 0x10;
        OPAQUE_FD_BIT_KHR :: 0x1;
        OPAQUE_WIN32_BIT_KHR :: 0x2;
        OPAQUE_WIN32_KMT_BIT_KHR :: 0x4;
        D3D12_FENCE_BIT_KHR :: 0x8;
        SYNC_FD_BIT_KHR :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT :: OPAQUE_FD_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_BIT :: OPAQUE_WIN32_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT :: OPAQUE_WIN32_KMT_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_D3D12_FENCE_BIT :: D3D12_FENCE_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_SYNC_FD_BIT :: SYNC_FD_BIT;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD_BIT_KHR :: OPAQUE_FD_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_BIT_KHR :: OPAQUE_WIN32_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32_KMT_BIT_KHR :: OPAQUE_WIN32_KMT_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_D3D12_FENCE_BIT_KHR :: D3D12_FENCE_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_SYNC_FD_BIT_KHR :: SYNC_FD_BIT_KHR;
        VK_EXTERNAL_SEMAPHORE_HANDLE_TYPE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkExternalSemaphoreProperties :: struct {
        sType: VkStructureType = .EXTERNAL_SEMAPHORE_PROPERTIES;
        pNext: *void;
        exportFromImportedHandleTypes: VkExternalSemaphoreHandleTypeFlags;
        compatibleHandleTypes: VkExternalSemaphoreHandleTypeFlags;
        externalSemaphoreFeatures: VkExternalSemaphoreFeatureFlags;
    }

}

#if OS == .LINUX {
    VkExternalSemaphoreProperties :: struct {
        sType: VkStructureType = .EXTERNAL_SEMAPHORE_PROPERTIES;
        pNext: *void;
        exportFromImportedHandleTypes: VkExternalSemaphoreHandleTypeFlags;
        compatibleHandleTypes: VkExternalSemaphoreHandleTypeFlags;
        externalSemaphoreFeatures: VkExternalSemaphoreFeatureFlags;
    }

}

#if OS == .WINDOWS {
    VkFence_T :: struct {}

}

#if OS == .LINUX {
    VkFence_T :: struct {}

}

#if OS == .WINDOWS {
    VkFenceCreateFlagBits :: enum_flags u32 {
        SIGNALED_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FENCE_CREATE_SIGNALED_BIT :: SIGNALED_BIT;
        VK_FENCE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFenceCreateFlagBits :: enum_flags u32 {
        SIGNALED_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FENCE_CREATE_SIGNALED_BIT :: SIGNALED_BIT;
        VK_FENCE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkFenceCreateInfo :: struct {
        sType: VkStructureType = .FENCE_CREATE_INFO;
        pNext: *void;
        flags: VkFenceCreateFlags;
    }

}

#if OS == .LINUX {
    VkFenceCreateInfo :: struct {
        sType: VkStructureType = .FENCE_CREATE_INFO;
        pNext: *void;
        flags: VkFenceCreateFlags;
    }

}

#if OS == .WINDOWS {
    VkFenceGetFdInfoKHR :: struct {
        sType: VkStructureType = .FENCE_GET_FD_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        handleType: VkExternalFenceHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkFenceGetFdInfoKHR :: struct {
        sType: VkStructureType = .FENCE_GET_FD_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        handleType: VkExternalFenceHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkFenceGetWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .FENCE_GET_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        handleType: VkExternalFenceHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkFenceImportFlagBits :: enum_flags u32 {
        TEMPORARY_BIT :: 0x1;
        TEMPORARY_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FENCE_IMPORT_TEMPORARY_BIT :: TEMPORARY_BIT;
        VK_FENCE_IMPORT_TEMPORARY_BIT_KHR :: TEMPORARY_BIT_KHR;
        VK_FENCE_IMPORT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFenceImportFlagBits :: enum_flags u32 {
        TEMPORARY_BIT :: 0x1;
        TEMPORARY_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FENCE_IMPORT_TEMPORARY_BIT :: TEMPORARY_BIT;
        VK_FENCE_IMPORT_TEMPORARY_BIT_KHR :: TEMPORARY_BIT_KHR;
        VK_FENCE_IMPORT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkFilter :: enum u32 {
        NEAREST :: 0;
        LINEAR :: 1;
        CUBIC_IMG :: 1000015000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_FILTER_NEAREST :: NEAREST;
        VK_FILTER_LINEAR :: LINEAR;
        VK_FILTER_CUBIC_IMG :: CUBIC_IMG;
        VK_FILTER_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FILTER_END_RANGE :: END_RANGE;
        VK_FILTER_RANGE_SIZE :: RANGE_SIZE;
        VK_FILTER_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFilter :: enum u32 {
        NEAREST :: 0;
        LINEAR :: 1;
        CUBIC_IMG :: 1000015000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_FILTER_NEAREST :: NEAREST;
        VK_FILTER_LINEAR :: LINEAR;
        VK_FILTER_CUBIC_IMG :: CUBIC_IMG;
        VK_FILTER_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FILTER_END_RANGE :: END_RANGE;
        VK_FILTER_RANGE_SIZE :: RANGE_SIZE;
        VK_FILTER_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkFormat :: enum u32 {
        UNDEFINED :: 0;
        R4G4_UNORM_PACK8 :: 1;
        R4G4B4A4_UNORM_PACK16 :: 2;
        B4G4R4A4_UNORM_PACK16 :: 3;
        R5G6B5_UNORM_PACK16 :: 4;
        B5G6R5_UNORM_PACK16 :: 5;
        R5G5B5A1_UNORM_PACK16 :: 6;
        B5G5R5A1_UNORM_PACK16 :: 7;
        A1R5G5B5_UNORM_PACK16 :: 8;
        R8_UNORM :: 9;
        R8_SNORM :: 10;
        R8_USCALED :: 11;
        R8_SSCALED :: 12;
        R8_UINT :: 13;
        R8_SINT :: 14;
        R8_SRGB :: 15;
        R8G8_UNORM :: 16;
        R8G8_SNORM :: 17;
        R8G8_USCALED :: 18;
        R8G8_SSCALED :: 19;
        R8G8_UINT :: 20;
        R8G8_SINT :: 21;
        R8G8_SRGB :: 22;
        R8G8B8_UNORM :: 23;
        R8G8B8_SNORM :: 24;
        R8G8B8_USCALED :: 25;
        R8G8B8_SSCALED :: 26;
        R8G8B8_UINT :: 27;
        R8G8B8_SINT :: 28;
        R8G8B8_SRGB :: 29;
        B8G8R8_UNORM :: 30;
        B8G8R8_SNORM :: 31;
        B8G8R8_USCALED :: 32;
        B8G8R8_SSCALED :: 33;
        B8G8R8_UINT :: 34;
        B8G8R8_SINT :: 35;
        B8G8R8_SRGB :: 36;
        R8G8B8A8_UNORM :: 37;
        R8G8B8A8_SNORM :: 38;
        R8G8B8A8_USCALED :: 39;
        R8G8B8A8_SSCALED :: 40;
        R8G8B8A8_UINT :: 41;
        R8G8B8A8_SINT :: 42;
        R8G8B8A8_SRGB :: 43;
        B8G8R8A8_UNORM :: 44;
        B8G8R8A8_SNORM :: 45;
        B8G8R8A8_USCALED :: 46;
        B8G8R8A8_SSCALED :: 47;
        B8G8R8A8_UINT :: 48;
        B8G8R8A8_SINT :: 49;
        B8G8R8A8_SRGB :: 50;
        A8B8G8R8_UNORM_PACK32 :: 51;
        A8B8G8R8_SNORM_PACK32 :: 52;
        A8B8G8R8_USCALED_PACK32 :: 53;
        A8B8G8R8_SSCALED_PACK32 :: 54;
        A8B8G8R8_UINT_PACK32 :: 55;
        A8B8G8R8_SINT_PACK32 :: 56;
        A8B8G8R8_SRGB_PACK32 :: 57;
        A2R10G10B10_UNORM_PACK32 :: 58;
        A2R10G10B10_SNORM_PACK32 :: 59;
        A2R10G10B10_USCALED_PACK32 :: 60;
        A2R10G10B10_SSCALED_PACK32 :: 61;
        A2R10G10B10_UINT_PACK32 :: 62;
        A2R10G10B10_SINT_PACK32 :: 63;
        A2B10G10R10_UNORM_PACK32 :: 64;
        A2B10G10R10_SNORM_PACK32 :: 65;
        A2B10G10R10_USCALED_PACK32 :: 66;
        A2B10G10R10_SSCALED_PACK32 :: 67;
        A2B10G10R10_UINT_PACK32 :: 68;
        A2B10G10R10_SINT_PACK32 :: 69;
        R16_UNORM :: 70;
        R16_SNORM :: 71;
        R16_USCALED :: 72;
        R16_SSCALED :: 73;
        R16_UINT :: 74;
        R16_SINT :: 75;
        R16_SFLOAT :: 76;
        R16G16_UNORM :: 77;
        R16G16_SNORM :: 78;
        R16G16_USCALED :: 79;
        R16G16_SSCALED :: 80;
        R16G16_UINT :: 81;
        R16G16_SINT :: 82;
        R16G16_SFLOAT :: 83;
        R16G16B16_UNORM :: 84;
        R16G16B16_SNORM :: 85;
        R16G16B16_USCALED :: 86;
        R16G16B16_SSCALED :: 87;
        R16G16B16_UINT :: 88;
        R16G16B16_SINT :: 89;
        R16G16B16_SFLOAT :: 90;
        R16G16B16A16_UNORM :: 91;
        R16G16B16A16_SNORM :: 92;
        R16G16B16A16_USCALED :: 93;
        R16G16B16A16_SSCALED :: 94;
        R16G16B16A16_UINT :: 95;
        R16G16B16A16_SINT :: 96;
        R16G16B16A16_SFLOAT :: 97;
        R32_UINT :: 98;
        R32_SINT :: 99;
        R32_SFLOAT :: 100;
        R32G32_UINT :: 101;
        R32G32_SINT :: 102;
        R32G32_SFLOAT :: 103;
        R32G32B32_UINT :: 104;
        R32G32B32_SINT :: 105;
        R32G32B32_SFLOAT :: 106;
        R32G32B32A32_UINT :: 107;
        R32G32B32A32_SINT :: 108;
        R32G32B32A32_SFLOAT :: 109;
        R64_UINT :: 110;
        R64_SINT :: 111;
        R64_SFLOAT :: 112;
        R64G64_UINT :: 113;
        R64G64_SINT :: 114;
        R64G64_SFLOAT :: 115;
        R64G64B64_UINT :: 116;
        R64G64B64_SINT :: 117;
        R64G64B64_SFLOAT :: 118;
        R64G64B64A64_UINT :: 119;
        R64G64B64A64_SINT :: 120;
        R64G64B64A64_SFLOAT :: 121;
        B10G11R11_UFLOAT_PACK32 :: 122;
        E5B9G9R9_UFLOAT_PACK32 :: 123;
        D16_UNORM :: 124;
        X8_D24_UNORM_PACK32 :: 125;
        D32_SFLOAT :: 126;
        S8_UINT :: 127;
        D16_UNORM_S8_UINT :: 128;
        D24_UNORM_S8_UINT :: 129;
        D32_SFLOAT_S8_UINT :: 130;
        BC1_RGB_UNORM_BLOCK :: 131;
        BC1_RGB_SRGB_BLOCK :: 132;
        BC1_RGBA_UNORM_BLOCK :: 133;
        BC1_RGBA_SRGB_BLOCK :: 134;
        BC2_UNORM_BLOCK :: 135;
        BC2_SRGB_BLOCK :: 136;
        BC3_UNORM_BLOCK :: 137;
        BC3_SRGB_BLOCK :: 138;
        BC4_UNORM_BLOCK :: 139;
        BC4_SNORM_BLOCK :: 140;
        BC5_UNORM_BLOCK :: 141;
        BC5_SNORM_BLOCK :: 142;
        BC6H_UFLOAT_BLOCK :: 143;
        BC6H_SFLOAT_BLOCK :: 144;
        BC7_UNORM_BLOCK :: 145;
        BC7_SRGB_BLOCK :: 146;
        ETC2_R8G8B8_UNORM_BLOCK :: 147;
        ETC2_R8G8B8_SRGB_BLOCK :: 148;
        ETC2_R8G8B8A1_UNORM_BLOCK :: 149;
        ETC2_R8G8B8A1_SRGB_BLOCK :: 150;
        ETC2_R8G8B8A8_UNORM_BLOCK :: 151;
        ETC2_R8G8B8A8_SRGB_BLOCK :: 152;
        EAC_R11_UNORM_BLOCK :: 153;
        EAC_R11_SNORM_BLOCK :: 154;
        EAC_R11G11_UNORM_BLOCK :: 155;
        EAC_R11G11_SNORM_BLOCK :: 156;
        ASTC_4x4_UNORM_BLOCK :: 157;
        ASTC_4x4_SRGB_BLOCK :: 158;
        ASTC_5x4_UNORM_BLOCK :: 159;
        ASTC_5x4_SRGB_BLOCK :: 160;
        ASTC_5x5_UNORM_BLOCK :: 161;
        ASTC_5x5_SRGB_BLOCK :: 162;
        ASTC_6x5_UNORM_BLOCK :: 163;
        ASTC_6x5_SRGB_BLOCK :: 164;
        ASTC_6x6_UNORM_BLOCK :: 165;
        ASTC_6x6_SRGB_BLOCK :: 166;
        ASTC_8x5_UNORM_BLOCK :: 167;
        ASTC_8x5_SRGB_BLOCK :: 168;
        ASTC_8x6_UNORM_BLOCK :: 169;
        ASTC_8x6_SRGB_BLOCK :: 170;
        ASTC_8x8_UNORM_BLOCK :: 171;
        ASTC_8x8_SRGB_BLOCK :: 172;
        ASTC_10x5_UNORM_BLOCK :: 173;
        ASTC_10x5_SRGB_BLOCK :: 174;
        ASTC_10x6_UNORM_BLOCK :: 175;
        ASTC_10x6_SRGB_BLOCK :: 176;
        ASTC_10x8_UNORM_BLOCK :: 177;
        ASTC_10x8_SRGB_BLOCK :: 178;
        ASTC_10x10_UNORM_BLOCK :: 179;
        ASTC_10x10_SRGB_BLOCK :: 180;
        ASTC_12x10_UNORM_BLOCK :: 181;
        ASTC_12x10_SRGB_BLOCK :: 182;
        ASTC_12x12_UNORM_BLOCK :: 183;
        ASTC_12x12_SRGB_BLOCK :: 184;
        G8B8G8R8_422_UNORM :: 1000156000;
        B8G8R8G8_422_UNORM :: 1000156001;
        G8_B8_R8_3PLANE_420_UNORM :: 1000156002;
        G8_B8R8_2PLANE_420_UNORM :: 1000156003;
        G8_B8_R8_3PLANE_422_UNORM :: 1000156004;
        G8_B8R8_2PLANE_422_UNORM :: 1000156005;
        G8_B8_R8_3PLANE_444_UNORM :: 1000156006;
        R10X6_UNORM_PACK16 :: 1000156007;
        R10X6G10X6_UNORM_2PACK16 :: 1000156008;
        R10X6G10X6B10X6A10X6_UNORM_4PACK16 :: 1000156009;
        G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 :: 1000156010;
        B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 :: 1000156011;
        G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 :: 1000156012;
        G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 :: 1000156013;
        G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 :: 1000156014;
        G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 :: 1000156015;
        G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 :: 1000156016;
        R12X4_UNORM_PACK16 :: 1000156017;
        R12X4G12X4_UNORM_2PACK16 :: 1000156018;
        R12X4G12X4B12X4A12X4_UNORM_4PACK16 :: 1000156019;
        G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 :: 1000156020;
        B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 :: 1000156021;
        G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 :: 1000156022;
        G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 :: 1000156023;
        G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 :: 1000156024;
        G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 :: 1000156025;
        G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 :: 1000156026;
        G16B16G16R16_422_UNORM :: 1000156027;
        B16G16R16G16_422_UNORM :: 1000156028;
        G16_B16_R16_3PLANE_420_UNORM :: 1000156029;
        G16_B16R16_2PLANE_420_UNORM :: 1000156030;
        G16_B16_R16_3PLANE_422_UNORM :: 1000156031;
        G16_B16R16_2PLANE_422_UNORM :: 1000156032;
        G16_B16_R16_3PLANE_444_UNORM :: 1000156033;
        PVRTC1_2BPP_UNORM_BLOCK_IMG :: 1000054000;
        PVRTC1_4BPP_UNORM_BLOCK_IMG :: 1000054001;
        PVRTC2_2BPP_UNORM_BLOCK_IMG :: 1000054002;
        PVRTC2_4BPP_UNORM_BLOCK_IMG :: 1000054003;
        PVRTC1_2BPP_SRGB_BLOCK_IMG :: 1000054004;
        PVRTC1_4BPP_SRGB_BLOCK_IMG :: 1000054005;
        PVRTC2_2BPP_SRGB_BLOCK_IMG :: 1000054006;
        PVRTC2_4BPP_SRGB_BLOCK_IMG :: 1000054007;
        G8B8G8R8_422_UNORM_KHR :: 1000156000;
        B8G8R8G8_422_UNORM_KHR :: 1000156001;
        G8_B8_R8_3PLANE_420_UNORM_KHR :: 1000156002;
        G8_B8R8_2PLANE_420_UNORM_KHR :: 1000156003;
        G8_B8_R8_3PLANE_422_UNORM_KHR :: 1000156004;
        G8_B8R8_2PLANE_422_UNORM_KHR :: 1000156005;
        G8_B8_R8_3PLANE_444_UNORM_KHR :: 1000156006;
        R10X6_UNORM_PACK16_KHR :: 1000156007;
        R10X6G10X6_UNORM_2PACK16_KHR :: 1000156008;
        R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR :: 1000156009;
        G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR :: 1000156010;
        B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR :: 1000156011;
        G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR :: 1000156012;
        G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR :: 1000156013;
        G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR :: 1000156014;
        G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR :: 1000156015;
        G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR :: 1000156016;
        R12X4_UNORM_PACK16_KHR :: 1000156017;
        R12X4G12X4_UNORM_2PACK16_KHR :: 1000156018;
        R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR :: 1000156019;
        G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR :: 1000156020;
        B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR :: 1000156021;
        G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR :: 1000156022;
        G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR :: 1000156023;
        G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR :: 1000156024;
        G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR :: 1000156025;
        G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR :: 1000156026;
        G16B16G16R16_422_UNORM_KHR :: 1000156027;
        B16G16R16G16_422_UNORM_KHR :: 1000156028;
        G16_B16_R16_3PLANE_420_UNORM_KHR :: 1000156029;
        G16_B16R16_2PLANE_420_UNORM_KHR :: 1000156030;
        G16_B16_R16_3PLANE_422_UNORM_KHR :: 1000156031;
        G16_B16R16_2PLANE_422_UNORM_KHR :: 1000156032;
        G16_B16_R16_3PLANE_444_UNORM_KHR :: 1000156033;
        BEGIN_RANGE :: 0;
        END_RANGE :: 184;
        RANGE_SIZE :: 185;
        MAX_ENUM :: 2147483647;

        VK_FORMAT_UNDEFINED :: UNDEFINED;
        VK_FORMAT_R4G4_UNORM_PACK8 :: R4G4_UNORM_PACK8;
        VK_FORMAT_R4G4B4A4_UNORM_PACK16 :: R4G4B4A4_UNORM_PACK16;
        VK_FORMAT_B4G4R4A4_UNORM_PACK16 :: B4G4R4A4_UNORM_PACK16;
        VK_FORMAT_R5G6B5_UNORM_PACK16 :: R5G6B5_UNORM_PACK16;
        VK_FORMAT_B5G6R5_UNORM_PACK16 :: B5G6R5_UNORM_PACK16;
        VK_FORMAT_R5G5B5A1_UNORM_PACK16 :: R5G5B5A1_UNORM_PACK16;
        VK_FORMAT_B5G5R5A1_UNORM_PACK16 :: B5G5R5A1_UNORM_PACK16;
        VK_FORMAT_A1R5G5B5_UNORM_PACK16 :: A1R5G5B5_UNORM_PACK16;
        VK_FORMAT_R8_UNORM :: R8_UNORM;
        VK_FORMAT_R8_SNORM :: R8_SNORM;
        VK_FORMAT_R8_USCALED :: R8_USCALED;
        VK_FORMAT_R8_SSCALED :: R8_SSCALED;
        VK_FORMAT_R8_UINT :: R8_UINT;
        VK_FORMAT_R8_SINT :: R8_SINT;
        VK_FORMAT_R8_SRGB :: R8_SRGB;
        VK_FORMAT_R8G8_UNORM :: R8G8_UNORM;
        VK_FORMAT_R8G8_SNORM :: R8G8_SNORM;
        VK_FORMAT_R8G8_USCALED :: R8G8_USCALED;
        VK_FORMAT_R8G8_SSCALED :: R8G8_SSCALED;
        VK_FORMAT_R8G8_UINT :: R8G8_UINT;
        VK_FORMAT_R8G8_SINT :: R8G8_SINT;
        VK_FORMAT_R8G8_SRGB :: R8G8_SRGB;
        VK_FORMAT_R8G8B8_UNORM :: R8G8B8_UNORM;
        VK_FORMAT_R8G8B8_SNORM :: R8G8B8_SNORM;
        VK_FORMAT_R8G8B8_USCALED :: R8G8B8_USCALED;
        VK_FORMAT_R8G8B8_SSCALED :: R8G8B8_SSCALED;
        VK_FORMAT_R8G8B8_UINT :: R8G8B8_UINT;
        VK_FORMAT_R8G8B8_SINT :: R8G8B8_SINT;
        VK_FORMAT_R8G8B8_SRGB :: R8G8B8_SRGB;
        VK_FORMAT_B8G8R8_UNORM :: B8G8R8_UNORM;
        VK_FORMAT_B8G8R8_SNORM :: B8G8R8_SNORM;
        VK_FORMAT_B8G8R8_USCALED :: B8G8R8_USCALED;
        VK_FORMAT_B8G8R8_SSCALED :: B8G8R8_SSCALED;
        VK_FORMAT_B8G8R8_UINT :: B8G8R8_UINT;
        VK_FORMAT_B8G8R8_SINT :: B8G8R8_SINT;
        VK_FORMAT_B8G8R8_SRGB :: B8G8R8_SRGB;
        VK_FORMAT_R8G8B8A8_UNORM :: R8G8B8A8_UNORM;
        VK_FORMAT_R8G8B8A8_SNORM :: R8G8B8A8_SNORM;
        VK_FORMAT_R8G8B8A8_USCALED :: R8G8B8A8_USCALED;
        VK_FORMAT_R8G8B8A8_SSCALED :: R8G8B8A8_SSCALED;
        VK_FORMAT_R8G8B8A8_UINT :: R8G8B8A8_UINT;
        VK_FORMAT_R8G8B8A8_SINT :: R8G8B8A8_SINT;
        VK_FORMAT_R8G8B8A8_SRGB :: R8G8B8A8_SRGB;
        VK_FORMAT_B8G8R8A8_UNORM :: B8G8R8A8_UNORM;
        VK_FORMAT_B8G8R8A8_SNORM :: B8G8R8A8_SNORM;
        VK_FORMAT_B8G8R8A8_USCALED :: B8G8R8A8_USCALED;
        VK_FORMAT_B8G8R8A8_SSCALED :: B8G8R8A8_SSCALED;
        VK_FORMAT_B8G8R8A8_UINT :: B8G8R8A8_UINT;
        VK_FORMAT_B8G8R8A8_SINT :: B8G8R8A8_SINT;
        VK_FORMAT_B8G8R8A8_SRGB :: B8G8R8A8_SRGB;
        VK_FORMAT_A8B8G8R8_UNORM_PACK32 :: A8B8G8R8_UNORM_PACK32;
        VK_FORMAT_A8B8G8R8_SNORM_PACK32 :: A8B8G8R8_SNORM_PACK32;
        VK_FORMAT_A8B8G8R8_USCALED_PACK32 :: A8B8G8R8_USCALED_PACK32;
        VK_FORMAT_A8B8G8R8_SSCALED_PACK32 :: A8B8G8R8_SSCALED_PACK32;
        VK_FORMAT_A8B8G8R8_UINT_PACK32 :: A8B8G8R8_UINT_PACK32;
        VK_FORMAT_A8B8G8R8_SINT_PACK32 :: A8B8G8R8_SINT_PACK32;
        VK_FORMAT_A8B8G8R8_SRGB_PACK32 :: A8B8G8R8_SRGB_PACK32;
        VK_FORMAT_A2R10G10B10_UNORM_PACK32 :: A2R10G10B10_UNORM_PACK32;
        VK_FORMAT_A2R10G10B10_SNORM_PACK32 :: A2R10G10B10_SNORM_PACK32;
        VK_FORMAT_A2R10G10B10_USCALED_PACK32 :: A2R10G10B10_USCALED_PACK32;
        VK_FORMAT_A2R10G10B10_SSCALED_PACK32 :: A2R10G10B10_SSCALED_PACK32;
        VK_FORMAT_A2R10G10B10_UINT_PACK32 :: A2R10G10B10_UINT_PACK32;
        VK_FORMAT_A2R10G10B10_SINT_PACK32 :: A2R10G10B10_SINT_PACK32;
        VK_FORMAT_A2B10G10R10_UNORM_PACK32 :: A2B10G10R10_UNORM_PACK32;
        VK_FORMAT_A2B10G10R10_SNORM_PACK32 :: A2B10G10R10_SNORM_PACK32;
        VK_FORMAT_A2B10G10R10_USCALED_PACK32 :: A2B10G10R10_USCALED_PACK32;
        VK_FORMAT_A2B10G10R10_SSCALED_PACK32 :: A2B10G10R10_SSCALED_PACK32;
        VK_FORMAT_A2B10G10R10_UINT_PACK32 :: A2B10G10R10_UINT_PACK32;
        VK_FORMAT_A2B10G10R10_SINT_PACK32 :: A2B10G10R10_SINT_PACK32;
        VK_FORMAT_R16_UNORM :: R16_UNORM;
        VK_FORMAT_R16_SNORM :: R16_SNORM;
        VK_FORMAT_R16_USCALED :: R16_USCALED;
        VK_FORMAT_R16_SSCALED :: R16_SSCALED;
        VK_FORMAT_R16_UINT :: R16_UINT;
        VK_FORMAT_R16_SINT :: R16_SINT;
        VK_FORMAT_R16_SFLOAT :: R16_SFLOAT;
        VK_FORMAT_R16G16_UNORM :: R16G16_UNORM;
        VK_FORMAT_R16G16_SNORM :: R16G16_SNORM;
        VK_FORMAT_R16G16_USCALED :: R16G16_USCALED;
        VK_FORMAT_R16G16_SSCALED :: R16G16_SSCALED;
        VK_FORMAT_R16G16_UINT :: R16G16_UINT;
        VK_FORMAT_R16G16_SINT :: R16G16_SINT;
        VK_FORMAT_R16G16_SFLOAT :: R16G16_SFLOAT;
        VK_FORMAT_R16G16B16_UNORM :: R16G16B16_UNORM;
        VK_FORMAT_R16G16B16_SNORM :: R16G16B16_SNORM;
        VK_FORMAT_R16G16B16_USCALED :: R16G16B16_USCALED;
        VK_FORMAT_R16G16B16_SSCALED :: R16G16B16_SSCALED;
        VK_FORMAT_R16G16B16_UINT :: R16G16B16_UINT;
        VK_FORMAT_R16G16B16_SINT :: R16G16B16_SINT;
        VK_FORMAT_R16G16B16_SFLOAT :: R16G16B16_SFLOAT;
        VK_FORMAT_R16G16B16A16_UNORM :: R16G16B16A16_UNORM;
        VK_FORMAT_R16G16B16A16_SNORM :: R16G16B16A16_SNORM;
        VK_FORMAT_R16G16B16A16_USCALED :: R16G16B16A16_USCALED;
        VK_FORMAT_R16G16B16A16_SSCALED :: R16G16B16A16_SSCALED;
        VK_FORMAT_R16G16B16A16_UINT :: R16G16B16A16_UINT;
        VK_FORMAT_R16G16B16A16_SINT :: R16G16B16A16_SINT;
        VK_FORMAT_R16G16B16A16_SFLOAT :: R16G16B16A16_SFLOAT;
        VK_FORMAT_R32_UINT :: R32_UINT;
        VK_FORMAT_R32_SINT :: R32_SINT;
        VK_FORMAT_R32_SFLOAT :: R32_SFLOAT;
        VK_FORMAT_R32G32_UINT :: R32G32_UINT;
        VK_FORMAT_R32G32_SINT :: R32G32_SINT;
        VK_FORMAT_R32G32_SFLOAT :: R32G32_SFLOAT;
        VK_FORMAT_R32G32B32_UINT :: R32G32B32_UINT;
        VK_FORMAT_R32G32B32_SINT :: R32G32B32_SINT;
        VK_FORMAT_R32G32B32_SFLOAT :: R32G32B32_SFLOAT;
        VK_FORMAT_R32G32B32A32_UINT :: R32G32B32A32_UINT;
        VK_FORMAT_R32G32B32A32_SINT :: R32G32B32A32_SINT;
        VK_FORMAT_R32G32B32A32_SFLOAT :: R32G32B32A32_SFLOAT;
        VK_FORMAT_R64_UINT :: R64_UINT;
        VK_FORMAT_R64_SINT :: R64_SINT;
        VK_FORMAT_R64_SFLOAT :: R64_SFLOAT;
        VK_FORMAT_R64G64_UINT :: R64G64_UINT;
        VK_FORMAT_R64G64_SINT :: R64G64_SINT;
        VK_FORMAT_R64G64_SFLOAT :: R64G64_SFLOAT;
        VK_FORMAT_R64G64B64_UINT :: R64G64B64_UINT;
        VK_FORMAT_R64G64B64_SINT :: R64G64B64_SINT;
        VK_FORMAT_R64G64B64_SFLOAT :: R64G64B64_SFLOAT;
        VK_FORMAT_R64G64B64A64_UINT :: R64G64B64A64_UINT;
        VK_FORMAT_R64G64B64A64_SINT :: R64G64B64A64_SINT;
        VK_FORMAT_R64G64B64A64_SFLOAT :: R64G64B64A64_SFLOAT;
        VK_FORMAT_B10G11R11_UFLOAT_PACK32 :: B10G11R11_UFLOAT_PACK32;
        VK_FORMAT_E5B9G9R9_UFLOAT_PACK32 :: E5B9G9R9_UFLOAT_PACK32;
        VK_FORMAT_D16_UNORM :: D16_UNORM;
        VK_FORMAT_X8_D24_UNORM_PACK32 :: X8_D24_UNORM_PACK32;
        VK_FORMAT_D32_SFLOAT :: D32_SFLOAT;
        VK_FORMAT_S8_UINT :: S8_UINT;
        VK_FORMAT_D16_UNORM_S8_UINT :: D16_UNORM_S8_UINT;
        VK_FORMAT_D24_UNORM_S8_UINT :: D24_UNORM_S8_UINT;
        VK_FORMAT_D32_SFLOAT_S8_UINT :: D32_SFLOAT_S8_UINT;
        VK_FORMAT_BC1_RGB_UNORM_BLOCK :: BC1_RGB_UNORM_BLOCK;
        VK_FORMAT_BC1_RGB_SRGB_BLOCK :: BC1_RGB_SRGB_BLOCK;
        VK_FORMAT_BC1_RGBA_UNORM_BLOCK :: BC1_RGBA_UNORM_BLOCK;
        VK_FORMAT_BC1_RGBA_SRGB_BLOCK :: BC1_RGBA_SRGB_BLOCK;
        VK_FORMAT_BC2_UNORM_BLOCK :: BC2_UNORM_BLOCK;
        VK_FORMAT_BC2_SRGB_BLOCK :: BC2_SRGB_BLOCK;
        VK_FORMAT_BC3_UNORM_BLOCK :: BC3_UNORM_BLOCK;
        VK_FORMAT_BC3_SRGB_BLOCK :: BC3_SRGB_BLOCK;
        VK_FORMAT_BC4_UNORM_BLOCK :: BC4_UNORM_BLOCK;
        VK_FORMAT_BC4_SNORM_BLOCK :: BC4_SNORM_BLOCK;
        VK_FORMAT_BC5_UNORM_BLOCK :: BC5_UNORM_BLOCK;
        VK_FORMAT_BC5_SNORM_BLOCK :: BC5_SNORM_BLOCK;
        VK_FORMAT_BC6H_UFLOAT_BLOCK :: BC6H_UFLOAT_BLOCK;
        VK_FORMAT_BC6H_SFLOAT_BLOCK :: BC6H_SFLOAT_BLOCK;
        VK_FORMAT_BC7_UNORM_BLOCK :: BC7_UNORM_BLOCK;
        VK_FORMAT_BC7_SRGB_BLOCK :: BC7_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK :: ETC2_R8G8B8_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK :: ETC2_R8G8B8_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A1_UNORM_BLOCK :: ETC2_R8G8B8A1_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A1_SRGB_BLOCK :: ETC2_R8G8B8A1_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK :: ETC2_R8G8B8A8_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK :: ETC2_R8G8B8A8_SRGB_BLOCK;
        VK_FORMAT_EAC_R11_UNORM_BLOCK :: EAC_R11_UNORM_BLOCK;
        VK_FORMAT_EAC_R11_SNORM_BLOCK :: EAC_R11_SNORM_BLOCK;
        VK_FORMAT_EAC_R11G11_UNORM_BLOCK :: EAC_R11G11_UNORM_BLOCK;
        VK_FORMAT_EAC_R11G11_SNORM_BLOCK :: EAC_R11G11_SNORM_BLOCK;
        VK_FORMAT_ASTC_4x4_UNORM_BLOCK :: ASTC_4x4_UNORM_BLOCK;
        VK_FORMAT_ASTC_4x4_SRGB_BLOCK :: ASTC_4x4_SRGB_BLOCK;
        VK_FORMAT_ASTC_5x4_UNORM_BLOCK :: ASTC_5x4_UNORM_BLOCK;
        VK_FORMAT_ASTC_5x4_SRGB_BLOCK :: ASTC_5x4_SRGB_BLOCK;
        VK_FORMAT_ASTC_5x5_UNORM_BLOCK :: ASTC_5x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_5x5_SRGB_BLOCK :: ASTC_5x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_6x5_UNORM_BLOCK :: ASTC_6x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_6x5_SRGB_BLOCK :: ASTC_6x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_6x6_UNORM_BLOCK :: ASTC_6x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_6x6_SRGB_BLOCK :: ASTC_6x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x5_UNORM_BLOCK :: ASTC_8x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x5_SRGB_BLOCK :: ASTC_8x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x6_UNORM_BLOCK :: ASTC_8x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x6_SRGB_BLOCK :: ASTC_8x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x8_UNORM_BLOCK :: ASTC_8x8_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x8_SRGB_BLOCK :: ASTC_8x8_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x5_UNORM_BLOCK :: ASTC_10x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x5_SRGB_BLOCK :: ASTC_10x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x6_UNORM_BLOCK :: ASTC_10x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x6_SRGB_BLOCK :: ASTC_10x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x8_UNORM_BLOCK :: ASTC_10x8_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x8_SRGB_BLOCK :: ASTC_10x8_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x10_UNORM_BLOCK :: ASTC_10x10_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x10_SRGB_BLOCK :: ASTC_10x10_SRGB_BLOCK;
        VK_FORMAT_ASTC_12x10_UNORM_BLOCK :: ASTC_12x10_UNORM_BLOCK;
        VK_FORMAT_ASTC_12x10_SRGB_BLOCK :: ASTC_12x10_SRGB_BLOCK;
        VK_FORMAT_ASTC_12x12_UNORM_BLOCK :: ASTC_12x12_UNORM_BLOCK;
        VK_FORMAT_ASTC_12x12_SRGB_BLOCK :: ASTC_12x12_SRGB_BLOCK;
        VK_FORMAT_G8B8G8R8_422_UNORM :: G8B8G8R8_422_UNORM;
        VK_FORMAT_B8G8R8G8_422_UNORM :: B8G8R8G8_422_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM :: G8_B8_R8_3PLANE_420_UNORM;
        VK_FORMAT_G8_B8R8_2PLANE_420_UNORM :: G8_B8R8_2PLANE_420_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM :: G8_B8_R8_3PLANE_422_UNORM;
        VK_FORMAT_G8_B8R8_2PLANE_422_UNORM :: G8_B8R8_2PLANE_422_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM :: G8_B8_R8_3PLANE_444_UNORM;
        VK_FORMAT_R10X6_UNORM_PACK16 :: R10X6_UNORM_PACK16;
        VK_FORMAT_R10X6G10X6_UNORM_2PACK16 :: R10X6G10X6_UNORM_2PACK16;
        VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16 :: R10X6G10X6B10X6A10X6_UNORM_4PACK16;
        VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 :: G10X6B10X6G10X6R10X6_422_UNORM_4PACK16;
        VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 :: B10X6G10X6R10X6G10X6_422_UNORM_4PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 :: G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 :: G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16;
        VK_FORMAT_R12X4_UNORM_PACK16 :: R12X4_UNORM_PACK16;
        VK_FORMAT_R12X4G12X4_UNORM_2PACK16 :: R12X4G12X4_UNORM_2PACK16;
        VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16 :: R12X4G12X4B12X4A12X4_UNORM_4PACK16;
        VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 :: G12X4B12X4G12X4R12X4_422_UNORM_4PACK16;
        VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 :: B12X4G12X4R12X4G12X4_422_UNORM_4PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 :: G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 :: G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16;
        VK_FORMAT_G16B16G16R16_422_UNORM :: G16B16G16R16_422_UNORM;
        VK_FORMAT_B16G16R16G16_422_UNORM :: B16G16R16G16_422_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM :: G16_B16_R16_3PLANE_420_UNORM;
        VK_FORMAT_G16_B16R16_2PLANE_420_UNORM :: G16_B16R16_2PLANE_420_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM :: G16_B16_R16_3PLANE_422_UNORM;
        VK_FORMAT_G16_B16R16_2PLANE_422_UNORM :: G16_B16R16_2PLANE_422_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM :: G16_B16_R16_3PLANE_444_UNORM;
        VK_FORMAT_PVRTC1_2BPP_UNORM_BLOCK_IMG :: PVRTC1_2BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC1_4BPP_UNORM_BLOCK_IMG :: PVRTC1_4BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC2_2BPP_UNORM_BLOCK_IMG :: PVRTC2_2BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC2_4BPP_UNORM_BLOCK_IMG :: PVRTC2_4BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC1_2BPP_SRGB_BLOCK_IMG :: PVRTC1_2BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC1_4BPP_SRGB_BLOCK_IMG :: PVRTC1_4BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC2_2BPP_SRGB_BLOCK_IMG :: PVRTC2_2BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC2_4BPP_SRGB_BLOCK_IMG :: PVRTC2_4BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_G8B8G8R8_422_UNORM_KHR :: G8B8G8R8_422_UNORM_KHR;
        VK_FORMAT_B8G8R8G8_422_UNORM_KHR :: B8G8R8G8_422_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM_KHR :: G8_B8_R8_3PLANE_420_UNORM_KHR;
        VK_FORMAT_G8_B8R8_2PLANE_420_UNORM_KHR :: G8_B8R8_2PLANE_420_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM_KHR :: G8_B8_R8_3PLANE_422_UNORM_KHR;
        VK_FORMAT_G8_B8R8_2PLANE_422_UNORM_KHR :: G8_B8R8_2PLANE_422_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM_KHR :: G8_B8_R8_3PLANE_444_UNORM_KHR;
        VK_FORMAT_R10X6_UNORM_PACK16_KHR :: R10X6_UNORM_PACK16_KHR;
        VK_FORMAT_R10X6G10X6_UNORM_2PACK16_KHR :: R10X6G10X6_UNORM_2PACK16_KHR;
        VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR :: R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR;
        VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR :: G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR;
        VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR :: B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR :: G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR :: G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR;
        VK_FORMAT_R12X4_UNORM_PACK16_KHR :: R12X4_UNORM_PACK16_KHR;
        VK_FORMAT_R12X4G12X4_UNORM_2PACK16_KHR :: R12X4G12X4_UNORM_2PACK16_KHR;
        VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR :: R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR;
        VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR :: G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR;
        VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR :: B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR :: G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR :: G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR;
        VK_FORMAT_G16B16G16R16_422_UNORM_KHR :: G16B16G16R16_422_UNORM_KHR;
        VK_FORMAT_B16G16R16G16_422_UNORM_KHR :: B16G16R16G16_422_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM_KHR :: G16_B16_R16_3PLANE_420_UNORM_KHR;
        VK_FORMAT_G16_B16R16_2PLANE_420_UNORM_KHR :: G16_B16R16_2PLANE_420_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM_KHR :: G16_B16_R16_3PLANE_422_UNORM_KHR;
        VK_FORMAT_G16_B16R16_2PLANE_422_UNORM_KHR :: G16_B16R16_2PLANE_422_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM_KHR :: G16_B16_R16_3PLANE_444_UNORM_KHR;
        VK_FORMAT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FORMAT_END_RANGE :: END_RANGE;
        VK_FORMAT_RANGE_SIZE :: RANGE_SIZE;
        VK_FORMAT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFormat :: enum u32 {
        UNDEFINED :: 0;
        R4G4_UNORM_PACK8 :: 1;
        R4G4B4A4_UNORM_PACK16 :: 2;
        B4G4R4A4_UNORM_PACK16 :: 3;
        R5G6B5_UNORM_PACK16 :: 4;
        B5G6R5_UNORM_PACK16 :: 5;
        R5G5B5A1_UNORM_PACK16 :: 6;
        B5G5R5A1_UNORM_PACK16 :: 7;
        A1R5G5B5_UNORM_PACK16 :: 8;
        R8_UNORM :: 9;
        R8_SNORM :: 10;
        R8_USCALED :: 11;
        R8_SSCALED :: 12;
        R8_UINT :: 13;
        R8_SINT :: 14;
        R8_SRGB :: 15;
        R8G8_UNORM :: 16;
        R8G8_SNORM :: 17;
        R8G8_USCALED :: 18;
        R8G8_SSCALED :: 19;
        R8G8_UINT :: 20;
        R8G8_SINT :: 21;
        R8G8_SRGB :: 22;
        R8G8B8_UNORM :: 23;
        R8G8B8_SNORM :: 24;
        R8G8B8_USCALED :: 25;
        R8G8B8_SSCALED :: 26;
        R8G8B8_UINT :: 27;
        R8G8B8_SINT :: 28;
        R8G8B8_SRGB :: 29;
        B8G8R8_UNORM :: 30;
        B8G8R8_SNORM :: 31;
        B8G8R8_USCALED :: 32;
        B8G8R8_SSCALED :: 33;
        B8G8R8_UINT :: 34;
        B8G8R8_SINT :: 35;
        B8G8R8_SRGB :: 36;
        R8G8B8A8_UNORM :: 37;
        R8G8B8A8_SNORM :: 38;
        R8G8B8A8_USCALED :: 39;
        R8G8B8A8_SSCALED :: 40;
        R8G8B8A8_UINT :: 41;
        R8G8B8A8_SINT :: 42;
        R8G8B8A8_SRGB :: 43;
        B8G8R8A8_UNORM :: 44;
        B8G8R8A8_SNORM :: 45;
        B8G8R8A8_USCALED :: 46;
        B8G8R8A8_SSCALED :: 47;
        B8G8R8A8_UINT :: 48;
        B8G8R8A8_SINT :: 49;
        B8G8R8A8_SRGB :: 50;
        A8B8G8R8_UNORM_PACK32 :: 51;
        A8B8G8R8_SNORM_PACK32 :: 52;
        A8B8G8R8_USCALED_PACK32 :: 53;
        A8B8G8R8_SSCALED_PACK32 :: 54;
        A8B8G8R8_UINT_PACK32 :: 55;
        A8B8G8R8_SINT_PACK32 :: 56;
        A8B8G8R8_SRGB_PACK32 :: 57;
        A2R10G10B10_UNORM_PACK32 :: 58;
        A2R10G10B10_SNORM_PACK32 :: 59;
        A2R10G10B10_USCALED_PACK32 :: 60;
        A2R10G10B10_SSCALED_PACK32 :: 61;
        A2R10G10B10_UINT_PACK32 :: 62;
        A2R10G10B10_SINT_PACK32 :: 63;
        A2B10G10R10_UNORM_PACK32 :: 64;
        A2B10G10R10_SNORM_PACK32 :: 65;
        A2B10G10R10_USCALED_PACK32 :: 66;
        A2B10G10R10_SSCALED_PACK32 :: 67;
        A2B10G10R10_UINT_PACK32 :: 68;
        A2B10G10R10_SINT_PACK32 :: 69;
        R16_UNORM :: 70;
        R16_SNORM :: 71;
        R16_USCALED :: 72;
        R16_SSCALED :: 73;
        R16_UINT :: 74;
        R16_SINT :: 75;
        R16_SFLOAT :: 76;
        R16G16_UNORM :: 77;
        R16G16_SNORM :: 78;
        R16G16_USCALED :: 79;
        R16G16_SSCALED :: 80;
        R16G16_UINT :: 81;
        R16G16_SINT :: 82;
        R16G16_SFLOAT :: 83;
        R16G16B16_UNORM :: 84;
        R16G16B16_SNORM :: 85;
        R16G16B16_USCALED :: 86;
        R16G16B16_SSCALED :: 87;
        R16G16B16_UINT :: 88;
        R16G16B16_SINT :: 89;
        R16G16B16_SFLOAT :: 90;
        R16G16B16A16_UNORM :: 91;
        R16G16B16A16_SNORM :: 92;
        R16G16B16A16_USCALED :: 93;
        R16G16B16A16_SSCALED :: 94;
        R16G16B16A16_UINT :: 95;
        R16G16B16A16_SINT :: 96;
        R16G16B16A16_SFLOAT :: 97;
        R32_UINT :: 98;
        R32_SINT :: 99;
        R32_SFLOAT :: 100;
        R32G32_UINT :: 101;
        R32G32_SINT :: 102;
        R32G32_SFLOAT :: 103;
        R32G32B32_UINT :: 104;
        R32G32B32_SINT :: 105;
        R32G32B32_SFLOAT :: 106;
        R32G32B32A32_UINT :: 107;
        R32G32B32A32_SINT :: 108;
        R32G32B32A32_SFLOAT :: 109;
        R64_UINT :: 110;
        R64_SINT :: 111;
        R64_SFLOAT :: 112;
        R64G64_UINT :: 113;
        R64G64_SINT :: 114;
        R64G64_SFLOAT :: 115;
        R64G64B64_UINT :: 116;
        R64G64B64_SINT :: 117;
        R64G64B64_SFLOAT :: 118;
        R64G64B64A64_UINT :: 119;
        R64G64B64A64_SINT :: 120;
        R64G64B64A64_SFLOAT :: 121;
        B10G11R11_UFLOAT_PACK32 :: 122;
        E5B9G9R9_UFLOAT_PACK32 :: 123;
        D16_UNORM :: 124;
        X8_D24_UNORM_PACK32 :: 125;
        D32_SFLOAT :: 126;
        S8_UINT :: 127;
        D16_UNORM_S8_UINT :: 128;
        D24_UNORM_S8_UINT :: 129;
        D32_SFLOAT_S8_UINT :: 130;
        BC1_RGB_UNORM_BLOCK :: 131;
        BC1_RGB_SRGB_BLOCK :: 132;
        BC1_RGBA_UNORM_BLOCK :: 133;
        BC1_RGBA_SRGB_BLOCK :: 134;
        BC2_UNORM_BLOCK :: 135;
        BC2_SRGB_BLOCK :: 136;
        BC3_UNORM_BLOCK :: 137;
        BC3_SRGB_BLOCK :: 138;
        BC4_UNORM_BLOCK :: 139;
        BC4_SNORM_BLOCK :: 140;
        BC5_UNORM_BLOCK :: 141;
        BC5_SNORM_BLOCK :: 142;
        BC6H_UFLOAT_BLOCK :: 143;
        BC6H_SFLOAT_BLOCK :: 144;
        BC7_UNORM_BLOCK :: 145;
        BC7_SRGB_BLOCK :: 146;
        ETC2_R8G8B8_UNORM_BLOCK :: 147;
        ETC2_R8G8B8_SRGB_BLOCK :: 148;
        ETC2_R8G8B8A1_UNORM_BLOCK :: 149;
        ETC2_R8G8B8A1_SRGB_BLOCK :: 150;
        ETC2_R8G8B8A8_UNORM_BLOCK :: 151;
        ETC2_R8G8B8A8_SRGB_BLOCK :: 152;
        EAC_R11_UNORM_BLOCK :: 153;
        EAC_R11_SNORM_BLOCK :: 154;
        EAC_R11G11_UNORM_BLOCK :: 155;
        EAC_R11G11_SNORM_BLOCK :: 156;
        ASTC_4x4_UNORM_BLOCK :: 157;
        ASTC_4x4_SRGB_BLOCK :: 158;
        ASTC_5x4_UNORM_BLOCK :: 159;
        ASTC_5x4_SRGB_BLOCK :: 160;
        ASTC_5x5_UNORM_BLOCK :: 161;
        ASTC_5x5_SRGB_BLOCK :: 162;
        ASTC_6x5_UNORM_BLOCK :: 163;
        ASTC_6x5_SRGB_BLOCK :: 164;
        ASTC_6x6_UNORM_BLOCK :: 165;
        ASTC_6x6_SRGB_BLOCK :: 166;
        ASTC_8x5_UNORM_BLOCK :: 167;
        ASTC_8x5_SRGB_BLOCK :: 168;
        ASTC_8x6_UNORM_BLOCK :: 169;
        ASTC_8x6_SRGB_BLOCK :: 170;
        ASTC_8x8_UNORM_BLOCK :: 171;
        ASTC_8x8_SRGB_BLOCK :: 172;
        ASTC_10x5_UNORM_BLOCK :: 173;
        ASTC_10x5_SRGB_BLOCK :: 174;
        ASTC_10x6_UNORM_BLOCK :: 175;
        ASTC_10x6_SRGB_BLOCK :: 176;
        ASTC_10x8_UNORM_BLOCK :: 177;
        ASTC_10x8_SRGB_BLOCK :: 178;
        ASTC_10x10_UNORM_BLOCK :: 179;
        ASTC_10x10_SRGB_BLOCK :: 180;
        ASTC_12x10_UNORM_BLOCK :: 181;
        ASTC_12x10_SRGB_BLOCK :: 182;
        ASTC_12x12_UNORM_BLOCK :: 183;
        ASTC_12x12_SRGB_BLOCK :: 184;
        G8B8G8R8_422_UNORM :: 1000156000;
        B8G8R8G8_422_UNORM :: 1000156001;
        G8_B8_R8_3PLANE_420_UNORM :: 1000156002;
        G8_B8R8_2PLANE_420_UNORM :: 1000156003;
        G8_B8_R8_3PLANE_422_UNORM :: 1000156004;
        G8_B8R8_2PLANE_422_UNORM :: 1000156005;
        G8_B8_R8_3PLANE_444_UNORM :: 1000156006;
        R10X6_UNORM_PACK16 :: 1000156007;
        R10X6G10X6_UNORM_2PACK16 :: 1000156008;
        R10X6G10X6B10X6A10X6_UNORM_4PACK16 :: 1000156009;
        G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 :: 1000156010;
        B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 :: 1000156011;
        G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 :: 1000156012;
        G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 :: 1000156013;
        G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 :: 1000156014;
        G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 :: 1000156015;
        G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 :: 1000156016;
        R12X4_UNORM_PACK16 :: 1000156017;
        R12X4G12X4_UNORM_2PACK16 :: 1000156018;
        R12X4G12X4B12X4A12X4_UNORM_4PACK16 :: 1000156019;
        G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 :: 1000156020;
        B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 :: 1000156021;
        G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 :: 1000156022;
        G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 :: 1000156023;
        G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 :: 1000156024;
        G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 :: 1000156025;
        G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 :: 1000156026;
        G16B16G16R16_422_UNORM :: 1000156027;
        B16G16R16G16_422_UNORM :: 1000156028;
        G16_B16_R16_3PLANE_420_UNORM :: 1000156029;
        G16_B16R16_2PLANE_420_UNORM :: 1000156030;
        G16_B16_R16_3PLANE_422_UNORM :: 1000156031;
        G16_B16R16_2PLANE_422_UNORM :: 1000156032;
        G16_B16_R16_3PLANE_444_UNORM :: 1000156033;
        PVRTC1_2BPP_UNORM_BLOCK_IMG :: 1000054000;
        PVRTC1_4BPP_UNORM_BLOCK_IMG :: 1000054001;
        PVRTC2_2BPP_UNORM_BLOCK_IMG :: 1000054002;
        PVRTC2_4BPP_UNORM_BLOCK_IMG :: 1000054003;
        PVRTC1_2BPP_SRGB_BLOCK_IMG :: 1000054004;
        PVRTC1_4BPP_SRGB_BLOCK_IMG :: 1000054005;
        PVRTC2_2BPP_SRGB_BLOCK_IMG :: 1000054006;
        PVRTC2_4BPP_SRGB_BLOCK_IMG :: 1000054007;
        G8B8G8R8_422_UNORM_KHR :: 1000156000;
        B8G8R8G8_422_UNORM_KHR :: 1000156001;
        G8_B8_R8_3PLANE_420_UNORM_KHR :: 1000156002;
        G8_B8R8_2PLANE_420_UNORM_KHR :: 1000156003;
        G8_B8_R8_3PLANE_422_UNORM_KHR :: 1000156004;
        G8_B8R8_2PLANE_422_UNORM_KHR :: 1000156005;
        G8_B8_R8_3PLANE_444_UNORM_KHR :: 1000156006;
        R10X6_UNORM_PACK16_KHR :: 1000156007;
        R10X6G10X6_UNORM_2PACK16_KHR :: 1000156008;
        R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR :: 1000156009;
        G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR :: 1000156010;
        B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR :: 1000156011;
        G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR :: 1000156012;
        G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR :: 1000156013;
        G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR :: 1000156014;
        G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR :: 1000156015;
        G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR :: 1000156016;
        R12X4_UNORM_PACK16_KHR :: 1000156017;
        R12X4G12X4_UNORM_2PACK16_KHR :: 1000156018;
        R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR :: 1000156019;
        G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR :: 1000156020;
        B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR :: 1000156021;
        G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR :: 1000156022;
        G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR :: 1000156023;
        G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR :: 1000156024;
        G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR :: 1000156025;
        G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR :: 1000156026;
        G16B16G16R16_422_UNORM_KHR :: 1000156027;
        B16G16R16G16_422_UNORM_KHR :: 1000156028;
        G16_B16_R16_3PLANE_420_UNORM_KHR :: 1000156029;
        G16_B16R16_2PLANE_420_UNORM_KHR :: 1000156030;
        G16_B16_R16_3PLANE_422_UNORM_KHR :: 1000156031;
        G16_B16R16_2PLANE_422_UNORM_KHR :: 1000156032;
        G16_B16_R16_3PLANE_444_UNORM_KHR :: 1000156033;
        BEGIN_RANGE :: 0;
        END_RANGE :: 184;
        RANGE_SIZE :: 185;
        MAX_ENUM :: 2147483647;

        VK_FORMAT_UNDEFINED :: UNDEFINED;
        VK_FORMAT_R4G4_UNORM_PACK8 :: R4G4_UNORM_PACK8;
        VK_FORMAT_R4G4B4A4_UNORM_PACK16 :: R4G4B4A4_UNORM_PACK16;
        VK_FORMAT_B4G4R4A4_UNORM_PACK16 :: B4G4R4A4_UNORM_PACK16;
        VK_FORMAT_R5G6B5_UNORM_PACK16 :: R5G6B5_UNORM_PACK16;
        VK_FORMAT_B5G6R5_UNORM_PACK16 :: B5G6R5_UNORM_PACK16;
        VK_FORMAT_R5G5B5A1_UNORM_PACK16 :: R5G5B5A1_UNORM_PACK16;
        VK_FORMAT_B5G5R5A1_UNORM_PACK16 :: B5G5R5A1_UNORM_PACK16;
        VK_FORMAT_A1R5G5B5_UNORM_PACK16 :: A1R5G5B5_UNORM_PACK16;
        VK_FORMAT_R8_UNORM :: R8_UNORM;
        VK_FORMAT_R8_SNORM :: R8_SNORM;
        VK_FORMAT_R8_USCALED :: R8_USCALED;
        VK_FORMAT_R8_SSCALED :: R8_SSCALED;
        VK_FORMAT_R8_UINT :: R8_UINT;
        VK_FORMAT_R8_SINT :: R8_SINT;
        VK_FORMAT_R8_SRGB :: R8_SRGB;
        VK_FORMAT_R8G8_UNORM :: R8G8_UNORM;
        VK_FORMAT_R8G8_SNORM :: R8G8_SNORM;
        VK_FORMAT_R8G8_USCALED :: R8G8_USCALED;
        VK_FORMAT_R8G8_SSCALED :: R8G8_SSCALED;
        VK_FORMAT_R8G8_UINT :: R8G8_UINT;
        VK_FORMAT_R8G8_SINT :: R8G8_SINT;
        VK_FORMAT_R8G8_SRGB :: R8G8_SRGB;
        VK_FORMAT_R8G8B8_UNORM :: R8G8B8_UNORM;
        VK_FORMAT_R8G8B8_SNORM :: R8G8B8_SNORM;
        VK_FORMAT_R8G8B8_USCALED :: R8G8B8_USCALED;
        VK_FORMAT_R8G8B8_SSCALED :: R8G8B8_SSCALED;
        VK_FORMAT_R8G8B8_UINT :: R8G8B8_UINT;
        VK_FORMAT_R8G8B8_SINT :: R8G8B8_SINT;
        VK_FORMAT_R8G8B8_SRGB :: R8G8B8_SRGB;
        VK_FORMAT_B8G8R8_UNORM :: B8G8R8_UNORM;
        VK_FORMAT_B8G8R8_SNORM :: B8G8R8_SNORM;
        VK_FORMAT_B8G8R8_USCALED :: B8G8R8_USCALED;
        VK_FORMAT_B8G8R8_SSCALED :: B8G8R8_SSCALED;
        VK_FORMAT_B8G8R8_UINT :: B8G8R8_UINT;
        VK_FORMAT_B8G8R8_SINT :: B8G8R8_SINT;
        VK_FORMAT_B8G8R8_SRGB :: B8G8R8_SRGB;
        VK_FORMAT_R8G8B8A8_UNORM :: R8G8B8A8_UNORM;
        VK_FORMAT_R8G8B8A8_SNORM :: R8G8B8A8_SNORM;
        VK_FORMAT_R8G8B8A8_USCALED :: R8G8B8A8_USCALED;
        VK_FORMAT_R8G8B8A8_SSCALED :: R8G8B8A8_SSCALED;
        VK_FORMAT_R8G8B8A8_UINT :: R8G8B8A8_UINT;
        VK_FORMAT_R8G8B8A8_SINT :: R8G8B8A8_SINT;
        VK_FORMAT_R8G8B8A8_SRGB :: R8G8B8A8_SRGB;
        VK_FORMAT_B8G8R8A8_UNORM :: B8G8R8A8_UNORM;
        VK_FORMAT_B8G8R8A8_SNORM :: B8G8R8A8_SNORM;
        VK_FORMAT_B8G8R8A8_USCALED :: B8G8R8A8_USCALED;
        VK_FORMAT_B8G8R8A8_SSCALED :: B8G8R8A8_SSCALED;
        VK_FORMAT_B8G8R8A8_UINT :: B8G8R8A8_UINT;
        VK_FORMAT_B8G8R8A8_SINT :: B8G8R8A8_SINT;
        VK_FORMAT_B8G8R8A8_SRGB :: B8G8R8A8_SRGB;
        VK_FORMAT_A8B8G8R8_UNORM_PACK32 :: A8B8G8R8_UNORM_PACK32;
        VK_FORMAT_A8B8G8R8_SNORM_PACK32 :: A8B8G8R8_SNORM_PACK32;
        VK_FORMAT_A8B8G8R8_USCALED_PACK32 :: A8B8G8R8_USCALED_PACK32;
        VK_FORMAT_A8B8G8R8_SSCALED_PACK32 :: A8B8G8R8_SSCALED_PACK32;
        VK_FORMAT_A8B8G8R8_UINT_PACK32 :: A8B8G8R8_UINT_PACK32;
        VK_FORMAT_A8B8G8R8_SINT_PACK32 :: A8B8G8R8_SINT_PACK32;
        VK_FORMAT_A8B8G8R8_SRGB_PACK32 :: A8B8G8R8_SRGB_PACK32;
        VK_FORMAT_A2R10G10B10_UNORM_PACK32 :: A2R10G10B10_UNORM_PACK32;
        VK_FORMAT_A2R10G10B10_SNORM_PACK32 :: A2R10G10B10_SNORM_PACK32;
        VK_FORMAT_A2R10G10B10_USCALED_PACK32 :: A2R10G10B10_USCALED_PACK32;
        VK_FORMAT_A2R10G10B10_SSCALED_PACK32 :: A2R10G10B10_SSCALED_PACK32;
        VK_FORMAT_A2R10G10B10_UINT_PACK32 :: A2R10G10B10_UINT_PACK32;
        VK_FORMAT_A2R10G10B10_SINT_PACK32 :: A2R10G10B10_SINT_PACK32;
        VK_FORMAT_A2B10G10R10_UNORM_PACK32 :: A2B10G10R10_UNORM_PACK32;
        VK_FORMAT_A2B10G10R10_SNORM_PACK32 :: A2B10G10R10_SNORM_PACK32;
        VK_FORMAT_A2B10G10R10_USCALED_PACK32 :: A2B10G10R10_USCALED_PACK32;
        VK_FORMAT_A2B10G10R10_SSCALED_PACK32 :: A2B10G10R10_SSCALED_PACK32;
        VK_FORMAT_A2B10G10R10_UINT_PACK32 :: A2B10G10R10_UINT_PACK32;
        VK_FORMAT_A2B10G10R10_SINT_PACK32 :: A2B10G10R10_SINT_PACK32;
        VK_FORMAT_R16_UNORM :: R16_UNORM;
        VK_FORMAT_R16_SNORM :: R16_SNORM;
        VK_FORMAT_R16_USCALED :: R16_USCALED;
        VK_FORMAT_R16_SSCALED :: R16_SSCALED;
        VK_FORMAT_R16_UINT :: R16_UINT;
        VK_FORMAT_R16_SINT :: R16_SINT;
        VK_FORMAT_R16_SFLOAT :: R16_SFLOAT;
        VK_FORMAT_R16G16_UNORM :: R16G16_UNORM;
        VK_FORMAT_R16G16_SNORM :: R16G16_SNORM;
        VK_FORMAT_R16G16_USCALED :: R16G16_USCALED;
        VK_FORMAT_R16G16_SSCALED :: R16G16_SSCALED;
        VK_FORMAT_R16G16_UINT :: R16G16_UINT;
        VK_FORMAT_R16G16_SINT :: R16G16_SINT;
        VK_FORMAT_R16G16_SFLOAT :: R16G16_SFLOAT;
        VK_FORMAT_R16G16B16_UNORM :: R16G16B16_UNORM;
        VK_FORMAT_R16G16B16_SNORM :: R16G16B16_SNORM;
        VK_FORMAT_R16G16B16_USCALED :: R16G16B16_USCALED;
        VK_FORMAT_R16G16B16_SSCALED :: R16G16B16_SSCALED;
        VK_FORMAT_R16G16B16_UINT :: R16G16B16_UINT;
        VK_FORMAT_R16G16B16_SINT :: R16G16B16_SINT;
        VK_FORMAT_R16G16B16_SFLOAT :: R16G16B16_SFLOAT;
        VK_FORMAT_R16G16B16A16_UNORM :: R16G16B16A16_UNORM;
        VK_FORMAT_R16G16B16A16_SNORM :: R16G16B16A16_SNORM;
        VK_FORMAT_R16G16B16A16_USCALED :: R16G16B16A16_USCALED;
        VK_FORMAT_R16G16B16A16_SSCALED :: R16G16B16A16_SSCALED;
        VK_FORMAT_R16G16B16A16_UINT :: R16G16B16A16_UINT;
        VK_FORMAT_R16G16B16A16_SINT :: R16G16B16A16_SINT;
        VK_FORMAT_R16G16B16A16_SFLOAT :: R16G16B16A16_SFLOAT;
        VK_FORMAT_R32_UINT :: R32_UINT;
        VK_FORMAT_R32_SINT :: R32_SINT;
        VK_FORMAT_R32_SFLOAT :: R32_SFLOAT;
        VK_FORMAT_R32G32_UINT :: R32G32_UINT;
        VK_FORMAT_R32G32_SINT :: R32G32_SINT;
        VK_FORMAT_R32G32_SFLOAT :: R32G32_SFLOAT;
        VK_FORMAT_R32G32B32_UINT :: R32G32B32_UINT;
        VK_FORMAT_R32G32B32_SINT :: R32G32B32_SINT;
        VK_FORMAT_R32G32B32_SFLOAT :: R32G32B32_SFLOAT;
        VK_FORMAT_R32G32B32A32_UINT :: R32G32B32A32_UINT;
        VK_FORMAT_R32G32B32A32_SINT :: R32G32B32A32_SINT;
        VK_FORMAT_R32G32B32A32_SFLOAT :: R32G32B32A32_SFLOAT;
        VK_FORMAT_R64_UINT :: R64_UINT;
        VK_FORMAT_R64_SINT :: R64_SINT;
        VK_FORMAT_R64_SFLOAT :: R64_SFLOAT;
        VK_FORMAT_R64G64_UINT :: R64G64_UINT;
        VK_FORMAT_R64G64_SINT :: R64G64_SINT;
        VK_FORMAT_R64G64_SFLOAT :: R64G64_SFLOAT;
        VK_FORMAT_R64G64B64_UINT :: R64G64B64_UINT;
        VK_FORMAT_R64G64B64_SINT :: R64G64B64_SINT;
        VK_FORMAT_R64G64B64_SFLOAT :: R64G64B64_SFLOAT;
        VK_FORMAT_R64G64B64A64_UINT :: R64G64B64A64_UINT;
        VK_FORMAT_R64G64B64A64_SINT :: R64G64B64A64_SINT;
        VK_FORMAT_R64G64B64A64_SFLOAT :: R64G64B64A64_SFLOAT;
        VK_FORMAT_B10G11R11_UFLOAT_PACK32 :: B10G11R11_UFLOAT_PACK32;
        VK_FORMAT_E5B9G9R9_UFLOAT_PACK32 :: E5B9G9R9_UFLOAT_PACK32;
        VK_FORMAT_D16_UNORM :: D16_UNORM;
        VK_FORMAT_X8_D24_UNORM_PACK32 :: X8_D24_UNORM_PACK32;
        VK_FORMAT_D32_SFLOAT :: D32_SFLOAT;
        VK_FORMAT_S8_UINT :: S8_UINT;
        VK_FORMAT_D16_UNORM_S8_UINT :: D16_UNORM_S8_UINT;
        VK_FORMAT_D24_UNORM_S8_UINT :: D24_UNORM_S8_UINT;
        VK_FORMAT_D32_SFLOAT_S8_UINT :: D32_SFLOAT_S8_UINT;
        VK_FORMAT_BC1_RGB_UNORM_BLOCK :: BC1_RGB_UNORM_BLOCK;
        VK_FORMAT_BC1_RGB_SRGB_BLOCK :: BC1_RGB_SRGB_BLOCK;
        VK_FORMAT_BC1_RGBA_UNORM_BLOCK :: BC1_RGBA_UNORM_BLOCK;
        VK_FORMAT_BC1_RGBA_SRGB_BLOCK :: BC1_RGBA_SRGB_BLOCK;
        VK_FORMAT_BC2_UNORM_BLOCK :: BC2_UNORM_BLOCK;
        VK_FORMAT_BC2_SRGB_BLOCK :: BC2_SRGB_BLOCK;
        VK_FORMAT_BC3_UNORM_BLOCK :: BC3_UNORM_BLOCK;
        VK_FORMAT_BC3_SRGB_BLOCK :: BC3_SRGB_BLOCK;
        VK_FORMAT_BC4_UNORM_BLOCK :: BC4_UNORM_BLOCK;
        VK_FORMAT_BC4_SNORM_BLOCK :: BC4_SNORM_BLOCK;
        VK_FORMAT_BC5_UNORM_BLOCK :: BC5_UNORM_BLOCK;
        VK_FORMAT_BC5_SNORM_BLOCK :: BC5_SNORM_BLOCK;
        VK_FORMAT_BC6H_UFLOAT_BLOCK :: BC6H_UFLOAT_BLOCK;
        VK_FORMAT_BC6H_SFLOAT_BLOCK :: BC6H_SFLOAT_BLOCK;
        VK_FORMAT_BC7_UNORM_BLOCK :: BC7_UNORM_BLOCK;
        VK_FORMAT_BC7_SRGB_BLOCK :: BC7_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK :: ETC2_R8G8B8_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK :: ETC2_R8G8B8_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A1_UNORM_BLOCK :: ETC2_R8G8B8A1_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A1_SRGB_BLOCK :: ETC2_R8G8B8A1_SRGB_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK :: ETC2_R8G8B8A8_UNORM_BLOCK;
        VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK :: ETC2_R8G8B8A8_SRGB_BLOCK;
        VK_FORMAT_EAC_R11_UNORM_BLOCK :: EAC_R11_UNORM_BLOCK;
        VK_FORMAT_EAC_R11_SNORM_BLOCK :: EAC_R11_SNORM_BLOCK;
        VK_FORMAT_EAC_R11G11_UNORM_BLOCK :: EAC_R11G11_UNORM_BLOCK;
        VK_FORMAT_EAC_R11G11_SNORM_BLOCK :: EAC_R11G11_SNORM_BLOCK;
        VK_FORMAT_ASTC_4x4_UNORM_BLOCK :: ASTC_4x4_UNORM_BLOCK;
        VK_FORMAT_ASTC_4x4_SRGB_BLOCK :: ASTC_4x4_SRGB_BLOCK;
        VK_FORMAT_ASTC_5x4_UNORM_BLOCK :: ASTC_5x4_UNORM_BLOCK;
        VK_FORMAT_ASTC_5x4_SRGB_BLOCK :: ASTC_5x4_SRGB_BLOCK;
        VK_FORMAT_ASTC_5x5_UNORM_BLOCK :: ASTC_5x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_5x5_SRGB_BLOCK :: ASTC_5x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_6x5_UNORM_BLOCK :: ASTC_6x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_6x5_SRGB_BLOCK :: ASTC_6x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_6x6_UNORM_BLOCK :: ASTC_6x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_6x6_SRGB_BLOCK :: ASTC_6x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x5_UNORM_BLOCK :: ASTC_8x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x5_SRGB_BLOCK :: ASTC_8x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x6_UNORM_BLOCK :: ASTC_8x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x6_SRGB_BLOCK :: ASTC_8x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_8x8_UNORM_BLOCK :: ASTC_8x8_UNORM_BLOCK;
        VK_FORMAT_ASTC_8x8_SRGB_BLOCK :: ASTC_8x8_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x5_UNORM_BLOCK :: ASTC_10x5_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x5_SRGB_BLOCK :: ASTC_10x5_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x6_UNORM_BLOCK :: ASTC_10x6_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x6_SRGB_BLOCK :: ASTC_10x6_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x8_UNORM_BLOCK :: ASTC_10x8_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x8_SRGB_BLOCK :: ASTC_10x8_SRGB_BLOCK;
        VK_FORMAT_ASTC_10x10_UNORM_BLOCK :: ASTC_10x10_UNORM_BLOCK;
        VK_FORMAT_ASTC_10x10_SRGB_BLOCK :: ASTC_10x10_SRGB_BLOCK;
        VK_FORMAT_ASTC_12x10_UNORM_BLOCK :: ASTC_12x10_UNORM_BLOCK;
        VK_FORMAT_ASTC_12x10_SRGB_BLOCK :: ASTC_12x10_SRGB_BLOCK;
        VK_FORMAT_ASTC_12x12_UNORM_BLOCK :: ASTC_12x12_UNORM_BLOCK;
        VK_FORMAT_ASTC_12x12_SRGB_BLOCK :: ASTC_12x12_SRGB_BLOCK;
        VK_FORMAT_G8B8G8R8_422_UNORM :: G8B8G8R8_422_UNORM;
        VK_FORMAT_B8G8R8G8_422_UNORM :: B8G8R8G8_422_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM :: G8_B8_R8_3PLANE_420_UNORM;
        VK_FORMAT_G8_B8R8_2PLANE_420_UNORM :: G8_B8R8_2PLANE_420_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM :: G8_B8_R8_3PLANE_422_UNORM;
        VK_FORMAT_G8_B8R8_2PLANE_422_UNORM :: G8_B8R8_2PLANE_422_UNORM;
        VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM :: G8_B8_R8_3PLANE_444_UNORM;
        VK_FORMAT_R10X6_UNORM_PACK16 :: R10X6_UNORM_PACK16;
        VK_FORMAT_R10X6G10X6_UNORM_2PACK16 :: R10X6G10X6_UNORM_2PACK16;
        VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16 :: R10X6G10X6B10X6A10X6_UNORM_4PACK16;
        VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 :: G10X6B10X6G10X6R10X6_422_UNORM_4PACK16;
        VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 :: B10X6G10X6R10X6G10X6_422_UNORM_4PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 :: G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 :: G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 :: G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16;
        VK_FORMAT_R12X4_UNORM_PACK16 :: R12X4_UNORM_PACK16;
        VK_FORMAT_R12X4G12X4_UNORM_2PACK16 :: R12X4G12X4_UNORM_2PACK16;
        VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16 :: R12X4G12X4B12X4A12X4_UNORM_4PACK16;
        VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 :: G12X4B12X4G12X4R12X4_422_UNORM_4PACK16;
        VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 :: B12X4G12X4R12X4G12X4_422_UNORM_4PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 :: G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 :: G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 :: G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16;
        VK_FORMAT_G16B16G16R16_422_UNORM :: G16B16G16R16_422_UNORM;
        VK_FORMAT_B16G16R16G16_422_UNORM :: B16G16R16G16_422_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM :: G16_B16_R16_3PLANE_420_UNORM;
        VK_FORMAT_G16_B16R16_2PLANE_420_UNORM :: G16_B16R16_2PLANE_420_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM :: G16_B16_R16_3PLANE_422_UNORM;
        VK_FORMAT_G16_B16R16_2PLANE_422_UNORM :: G16_B16R16_2PLANE_422_UNORM;
        VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM :: G16_B16_R16_3PLANE_444_UNORM;
        VK_FORMAT_PVRTC1_2BPP_UNORM_BLOCK_IMG :: PVRTC1_2BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC1_4BPP_UNORM_BLOCK_IMG :: PVRTC1_4BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC2_2BPP_UNORM_BLOCK_IMG :: PVRTC2_2BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC2_4BPP_UNORM_BLOCK_IMG :: PVRTC2_4BPP_UNORM_BLOCK_IMG;
        VK_FORMAT_PVRTC1_2BPP_SRGB_BLOCK_IMG :: PVRTC1_2BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC1_4BPP_SRGB_BLOCK_IMG :: PVRTC1_4BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC2_2BPP_SRGB_BLOCK_IMG :: PVRTC2_2BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_PVRTC2_4BPP_SRGB_BLOCK_IMG :: PVRTC2_4BPP_SRGB_BLOCK_IMG;
        VK_FORMAT_G8B8G8R8_422_UNORM_KHR :: G8B8G8R8_422_UNORM_KHR;
        VK_FORMAT_B8G8R8G8_422_UNORM_KHR :: B8G8R8G8_422_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM_KHR :: G8_B8_R8_3PLANE_420_UNORM_KHR;
        VK_FORMAT_G8_B8R8_2PLANE_420_UNORM_KHR :: G8_B8R8_2PLANE_420_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM_KHR :: G8_B8_R8_3PLANE_422_UNORM_KHR;
        VK_FORMAT_G8_B8R8_2PLANE_422_UNORM_KHR :: G8_B8R8_2PLANE_422_UNORM_KHR;
        VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM_KHR :: G8_B8_R8_3PLANE_444_UNORM_KHR;
        VK_FORMAT_R10X6_UNORM_PACK16_KHR :: R10X6_UNORM_PACK16_KHR;
        VK_FORMAT_R10X6G10X6_UNORM_2PACK16_KHR :: R10X6G10X6_UNORM_2PACK16_KHR;
        VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR :: R10X6G10X6B10X6A10X6_UNORM_4PACK16_KHR;
        VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR :: G10X6B10X6G10X6R10X6_422_UNORM_4PACK16_KHR;
        VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR :: B10X6G10X6R10X6G10X6_422_UNORM_4PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR :: G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR :: G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR :: G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16_KHR;
        VK_FORMAT_R12X4_UNORM_PACK16_KHR :: R12X4_UNORM_PACK16_KHR;
        VK_FORMAT_R12X4G12X4_UNORM_2PACK16_KHR :: R12X4G12X4_UNORM_2PACK16_KHR;
        VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR :: R12X4G12X4B12X4A12X4_UNORM_4PACK16_KHR;
        VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR :: G12X4B12X4G12X4R12X4_422_UNORM_4PACK16_KHR;
        VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR :: B12X4G12X4R12X4G12X4_422_UNORM_4PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR :: G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR :: G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16_KHR;
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR :: G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16_KHR;
        VK_FORMAT_G16B16G16R16_422_UNORM_KHR :: G16B16G16R16_422_UNORM_KHR;
        VK_FORMAT_B16G16R16G16_422_UNORM_KHR :: B16G16R16G16_422_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM_KHR :: G16_B16_R16_3PLANE_420_UNORM_KHR;
        VK_FORMAT_G16_B16R16_2PLANE_420_UNORM_KHR :: G16_B16R16_2PLANE_420_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM_KHR :: G16_B16_R16_3PLANE_422_UNORM_KHR;
        VK_FORMAT_G16_B16R16_2PLANE_422_UNORM_KHR :: G16_B16R16_2PLANE_422_UNORM_KHR;
        VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM_KHR :: G16_B16_R16_3PLANE_444_UNORM_KHR;
        VK_FORMAT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FORMAT_END_RANGE :: END_RANGE;
        VK_FORMAT_RANGE_SIZE :: RANGE_SIZE;
        VK_FORMAT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkFormatFeatureFlagBits :: enum_flags u32 {
        SAMPLED_IMAGE_BIT :: 0x1;
        STORAGE_IMAGE_BIT :: 0x2;
        STORAGE_IMAGE_ATOMIC_BIT :: 0x4;
        UNIFORM_TEXEL_BUFFER_BIT :: 0x8;
        STORAGE_TEXEL_BUFFER_BIT :: 0x10;
        STORAGE_TEXEL_BUFFER_ATOMIC_BIT :: 0x20;
        VERTEX_BUFFER_BIT :: 0x40;
        COLOR_ATTACHMENT_BIT :: 0x80;
        COLOR_ATTACHMENT_BLEND_BIT :: 0x100;
        DEPTH_STENCIL_ATTACHMENT_BIT :: 0x200;
        BLIT_SRC_BIT :: 0x400;
        BLIT_DST_BIT :: 0x800;
        SAMPLED_IMAGE_FILTER_LINEAR_BIT :: 0x1000;
        TRANSFER_SRC_BIT :: 0x4000;
        TRANSFER_DST_BIT :: 0x8000;
        MIDPOINT_CHROMA_SAMPLES_BIT :: 0x20000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT :: 0x40000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT :: 0x80000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT :: 0x100000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT :: 0x200000;
        DISJOINT_BIT :: 0x400000;
        COSITED_CHROMA_SAMPLES_BIT :: 0x800000;
        SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG :: 0x2000;
        SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT :: 0x10000;
        TRANSFER_SRC_BIT_KHR :: 0x4000;
        TRANSFER_DST_BIT_KHR :: 0x8000;
        MIDPOINT_CHROMA_SAMPLES_BIT_KHR :: 0x20000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR :: 0x40000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR :: 0x80000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR :: 0x100000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR :: 0x200000;
        DISJOINT_BIT_KHR :: 0x400000;
        COSITED_CHROMA_SAMPLES_BIT_KHR :: 0x800000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT :: SAMPLED_IMAGE_BIT;
        VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT :: STORAGE_IMAGE_BIT;
        VK_FORMAT_FEATURE_STORAGE_IMAGE_ATOMIC_BIT :: STORAGE_IMAGE_ATOMIC_BIT;
        VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT :: UNIFORM_TEXEL_BUFFER_BIT;
        VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_BIT :: STORAGE_TEXEL_BUFFER_BIT;
        VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_ATOMIC_BIT :: STORAGE_TEXEL_BUFFER_ATOMIC_BIT;
        VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT :: VERTEX_BUFFER_BIT;
        VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT :: COLOR_ATTACHMENT_BIT;
        VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT :: COLOR_ATTACHMENT_BLEND_BIT;
        VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT :: DEPTH_STENCIL_ATTACHMENT_BIT;
        VK_FORMAT_FEATURE_BLIT_SRC_BIT :: BLIT_SRC_BIT;
        VK_FORMAT_FEATURE_BLIT_DST_BIT :: BLIT_DST_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT :: SAMPLED_IMAGE_FILTER_LINEAR_BIT;
        VK_FORMAT_FEATURE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_FORMAT_FEATURE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_FORMAT_FEATURE_MIDPOINT_CHROMA_SAMPLES_BIT :: MIDPOINT_CHROMA_SAMPLES_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT;
        VK_FORMAT_FEATURE_DISJOINT_BIT :: DISJOINT_BIT;
        VK_FORMAT_FEATURE_COSITED_CHROMA_SAMPLES_BIT :: COSITED_CHROMA_SAMPLES_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG :: SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT :: SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT;
        VK_FORMAT_FEATURE_TRANSFER_SRC_BIT_KHR :: TRANSFER_SRC_BIT_KHR;
        VK_FORMAT_FEATURE_TRANSFER_DST_BIT_KHR :: TRANSFER_DST_BIT_KHR;
        VK_FORMAT_FEATURE_MIDPOINT_CHROMA_SAMPLES_BIT_KHR :: MIDPOINT_CHROMA_SAMPLES_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR;
        VK_FORMAT_FEATURE_DISJOINT_BIT_KHR :: DISJOINT_BIT_KHR;
        VK_FORMAT_FEATURE_COSITED_CHROMA_SAMPLES_BIT_KHR :: COSITED_CHROMA_SAMPLES_BIT_KHR;
        VK_FORMAT_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFormatFeatureFlagBits :: enum_flags u32 {
        SAMPLED_IMAGE_BIT :: 0x1;
        STORAGE_IMAGE_BIT :: 0x2;
        STORAGE_IMAGE_ATOMIC_BIT :: 0x4;
        UNIFORM_TEXEL_BUFFER_BIT :: 0x8;
        STORAGE_TEXEL_BUFFER_BIT :: 0x10;
        STORAGE_TEXEL_BUFFER_ATOMIC_BIT :: 0x20;
        VERTEX_BUFFER_BIT :: 0x40;
        COLOR_ATTACHMENT_BIT :: 0x80;
        COLOR_ATTACHMENT_BLEND_BIT :: 0x100;
        DEPTH_STENCIL_ATTACHMENT_BIT :: 0x200;
        BLIT_SRC_BIT :: 0x400;
        BLIT_DST_BIT :: 0x800;
        SAMPLED_IMAGE_FILTER_LINEAR_BIT :: 0x1000;
        TRANSFER_SRC_BIT :: 0x4000;
        TRANSFER_DST_BIT :: 0x8000;
        MIDPOINT_CHROMA_SAMPLES_BIT :: 0x20000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT :: 0x40000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT :: 0x80000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT :: 0x100000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT :: 0x200000;
        DISJOINT_BIT :: 0x400000;
        COSITED_CHROMA_SAMPLES_BIT :: 0x800000;
        SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG :: 0x2000;
        SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT :: 0x10000;
        TRANSFER_SRC_BIT_KHR :: 0x4000;
        TRANSFER_DST_BIT_KHR :: 0x8000;
        MIDPOINT_CHROMA_SAMPLES_BIT_KHR :: 0x20000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR :: 0x40000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR :: 0x80000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR :: 0x100000;
        SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR :: 0x200000;
        DISJOINT_BIT_KHR :: 0x400000;
        COSITED_CHROMA_SAMPLES_BIT_KHR :: 0x800000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT :: SAMPLED_IMAGE_BIT;
        VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT :: STORAGE_IMAGE_BIT;
        VK_FORMAT_FEATURE_STORAGE_IMAGE_ATOMIC_BIT :: STORAGE_IMAGE_ATOMIC_BIT;
        VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT :: UNIFORM_TEXEL_BUFFER_BIT;
        VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_BIT :: STORAGE_TEXEL_BUFFER_BIT;
        VK_FORMAT_FEATURE_STORAGE_TEXEL_BUFFER_ATOMIC_BIT :: STORAGE_TEXEL_BUFFER_ATOMIC_BIT;
        VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT :: VERTEX_BUFFER_BIT;
        VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT :: COLOR_ATTACHMENT_BIT;
        VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT :: COLOR_ATTACHMENT_BLEND_BIT;
        VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT :: DEPTH_STENCIL_ATTACHMENT_BIT;
        VK_FORMAT_FEATURE_BLIT_SRC_BIT :: BLIT_SRC_BIT;
        VK_FORMAT_FEATURE_BLIT_DST_BIT :: BLIT_DST_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT :: SAMPLED_IMAGE_FILTER_LINEAR_BIT;
        VK_FORMAT_FEATURE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_FORMAT_FEATURE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_FORMAT_FEATURE_MIDPOINT_CHROMA_SAMPLES_BIT :: MIDPOINT_CHROMA_SAMPLES_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT;
        VK_FORMAT_FEATURE_DISJOINT_BIT :: DISJOINT_BIT;
        VK_FORMAT_FEATURE_COSITED_CHROMA_SAMPLES_BIT :: COSITED_CHROMA_SAMPLES_BIT;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG :: SAMPLED_IMAGE_FILTER_CUBIC_BIT_IMG;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT :: SAMPLED_IMAGE_FILTER_MINMAX_BIT_EXT;
        VK_FORMAT_FEATURE_TRANSFER_SRC_BIT_KHR :: TRANSFER_SRC_BIT_KHR;
        VK_FORMAT_FEATURE_TRANSFER_DST_BIT_KHR :: TRANSFER_DST_BIT_KHR;
        VK_FORMAT_FEATURE_MIDPOINT_CHROMA_SAMPLES_BIT_KHR :: MIDPOINT_CHROMA_SAMPLES_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_SEPARATE_RECONSTRUCTION_FILTER_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_BIT_KHR;
        VK_FORMAT_FEATURE_SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR :: SAMPLED_IMAGE_YCBCR_CONVERSION_CHROMA_RECONSTRUCTION_EXPLICIT_FORCEABLE_BIT_KHR;
        VK_FORMAT_FEATURE_DISJOINT_BIT_KHR :: DISJOINT_BIT_KHR;
        VK_FORMAT_FEATURE_COSITED_CHROMA_SAMPLES_BIT_KHR :: COSITED_CHROMA_SAMPLES_BIT_KHR;
        VK_FORMAT_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkFormatProperties :: struct {
        linearTilingFeatures: VkFormatFeatureFlags;
        optimalTilingFeatures: VkFormatFeatureFlags;
        bufferFeatures: VkFormatFeatureFlags;
    }

}

#if OS == .LINUX {
    VkFormatProperties :: struct {
        linearTilingFeatures: VkFormatFeatureFlags;
        optimalTilingFeatures: VkFormatFeatureFlags;
        bufferFeatures: VkFormatFeatureFlags;
    }

}

#if OS == .WINDOWS {
    VkFormatProperties2 :: struct {
        sType: VkStructureType = .FORMAT_PROPERTIES_2;
        pNext: *void;
        formatProperties: VkFormatProperties;
    }

}

#if OS == .LINUX {
    VkFormatProperties2 :: struct {
        sType: VkStructureType = .FORMAT_PROPERTIES_2;
        pNext: *void;
        formatProperties: VkFormatProperties;
    }

}

#if OS == .WINDOWS {
    VkFramebuffer_T :: struct {}

}

#if OS == .LINUX {
    VkFramebuffer_T :: struct {}

}

#if OS == .WINDOWS {
    VkFramebufferCreateInfo :: struct {
        sType: VkStructureType = .FRAMEBUFFER_CREATE_INFO;
        pNext: *void;
        flags: VkFramebufferCreateFlags;
        renderPass: VkRenderPass;
        attachmentCount: u32;
        pAttachments: *VkImageView;
        width: u32;
        height: u32;
        layers: u32;
    }

}

#if OS == .LINUX {
    VkFramebufferCreateInfo :: struct {
        sType: VkStructureType = .FRAMEBUFFER_CREATE_INFO;
        pNext: *void;
        flags: VkFramebufferCreateFlags;
        renderPass: VkRenderPass;
        attachmentCount: u32;
        pAttachments: *VkImageView;
        width: u32;
        height: u32;
        layers: u32;
    }

}

#if OS == .WINDOWS {
    VkFrontFace :: enum u32 {
        COUNTER_CLOCKWISE :: 0;
        CLOCKWISE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_FRONT_FACE_COUNTER_CLOCKWISE :: COUNTER_CLOCKWISE;
        VK_FRONT_FACE_CLOCKWISE :: CLOCKWISE;
        VK_FRONT_FACE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FRONT_FACE_END_RANGE :: END_RANGE;
        VK_FRONT_FACE_RANGE_SIZE :: RANGE_SIZE;
        VK_FRONT_FACE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkFrontFace :: enum u32 {
        COUNTER_CLOCKWISE :: 0;
        CLOCKWISE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_FRONT_FACE_COUNTER_CLOCKWISE :: COUNTER_CLOCKWISE;
        VK_FRONT_FACE_CLOCKWISE :: CLOCKWISE;
        VK_FRONT_FACE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_FRONT_FACE_END_RANGE :: END_RANGE;
        VK_FRONT_FACE_RANGE_SIZE :: RANGE_SIZE;
        VK_FRONT_FACE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkGeometryAABBNVX :: struct {
        sType: VkStructureType = .GEOMETRY_AABB_NVX;
        pNext: *void;
        aabbData: VkBuffer;
        numAABBs: u32;
        stride: u32;
        offset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkGeometryAABBNVX :: struct {
        sType: VkStructureType = .GEOMETRY_AABB_NVX;
        pNext: *void;
        aabbData: VkBuffer;
        numAABBs: u32;
        stride: u32;
        offset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkGeometryDataNVX :: struct {
        triangles: VkGeometryTrianglesNVX;
        aabbs: VkGeometryAABBNVX;
    }

}

#if OS == .LINUX {
    VkGeometryDataNVX :: struct {
        triangles: VkGeometryTrianglesNVX;
        aabbs: VkGeometryAABBNVX;
    }

}

#if OS == .WINDOWS {
    VkGeometryFlagBitsNVX :: enum_flags u32 {
        OPAQUE_BIT_NVX :: 0x1;
        NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_GEOMETRY_OPAQUE_BIT_NVX :: OPAQUE_BIT_NVX;
        VK_GEOMETRY_NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX :: NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX;
        VK_GEOMETRY_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkGeometryFlagBitsNVX :: enum_flags u32 {
        OPAQUE_BIT_NVX :: 0x1;
        NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_GEOMETRY_OPAQUE_BIT_NVX :: OPAQUE_BIT_NVX;
        VK_GEOMETRY_NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX :: NO_DUPLICATE_ANY_HIT_INVOCATION_BIT_NVX;
        VK_GEOMETRY_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkGeometryInstanceFlagBitsNVX :: enum_flags u32 {
        TRIANGLE_CULL_DISABLE_BIT_NVX :: 0x1;
        TRIANGLE_CULL_FLIP_WINDING_BIT_NVX :: 0x2;
        FORCE_OPAQUE_BIT_NVX :: 0x4;
        FORCE_NO_OPAQUE_BIT_NVX :: 0x8;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_GEOMETRY_INSTANCE_TRIANGLE_CULL_DISABLE_BIT_NVX :: TRIANGLE_CULL_DISABLE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_TRIANGLE_CULL_FLIP_WINDING_BIT_NVX :: TRIANGLE_CULL_FLIP_WINDING_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FORCE_OPAQUE_BIT_NVX :: FORCE_OPAQUE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FORCE_NO_OPAQUE_BIT_NVX :: FORCE_NO_OPAQUE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkGeometryInstanceFlagBitsNVX :: enum_flags u32 {
        TRIANGLE_CULL_DISABLE_BIT_NVX :: 0x1;
        TRIANGLE_CULL_FLIP_WINDING_BIT_NVX :: 0x2;
        FORCE_OPAQUE_BIT_NVX :: 0x4;
        FORCE_NO_OPAQUE_BIT_NVX :: 0x8;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_GEOMETRY_INSTANCE_TRIANGLE_CULL_DISABLE_BIT_NVX :: TRIANGLE_CULL_DISABLE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_TRIANGLE_CULL_FLIP_WINDING_BIT_NVX :: TRIANGLE_CULL_FLIP_WINDING_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FORCE_OPAQUE_BIT_NVX :: FORCE_OPAQUE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FORCE_NO_OPAQUE_BIT_NVX :: FORCE_NO_OPAQUE_BIT_NVX;
        VK_GEOMETRY_INSTANCE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkGeometryNVX :: struct {
        sType: VkStructureType = .GEOMETRY_NVX;
        pNext: *void;
        geometryType: VkGeometryTypeNVX;
        geometry: VkGeometryDataNVX;
        flags: VkGeometryFlagsNVX;
    }

}

#if OS == .LINUX {
    VkGeometryNVX :: struct {
        sType: VkStructureType = .GEOMETRY_NVX;
        pNext: *void;
        geometryType: VkGeometryTypeNVX;
        geometry: VkGeometryDataNVX;
        flags: VkGeometryFlagsNVX;
    }

}

#if OS == .WINDOWS {
    VkGeometryTrianglesNVX :: struct {
        sType: VkStructureType = .GEOMETRY_TRIANGLES_NVX;
        pNext: *void;
        vertexData: VkBuffer;
        vertexOffset: VkDeviceSize;
        vertexCount: u32;
        vertexStride: VkDeviceSize;
        vertexFormat: VkFormat;
        indexData: VkBuffer;
        indexOffset: VkDeviceSize;
        indexCount: u32;
        indexType: VkIndexType;
        transformData: VkBuffer;
        transformOffset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkGeometryTrianglesNVX :: struct {
        sType: VkStructureType = .GEOMETRY_TRIANGLES_NVX;
        pNext: *void;
        vertexData: VkBuffer;
        vertexOffset: VkDeviceSize;
        vertexCount: u32;
        vertexStride: VkDeviceSize;
        vertexFormat: VkFormat;
        indexData: VkBuffer;
        indexOffset: VkDeviceSize;
        indexCount: u32;
        indexType: VkIndexType;
        transformData: VkBuffer;
        transformOffset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkGeometryTypeNVX :: enum u32 {
        TRIANGLES_NVX :: 0;
        AABBS_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_GEOMETRY_TYPE_TRIANGLES_NVX :: TRIANGLES_NVX;
        VK_GEOMETRY_TYPE_AABBS_NVX :: AABBS_NVX;
        VK_GEOMETRY_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_GEOMETRY_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_GEOMETRY_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_GEOMETRY_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkGeometryTypeNVX :: enum u32 {
        TRIANGLES_NVX :: 0;
        AABBS_NVX :: 1;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 1;
        RANGE_SIZE_NVX :: 2;
        MAX_ENUM_NVX :: 2147483647;

        VK_GEOMETRY_TYPE_TRIANGLES_NVX :: TRIANGLES_NVX;
        VK_GEOMETRY_TYPE_AABBS_NVX :: AABBS_NVX;
        VK_GEOMETRY_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_GEOMETRY_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_GEOMETRY_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_GEOMETRY_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkGraphicsPipelineCreateInfo :: struct {
        sType: VkStructureType = .GRAPHICS_PIPELINE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stageCount: u32;
        pStages: *VkPipelineShaderStageCreateInfo;
        pVertexInputState: *VkPipelineVertexInputStateCreateInfo;
        pInputAssemblyState: *VkPipelineInputAssemblyStateCreateInfo;
        pTessellationState: *VkPipelineTessellationStateCreateInfo;
        pViewportState: *VkPipelineViewportStateCreateInfo;
        pRasterizationState: *VkPipelineRasterizationStateCreateInfo;
        pMultisampleState: *VkPipelineMultisampleStateCreateInfo;
        pDepthStencilState: *VkPipelineDepthStencilStateCreateInfo;
        pColorBlendState: *VkPipelineColorBlendStateCreateInfo;
        pDynamicState: *VkPipelineDynamicStateCreateInfo;
        layout: VkPipelineLayout;
        renderPass: VkRenderPass;
        subpass: u32;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .LINUX {
    VkGraphicsPipelineCreateInfo :: struct {
        sType: VkStructureType = .GRAPHICS_PIPELINE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stageCount: u32;
        pStages: *VkPipelineShaderStageCreateInfo;
        pVertexInputState: *VkPipelineVertexInputStateCreateInfo;
        pInputAssemblyState: *VkPipelineInputAssemblyStateCreateInfo;
        pTessellationState: *VkPipelineTessellationStateCreateInfo;
        pViewportState: *VkPipelineViewportStateCreateInfo;
        pRasterizationState: *VkPipelineRasterizationStateCreateInfo;
        pMultisampleState: *VkPipelineMultisampleStateCreateInfo;
        pDepthStencilState: *VkPipelineDepthStencilStateCreateInfo;
        pColorBlendState: *VkPipelineColorBlendStateCreateInfo;
        pDynamicState: *VkPipelineDynamicStateCreateInfo;
        layout: VkPipelineLayout;
        renderPass: VkRenderPass;
        subpass: u32;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .WINDOWS {
    VkHdrMetadataEXT :: struct {
        sType: VkStructureType = .HDR_METADATA_EXT;
        pNext: *void;
        displayPrimaryRed: VkXYColorEXT;
        displayPrimaryGreen: VkXYColorEXT;
        displayPrimaryBlue: VkXYColorEXT;
        whitePoint: VkXYColorEXT;
        maxLuminance: float;
        minLuminance: float;
        maxContentLightLevel: float;
        maxFrameAverageLightLevel: float;
    }

}

#if OS == .LINUX {
    VkHdrMetadataEXT :: struct {
        sType: VkStructureType = .HDR_METADATA_EXT;
        pNext: *void;
        displayPrimaryRed: VkXYColorEXT;
        displayPrimaryGreen: VkXYColorEXT;
        displayPrimaryBlue: VkXYColorEXT;
        whitePoint: VkXYColorEXT;
        maxLuminance: float;
        minLuminance: float;
        maxContentLightLevel: float;
        maxFrameAverageLightLevel: float;
    }

}

#if OS == .WINDOWS {
    VkImage_T :: struct {}

}

#if OS == .LINUX {
    VkImage_T :: struct {}

}

#if OS == .WINDOWS {
    VkImageAspectFlagBits :: enum_flags u32 {
        COLOR_BIT :: 0x1;
        DEPTH_BIT :: 0x2;
        STENCIL_BIT :: 0x4;
        METADATA_BIT :: 0x8;
        PLANE_0_BIT :: 0x10;
        PLANE_1_BIT :: 0x20;
        PLANE_2_BIT :: 0x40;
        PLANE_0_BIT_KHR :: 0x10;
        PLANE_1_BIT_KHR :: 0x20;
        PLANE_2_BIT_KHR :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_ASPECT_COLOR_BIT :: COLOR_BIT;
        VK_IMAGE_ASPECT_DEPTH_BIT :: DEPTH_BIT;
        VK_IMAGE_ASPECT_STENCIL_BIT :: STENCIL_BIT;
        VK_IMAGE_ASPECT_METADATA_BIT :: METADATA_BIT;
        VK_IMAGE_ASPECT_PLANE_0_BIT :: PLANE_0_BIT;
        VK_IMAGE_ASPECT_PLANE_1_BIT :: PLANE_1_BIT;
        VK_IMAGE_ASPECT_PLANE_2_BIT :: PLANE_2_BIT;
        VK_IMAGE_ASPECT_PLANE_0_BIT_KHR :: PLANE_0_BIT_KHR;
        VK_IMAGE_ASPECT_PLANE_1_BIT_KHR :: PLANE_1_BIT_KHR;
        VK_IMAGE_ASPECT_PLANE_2_BIT_KHR :: PLANE_2_BIT_KHR;
        VK_IMAGE_ASPECT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageAspectFlagBits :: enum_flags u32 {
        COLOR_BIT :: 0x1;
        DEPTH_BIT :: 0x2;
        STENCIL_BIT :: 0x4;
        METADATA_BIT :: 0x8;
        PLANE_0_BIT :: 0x10;
        PLANE_1_BIT :: 0x20;
        PLANE_2_BIT :: 0x40;
        PLANE_0_BIT_KHR :: 0x10;
        PLANE_1_BIT_KHR :: 0x20;
        PLANE_2_BIT_KHR :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_ASPECT_COLOR_BIT :: COLOR_BIT;
        VK_IMAGE_ASPECT_DEPTH_BIT :: DEPTH_BIT;
        VK_IMAGE_ASPECT_STENCIL_BIT :: STENCIL_BIT;
        VK_IMAGE_ASPECT_METADATA_BIT :: METADATA_BIT;
        VK_IMAGE_ASPECT_PLANE_0_BIT :: PLANE_0_BIT;
        VK_IMAGE_ASPECT_PLANE_1_BIT :: PLANE_1_BIT;
        VK_IMAGE_ASPECT_PLANE_2_BIT :: PLANE_2_BIT;
        VK_IMAGE_ASPECT_PLANE_0_BIT_KHR :: PLANE_0_BIT_KHR;
        VK_IMAGE_ASPECT_PLANE_1_BIT_KHR :: PLANE_1_BIT_KHR;
        VK_IMAGE_ASPECT_PLANE_2_BIT_KHR :: PLANE_2_BIT_KHR;
        VK_IMAGE_ASPECT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageBlit :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffsets: [2] VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffsets: [2] VkOffset3D;
    }

}

#if OS == .LINUX {
    VkImageBlit :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffsets: [2] VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffsets: [2] VkOffset3D;
    }

}

#if OS == .WINDOWS {
    VkImageCopy :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffset: VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffset: VkOffset3D;
        extent: VkExtent3D;
    }

}

#if OS == .LINUX {
    VkImageCopy :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffset: VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffset: VkOffset3D;
        extent: VkExtent3D;
    }

}

#if OS == .WINDOWS {
    VkImageCreateFlagBits :: enum_flags u32 {
        SPARSE_BINDING_BIT :: 0x1;
        SPARSE_RESIDENCY_BIT :: 0x2;
        SPARSE_ALIASED_BIT :: 0x4;
        MUTABLE_FORMAT_BIT :: 0x8;
        CUBE_COMPATIBLE_BIT :: 0x10;
        ALIAS_BIT :: 0x400;
        SPLIT_INSTANCE_BIND_REGIONS_BIT :: 0x40;
        _2D_ARRAY_COMPATIBLE_BIT :: 0x20;
        BLOCK_TEXEL_VIEW_COMPATIBLE_BIT :: 0x80;
        EXTENDED_USAGE_BIT :: 0x100;
        PROTECTED_BIT :: 0x800;
        DISJOINT_BIT :: 0x200;
        CORNER_SAMPLED_BIT_NV :: 0x2000;
        SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT :: 0x1000;
        SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: 0x40;
        _2D_ARRAY_COMPATIBLE_BIT_KHR :: 0x20;
        BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR :: 0x80;
        EXTENDED_USAGE_BIT_KHR :: 0x100;
        DISJOINT_BIT_KHR :: 0x200;
        ALIAS_BIT_KHR :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_CREATE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_IMAGE_CREATE_SPARSE_RESIDENCY_BIT :: SPARSE_RESIDENCY_BIT;
        VK_IMAGE_CREATE_SPARSE_ALIASED_BIT :: SPARSE_ALIASED_BIT;
        VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT :: MUTABLE_FORMAT_BIT;
        VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT :: CUBE_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_ALIAS_BIT :: ALIAS_BIT;
        VK_IMAGE_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT :: SPLIT_INSTANCE_BIND_REGIONS_BIT;
        VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT :: _2D_ARRAY_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_BLOCK_TEXEL_VIEW_COMPATIBLE_BIT :: BLOCK_TEXEL_VIEW_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_EXTENDED_USAGE_BIT :: EXTENDED_USAGE_BIT;
        VK_IMAGE_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_IMAGE_CREATE_DISJOINT_BIT :: DISJOINT_BIT;
        VK_IMAGE_CREATE_CORNER_SAMPLED_BIT_NV :: CORNER_SAMPLED_BIT_NV;
        VK_IMAGE_CREATE_SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT :: SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT;
        VK_IMAGE_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR;
        VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT_KHR :: _2D_ARRAY_COMPATIBLE_BIT_KHR;
        VK_IMAGE_CREATE_BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR :: BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR;
        VK_IMAGE_CREATE_EXTENDED_USAGE_BIT_KHR :: EXTENDED_USAGE_BIT_KHR;
        VK_IMAGE_CREATE_DISJOINT_BIT_KHR :: DISJOINT_BIT_KHR;
        VK_IMAGE_CREATE_ALIAS_BIT_KHR :: ALIAS_BIT_KHR;
        VK_IMAGE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageCreateFlagBits :: enum_flags u32 {
        SPARSE_BINDING_BIT :: 0x1;
        SPARSE_RESIDENCY_BIT :: 0x2;
        SPARSE_ALIASED_BIT :: 0x4;
        MUTABLE_FORMAT_BIT :: 0x8;
        CUBE_COMPATIBLE_BIT :: 0x10;
        ALIAS_BIT :: 0x400;
        SPLIT_INSTANCE_BIND_REGIONS_BIT :: 0x40;
        _2D_ARRAY_COMPATIBLE_BIT :: 0x20;
        BLOCK_TEXEL_VIEW_COMPATIBLE_BIT :: 0x80;
        EXTENDED_USAGE_BIT :: 0x100;
        PROTECTED_BIT :: 0x800;
        DISJOINT_BIT :: 0x200;
        CORNER_SAMPLED_BIT_NV :: 0x2000;
        SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT :: 0x1000;
        SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: 0x40;
        _2D_ARRAY_COMPATIBLE_BIT_KHR :: 0x20;
        BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR :: 0x80;
        EXTENDED_USAGE_BIT_KHR :: 0x100;
        DISJOINT_BIT_KHR :: 0x200;
        ALIAS_BIT_KHR :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_CREATE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_IMAGE_CREATE_SPARSE_RESIDENCY_BIT :: SPARSE_RESIDENCY_BIT;
        VK_IMAGE_CREATE_SPARSE_ALIASED_BIT :: SPARSE_ALIASED_BIT;
        VK_IMAGE_CREATE_MUTABLE_FORMAT_BIT :: MUTABLE_FORMAT_BIT;
        VK_IMAGE_CREATE_CUBE_COMPATIBLE_BIT :: CUBE_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_ALIAS_BIT :: ALIAS_BIT;
        VK_IMAGE_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT :: SPLIT_INSTANCE_BIND_REGIONS_BIT;
        VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT :: _2D_ARRAY_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_BLOCK_TEXEL_VIEW_COMPATIBLE_BIT :: BLOCK_TEXEL_VIEW_COMPATIBLE_BIT;
        VK_IMAGE_CREATE_EXTENDED_USAGE_BIT :: EXTENDED_USAGE_BIT;
        VK_IMAGE_CREATE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_IMAGE_CREATE_DISJOINT_BIT :: DISJOINT_BIT;
        VK_IMAGE_CREATE_CORNER_SAMPLED_BIT_NV :: CORNER_SAMPLED_BIT_NV;
        VK_IMAGE_CREATE_SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT :: SAMPLE_LOCATIONS_COMPATIBLE_DEPTH_BIT_EXT;
        VK_IMAGE_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR;
        VK_IMAGE_CREATE_2D_ARRAY_COMPATIBLE_BIT_KHR :: _2D_ARRAY_COMPATIBLE_BIT_KHR;
        VK_IMAGE_CREATE_BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR :: BLOCK_TEXEL_VIEW_COMPATIBLE_BIT_KHR;
        VK_IMAGE_CREATE_EXTENDED_USAGE_BIT_KHR :: EXTENDED_USAGE_BIT_KHR;
        VK_IMAGE_CREATE_DISJOINT_BIT_KHR :: DISJOINT_BIT_KHR;
        VK_IMAGE_CREATE_ALIAS_BIT_KHR :: ALIAS_BIT_KHR;
        VK_IMAGE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_CREATE_INFO;
        pNext: *void;
        flags: VkImageCreateFlags;
        imageType: VkImageType;
        format: VkFormat;
        extent: VkExtent3D;
        mipLevels: u32;
        arrayLayers: u32;
        samples: VkSampleCountFlagBits;
        tiling: VkImageTiling;
        usage: VkImageUsageFlags;
        sharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
        initialLayout: VkImageLayout;
    }

}

#if OS == .LINUX {
    VkImageCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_CREATE_INFO;
        pNext: *void;
        flags: VkImageCreateFlags;
        imageType: VkImageType;
        format: VkFormat;
        extent: VkExtent3D;
        mipLevels: u32;
        arrayLayers: u32;
        samples: VkSampleCountFlagBits;
        tiling: VkImageTiling;
        usage: VkImageUsageFlags;
        sharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
        initialLayout: VkImageLayout;
    }

}

#if OS == .WINDOWS {
    VkImageFormatListCreateInfoKHR :: struct {
        sType: VkStructureType = .IMAGE_FORMAT_LIST_CREATE_INFO_KHR;
        pNext: *void;
        viewFormatCount: u32;
        pViewFormats: *VkFormat;
    }

}

#if OS == .LINUX {
    VkImageFormatListCreateInfoKHR :: struct {
        sType: VkStructureType = .IMAGE_FORMAT_LIST_CREATE_INFO_KHR;
        pNext: *void;
        viewFormatCount: u32;
        pViewFormats: *VkFormat;
    }

}

#if OS == .WINDOWS {
    VkImageFormatProperties :: struct {
        maxExtent: VkExtent3D;
        maxMipLevels: u32;
        maxArrayLayers: u32;
        sampleCounts: VkSampleCountFlags;
        maxResourceSize: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkImageFormatProperties :: struct {
        maxExtent: VkExtent3D;
        maxMipLevels: u32;
        maxArrayLayers: u32;
        sampleCounts: VkSampleCountFlags;
        maxResourceSize: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkImageFormatProperties2 :: struct {
        sType: VkStructureType = .IMAGE_FORMAT_PROPERTIES_2;
        pNext: *void;
        imageFormatProperties: VkImageFormatProperties;
    }

}

#if OS == .LINUX {
    VkImageFormatProperties2 :: struct {
        sType: VkStructureType = .IMAGE_FORMAT_PROPERTIES_2;
        pNext: *void;
        imageFormatProperties: VkImageFormatProperties;
    }

}

#if OS == .WINDOWS {
    VkImageLayout :: enum u32 {
        UNDEFINED :: 0;
        GENERAL :: 1;
        COLOR_ATTACHMENT_OPTIMAL :: 2;
        DEPTH_STENCIL_ATTACHMENT_OPTIMAL :: 3;
        DEPTH_STENCIL_READ_ONLY_OPTIMAL :: 4;
        SHADER_READ_ONLY_OPTIMAL :: 5;
        TRANSFER_SRC_OPTIMAL :: 6;
        TRANSFER_DST_OPTIMAL :: 7;
        PREINITIALIZED :: 8;
        DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL :: 1000117000;
        DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL :: 1000117001;
        PRESENT_SRC_KHR :: 1000001002;
        SHARED_PRESENT_KHR :: 1000111000;
        SHADING_RATE_OPTIMAL_NV :: 1000164003;
        DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR :: 1000117000;
        DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR :: 1000117001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 8;
        RANGE_SIZE :: 9;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_LAYOUT_UNDEFINED :: UNDEFINED;
        VK_IMAGE_LAYOUT_GENERAL :: GENERAL;
        VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL :: COLOR_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL :: DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL :: DEPTH_STENCIL_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL :: SHADER_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL :: TRANSFER_SRC_OPTIMAL;
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL :: TRANSFER_DST_OPTIMAL;
        VK_IMAGE_LAYOUT_PREINITIALIZED :: PREINITIALIZED;
        VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL :: DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL :: DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_PRESENT_SRC_KHR :: PRESENT_SRC_KHR;
        VK_IMAGE_LAYOUT_SHARED_PRESENT_KHR :: SHARED_PRESENT_KHR;
        VK_IMAGE_LAYOUT_SHADING_RATE_OPTIMAL_NV :: SHADING_RATE_OPTIMAL_NV;
        VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR :: DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR;
        VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR :: DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR;
        VK_IMAGE_LAYOUT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_LAYOUT_END_RANGE :: END_RANGE;
        VK_IMAGE_LAYOUT_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_LAYOUT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageLayout :: enum u32 {
        UNDEFINED :: 0;
        GENERAL :: 1;
        COLOR_ATTACHMENT_OPTIMAL :: 2;
        DEPTH_STENCIL_ATTACHMENT_OPTIMAL :: 3;
        DEPTH_STENCIL_READ_ONLY_OPTIMAL :: 4;
        SHADER_READ_ONLY_OPTIMAL :: 5;
        TRANSFER_SRC_OPTIMAL :: 6;
        TRANSFER_DST_OPTIMAL :: 7;
        PREINITIALIZED :: 8;
        DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL :: 1000117000;
        DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL :: 1000117001;
        PRESENT_SRC_KHR :: 1000001002;
        SHARED_PRESENT_KHR :: 1000111000;
        SHADING_RATE_OPTIMAL_NV :: 1000164003;
        DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR :: 1000117000;
        DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR :: 1000117001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 8;
        RANGE_SIZE :: 9;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_LAYOUT_UNDEFINED :: UNDEFINED;
        VK_IMAGE_LAYOUT_GENERAL :: GENERAL;
        VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL :: COLOR_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL :: DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_STENCIL_READ_ONLY_OPTIMAL :: DEPTH_STENCIL_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL :: SHADER_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL :: TRANSFER_SRC_OPTIMAL;
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL :: TRANSFER_DST_OPTIMAL;
        VK_IMAGE_LAYOUT_PREINITIALIZED :: PREINITIALIZED;
        VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL :: DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL;
        VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL :: DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL;
        VK_IMAGE_LAYOUT_PRESENT_SRC_KHR :: PRESENT_SRC_KHR;
        VK_IMAGE_LAYOUT_SHARED_PRESENT_KHR :: SHARED_PRESENT_KHR;
        VK_IMAGE_LAYOUT_SHADING_RATE_OPTIMAL_NV :: SHADING_RATE_OPTIMAL_NV;
        VK_IMAGE_LAYOUT_DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR :: DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL_KHR;
        VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR :: DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL_KHR;
        VK_IMAGE_LAYOUT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_LAYOUT_END_RANGE :: END_RANGE;
        VK_IMAGE_LAYOUT_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_LAYOUT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageMemoryBarrier :: struct {
        sType: VkStructureType = .IMAGE_MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        oldLayout: VkImageLayout;
        newLayout: VkImageLayout;
        srcQueueFamilyIndex: u32;
        dstQueueFamilyIndex: u32;
        image: VkImage;
        subresourceRange: VkImageSubresourceRange;
    }

}

#if OS == .LINUX {
    VkImageMemoryBarrier :: struct {
        sType: VkStructureType = .IMAGE_MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        oldLayout: VkImageLayout;
        newLayout: VkImageLayout;
        srcQueueFamilyIndex: u32;
        dstQueueFamilyIndex: u32;
        image: VkImage;
        subresourceRange: VkImageSubresourceRange;
    }

}

#if OS == .WINDOWS {
    VkImageMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .IMAGE_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        image: VkImage;
    }

}

#if OS == .LINUX {
    VkImageMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .IMAGE_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        image: VkImage;
    }

}

#if OS == .WINDOWS {
    VkImagePlaneMemoryRequirementsInfo :: struct {
        sType: VkStructureType = .IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO;
        pNext: *void;
        planeAspect: VkImageAspectFlagBits;
    }

}

#if OS == .LINUX {
    VkImagePlaneMemoryRequirementsInfo :: struct {
        sType: VkStructureType = .IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO;
        pNext: *void;
        planeAspect: VkImageAspectFlagBits;
    }

}

#if OS == .WINDOWS {
    VkImageResolve :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffset: VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffset: VkOffset3D;
        extent: VkExtent3D;
    }

}

#if OS == .LINUX {
    VkImageResolve :: struct {
        srcSubresource: VkImageSubresourceLayers;
        srcOffset: VkOffset3D;
        dstSubresource: VkImageSubresourceLayers;
        dstOffset: VkOffset3D;
        extent: VkExtent3D;
    }

}

#if OS == .WINDOWS {
    VkImageSparseMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        image: VkImage;
    }

}

#if OS == .LINUX {
    VkImageSparseMemoryRequirementsInfo2 :: struct {
        sType: VkStructureType = .IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2;
        pNext: *void;
        image: VkImage;
    }

}

#if OS == .WINDOWS {
    VkImageSubresource :: struct {
        aspectMask: VkImageAspectFlags;
        mipLevel: u32;
        arrayLayer: u32;
    }

}

#if OS == .LINUX {
    VkImageSubresource :: struct {
        aspectMask: VkImageAspectFlags;
        mipLevel: u32;
        arrayLayer: u32;
    }

}

#if OS == .WINDOWS {
    VkImageSubresourceLayers :: struct {
        aspectMask: VkImageAspectFlags;
        mipLevel: u32;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .LINUX {
    VkImageSubresourceLayers :: struct {
        aspectMask: VkImageAspectFlags;
        mipLevel: u32;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .WINDOWS {
    VkImageSubresourceRange :: struct {
        aspectMask: VkImageAspectFlags;
        baseMipLevel: u32;
        levelCount: u32;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .LINUX {
    VkImageSubresourceRange :: struct {
        aspectMask: VkImageAspectFlags;
        baseMipLevel: u32;
        levelCount: u32;
        baseArrayLayer: u32;
        layerCount: u32;
    }

}

#if OS == .WINDOWS {
    VkImageSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .IMAGE_SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
    }

}

#if OS == .LINUX {
    VkImageSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .IMAGE_SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        swapchain: VkSwapchainKHR;
    }

}

#if OS == .WINDOWS {
    VkImageTiling :: enum u32 {
        OPTIMAL :: 0;
        LINEAR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_TILING_OPTIMAL :: OPTIMAL;
        VK_IMAGE_TILING_LINEAR :: LINEAR;
        VK_IMAGE_TILING_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_TILING_END_RANGE :: END_RANGE;
        VK_IMAGE_TILING_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_TILING_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageTiling :: enum u32 {
        OPTIMAL :: 0;
        LINEAR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_TILING_OPTIMAL :: OPTIMAL;
        VK_IMAGE_TILING_LINEAR :: LINEAR;
        VK_IMAGE_TILING_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_TILING_END_RANGE :: END_RANGE;
        VK_IMAGE_TILING_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_TILING_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageType :: enum u32 {
        _1D :: 0;
        _2D :: 1;
        _3D :: 2;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_TYPE_1D :: _1D;
        VK_IMAGE_TYPE_2D :: _2D;
        VK_IMAGE_TYPE_3D :: _3D;
        VK_IMAGE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_TYPE_END_RANGE :: END_RANGE;
        VK_IMAGE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageType :: enum u32 {
        _1D :: 0;
        _2D :: 1;
        _3D :: 2;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_TYPE_1D :: _1D;
        VK_IMAGE_TYPE_2D :: _2D;
        VK_IMAGE_TYPE_3D :: _3D;
        VK_IMAGE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_TYPE_END_RANGE :: END_RANGE;
        VK_IMAGE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageUsageFlagBits :: enum_flags u32 {
        TRANSFER_SRC_BIT :: 0x1;
        TRANSFER_DST_BIT :: 0x2;
        SAMPLED_BIT :: 0x4;
        STORAGE_BIT :: 0x8;
        COLOR_ATTACHMENT_BIT :: 0x10;
        DEPTH_STENCIL_ATTACHMENT_BIT :: 0x20;
        TRANSIENT_ATTACHMENT_BIT :: 0x40;
        INPUT_ATTACHMENT_BIT :: 0x80;
        SHADING_RATE_IMAGE_BIT_NV :: 0x100;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_USAGE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_IMAGE_USAGE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_IMAGE_USAGE_SAMPLED_BIT :: SAMPLED_BIT;
        VK_IMAGE_USAGE_STORAGE_BIT :: STORAGE_BIT;
        VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT :: COLOR_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT :: DEPTH_STENCIL_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_TRANSIENT_ATTACHMENT_BIT :: TRANSIENT_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT :: INPUT_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_SHADING_RATE_IMAGE_BIT_NV :: SHADING_RATE_IMAGE_BIT_NV;
        VK_IMAGE_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageUsageFlagBits :: enum_flags u32 {
        TRANSFER_SRC_BIT :: 0x1;
        TRANSFER_DST_BIT :: 0x2;
        SAMPLED_BIT :: 0x4;
        STORAGE_BIT :: 0x8;
        COLOR_ATTACHMENT_BIT :: 0x10;
        DEPTH_STENCIL_ATTACHMENT_BIT :: 0x20;
        TRANSIENT_ATTACHMENT_BIT :: 0x40;
        INPUT_ATTACHMENT_BIT :: 0x80;
        SHADING_RATE_IMAGE_BIT_NV :: 0x100;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_IMAGE_USAGE_TRANSFER_SRC_BIT :: TRANSFER_SRC_BIT;
        VK_IMAGE_USAGE_TRANSFER_DST_BIT :: TRANSFER_DST_BIT;
        VK_IMAGE_USAGE_SAMPLED_BIT :: SAMPLED_BIT;
        VK_IMAGE_USAGE_STORAGE_BIT :: STORAGE_BIT;
        VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT :: COLOR_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT :: DEPTH_STENCIL_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_TRANSIENT_ATTACHMENT_BIT :: TRANSIENT_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT :: INPUT_ATTACHMENT_BIT;
        VK_IMAGE_USAGE_SHADING_RATE_IMAGE_BIT_NV :: SHADING_RATE_IMAGE_BIT_NV;
        VK_IMAGE_USAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageView_T :: struct {}

}

#if OS == .LINUX {
    VkImageView_T :: struct {}

}

#if OS == .WINDOWS {
    VkImageViewASTCDecodeModeEXT :: struct {
        sType: VkStructureType = .IMAGE_VIEW_ASTC_DECODE_MODE_EXT;
        pNext: *void;
        decodeMode: VkFormat;
    }

}

#if OS == .LINUX {
    VkImageViewASTCDecodeModeEXT :: struct {
        sType: VkStructureType = .IMAGE_VIEW_ASTC_DECODE_MODE_EXT;
        pNext: *void;
        decodeMode: VkFormat;
    }

}

#if OS == .WINDOWS {
    VkImageViewCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_VIEW_CREATE_INFO;
        pNext: *void;
        flags: VkImageViewCreateFlags;
        image: VkImage;
        viewType: VkImageViewType;
        format: VkFormat;
        components: VkComponentMapping;
        subresourceRange: VkImageSubresourceRange;
    }

}

#if OS == .LINUX {
    VkImageViewCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_VIEW_CREATE_INFO;
        pNext: *void;
        flags: VkImageViewCreateFlags;
        image: VkImage;
        viewType: VkImageViewType;
        format: VkFormat;
        components: VkComponentMapping;
        subresourceRange: VkImageSubresourceRange;
    }

}

#if OS == .WINDOWS {
    VkImageViewType :: enum u32 {
        _1D :: 0;
        _2D :: 1;
        _3D :: 2;
        CUBE :: 3;
        _1D_ARRAY :: 4;
        _2D_ARRAY :: 5;
        CUBE_ARRAY :: 6;
        BEGIN_RANGE :: 0;
        END_RANGE :: 6;
        RANGE_SIZE :: 7;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_VIEW_TYPE_1D :: _1D;
        VK_IMAGE_VIEW_TYPE_2D :: _2D;
        VK_IMAGE_VIEW_TYPE_3D :: _3D;
        VK_IMAGE_VIEW_TYPE_CUBE :: CUBE;
        VK_IMAGE_VIEW_TYPE_1D_ARRAY :: _1D_ARRAY;
        VK_IMAGE_VIEW_TYPE_2D_ARRAY :: _2D_ARRAY;
        VK_IMAGE_VIEW_TYPE_CUBE_ARRAY :: CUBE_ARRAY;
        VK_IMAGE_VIEW_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_VIEW_TYPE_END_RANGE :: END_RANGE;
        VK_IMAGE_VIEW_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_VIEW_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkImageViewType :: enum u32 {
        _1D :: 0;
        _2D :: 1;
        _3D :: 2;
        CUBE :: 3;
        _1D_ARRAY :: 4;
        _2D_ARRAY :: 5;
        CUBE_ARRAY :: 6;
        BEGIN_RANGE :: 0;
        END_RANGE :: 6;
        RANGE_SIZE :: 7;
        MAX_ENUM :: 2147483647;

        VK_IMAGE_VIEW_TYPE_1D :: _1D;
        VK_IMAGE_VIEW_TYPE_2D :: _2D;
        VK_IMAGE_VIEW_TYPE_3D :: _3D;
        VK_IMAGE_VIEW_TYPE_CUBE :: CUBE;
        VK_IMAGE_VIEW_TYPE_1D_ARRAY :: _1D_ARRAY;
        VK_IMAGE_VIEW_TYPE_2D_ARRAY :: _2D_ARRAY;
        VK_IMAGE_VIEW_TYPE_CUBE_ARRAY :: CUBE_ARRAY;
        VK_IMAGE_VIEW_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_IMAGE_VIEW_TYPE_END_RANGE :: END_RANGE;
        VK_IMAGE_VIEW_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_IMAGE_VIEW_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkImageViewUsageCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_VIEW_USAGE_CREATE_INFO;
        pNext: *void;
        usage: VkImageUsageFlags;
    }

}

#if OS == .LINUX {
    VkImageViewUsageCreateInfo :: struct {
        sType: VkStructureType = .IMAGE_VIEW_USAGE_CREATE_INFO;
        pNext: *void;
        usage: VkImageUsageFlags;
    }

}

#if OS == .WINDOWS {
    VkImportFenceFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_FENCE_FD_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        flags: VkFenceImportFlags;
        handleType: VkExternalFenceHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .LINUX {
    VkImportFenceFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_FENCE_FD_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        flags: VkFenceImportFlags;
        handleType: VkExternalFenceHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .WINDOWS {
    VkImportFenceWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        fence: VkFence;
        flags: VkFenceImportFlags;
        handleType: VkExternalFenceHandleTypeFlagBits;
        handle: HANDLE;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkImportMemoryFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_FD_INFO_KHR;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .LINUX {
    VkImportMemoryFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_FD_INFO_KHR;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .WINDOWS {
    VkImportMemoryHostPointerInfoEXT :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_HOST_POINTER_INFO_EXT;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
        pHostPointer: *void;
    }

}

#if OS == .LINUX {
    VkImportMemoryHostPointerInfoEXT :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_HOST_POINTER_INFO_EXT;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
        pHostPointer: *void;
    }

}

#if OS == .WINDOWS {
    VkImportMemoryWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
        handle: HANDLE;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkImportMemoryWin32HandleInfoNV :: struct {
        sType: VkStructureType = .IMPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagsNV;
        handle: HANDLE;
    }

}

#if OS == .WINDOWS {
    VkImportSemaphoreFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_SEMAPHORE_FD_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        flags: VkSemaphoreImportFlags;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .LINUX {
    VkImportSemaphoreFdInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_SEMAPHORE_FD_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        flags: VkSemaphoreImportFlags;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
        fd: s32;
    }

}

#if OS == .WINDOWS {
    VkImportSemaphoreWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        flags: VkSemaphoreImportFlags;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
        handle: HANDLE;
        name: *s16;
    }

}

#if OS == .WINDOWS {
    VkIndexType :: enum u32 {
        UINT16 :: 0;
        UINT32 :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_INDEX_TYPE_UINT16 :: UINT16;
        VK_INDEX_TYPE_UINT32 :: UINT32;
        VK_INDEX_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_INDEX_TYPE_END_RANGE :: END_RANGE;
        VK_INDEX_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_INDEX_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkIndexType :: enum u32 {
        UINT16 :: 0;
        UINT32 :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_INDEX_TYPE_UINT16 :: UINT16;
        VK_INDEX_TYPE_UINT32 :: UINT32;
        VK_INDEX_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_INDEX_TYPE_END_RANGE :: END_RANGE;
        VK_INDEX_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_INDEX_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutCreateInfoNVX :: struct {
        sType: VkStructureType = .INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX;
        pNext: *void;
        pipelineBindPoint: VkPipelineBindPoint;
        flags: VkIndirectCommandsLayoutUsageFlagsNVX;
        tokenCount: u32;
        pTokens: *VkIndirectCommandsLayoutTokenNVX;
    }

}

#if OS == .LINUX {
    VkIndirectCommandsLayoutCreateInfoNVX :: struct {
        sType: VkStructureType = .INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX;
        pNext: *void;
        pipelineBindPoint: VkPipelineBindPoint;
        flags: VkIndirectCommandsLayoutUsageFlagsNVX;
        tokenCount: u32;
        pTokens: *VkIndirectCommandsLayoutTokenNVX;
    }

}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutNVX_T :: struct {}

}

#if OS == .LINUX {
    VkIndirectCommandsLayoutNVX_T :: struct {}

}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutTokenNVX :: struct {
        tokenType: VkIndirectCommandsTokenTypeNVX;
        bindingUnit: u32;
        dynamicCount: u32;
        divisor: u32;
    }

}

#if OS == .LINUX {
    VkIndirectCommandsLayoutTokenNVX :: struct {
        tokenType: VkIndirectCommandsTokenTypeNVX;
        bindingUnit: u32;
        dynamicCount: u32;
        divisor: u32;
    }

}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutUsageFlagBitsNVX :: enum_flags u32 {
        UNORDERED_SEQUENCES_BIT_NVX :: 0x1;
        SPARSE_SEQUENCES_BIT_NVX :: 0x2;
        EMPTY_EXECUTIONS_BIT_NVX :: 0x4;
        INDEXED_SEQUENCES_BIT_NVX :: 0x8;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_UNORDERED_SEQUENCES_BIT_NVX :: UNORDERED_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_SPARSE_SEQUENCES_BIT_NVX :: SPARSE_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_EMPTY_EXECUTIONS_BIT_NVX :: EMPTY_EXECUTIONS_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_INDEXED_SEQUENCES_BIT_NVX :: INDEXED_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkIndirectCommandsLayoutUsageFlagBitsNVX :: enum_flags u32 {
        UNORDERED_SEQUENCES_BIT_NVX :: 0x1;
        SPARSE_SEQUENCES_BIT_NVX :: 0x2;
        EMPTY_EXECUTIONS_BIT_NVX :: 0x4;
        INDEXED_SEQUENCES_BIT_NVX :: 0x8;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_UNORDERED_SEQUENCES_BIT_NVX :: UNORDERED_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_SPARSE_SEQUENCES_BIT_NVX :: SPARSE_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_EMPTY_EXECUTIONS_BIT_NVX :: EMPTY_EXECUTIONS_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_INDEXED_SEQUENCES_BIT_NVX :: INDEXED_SEQUENCES_BIT_NVX;
        VK_INDIRECT_COMMANDS_LAYOUT_USAGE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkIndirectCommandsTokenNVX :: struct {
        tokenType: VkIndirectCommandsTokenTypeNVX;
        buffer: VkBuffer;
        offset: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkIndirectCommandsTokenNVX :: struct {
        tokenType: VkIndirectCommandsTokenTypeNVX;
        buffer: VkBuffer;
        offset: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkIndirectCommandsTokenTypeNVX :: enum u32 {
        PIPELINE_NVX :: 0;
        DESCRIPTOR_SET_NVX :: 1;
        INDEX_BUFFER_NVX :: 2;
        VERTEX_BUFFER_NVX :: 3;
        PUSH_CONSTANT_NVX :: 4;
        DRAW_INDEXED_NVX :: 5;
        DRAW_NVX :: 6;
        DISPATCH_NVX :: 7;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 7;
        RANGE_SIZE_NVX :: 8;
        MAX_ENUM_NVX :: 2147483647;

        VK_INDIRECT_COMMANDS_TOKEN_TYPE_PIPELINE_NVX :: PIPELINE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DESCRIPTOR_SET_NVX :: DESCRIPTOR_SET_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_INDEX_BUFFER_NVX :: INDEX_BUFFER_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_VERTEX_BUFFER_NVX :: VERTEX_BUFFER_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_PUSH_CONSTANT_NVX :: PUSH_CONSTANT_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DRAW_INDEXED_NVX :: DRAW_INDEXED_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DRAW_NVX :: DRAW_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DISPATCH_NVX :: DISPATCH_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkIndirectCommandsTokenTypeNVX :: enum u32 {
        PIPELINE_NVX :: 0;
        DESCRIPTOR_SET_NVX :: 1;
        INDEX_BUFFER_NVX :: 2;
        VERTEX_BUFFER_NVX :: 3;
        PUSH_CONSTANT_NVX :: 4;
        DRAW_INDEXED_NVX :: 5;
        DRAW_NVX :: 6;
        DISPATCH_NVX :: 7;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 7;
        RANGE_SIZE_NVX :: 8;
        MAX_ENUM_NVX :: 2147483647;

        VK_INDIRECT_COMMANDS_TOKEN_TYPE_PIPELINE_NVX :: PIPELINE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DESCRIPTOR_SET_NVX :: DESCRIPTOR_SET_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_INDEX_BUFFER_NVX :: INDEX_BUFFER_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_VERTEX_BUFFER_NVX :: VERTEX_BUFFER_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_PUSH_CONSTANT_NVX :: PUSH_CONSTANT_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DRAW_INDEXED_NVX :: DRAW_INDEXED_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DRAW_NVX :: DRAW_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_DISPATCH_NVX :: DISPATCH_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_INDIRECT_COMMANDS_TOKEN_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkInputAttachmentAspectReference :: struct {
        subpass: u32;
        inputAttachmentIndex: u32;
        aspectMask: VkImageAspectFlags;
    }

}

#if OS == .LINUX {
    VkInputAttachmentAspectReference :: struct {
        subpass: u32;
        inputAttachmentIndex: u32;
        aspectMask: VkImageAspectFlags;
    }

}

#if OS == .WINDOWS {
    VkInstance_T :: struct {}

}

#if OS == .LINUX {
    VkInstance_T :: struct {}

}

#if OS == .WINDOWS {
    VkInstanceCreateInfo :: struct {
        sType: VkStructureType = .INSTANCE_CREATE_INFO;
        pNext: *void;
        flags: VkInstanceCreateFlags;
        pApplicationInfo: *VkApplicationInfo;
        enabledLayerCount: u32;
        ppEnabledLayerNames: **u8;
        enabledExtensionCount: u32;
        ppEnabledExtensionNames: **u8;
    }

}

#if OS == .LINUX {
    VkInstanceCreateInfo :: struct {
        sType: VkStructureType = .INSTANCE_CREATE_INFO;
        pNext: *void;
        flags: VkInstanceCreateFlags;
        pApplicationInfo: *VkApplicationInfo;
        enabledLayerCount: u32;
        ppEnabledLayerNames: **u8;
        enabledExtensionCount: u32;
        ppEnabledExtensionNames: **u8;
    }

}

#if OS == .WINDOWS {
    VkInternalAllocationType :: enum u32 {
        EXECUTABLE :: 0;
        BEGIN_RANGE :: 0;
        END_RANGE :: 0;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_INTERNAL_ALLOCATION_TYPE_EXECUTABLE :: EXECUTABLE;
        VK_INTERNAL_ALLOCATION_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_INTERNAL_ALLOCATION_TYPE_END_RANGE :: END_RANGE;
        VK_INTERNAL_ALLOCATION_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_INTERNAL_ALLOCATION_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkInternalAllocationType :: enum u32 {
        EXECUTABLE :: 0;
        BEGIN_RANGE :: 0;
        END_RANGE :: 0;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_INTERNAL_ALLOCATION_TYPE_EXECUTABLE :: EXECUTABLE;
        VK_INTERNAL_ALLOCATION_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_INTERNAL_ALLOCATION_TYPE_END_RANGE :: END_RANGE;
        VK_INTERNAL_ALLOCATION_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_INTERNAL_ALLOCATION_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkLayerProperties :: struct {
        layerName: [256] u8;
        specVersion: u32;
        implementationVersion: u32;
        description: [256] u8;
    }

}

#if OS == .LINUX {
    VkLayerProperties :: struct {
        layerName: [256] u8;
        specVersion: u32;
        implementationVersion: u32;
        description: [256] u8;
    }

}

#if OS == .WINDOWS {
    VkLogicOp :: enum u32 {
        CLEAR :: 0;
        AND :: 1;
        AND_REVERSE :: 2;
        COPY :: 3;
        AND_INVERTED :: 4;
        NO_OP :: 5;
        XOR :: 6;
        OR :: 7;
        NOR :: 8;
        EQUIVALENT :: 9;
        INVERT :: 10;
        OR_REVERSE :: 11;
        COPY_INVERTED :: 12;
        OR_INVERTED :: 13;
        NAND :: 14;
        SET :: 15;
        BEGIN_RANGE :: 0;
        END_RANGE :: 15;
        RANGE_SIZE :: 16;
        MAX_ENUM :: 2147483647;

        VK_LOGIC_OP_CLEAR :: CLEAR;
        VK_LOGIC_OP_AND :: AND;
        VK_LOGIC_OP_AND_REVERSE :: AND_REVERSE;
        VK_LOGIC_OP_COPY :: COPY;
        VK_LOGIC_OP_AND_INVERTED :: AND_INVERTED;
        VK_LOGIC_OP_NO_OP :: NO_OP;
        VK_LOGIC_OP_XOR :: XOR;
        VK_LOGIC_OP_OR :: OR;
        VK_LOGIC_OP_NOR :: NOR;
        VK_LOGIC_OP_EQUIVALENT :: EQUIVALENT;
        VK_LOGIC_OP_INVERT :: INVERT;
        VK_LOGIC_OP_OR_REVERSE :: OR_REVERSE;
        VK_LOGIC_OP_COPY_INVERTED :: COPY_INVERTED;
        VK_LOGIC_OP_OR_INVERTED :: OR_INVERTED;
        VK_LOGIC_OP_NAND :: NAND;
        VK_LOGIC_OP_SET :: SET;
        VK_LOGIC_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_LOGIC_OP_END_RANGE :: END_RANGE;
        VK_LOGIC_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_LOGIC_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkLogicOp :: enum u32 {
        CLEAR :: 0;
        AND :: 1;
        AND_REVERSE :: 2;
        COPY :: 3;
        AND_INVERTED :: 4;
        NO_OP :: 5;
        XOR :: 6;
        OR :: 7;
        NOR :: 8;
        EQUIVALENT :: 9;
        INVERT :: 10;
        OR_REVERSE :: 11;
        COPY_INVERTED :: 12;
        OR_INVERTED :: 13;
        NAND :: 14;
        SET :: 15;
        BEGIN_RANGE :: 0;
        END_RANGE :: 15;
        RANGE_SIZE :: 16;
        MAX_ENUM :: 2147483647;

        VK_LOGIC_OP_CLEAR :: CLEAR;
        VK_LOGIC_OP_AND :: AND;
        VK_LOGIC_OP_AND_REVERSE :: AND_REVERSE;
        VK_LOGIC_OP_COPY :: COPY;
        VK_LOGIC_OP_AND_INVERTED :: AND_INVERTED;
        VK_LOGIC_OP_NO_OP :: NO_OP;
        VK_LOGIC_OP_XOR :: XOR;
        VK_LOGIC_OP_OR :: OR;
        VK_LOGIC_OP_NOR :: NOR;
        VK_LOGIC_OP_EQUIVALENT :: EQUIVALENT;
        VK_LOGIC_OP_INVERT :: INVERT;
        VK_LOGIC_OP_OR_REVERSE :: OR_REVERSE;
        VK_LOGIC_OP_COPY_INVERTED :: COPY_INVERTED;
        VK_LOGIC_OP_OR_INVERTED :: OR_INVERTED;
        VK_LOGIC_OP_NAND :: NAND;
        VK_LOGIC_OP_SET :: SET;
        VK_LOGIC_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_LOGIC_OP_END_RANGE :: END_RANGE;
        VK_LOGIC_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_LOGIC_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkMappedMemoryRange :: struct {
        sType: VkStructureType = .MAPPED_MEMORY_RANGE;
        pNext: *void;
        memory: VkDeviceMemory;
        offset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkMappedMemoryRange :: struct {
        sType: VkStructureType = .MAPPED_MEMORY_RANGE;
        pNext: *void;
        memory: VkDeviceMemory;
        offset: VkDeviceSize;
        size: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkMemoryAllocateFlagBits :: enum_flags u32 {
        DEVICE_MASK_BIT :: 0x1;
        DEVICE_MASK_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_ALLOCATE_DEVICE_MASK_BIT :: DEVICE_MASK_BIT;
        VK_MEMORY_ALLOCATE_DEVICE_MASK_BIT_KHR :: DEVICE_MASK_BIT_KHR;
        VK_MEMORY_ALLOCATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkMemoryAllocateFlagBits :: enum_flags u32 {
        DEVICE_MASK_BIT :: 0x1;
        DEVICE_MASK_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_ALLOCATE_DEVICE_MASK_BIT :: DEVICE_MASK_BIT;
        VK_MEMORY_ALLOCATE_DEVICE_MASK_BIT_KHR :: DEVICE_MASK_BIT_KHR;
        VK_MEMORY_ALLOCATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkMemoryAllocateFlagsInfo :: struct {
        sType: VkStructureType = .MEMORY_ALLOCATE_FLAGS_INFO;
        pNext: *void;
        flags: VkMemoryAllocateFlags;
        deviceMask: u32;
    }

}

#if OS == .LINUX {
    VkMemoryAllocateFlagsInfo :: struct {
        sType: VkStructureType = .MEMORY_ALLOCATE_FLAGS_INFO;
        pNext: *void;
        flags: VkMemoryAllocateFlags;
        deviceMask: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryAllocateInfo :: struct {
        sType: VkStructureType = .MEMORY_ALLOCATE_INFO;
        pNext: *void;
        allocationSize: VkDeviceSize;
        memoryTypeIndex: u32;
    }

}

#if OS == .LINUX {
    VkMemoryAllocateInfo :: struct {
        sType: VkStructureType = .MEMORY_ALLOCATE_INFO;
        pNext: *void;
        allocationSize: VkDeviceSize;
        memoryTypeIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryBarrier :: struct {
        sType: VkStructureType = .MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
    }

}

#if OS == .LINUX {
    VkMemoryBarrier :: struct {
        sType: VkStructureType = .MEMORY_BARRIER;
        pNext: *void;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
    }

}

#if OS == .WINDOWS {
    VkMemoryDedicatedAllocateInfo :: struct {
        sType: VkStructureType = .MEMORY_DEDICATED_ALLOCATE_INFO;
        pNext: *void;
        image: VkImage;
        buffer: VkBuffer;
    }

}

#if OS == .LINUX {
    VkMemoryDedicatedAllocateInfo :: struct {
        sType: VkStructureType = .MEMORY_DEDICATED_ALLOCATE_INFO;
        pNext: *void;
        image: VkImage;
        buffer: VkBuffer;
    }

}

#if OS == .WINDOWS {
    VkMemoryDedicatedRequirements :: struct {
        sType: VkStructureType = .MEMORY_DEDICATED_REQUIREMENTS;
        pNext: *void;
        prefersDedicatedAllocation: VkBool32;
        requiresDedicatedAllocation: VkBool32;
    }

}

#if OS == .LINUX {
    VkMemoryDedicatedRequirements :: struct {
        sType: VkStructureType = .MEMORY_DEDICATED_REQUIREMENTS;
        pNext: *void;
        prefersDedicatedAllocation: VkBool32;
        requiresDedicatedAllocation: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkMemoryFdPropertiesKHR :: struct {
        sType: VkStructureType = .MEMORY_FD_PROPERTIES_KHR;
        pNext: *void;
        memoryTypeBits: u32;
    }

}

#if OS == .LINUX {
    VkMemoryFdPropertiesKHR :: struct {
        sType: VkStructureType = .MEMORY_FD_PROPERTIES_KHR;
        pNext: *void;
        memoryTypeBits: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryGetFdInfoKHR :: struct {
        sType: VkStructureType = .MEMORY_GET_FD_INFO_KHR;
        pNext: *void;
        memory: VkDeviceMemory;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkMemoryGetFdInfoKHR :: struct {
        sType: VkStructureType = .MEMORY_GET_FD_INFO_KHR;
        pNext: *void;
        memory: VkDeviceMemory;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkMemoryGetWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .MEMORY_GET_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        memory: VkDeviceMemory;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkMemoryHeap :: struct {
        size: VkDeviceSize;
        flags: VkMemoryHeapFlags;
    }

}

#if OS == .LINUX {
    VkMemoryHeap :: struct {
        size: VkDeviceSize;
        flags: VkMemoryHeapFlags;
    }

}

#if OS == .WINDOWS {
    VkMemoryHeapFlagBits :: enum_flags u32 {
        DEVICE_LOCAL_BIT :: 0x1;
        MULTI_INSTANCE_BIT :: 0x2;
        MULTI_INSTANCE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_HEAP_DEVICE_LOCAL_BIT :: DEVICE_LOCAL_BIT;
        VK_MEMORY_HEAP_MULTI_INSTANCE_BIT :: MULTI_INSTANCE_BIT;
        VK_MEMORY_HEAP_MULTI_INSTANCE_BIT_KHR :: MULTI_INSTANCE_BIT_KHR;
        VK_MEMORY_HEAP_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkMemoryHeapFlagBits :: enum_flags u32 {
        DEVICE_LOCAL_BIT :: 0x1;
        MULTI_INSTANCE_BIT :: 0x2;
        MULTI_INSTANCE_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_HEAP_DEVICE_LOCAL_BIT :: DEVICE_LOCAL_BIT;
        VK_MEMORY_HEAP_MULTI_INSTANCE_BIT :: MULTI_INSTANCE_BIT;
        VK_MEMORY_HEAP_MULTI_INSTANCE_BIT_KHR :: MULTI_INSTANCE_BIT_KHR;
        VK_MEMORY_HEAP_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkMemoryHostPointerPropertiesEXT :: struct {
        sType: VkStructureType = .MEMORY_HOST_POINTER_PROPERTIES_EXT;
        pNext: *void;
        memoryTypeBits: u32;
    }

}

#if OS == .LINUX {
    VkMemoryHostPointerPropertiesEXT :: struct {
        sType: VkStructureType = .MEMORY_HOST_POINTER_PROPERTIES_EXT;
        pNext: *void;
        memoryTypeBits: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryPropertyFlagBits :: enum_flags u32 {
        DEVICE_LOCAL_BIT :: 0x1;
        HOST_VISIBLE_BIT :: 0x2;
        HOST_COHERENT_BIT :: 0x4;
        HOST_CACHED_BIT :: 0x8;
        LAZILY_ALLOCATED_BIT :: 0x10;
        PROTECTED_BIT :: 0x20;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT :: DEVICE_LOCAL_BIT;
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT :: HOST_VISIBLE_BIT;
        VK_MEMORY_PROPERTY_HOST_COHERENT_BIT :: HOST_COHERENT_BIT;
        VK_MEMORY_PROPERTY_HOST_CACHED_BIT :: HOST_CACHED_BIT;
        VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT :: LAZILY_ALLOCATED_BIT;
        VK_MEMORY_PROPERTY_PROTECTED_BIT :: PROTECTED_BIT;
        VK_MEMORY_PROPERTY_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkMemoryPropertyFlagBits :: enum_flags u32 {
        DEVICE_LOCAL_BIT :: 0x1;
        HOST_VISIBLE_BIT :: 0x2;
        HOST_COHERENT_BIT :: 0x4;
        HOST_CACHED_BIT :: 0x8;
        LAZILY_ALLOCATED_BIT :: 0x10;
        PROTECTED_BIT :: 0x20;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT :: DEVICE_LOCAL_BIT;
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT :: HOST_VISIBLE_BIT;
        VK_MEMORY_PROPERTY_HOST_COHERENT_BIT :: HOST_COHERENT_BIT;
        VK_MEMORY_PROPERTY_HOST_CACHED_BIT :: HOST_CACHED_BIT;
        VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT :: LAZILY_ALLOCATED_BIT;
        VK_MEMORY_PROPERTY_PROTECTED_BIT :: PROTECTED_BIT;
        VK_MEMORY_PROPERTY_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkMemoryRequirements :: struct {
        size: VkDeviceSize;
        alignment: VkDeviceSize;
        memoryTypeBits: u32;
    }

}

#if OS == .LINUX {
    VkMemoryRequirements :: struct {
        size: VkDeviceSize;
        alignment: VkDeviceSize;
        memoryTypeBits: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryRequirements2 :: struct {
        sType: VkStructureType = .MEMORY_REQUIREMENTS_2;
        pNext: *void;
        memoryRequirements: VkMemoryRequirements;
    }

}

#if OS == .LINUX {
    VkMemoryRequirements2 :: struct {
        sType: VkStructureType = .MEMORY_REQUIREMENTS_2;
        pNext: *void;
        memoryRequirements: VkMemoryRequirements;
    }

}

#if OS == .WINDOWS {
    VkMemoryType :: struct {
        propertyFlags: VkMemoryPropertyFlags;
        heapIndex: u32;
    }

}

#if OS == .LINUX {
    VkMemoryType :: struct {
        propertyFlags: VkMemoryPropertyFlags;
        heapIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkMemoryWin32HandlePropertiesKHR :: struct {
        sType: VkStructureType = .MEMORY_WIN32_HANDLE_PROPERTIES_KHR;
        pNext: *void;
        memoryTypeBits: u32;
    }

}

#if OS == .WINDOWS {
    VkMultisamplePropertiesEXT :: struct {
        sType: VkStructureType = .MULTISAMPLE_PROPERTIES_EXT;
        pNext: *void;
        maxSampleLocationGridSize: VkExtent2D;
    }

}

#if OS == .LINUX {
    VkMultisamplePropertiesEXT :: struct {
        sType: VkStructureType = .MULTISAMPLE_PROPERTIES_EXT;
        pNext: *void;
        maxSampleLocationGridSize: VkExtent2D;
    }

}

#if OS == .WINDOWS {
    VkObjectEntryTypeNVX :: enum u32 {
        DESCRIPTOR_SET_NVX :: 0;
        PIPELINE_NVX :: 1;
        INDEX_BUFFER_NVX :: 2;
        VERTEX_BUFFER_NVX :: 3;
        PUSH_CONSTANT_NVX :: 4;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 4;
        RANGE_SIZE_NVX :: 5;
        MAX_ENUM_NVX :: 2147483647;

        VK_OBJECT_ENTRY_TYPE_DESCRIPTOR_SET_NVX :: DESCRIPTOR_SET_NVX;
        VK_OBJECT_ENTRY_TYPE_PIPELINE_NVX :: PIPELINE_NVX;
        VK_OBJECT_ENTRY_TYPE_INDEX_BUFFER_NVX :: INDEX_BUFFER_NVX;
        VK_OBJECT_ENTRY_TYPE_VERTEX_BUFFER_NVX :: VERTEX_BUFFER_NVX;
        VK_OBJECT_ENTRY_TYPE_PUSH_CONSTANT_NVX :: PUSH_CONSTANT_NVX;
        VK_OBJECT_ENTRY_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_OBJECT_ENTRY_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_OBJECT_ENTRY_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_OBJECT_ENTRY_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkObjectEntryTypeNVX :: enum u32 {
        DESCRIPTOR_SET_NVX :: 0;
        PIPELINE_NVX :: 1;
        INDEX_BUFFER_NVX :: 2;
        VERTEX_BUFFER_NVX :: 3;
        PUSH_CONSTANT_NVX :: 4;
        BEGIN_RANGE_NVX :: 0;
        END_RANGE_NVX :: 4;
        RANGE_SIZE_NVX :: 5;
        MAX_ENUM_NVX :: 2147483647;

        VK_OBJECT_ENTRY_TYPE_DESCRIPTOR_SET_NVX :: DESCRIPTOR_SET_NVX;
        VK_OBJECT_ENTRY_TYPE_PIPELINE_NVX :: PIPELINE_NVX;
        VK_OBJECT_ENTRY_TYPE_INDEX_BUFFER_NVX :: INDEX_BUFFER_NVX;
        VK_OBJECT_ENTRY_TYPE_VERTEX_BUFFER_NVX :: VERTEX_BUFFER_NVX;
        VK_OBJECT_ENTRY_TYPE_PUSH_CONSTANT_NVX :: PUSH_CONSTANT_NVX;
        VK_OBJECT_ENTRY_TYPE_BEGIN_RANGE_NVX :: BEGIN_RANGE_NVX;
        VK_OBJECT_ENTRY_TYPE_END_RANGE_NVX :: END_RANGE_NVX;
        VK_OBJECT_ENTRY_TYPE_RANGE_SIZE_NVX :: RANGE_SIZE_NVX;
        VK_OBJECT_ENTRY_TYPE_MAX_ENUM_NVX :: MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkObjectEntryUsageFlagBitsNVX :: enum_flags u32 {
        GRAPHICS_BIT_NVX :: 0x1;
        COMPUTE_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_OBJECT_ENTRY_USAGE_GRAPHICS_BIT_NVX :: GRAPHICS_BIT_NVX;
        VK_OBJECT_ENTRY_USAGE_COMPUTE_BIT_NVX :: COMPUTE_BIT_NVX;
        VK_OBJECT_ENTRY_USAGE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .LINUX {
    VkObjectEntryUsageFlagBitsNVX :: enum_flags u32 {
        GRAPHICS_BIT_NVX :: 0x1;
        COMPUTE_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM_NVX :: 0x7fffffff;

        VK_OBJECT_ENTRY_USAGE_GRAPHICS_BIT_NVX :: GRAPHICS_BIT_NVX;
        VK_OBJECT_ENTRY_USAGE_COMPUTE_BIT_NVX :: COMPUTE_BIT_NVX;
        VK_OBJECT_ENTRY_USAGE_FLAG_BITS_MAX_ENUM_NVX :: FLAG_BITS_MAX_ENUM_NVX;
    }

}

#if OS == .WINDOWS {
    VkObjectTableCreateInfoNVX :: struct {
        sType: VkStructureType = .OBJECT_TABLE_CREATE_INFO_NVX;
        pNext: *void;
        objectCount: u32;
        pObjectEntryTypes: *VkObjectEntryTypeNVX;
        pObjectEntryCounts: *u32;
        pObjectEntryUsageFlags: *VkObjectEntryUsageFlagsNVX;
        maxUniformBuffersPerDescriptor: u32;
        maxStorageBuffersPerDescriptor: u32;
        maxStorageImagesPerDescriptor: u32;
        maxSampledImagesPerDescriptor: u32;
        maxPipelineLayouts: u32;
    }

}

#if OS == .LINUX {
    VkObjectTableCreateInfoNVX :: struct {
        sType: VkStructureType = .OBJECT_TABLE_CREATE_INFO_NVX;
        pNext: *void;
        objectCount: u32;
        pObjectEntryTypes: *VkObjectEntryTypeNVX;
        pObjectEntryCounts: *u32;
        pObjectEntryUsageFlags: *VkObjectEntryUsageFlagsNVX;
        maxUniformBuffersPerDescriptor: u32;
        maxStorageBuffersPerDescriptor: u32;
        maxStorageImagesPerDescriptor: u32;
        maxSampledImagesPerDescriptor: u32;
        maxPipelineLayouts: u32;
    }

}

#if OS == .WINDOWS {
    VkObjectTableDescriptorSetEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipelineLayout: VkPipelineLayout;
        descriptorSet: VkDescriptorSet;
    }

}

#if OS == .LINUX {
    VkObjectTableDescriptorSetEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipelineLayout: VkPipelineLayout;
        descriptorSet: VkDescriptorSet;
    }

}

#if OS == .WINDOWS {
    VkObjectTableEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
    }

}

#if OS == .LINUX {
    VkObjectTableEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
    }

}

#if OS == .WINDOWS {
    VkObjectTableIndexBufferEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        buffer: VkBuffer;
        indexType: VkIndexType;
    }

}

#if OS == .LINUX {
    VkObjectTableIndexBufferEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        buffer: VkBuffer;
        indexType: VkIndexType;
    }

}

#if OS == .WINDOWS {
    VkObjectTableNVX_T :: struct {}

}

#if OS == .LINUX {
    VkObjectTableNVX_T :: struct {}

}

#if OS == .WINDOWS {
    VkObjectTablePipelineEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipeline: VkPipeline;
    }

}

#if OS == .LINUX {
    VkObjectTablePipelineEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipeline: VkPipeline;
    }

}

#if OS == .WINDOWS {
    VkObjectTablePushConstantEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipelineLayout: VkPipelineLayout;
        stageFlags: VkShaderStageFlags;
    }

}

#if OS == .LINUX {
    VkObjectTablePushConstantEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        pipelineLayout: VkPipelineLayout;
        stageFlags: VkShaderStageFlags;
    }

}

#if OS == .WINDOWS {
    VkObjectTableVertexBufferEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        buffer: VkBuffer;
    }

}

#if OS == .LINUX {
    VkObjectTableVertexBufferEntryNVX :: struct {
        type: VkObjectEntryTypeNVX;
        flags: VkObjectEntryUsageFlagsNVX;
        buffer: VkBuffer;
    }

}

#if OS == .WINDOWS {
    VkObjectType :: enum u32 {
        UNKNOWN :: 0;
        INSTANCE :: 1;
        PHYSICAL_DEVICE :: 2;
        DEVICE :: 3;
        QUEUE :: 4;
        SEMAPHORE :: 5;
        COMMAND_BUFFER :: 6;
        FENCE :: 7;
        DEVICE_MEMORY :: 8;
        BUFFER :: 9;
        IMAGE :: 10;
        EVENT :: 11;
        QUERY_POOL :: 12;
        BUFFER_VIEW :: 13;
        IMAGE_VIEW :: 14;
        SHADER_MODULE :: 15;
        PIPELINE_CACHE :: 16;
        PIPELINE_LAYOUT :: 17;
        RENDER_PASS :: 18;
        PIPELINE :: 19;
        DESCRIPTOR_SET_LAYOUT :: 20;
        SAMPLER :: 21;
        DESCRIPTOR_POOL :: 22;
        DESCRIPTOR_SET :: 23;
        FRAMEBUFFER :: 24;
        COMMAND_POOL :: 25;
        SAMPLER_YCBCR_CONVERSION :: 1000156000;
        DESCRIPTOR_UPDATE_TEMPLATE :: 1000085000;
        SURFACE_KHR :: 1000000000;
        SWAPCHAIN_KHR :: 1000001000;
        DISPLAY_KHR :: 1000002000;
        DISPLAY_MODE_KHR :: 1000002001;
        DEBUG_REPORT_CALLBACK_EXT :: 1000011000;
        OBJECT_TABLE_NVX :: 1000086000;
        INDIRECT_COMMANDS_LAYOUT_NVX :: 1000086001;
        DEBUG_UTILS_MESSENGER_EXT :: 1000128000;
        VALIDATION_CACHE_EXT :: 1000160000;
        ACCELERATION_STRUCTURE_NVX :: 1000165000;
        DESCRIPTOR_UPDATE_TEMPLATE_KHR :: 1000085000;
        SAMPLER_YCBCR_CONVERSION_KHR :: 1000156000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 25;
        RANGE_SIZE :: 26;
        MAX_ENUM :: 2147483647;

        VK_OBJECT_TYPE_UNKNOWN :: UNKNOWN;
        VK_OBJECT_TYPE_INSTANCE :: INSTANCE;
        VK_OBJECT_TYPE_PHYSICAL_DEVICE :: PHYSICAL_DEVICE;
        VK_OBJECT_TYPE_DEVICE :: DEVICE;
        VK_OBJECT_TYPE_QUEUE :: QUEUE;
        VK_OBJECT_TYPE_SEMAPHORE :: SEMAPHORE;
        VK_OBJECT_TYPE_COMMAND_BUFFER :: COMMAND_BUFFER;
        VK_OBJECT_TYPE_FENCE :: FENCE;
        VK_OBJECT_TYPE_DEVICE_MEMORY :: DEVICE_MEMORY;
        VK_OBJECT_TYPE_BUFFER :: BUFFER;
        VK_OBJECT_TYPE_IMAGE :: IMAGE;
        VK_OBJECT_TYPE_EVENT :: EVENT;
        VK_OBJECT_TYPE_QUERY_POOL :: QUERY_POOL;
        VK_OBJECT_TYPE_BUFFER_VIEW :: BUFFER_VIEW;
        VK_OBJECT_TYPE_IMAGE_VIEW :: IMAGE_VIEW;
        VK_OBJECT_TYPE_SHADER_MODULE :: SHADER_MODULE;
        VK_OBJECT_TYPE_PIPELINE_CACHE :: PIPELINE_CACHE;
        VK_OBJECT_TYPE_PIPELINE_LAYOUT :: PIPELINE_LAYOUT;
        VK_OBJECT_TYPE_RENDER_PASS :: RENDER_PASS;
        VK_OBJECT_TYPE_PIPELINE :: PIPELINE;
        VK_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT :: DESCRIPTOR_SET_LAYOUT;
        VK_OBJECT_TYPE_SAMPLER :: SAMPLER;
        VK_OBJECT_TYPE_DESCRIPTOR_POOL :: DESCRIPTOR_POOL;
        VK_OBJECT_TYPE_DESCRIPTOR_SET :: DESCRIPTOR_SET;
        VK_OBJECT_TYPE_FRAMEBUFFER :: FRAMEBUFFER;
        VK_OBJECT_TYPE_COMMAND_POOL :: COMMAND_POOL;
        VK_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION :: SAMPLER_YCBCR_CONVERSION;
        VK_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE :: DESCRIPTOR_UPDATE_TEMPLATE;
        VK_OBJECT_TYPE_SURFACE_KHR :: SURFACE_KHR;
        VK_OBJECT_TYPE_SWAPCHAIN_KHR :: SWAPCHAIN_KHR;
        VK_OBJECT_TYPE_DISPLAY_KHR :: DISPLAY_KHR;
        VK_OBJECT_TYPE_DISPLAY_MODE_KHR :: DISPLAY_MODE_KHR;
        VK_OBJECT_TYPE_DEBUG_REPORT_CALLBACK_EXT :: DEBUG_REPORT_CALLBACK_EXT;
        VK_OBJECT_TYPE_OBJECT_TABLE_NVX :: OBJECT_TABLE_NVX;
        VK_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX :: INDIRECT_COMMANDS_LAYOUT_NVX;
        VK_OBJECT_TYPE_DEBUG_UTILS_MESSENGER_EXT :: DEBUG_UTILS_MESSENGER_EXT;
        VK_OBJECT_TYPE_VALIDATION_CACHE_EXT :: VALIDATION_CACHE_EXT;
        VK_OBJECT_TYPE_ACCELERATION_STRUCTURE_NVX :: ACCELERATION_STRUCTURE_NVX;
        VK_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_KHR :: DESCRIPTOR_UPDATE_TEMPLATE_KHR;
        VK_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_KHR :: SAMPLER_YCBCR_CONVERSION_KHR;
        VK_OBJECT_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_OBJECT_TYPE_END_RANGE :: END_RANGE;
        VK_OBJECT_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_OBJECT_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkObjectType :: enum u32 {
        UNKNOWN :: 0;
        INSTANCE :: 1;
        PHYSICAL_DEVICE :: 2;
        DEVICE :: 3;
        QUEUE :: 4;
        SEMAPHORE :: 5;
        COMMAND_BUFFER :: 6;
        FENCE :: 7;
        DEVICE_MEMORY :: 8;
        BUFFER :: 9;
        IMAGE :: 10;
        EVENT :: 11;
        QUERY_POOL :: 12;
        BUFFER_VIEW :: 13;
        IMAGE_VIEW :: 14;
        SHADER_MODULE :: 15;
        PIPELINE_CACHE :: 16;
        PIPELINE_LAYOUT :: 17;
        RENDER_PASS :: 18;
        PIPELINE :: 19;
        DESCRIPTOR_SET_LAYOUT :: 20;
        SAMPLER :: 21;
        DESCRIPTOR_POOL :: 22;
        DESCRIPTOR_SET :: 23;
        FRAMEBUFFER :: 24;
        COMMAND_POOL :: 25;
        SAMPLER_YCBCR_CONVERSION :: 1000156000;
        DESCRIPTOR_UPDATE_TEMPLATE :: 1000085000;
        SURFACE_KHR :: 1000000000;
        SWAPCHAIN_KHR :: 1000001000;
        DISPLAY_KHR :: 1000002000;
        DISPLAY_MODE_KHR :: 1000002001;
        DEBUG_REPORT_CALLBACK_EXT :: 1000011000;
        OBJECT_TABLE_NVX :: 1000086000;
        INDIRECT_COMMANDS_LAYOUT_NVX :: 1000086001;
        DEBUG_UTILS_MESSENGER_EXT :: 1000128000;
        VALIDATION_CACHE_EXT :: 1000160000;
        ACCELERATION_STRUCTURE_NVX :: 1000165000;
        DESCRIPTOR_UPDATE_TEMPLATE_KHR :: 1000085000;
        SAMPLER_YCBCR_CONVERSION_KHR :: 1000156000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 25;
        RANGE_SIZE :: 26;
        MAX_ENUM :: 2147483647;

        VK_OBJECT_TYPE_UNKNOWN :: UNKNOWN;
        VK_OBJECT_TYPE_INSTANCE :: INSTANCE;
        VK_OBJECT_TYPE_PHYSICAL_DEVICE :: PHYSICAL_DEVICE;
        VK_OBJECT_TYPE_DEVICE :: DEVICE;
        VK_OBJECT_TYPE_QUEUE :: QUEUE;
        VK_OBJECT_TYPE_SEMAPHORE :: SEMAPHORE;
        VK_OBJECT_TYPE_COMMAND_BUFFER :: COMMAND_BUFFER;
        VK_OBJECT_TYPE_FENCE :: FENCE;
        VK_OBJECT_TYPE_DEVICE_MEMORY :: DEVICE_MEMORY;
        VK_OBJECT_TYPE_BUFFER :: BUFFER;
        VK_OBJECT_TYPE_IMAGE :: IMAGE;
        VK_OBJECT_TYPE_EVENT :: EVENT;
        VK_OBJECT_TYPE_QUERY_POOL :: QUERY_POOL;
        VK_OBJECT_TYPE_BUFFER_VIEW :: BUFFER_VIEW;
        VK_OBJECT_TYPE_IMAGE_VIEW :: IMAGE_VIEW;
        VK_OBJECT_TYPE_SHADER_MODULE :: SHADER_MODULE;
        VK_OBJECT_TYPE_PIPELINE_CACHE :: PIPELINE_CACHE;
        VK_OBJECT_TYPE_PIPELINE_LAYOUT :: PIPELINE_LAYOUT;
        VK_OBJECT_TYPE_RENDER_PASS :: RENDER_PASS;
        VK_OBJECT_TYPE_PIPELINE :: PIPELINE;
        VK_OBJECT_TYPE_DESCRIPTOR_SET_LAYOUT :: DESCRIPTOR_SET_LAYOUT;
        VK_OBJECT_TYPE_SAMPLER :: SAMPLER;
        VK_OBJECT_TYPE_DESCRIPTOR_POOL :: DESCRIPTOR_POOL;
        VK_OBJECT_TYPE_DESCRIPTOR_SET :: DESCRIPTOR_SET;
        VK_OBJECT_TYPE_FRAMEBUFFER :: FRAMEBUFFER;
        VK_OBJECT_TYPE_COMMAND_POOL :: COMMAND_POOL;
        VK_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION :: SAMPLER_YCBCR_CONVERSION;
        VK_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE :: DESCRIPTOR_UPDATE_TEMPLATE;
        VK_OBJECT_TYPE_SURFACE_KHR :: SURFACE_KHR;
        VK_OBJECT_TYPE_SWAPCHAIN_KHR :: SWAPCHAIN_KHR;
        VK_OBJECT_TYPE_DISPLAY_KHR :: DISPLAY_KHR;
        VK_OBJECT_TYPE_DISPLAY_MODE_KHR :: DISPLAY_MODE_KHR;
        VK_OBJECT_TYPE_DEBUG_REPORT_CALLBACK_EXT :: DEBUG_REPORT_CALLBACK_EXT;
        VK_OBJECT_TYPE_OBJECT_TABLE_NVX :: OBJECT_TABLE_NVX;
        VK_OBJECT_TYPE_INDIRECT_COMMANDS_LAYOUT_NVX :: INDIRECT_COMMANDS_LAYOUT_NVX;
        VK_OBJECT_TYPE_DEBUG_UTILS_MESSENGER_EXT :: DEBUG_UTILS_MESSENGER_EXT;
        VK_OBJECT_TYPE_VALIDATION_CACHE_EXT :: VALIDATION_CACHE_EXT;
        VK_OBJECT_TYPE_ACCELERATION_STRUCTURE_NVX :: ACCELERATION_STRUCTURE_NVX;
        VK_OBJECT_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_KHR :: DESCRIPTOR_UPDATE_TEMPLATE_KHR;
        VK_OBJECT_TYPE_SAMPLER_YCBCR_CONVERSION_KHR :: SAMPLER_YCBCR_CONVERSION_KHR;
        VK_OBJECT_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_OBJECT_TYPE_END_RANGE :: END_RANGE;
        VK_OBJECT_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_OBJECT_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkOffset2D :: struct {
        x: s32;
        y: s32;
    }

}

#if OS == .LINUX {
    VkOffset2D :: struct {
        x: s32;
        y: s32;
    }

}

#if OS == .WINDOWS {
    VkOffset3D :: struct {
        x: s32;
        y: s32;
        z: s32;
    }

}

#if OS == .LINUX {
    VkOffset3D :: struct {
        x: s32;
        y: s32;
        z: s32;
    }

}

#if OS == .WINDOWS {
    VkPastPresentationTimingGOOGLE :: struct {
        presentID: u32;
        desiredPresentTime: u64;
        actualPresentTime: u64;
        earliestPresentTime: u64;
        presentMargin: u64;
    }

}

#if OS == .LINUX {
    VkPastPresentationTimingGOOGLE :: struct {
        presentID: u32;
        desiredPresentTime: u64;
        actualPresentTime: u64;
        earliestPresentTime: u64;
        presentMargin: u64;
    }

}

#if OS == .WINDOWS {
    VkPeerMemoryFeatureFlagBits :: enum_flags u32 {
        COPY_SRC_BIT :: 0x1;
        COPY_DST_BIT :: 0x2;
        GENERIC_SRC_BIT :: 0x4;
        GENERIC_DST_BIT :: 0x8;
        COPY_SRC_BIT_KHR :: 0x1;
        COPY_DST_BIT_KHR :: 0x2;
        GENERIC_SRC_BIT_KHR :: 0x4;
        GENERIC_DST_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PEER_MEMORY_FEATURE_COPY_SRC_BIT :: COPY_SRC_BIT;
        VK_PEER_MEMORY_FEATURE_COPY_DST_BIT :: COPY_DST_BIT;
        VK_PEER_MEMORY_FEATURE_GENERIC_SRC_BIT :: GENERIC_SRC_BIT;
        VK_PEER_MEMORY_FEATURE_GENERIC_DST_BIT :: GENERIC_DST_BIT;
        VK_PEER_MEMORY_FEATURE_COPY_SRC_BIT_KHR :: COPY_SRC_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_COPY_DST_BIT_KHR :: COPY_DST_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_GENERIC_SRC_BIT_KHR :: GENERIC_SRC_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_GENERIC_DST_BIT_KHR :: GENERIC_DST_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPeerMemoryFeatureFlagBits :: enum_flags u32 {
        COPY_SRC_BIT :: 0x1;
        COPY_DST_BIT :: 0x2;
        GENERIC_SRC_BIT :: 0x4;
        GENERIC_DST_BIT :: 0x8;
        COPY_SRC_BIT_KHR :: 0x1;
        COPY_DST_BIT_KHR :: 0x2;
        GENERIC_SRC_BIT_KHR :: 0x4;
        GENERIC_DST_BIT_KHR :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PEER_MEMORY_FEATURE_COPY_SRC_BIT :: COPY_SRC_BIT;
        VK_PEER_MEMORY_FEATURE_COPY_DST_BIT :: COPY_DST_BIT;
        VK_PEER_MEMORY_FEATURE_GENERIC_SRC_BIT :: GENERIC_SRC_BIT;
        VK_PEER_MEMORY_FEATURE_GENERIC_DST_BIT :: GENERIC_DST_BIT;
        VK_PEER_MEMORY_FEATURE_COPY_SRC_BIT_KHR :: COPY_SRC_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_COPY_DST_BIT_KHR :: COPY_DST_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_GENERIC_SRC_BIT_KHR :: GENERIC_SRC_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_GENERIC_DST_BIT_KHR :: GENERIC_DST_BIT_KHR;
        VK_PEER_MEMORY_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDevice16BitStorageFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES;
        pNext: *void;
        storageBuffer16BitAccess: VkBool32;
        uniformAndStorageBuffer16BitAccess: VkBool32;
        storagePushConstant16: VkBool32;
        storageInputOutput16: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDevice16BitStorageFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES;
        pNext: *void;
        storageBuffer16BitAccess: VkBool32;
        uniformAndStorageBuffer16BitAccess: VkBool32;
        storagePushConstant16: VkBool32;
        storageInputOutput16: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDevice8BitStorageFeaturesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR;
        pNext: *void;
        storageBuffer8BitAccess: VkBool32;
        uniformAndStorageBuffer8BitAccess: VkBool32;
        storagePushConstant8: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDevice8BitStorageFeaturesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR;
        pNext: *void;
        storageBuffer8BitAccess: VkBool32;
        uniformAndStorageBuffer8BitAccess: VkBool32;
        storagePushConstant8: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDevice_T :: struct {}

}

#if OS == .LINUX {
    VkPhysicalDevice_T :: struct {}

}

#if OS == .WINDOWS {
    VkPhysicalDeviceASTCDecodeFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT;
        pNext: *void;
        decodeModeSharedExponent: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceASTCDecodeFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT;
        pNext: *void;
        decodeModeSharedExponent: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceBlendOperationAdvancedFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT;
        pNext: *void;
        advancedBlendCoherentOperations: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceBlendOperationAdvancedFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT;
        pNext: *void;
        advancedBlendCoherentOperations: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceBlendOperationAdvancedPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT;
        pNext: *void;
        advancedBlendMaxColorAttachments: u32;
        advancedBlendIndependentBlend: VkBool32;
        advancedBlendNonPremultipliedSrcColor: VkBool32;
        advancedBlendNonPremultipliedDstColor: VkBool32;
        advancedBlendCorrelatedOverlap: VkBool32;
        advancedBlendAllOperations: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceBlendOperationAdvancedPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT;
        pNext: *void;
        advancedBlendMaxColorAttachments: u32;
        advancedBlendIndependentBlend: VkBool32;
        advancedBlendNonPremultipliedSrcColor: VkBool32;
        advancedBlendNonPremultipliedDstColor: VkBool32;
        advancedBlendCorrelatedOverlap: VkBool32;
        advancedBlendAllOperations: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceComputeShaderDerivativesFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV;
        pNext: *void;
        computeDerivativeGroupQuads: VkBool32;
        computeDerivativeGroupLinear: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceComputeShaderDerivativesFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV;
        pNext: *void;
        computeDerivativeGroupQuads: VkBool32;
        computeDerivativeGroupLinear: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceConditionalRenderingFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT;
        pNext: *void;
        conditionalRendering: VkBool32;
        inheritedConditionalRendering: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceConditionalRenderingFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT;
        pNext: *void;
        conditionalRendering: VkBool32;
        inheritedConditionalRendering: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceConservativeRasterizationPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT;
        pNext: *void;
        primitiveOverestimationSize: float;
        maxExtraPrimitiveOverestimationSize: float;
        extraPrimitiveOverestimationSizeGranularity: float;
        primitiveUnderestimation: VkBool32;
        conservativePointAndLineRasterization: VkBool32;
        degenerateTrianglesRasterized: VkBool32;
        degenerateLinesRasterized: VkBool32;
        fullyCoveredFragmentShaderInputVariable: VkBool32;
        conservativeRasterizationPostDepthCoverage: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceConservativeRasterizationPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT;
        pNext: *void;
        primitiveOverestimationSize: float;
        maxExtraPrimitiveOverestimationSize: float;
        extraPrimitiveOverestimationSizeGranularity: float;
        primitiveUnderestimation: VkBool32;
        conservativePointAndLineRasterization: VkBool32;
        degenerateTrianglesRasterized: VkBool32;
        degenerateLinesRasterized: VkBool32;
        fullyCoveredFragmentShaderInputVariable: VkBool32;
        conservativeRasterizationPostDepthCoverage: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceCornerSampledImageFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV;
        pNext: *void;
        cornerSampledImage: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceCornerSampledImageFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV;
        pNext: *void;
        cornerSampledImage: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceDescriptorIndexingFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT;
        pNext: *void;
        shaderInputAttachmentArrayDynamicIndexing: VkBool32;
        shaderUniformTexelBufferArrayDynamicIndexing: VkBool32;
        shaderStorageTexelBufferArrayDynamicIndexing: VkBool32;
        shaderUniformBufferArrayNonUniformIndexing: VkBool32;
        shaderSampledImageArrayNonUniformIndexing: VkBool32;
        shaderStorageBufferArrayNonUniformIndexing: VkBool32;
        shaderStorageImageArrayNonUniformIndexing: VkBool32;
        shaderInputAttachmentArrayNonUniformIndexing: VkBool32;
        shaderUniformTexelBufferArrayNonUniformIndexing: VkBool32;
        shaderStorageTexelBufferArrayNonUniformIndexing: VkBool32;
        descriptorBindingUniformBufferUpdateAfterBind: VkBool32;
        descriptorBindingSampledImageUpdateAfterBind: VkBool32;
        descriptorBindingStorageImageUpdateAfterBind: VkBool32;
        descriptorBindingStorageBufferUpdateAfterBind: VkBool32;
        descriptorBindingUniformTexelBufferUpdateAfterBind: VkBool32;
        descriptorBindingStorageTexelBufferUpdateAfterBind: VkBool32;
        descriptorBindingUpdateUnusedWhilePending: VkBool32;
        descriptorBindingPartiallyBound: VkBool32;
        descriptorBindingVariableDescriptorCount: VkBool32;
        runtimeDescriptorArray: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceDescriptorIndexingFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT;
        pNext: *void;
        shaderInputAttachmentArrayDynamicIndexing: VkBool32;
        shaderUniformTexelBufferArrayDynamicIndexing: VkBool32;
        shaderStorageTexelBufferArrayDynamicIndexing: VkBool32;
        shaderUniformBufferArrayNonUniformIndexing: VkBool32;
        shaderSampledImageArrayNonUniformIndexing: VkBool32;
        shaderStorageBufferArrayNonUniformIndexing: VkBool32;
        shaderStorageImageArrayNonUniformIndexing: VkBool32;
        shaderInputAttachmentArrayNonUniformIndexing: VkBool32;
        shaderUniformTexelBufferArrayNonUniformIndexing: VkBool32;
        shaderStorageTexelBufferArrayNonUniformIndexing: VkBool32;
        descriptorBindingUniformBufferUpdateAfterBind: VkBool32;
        descriptorBindingSampledImageUpdateAfterBind: VkBool32;
        descriptorBindingStorageImageUpdateAfterBind: VkBool32;
        descriptorBindingStorageBufferUpdateAfterBind: VkBool32;
        descriptorBindingUniformTexelBufferUpdateAfterBind: VkBool32;
        descriptorBindingStorageTexelBufferUpdateAfterBind: VkBool32;
        descriptorBindingUpdateUnusedWhilePending: VkBool32;
        descriptorBindingPartiallyBound: VkBool32;
        descriptorBindingVariableDescriptorCount: VkBool32;
        runtimeDescriptorArray: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceDescriptorIndexingPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT;
        pNext: *void;
        maxUpdateAfterBindDescriptorsInAllPools: u32;
        shaderUniformBufferArrayNonUniformIndexingNative: VkBool32;
        shaderSampledImageArrayNonUniformIndexingNative: VkBool32;
        shaderStorageBufferArrayNonUniformIndexingNative: VkBool32;
        shaderStorageImageArrayNonUniformIndexingNative: VkBool32;
        shaderInputAttachmentArrayNonUniformIndexingNative: VkBool32;
        robustBufferAccessUpdateAfterBind: VkBool32;
        quadDivergentImplicitLod: VkBool32;
        maxPerStageDescriptorUpdateAfterBindSamplers: u32;
        maxPerStageDescriptorUpdateAfterBindUniformBuffers: u32;
        maxPerStageDescriptorUpdateAfterBindStorageBuffers: u32;
        maxPerStageDescriptorUpdateAfterBindSampledImages: u32;
        maxPerStageDescriptorUpdateAfterBindStorageImages: u32;
        maxPerStageDescriptorUpdateAfterBindInputAttachments: u32;
        maxPerStageUpdateAfterBindResources: u32;
        maxDescriptorSetUpdateAfterBindSamplers: u32;
        maxDescriptorSetUpdateAfterBindUniformBuffers: u32;
        maxDescriptorSetUpdateAfterBindUniformBuffersDynamic: u32;
        maxDescriptorSetUpdateAfterBindStorageBuffers: u32;
        maxDescriptorSetUpdateAfterBindStorageBuffersDynamic: u32;
        maxDescriptorSetUpdateAfterBindSampledImages: u32;
        maxDescriptorSetUpdateAfterBindStorageImages: u32;
        maxDescriptorSetUpdateAfterBindInputAttachments: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceDescriptorIndexingPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT;
        pNext: *void;
        maxUpdateAfterBindDescriptorsInAllPools: u32;
        shaderUniformBufferArrayNonUniformIndexingNative: VkBool32;
        shaderSampledImageArrayNonUniformIndexingNative: VkBool32;
        shaderStorageBufferArrayNonUniformIndexingNative: VkBool32;
        shaderStorageImageArrayNonUniformIndexingNative: VkBool32;
        shaderInputAttachmentArrayNonUniformIndexingNative: VkBool32;
        robustBufferAccessUpdateAfterBind: VkBool32;
        quadDivergentImplicitLod: VkBool32;
        maxPerStageDescriptorUpdateAfterBindSamplers: u32;
        maxPerStageDescriptorUpdateAfterBindUniformBuffers: u32;
        maxPerStageDescriptorUpdateAfterBindStorageBuffers: u32;
        maxPerStageDescriptorUpdateAfterBindSampledImages: u32;
        maxPerStageDescriptorUpdateAfterBindStorageImages: u32;
        maxPerStageDescriptorUpdateAfterBindInputAttachments: u32;
        maxPerStageUpdateAfterBindResources: u32;
        maxDescriptorSetUpdateAfterBindSamplers: u32;
        maxDescriptorSetUpdateAfterBindUniformBuffers: u32;
        maxDescriptorSetUpdateAfterBindUniformBuffersDynamic: u32;
        maxDescriptorSetUpdateAfterBindStorageBuffers: u32;
        maxDescriptorSetUpdateAfterBindStorageBuffersDynamic: u32;
        maxDescriptorSetUpdateAfterBindSampledImages: u32;
        maxDescriptorSetUpdateAfterBindStorageImages: u32;
        maxDescriptorSetUpdateAfterBindInputAttachments: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceDiscardRectanglePropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT;
        pNext: *void;
        maxDiscardRectangles: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceDiscardRectanglePropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT;
        pNext: *void;
        maxDiscardRectangles: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExclusiveScissorFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV;
        pNext: *void;
        exclusiveScissor: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExclusiveScissorFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV;
        pNext: *void;
        exclusiveScissor: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalBufferInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO;
        pNext: *void;
        flags: VkBufferCreateFlags;
        usage: VkBufferUsageFlags;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExternalBufferInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO;
        pNext: *void;
        flags: VkBufferCreateFlags;
        usage: VkBufferUsageFlags;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalFenceInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO;
        pNext: *void;
        handleType: VkExternalFenceHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExternalFenceInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO;
        pNext: *void;
        handleType: VkExternalFenceHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalImageFormatInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExternalImageFormatInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO;
        pNext: *void;
        handleType: VkExternalMemoryHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalMemoryHostPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT;
        pNext: *void;
        minImportedHostPointerAlignment: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExternalMemoryHostPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT;
        pNext: *void;
        minImportedHostPointerAlignment: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalSemaphoreInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO;
        pNext: *void;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceExternalSemaphoreInfo :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO;
        pNext: *void;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceFeatures :: struct {
        robustBufferAccess: VkBool32;
        fullDrawIndexUint32: VkBool32;
        imageCubeArray: VkBool32;
        independentBlend: VkBool32;
        geometryShader: VkBool32;
        tessellationShader: VkBool32;
        sampleRateShading: VkBool32;
        dualSrcBlend: VkBool32;
        logicOp: VkBool32;
        multiDrawIndirect: VkBool32;
        drawIndirectFirstInstance: VkBool32;
        depthClamp: VkBool32;
        depthBiasClamp: VkBool32;
        fillModeNonSolid: VkBool32;
        depthBounds: VkBool32;
        wideLines: VkBool32;
        largePoints: VkBool32;
        alphaToOne: VkBool32;
        multiViewport: VkBool32;
        samplerAnisotropy: VkBool32;
        textureCompressionETC2: VkBool32;
        textureCompressionASTC_LDR: VkBool32;
        textureCompressionBC: VkBool32;
        occlusionQueryPrecise: VkBool32;
        pipelineStatisticsQuery: VkBool32;
        vertexPipelineStoresAndAtomics: VkBool32;
        fragmentStoresAndAtomics: VkBool32;
        shaderTessellationAndGeometryPointSize: VkBool32;
        shaderImageGatherExtended: VkBool32;
        shaderStorageImageExtendedFormats: VkBool32;
        shaderStorageImageMultisample: VkBool32;
        shaderStorageImageReadWithoutFormat: VkBool32;
        shaderStorageImageWriteWithoutFormat: VkBool32;
        shaderUniformBufferArrayDynamicIndexing: VkBool32;
        shaderSampledImageArrayDynamicIndexing: VkBool32;
        shaderStorageBufferArrayDynamicIndexing: VkBool32;
        shaderStorageImageArrayDynamicIndexing: VkBool32;
        shaderClipDistance: VkBool32;
        shaderCullDistance: VkBool32;
        shaderFloat64: VkBool32;
        shaderInt64: VkBool32;
        shaderInt16: VkBool32;
        shaderResourceResidency: VkBool32;
        shaderResourceMinLod: VkBool32;
        sparseBinding: VkBool32;
        sparseResidencyBuffer: VkBool32;
        sparseResidencyImage2D: VkBool32;
        sparseResidencyImage3D: VkBool32;
        sparseResidency2Samples: VkBool32;
        sparseResidency4Samples: VkBool32;
        sparseResidency8Samples: VkBool32;
        sparseResidency16Samples: VkBool32;
        sparseResidencyAliased: VkBool32;
        variableMultisampleRate: VkBool32;
        inheritedQueries: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceFeatures :: struct {
        robustBufferAccess: VkBool32;
        fullDrawIndexUint32: VkBool32;
        imageCubeArray: VkBool32;
        independentBlend: VkBool32;
        geometryShader: VkBool32;
        tessellationShader: VkBool32;
        sampleRateShading: VkBool32;
        dualSrcBlend: VkBool32;
        logicOp: VkBool32;
        multiDrawIndirect: VkBool32;
        drawIndirectFirstInstance: VkBool32;
        depthClamp: VkBool32;
        depthBiasClamp: VkBool32;
        fillModeNonSolid: VkBool32;
        depthBounds: VkBool32;
        wideLines: VkBool32;
        largePoints: VkBool32;
        alphaToOne: VkBool32;
        multiViewport: VkBool32;
        samplerAnisotropy: VkBool32;
        textureCompressionETC2: VkBool32;
        textureCompressionASTC_LDR: VkBool32;
        textureCompressionBC: VkBool32;
        occlusionQueryPrecise: VkBool32;
        pipelineStatisticsQuery: VkBool32;
        vertexPipelineStoresAndAtomics: VkBool32;
        fragmentStoresAndAtomics: VkBool32;
        shaderTessellationAndGeometryPointSize: VkBool32;
        shaderImageGatherExtended: VkBool32;
        shaderStorageImageExtendedFormats: VkBool32;
        shaderStorageImageMultisample: VkBool32;
        shaderStorageImageReadWithoutFormat: VkBool32;
        shaderStorageImageWriteWithoutFormat: VkBool32;
        shaderUniformBufferArrayDynamicIndexing: VkBool32;
        shaderSampledImageArrayDynamicIndexing: VkBool32;
        shaderStorageBufferArrayDynamicIndexing: VkBool32;
        shaderStorageImageArrayDynamicIndexing: VkBool32;
        shaderClipDistance: VkBool32;
        shaderCullDistance: VkBool32;
        shaderFloat64: VkBool32;
        shaderInt64: VkBool32;
        shaderInt16: VkBool32;
        shaderResourceResidency: VkBool32;
        shaderResourceMinLod: VkBool32;
        sparseBinding: VkBool32;
        sparseResidencyBuffer: VkBool32;
        sparseResidencyImage2D: VkBool32;
        sparseResidencyImage3D: VkBool32;
        sparseResidency2Samples: VkBool32;
        sparseResidency4Samples: VkBool32;
        sparseResidency8Samples: VkBool32;
        sparseResidency16Samples: VkBool32;
        sparseResidencyAliased: VkBool32;
        variableMultisampleRate: VkBool32;
        inheritedQueries: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceFeatures2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_FEATURES_2;
        pNext: *void;
        features: VkPhysicalDeviceFeatures;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceFeatures2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_FEATURES_2;
        pNext: *void;
        features: VkPhysicalDeviceFeatures;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceFragmentShaderBarycentricFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV;
        pNext: *void;
        fragmentShaderBarycentric: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceFragmentShaderBarycentricFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV;
        pNext: *void;
        fragmentShaderBarycentric: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceGroupProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_GROUP_PROPERTIES;
        pNext: *void;
        physicalDeviceCount: u32;
        physicalDevices: [32] VkPhysicalDevice;
        subsetAllocation: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceGroupProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_GROUP_PROPERTIES;
        pNext: *void;
        physicalDeviceCount: u32;
        physicalDevices: [32] VkPhysicalDevice;
        subsetAllocation: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceIDProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_ID_PROPERTIES;
        pNext: *void;
        deviceUUID: [16] u8;
        driverUUID: [16] u8;
        deviceLUID: [8] u8;
        deviceNodeMask: u32;
        deviceLUIDValid: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceIDProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_ID_PROPERTIES;
        pNext: *void;
        deviceUUID: [16] u8;
        driverUUID: [16] u8;
        deviceLUID: [8] u8;
        deviceNodeMask: u32;
        deviceLUIDValid: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceImageFormatInfo2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2;
        pNext: *void;
        format: VkFormat;
        type: VkImageType;
        tiling: VkImageTiling;
        usage: VkImageUsageFlags;
        flags: VkImageCreateFlags;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceImageFormatInfo2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2;
        pNext: *void;
        format: VkFormat;
        type: VkImageType;
        tiling: VkImageTiling;
        usage: VkImageUsageFlags;
        flags: VkImageCreateFlags;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceInlineUniformBlockFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT;
        pNext: *void;
        inlineUniformBlock: VkBool32;
        descriptorBindingInlineUniformBlockUpdateAfterBind: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceInlineUniformBlockFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT;
        pNext: *void;
        inlineUniformBlock: VkBool32;
        descriptorBindingInlineUniformBlockUpdateAfterBind: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceInlineUniformBlockPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT;
        pNext: *void;
        maxInlineUniformBlockSize: u32;
        maxPerStageDescriptorInlineUniformBlocks: u32;
        maxPerStageDescriptorUpdateAfterBindInlineUniformBlocks: u32;
        maxDescriptorSetInlineUniformBlocks: u32;
        maxDescriptorSetUpdateAfterBindInlineUniformBlocks: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceInlineUniformBlockPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT;
        pNext: *void;
        maxInlineUniformBlockSize: u32;
        maxPerStageDescriptorInlineUniformBlocks: u32;
        maxPerStageDescriptorUpdateAfterBindInlineUniformBlocks: u32;
        maxDescriptorSetInlineUniformBlocks: u32;
        maxDescriptorSetUpdateAfterBindInlineUniformBlocks: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceLimits :: struct {
        maxImageDimension1D: u32;
        maxImageDimension2D: u32;
        maxImageDimension3D: u32;
        maxImageDimensionCube: u32;
        maxImageArrayLayers: u32;
        maxTexelBufferElements: u32;
        maxUniformBufferRange: u32;
        maxStorageBufferRange: u32;
        maxPushConstantsSize: u32;
        maxMemoryAllocationCount: u32;
        maxSamplerAllocationCount: u32;
        bufferImageGranularity: VkDeviceSize;
        sparseAddressSpaceSize: VkDeviceSize;
        maxBoundDescriptorSets: u32;
        maxPerStageDescriptorSamplers: u32;
        maxPerStageDescriptorUniformBuffers: u32;
        maxPerStageDescriptorStorageBuffers: u32;
        maxPerStageDescriptorSampledImages: u32;
        maxPerStageDescriptorStorageImages: u32;
        maxPerStageDescriptorInputAttachments: u32;
        maxPerStageResources: u32;
        maxDescriptorSetSamplers: u32;
        maxDescriptorSetUniformBuffers: u32;
        maxDescriptorSetUniformBuffersDynamic: u32;
        maxDescriptorSetStorageBuffers: u32;
        maxDescriptorSetStorageBuffersDynamic: u32;
        maxDescriptorSetSampledImages: u32;
        maxDescriptorSetStorageImages: u32;
        maxDescriptorSetInputAttachments: u32;
        maxVertexInputAttributes: u32;
        maxVertexInputBindings: u32;
        maxVertexInputAttributeOffset: u32;
        maxVertexInputBindingStride: u32;
        maxVertexOutputComponents: u32;
        maxTessellationGenerationLevel: u32;
        maxTessellationPatchSize: u32;
        maxTessellationControlPerVertexInputComponents: u32;
        maxTessellationControlPerVertexOutputComponents: u32;
        maxTessellationControlPerPatchOutputComponents: u32;
        maxTessellationControlTotalOutputComponents: u32;
        maxTessellationEvaluationInputComponents: u32;
        maxTessellationEvaluationOutputComponents: u32;
        maxGeometryShaderInvocations: u32;
        maxGeometryInputComponents: u32;
        maxGeometryOutputComponents: u32;
        maxGeometryOutputVertices: u32;
        maxGeometryTotalOutputComponents: u32;
        maxFragmentInputComponents: u32;
        maxFragmentOutputAttachments: u32;
        maxFragmentDualSrcAttachments: u32;
        maxFragmentCombinedOutputResources: u32;
        maxComputeSharedMemorySize: u32;
        maxComputeWorkGroupCount: [3] u32;
        maxComputeWorkGroupInvocations: u32;
        maxComputeWorkGroupSize: [3] u32;
        subPixelPrecisionBits: u32;
        subTexelPrecisionBits: u32;
        mipmapPrecisionBits: u32;
        maxDrawIndexedIndexValue: u32;
        maxDrawIndirectCount: u32;
        maxSamplerLodBias: float;
        maxSamplerAnisotropy: float;
        maxViewports: u32;
        maxViewportDimensions: [2] u32;
        viewportBoundsRange: [2] float;
        viewportSubPixelBits: u32;
        minMemoryMapAlignment: u64;
        minTexelBufferOffsetAlignment: VkDeviceSize;
        minUniformBufferOffsetAlignment: VkDeviceSize;
        minStorageBufferOffsetAlignment: VkDeviceSize;
        minTexelOffset: s32;
        maxTexelOffset: u32;
        minTexelGatherOffset: s32;
        maxTexelGatherOffset: u32;
        minInterpolationOffset: float;
        maxInterpolationOffset: float;
        subPixelInterpolationOffsetBits: u32;
        maxFramebufferWidth: u32;
        maxFramebufferHeight: u32;
        maxFramebufferLayers: u32;
        framebufferColorSampleCounts: VkSampleCountFlags;
        framebufferDepthSampleCounts: VkSampleCountFlags;
        framebufferStencilSampleCounts: VkSampleCountFlags;
        framebufferNoAttachmentsSampleCounts: VkSampleCountFlags;
        maxColorAttachments: u32;
        sampledImageColorSampleCounts: VkSampleCountFlags;
        sampledImageIntegerSampleCounts: VkSampleCountFlags;
        sampledImageDepthSampleCounts: VkSampleCountFlags;
        sampledImageStencilSampleCounts: VkSampleCountFlags;
        storageImageSampleCounts: VkSampleCountFlags;
        maxSampleMaskWords: u32;
        timestampComputeAndGraphics: VkBool32;
        timestampPeriod: float;
        maxClipDistances: u32;
        maxCullDistances: u32;
        maxCombinedClipAndCullDistances: u32;
        discreteQueuePriorities: u32;
        pointSizeRange: [2] float;
        lineWidthRange: [2] float;
        pointSizeGranularity: float;
        lineWidthGranularity: float;
        strictLines: VkBool32;
        standardSampleLocations: VkBool32;
        optimalBufferCopyOffsetAlignment: VkDeviceSize;
        optimalBufferCopyRowPitchAlignment: VkDeviceSize;
        nonCoherentAtomSize: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceLimits :: struct {
        maxImageDimension1D: u32;
        maxImageDimension2D: u32;
        maxImageDimension3D: u32;
        maxImageDimensionCube: u32;
        maxImageArrayLayers: u32;
        maxTexelBufferElements: u32;
        maxUniformBufferRange: u32;
        maxStorageBufferRange: u32;
        maxPushConstantsSize: u32;
        maxMemoryAllocationCount: u32;
        maxSamplerAllocationCount: u32;
        bufferImageGranularity: VkDeviceSize;
        sparseAddressSpaceSize: VkDeviceSize;
        maxBoundDescriptorSets: u32;
        maxPerStageDescriptorSamplers: u32;
        maxPerStageDescriptorUniformBuffers: u32;
        maxPerStageDescriptorStorageBuffers: u32;
        maxPerStageDescriptorSampledImages: u32;
        maxPerStageDescriptorStorageImages: u32;
        maxPerStageDescriptorInputAttachments: u32;
        maxPerStageResources: u32;
        maxDescriptorSetSamplers: u32;
        maxDescriptorSetUniformBuffers: u32;
        maxDescriptorSetUniformBuffersDynamic: u32;
        maxDescriptorSetStorageBuffers: u32;
        maxDescriptorSetStorageBuffersDynamic: u32;
        maxDescriptorSetSampledImages: u32;
        maxDescriptorSetStorageImages: u32;
        maxDescriptorSetInputAttachments: u32;
        maxVertexInputAttributes: u32;
        maxVertexInputBindings: u32;
        maxVertexInputAttributeOffset: u32;
        maxVertexInputBindingStride: u32;
        maxVertexOutputComponents: u32;
        maxTessellationGenerationLevel: u32;
        maxTessellationPatchSize: u32;
        maxTessellationControlPerVertexInputComponents: u32;
        maxTessellationControlPerVertexOutputComponents: u32;
        maxTessellationControlPerPatchOutputComponents: u32;
        maxTessellationControlTotalOutputComponents: u32;
        maxTessellationEvaluationInputComponents: u32;
        maxTessellationEvaluationOutputComponents: u32;
        maxGeometryShaderInvocations: u32;
        maxGeometryInputComponents: u32;
        maxGeometryOutputComponents: u32;
        maxGeometryOutputVertices: u32;
        maxGeometryTotalOutputComponents: u32;
        maxFragmentInputComponents: u32;
        maxFragmentOutputAttachments: u32;
        maxFragmentDualSrcAttachments: u32;
        maxFragmentCombinedOutputResources: u32;
        maxComputeSharedMemorySize: u32;
        maxComputeWorkGroupCount: [3] u32;
        maxComputeWorkGroupInvocations: u32;
        maxComputeWorkGroupSize: [3] u32;
        subPixelPrecisionBits: u32;
        subTexelPrecisionBits: u32;
        mipmapPrecisionBits: u32;
        maxDrawIndexedIndexValue: u32;
        maxDrawIndirectCount: u32;
        maxSamplerLodBias: float;
        maxSamplerAnisotropy: float;
        maxViewports: u32;
        maxViewportDimensions: [2] u32;
        viewportBoundsRange: [2] float;
        viewportSubPixelBits: u32;
        minMemoryMapAlignment: u64;
        minTexelBufferOffsetAlignment: VkDeviceSize;
        minUniformBufferOffsetAlignment: VkDeviceSize;
        minStorageBufferOffsetAlignment: VkDeviceSize;
        minTexelOffset: s32;
        maxTexelOffset: u32;
        minTexelGatherOffset: s32;
        maxTexelGatherOffset: u32;
        minInterpolationOffset: float;
        maxInterpolationOffset: float;
        subPixelInterpolationOffsetBits: u32;
        maxFramebufferWidth: u32;
        maxFramebufferHeight: u32;
        maxFramebufferLayers: u32;
        framebufferColorSampleCounts: VkSampleCountFlags;
        framebufferDepthSampleCounts: VkSampleCountFlags;
        framebufferStencilSampleCounts: VkSampleCountFlags;
        framebufferNoAttachmentsSampleCounts: VkSampleCountFlags;
        maxColorAttachments: u32;
        sampledImageColorSampleCounts: VkSampleCountFlags;
        sampledImageIntegerSampleCounts: VkSampleCountFlags;
        sampledImageDepthSampleCounts: VkSampleCountFlags;
        sampledImageStencilSampleCounts: VkSampleCountFlags;
        storageImageSampleCounts: VkSampleCountFlags;
        maxSampleMaskWords: u32;
        timestampComputeAndGraphics: VkBool32;
        timestampPeriod: float;
        maxClipDistances: u32;
        maxCullDistances: u32;
        maxCombinedClipAndCullDistances: u32;
        discreteQueuePriorities: u32;
        pointSizeRange: [2] float;
        lineWidthRange: [2] float;
        pointSizeGranularity: float;
        lineWidthGranularity: float;
        strictLines: VkBool32;
        standardSampleLocations: VkBool32;
        optimalBufferCopyOffsetAlignment: VkDeviceSize;
        optimalBufferCopyRowPitchAlignment: VkDeviceSize;
        nonCoherentAtomSize: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMaintenance3Properties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES;
        pNext: *void;
        maxPerSetDescriptors: u32;
        maxMemoryAllocationSize: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMaintenance3Properties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES;
        pNext: *void;
        maxPerSetDescriptors: u32;
        maxMemoryAllocationSize: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMemoryProperties :: struct {
        memoryTypeCount: u32;
        memoryTypes: [32] VkMemoryType;
        memoryHeapCount: u32;
        memoryHeaps: [16] VkMemoryHeap;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMemoryProperties :: struct {
        memoryTypeCount: u32;
        memoryTypes: [32] VkMemoryType;
        memoryHeapCount: u32;
        memoryHeaps: [16] VkMemoryHeap;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMemoryProperties2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MEMORY_PROPERTIES_2;
        pNext: *void;
        memoryProperties: VkPhysicalDeviceMemoryProperties;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMemoryProperties2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MEMORY_PROPERTIES_2;
        pNext: *void;
        memoryProperties: VkPhysicalDeviceMemoryProperties;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMeshShaderFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV;
        pNext: *void;
        taskShader: VkBool32;
        meshShader: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMeshShaderFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV;
        pNext: *void;
        taskShader: VkBool32;
        meshShader: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMeshShaderPropertiesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV;
        pNext: *void;
        maxDrawMeshTasksCount: u32;
        maxTaskWorkGroupInvocations: u32;
        maxTaskWorkGroupSize: [3] u32;
        maxTaskTotalMemorySize: u32;
        maxTaskOutputCount: u32;
        maxMeshWorkGroupInvocations: u32;
        maxMeshWorkGroupSize: [3] u32;
        maxMeshTotalMemorySize: u32;
        maxMeshOutputVertices: u32;
        maxMeshOutputPrimitives: u32;
        maxMeshMultiviewViewCount: u32;
        meshOutputPerVertexGranularity: u32;
        meshOutputPerPrimitiveGranularity: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMeshShaderPropertiesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV;
        pNext: *void;
        maxDrawMeshTasksCount: u32;
        maxTaskWorkGroupInvocations: u32;
        maxTaskWorkGroupSize: [3] u32;
        maxTaskTotalMemorySize: u32;
        maxTaskOutputCount: u32;
        maxMeshWorkGroupInvocations: u32;
        maxMeshWorkGroupSize: [3] u32;
        maxMeshTotalMemorySize: u32;
        maxMeshOutputVertices: u32;
        maxMeshOutputPrimitives: u32;
        maxMeshMultiviewViewCount: u32;
        meshOutputPerVertexGranularity: u32;
        meshOutputPerPrimitiveGranularity: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMultiviewFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_FEATURES;
        pNext: *void;
        multiview: VkBool32;
        multiviewGeometryShader: VkBool32;
        multiviewTessellationShader: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMultiviewFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_FEATURES;
        pNext: *void;
        multiview: VkBool32;
        multiviewGeometryShader: VkBool32;
        multiviewTessellationShader: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMultiviewPerViewAttributesPropertiesNVX :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX;
        pNext: *void;
        perViewPositionAllComponents: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMultiviewPerViewAttributesPropertiesNVX :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX;
        pNext: *void;
        perViewPositionAllComponents: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceMultiviewProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES;
        pNext: *void;
        maxMultiviewViewCount: u32;
        maxMultiviewInstanceIndex: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceMultiviewProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES;
        pNext: *void;
        maxMultiviewViewCount: u32;
        maxMultiviewInstanceIndex: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDevicePointClippingProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES;
        pNext: *void;
        pointClippingBehavior: VkPointClippingBehavior;
    }

}

#if OS == .LINUX {
    VkPhysicalDevicePointClippingProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES;
        pNext: *void;
        pointClippingBehavior: VkPointClippingBehavior;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceProperties :: struct {
        apiVersion: u32;
        driverVersion: u32;
        vendorID: u32;
        deviceID: u32;
        deviceType: VkPhysicalDeviceType;
        deviceName: [256] u8;
        pipelineCacheUUID: [16] u8;
        limits: VkPhysicalDeviceLimits;
        sparseProperties: VkPhysicalDeviceSparseProperties;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceProperties :: struct {
        apiVersion: u32;
        driverVersion: u32;
        vendorID: u32;
        deviceID: u32;
        deviceType: VkPhysicalDeviceType;
        deviceName: [256] u8;
        pipelineCacheUUID: [16] u8;
        limits: VkPhysicalDeviceLimits;
        sparseProperties: VkPhysicalDeviceSparseProperties;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceProperties2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROPERTIES_2;
        pNext: *void;
        properties: VkPhysicalDeviceProperties;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceProperties2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROPERTIES_2;
        pNext: *void;
        properties: VkPhysicalDeviceProperties;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceProtectedMemoryFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES;
        pNext: *void;
        protectedMemory: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceProtectedMemoryFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES;
        pNext: *void;
        protectedMemory: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceProtectedMemoryProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES;
        pNext: *void;
        protectedNoFault: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceProtectedMemoryProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES;
        pNext: *void;
        protectedNoFault: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDevicePushDescriptorPropertiesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR;
        pNext: *void;
        maxPushDescriptors: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDevicePushDescriptorPropertiesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR;
        pNext: *void;
        maxPushDescriptors: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceRaytracingPropertiesNVX :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX;
        pNext: *void;
        shaderHeaderSize: u32;
        maxRecursionDepth: u32;
        maxGeometryCount: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceRaytracingPropertiesNVX :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX;
        pNext: *void;
        shaderHeaderSize: u32;
        maxRecursionDepth: u32;
        maxGeometryCount: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceRepresentativeFragmentTestFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV;
        pNext: *void;
        representativeFragmentTest: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceRepresentativeFragmentTestFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV;
        pNext: *void;
        representativeFragmentTest: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSampleLocationsPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT;
        pNext: *void;
        sampleLocationSampleCounts: VkSampleCountFlags;
        maxSampleLocationGridSize: VkExtent2D;
        sampleLocationCoordinateRange: [2] float;
        sampleLocationSubPixelBits: u32;
        variableSampleLocations: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSampleLocationsPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT;
        pNext: *void;
        sampleLocationSampleCounts: VkSampleCountFlags;
        maxSampleLocationGridSize: VkExtent2D;
        sampleLocationCoordinateRange: [2] float;
        sampleLocationSubPixelBits: u32;
        variableSampleLocations: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSamplerFilterMinmaxPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT;
        pNext: *void;
        filterMinmaxSingleComponentFormats: VkBool32;
        filterMinmaxImageComponentMapping: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSamplerFilterMinmaxPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT;
        pNext: *void;
        filterMinmaxSingleComponentFormats: VkBool32;
        filterMinmaxImageComponentMapping: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSamplerYcbcrConversionFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES;
        pNext: *void;
        samplerYcbcrConversion: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSamplerYcbcrConversionFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES;
        pNext: *void;
        samplerYcbcrConversion: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceShaderCorePropertiesAMD :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD;
        pNext: *void;
        shaderEngineCount: u32;
        shaderArraysPerEngineCount: u32;
        computeUnitsPerShaderArray: u32;
        simdPerComputeUnit: u32;
        wavefrontsPerSimd: u32;
        wavefrontSize: u32;
        sgprsPerSimd: u32;
        minSgprAllocation: u32;
        maxSgprAllocation: u32;
        sgprAllocationGranularity: u32;
        vgprsPerSimd: u32;
        minVgprAllocation: u32;
        maxVgprAllocation: u32;
        vgprAllocationGranularity: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceShaderCorePropertiesAMD :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD;
        pNext: *void;
        shaderEngineCount: u32;
        shaderArraysPerEngineCount: u32;
        computeUnitsPerShaderArray: u32;
        simdPerComputeUnit: u32;
        wavefrontsPerSimd: u32;
        wavefrontSize: u32;
        sgprsPerSimd: u32;
        minSgprAllocation: u32;
        maxSgprAllocation: u32;
        sgprAllocationGranularity: u32;
        vgprsPerSimd: u32;
        minVgprAllocation: u32;
        maxVgprAllocation: u32;
        vgprAllocationGranularity: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceShaderDrawParameterFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES;
        pNext: *void;
        shaderDrawParameters: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceShaderDrawParameterFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES;
        pNext: *void;
        shaderDrawParameters: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceShaderImageFootprintFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV;
        pNext: *void;
        imageFootprint: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceShaderImageFootprintFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV;
        pNext: *void;
        imageFootprint: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceShadingRateImageFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV;
        pNext: *void;
        shadingRateImage: VkBool32;
        shadingRateCoarseSampleOrder: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceShadingRateImageFeaturesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV;
        pNext: *void;
        shadingRateImage: VkBool32;
        shadingRateCoarseSampleOrder: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceShadingRateImagePropertiesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV;
        pNext: *void;
        shadingRateTexelSize: VkExtent2D;
        shadingRatePaletteSize: u32;
        shadingRateMaxCoarseSamples: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceShadingRateImagePropertiesNV :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV;
        pNext: *void;
        shadingRateTexelSize: VkExtent2D;
        shadingRatePaletteSize: u32;
        shadingRateMaxCoarseSamples: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSparseImageFormatInfo2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2;
        pNext: *void;
        format: VkFormat;
        type: VkImageType;
        samples: VkSampleCountFlagBits;
        usage: VkImageUsageFlags;
        tiling: VkImageTiling;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSparseImageFormatInfo2 :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2;
        pNext: *void;
        format: VkFormat;
        type: VkImageType;
        samples: VkSampleCountFlagBits;
        usage: VkImageUsageFlags;
        tiling: VkImageTiling;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSparseProperties :: struct {
        residencyStandard2DBlockShape: VkBool32;
        residencyStandard2DMultisampleBlockShape: VkBool32;
        residencyStandard3DBlockShape: VkBool32;
        residencyAlignedMipSize: VkBool32;
        residencyNonResidentStrict: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSparseProperties :: struct {
        residencyStandard2DBlockShape: VkBool32;
        residencyStandard2DMultisampleBlockShape: VkBool32;
        residencyStandard3DBlockShape: VkBool32;
        residencyAlignedMipSize: VkBool32;
        residencyNonResidentStrict: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSubgroupProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SUBGROUP_PROPERTIES;
        pNext: *void;
        subgroupSize: u32;
        supportedStages: VkShaderStageFlags;
        supportedOperations: VkSubgroupFeatureFlags;
        quadOperationsInAllStages: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSubgroupProperties :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SUBGROUP_PROPERTIES;
        pNext: *void;
        subgroupSize: u32;
        supportedStages: VkShaderStageFlags;
        supportedOperations: VkSubgroupFeatureFlags;
        quadOperationsInAllStages: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceSurfaceInfo2KHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SURFACE_INFO_2_KHR;
        pNext: *void;
        surface: VkSurfaceKHR;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceSurfaceInfo2KHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_SURFACE_INFO_2_KHR;
        pNext: *void;
        surface: VkSurfaceKHR;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceType :: enum u32 {
        OTHER :: 0;
        INTEGRATED_GPU :: 1;
        DISCRETE_GPU :: 2;
        VIRTUAL_GPU :: 3;
        CPU :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_PHYSICAL_DEVICE_TYPE_OTHER :: OTHER;
        VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU :: INTEGRATED_GPU;
        VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU :: DISCRETE_GPU;
        VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU :: VIRTUAL_GPU;
        VK_PHYSICAL_DEVICE_TYPE_CPU :: CPU;
        VK_PHYSICAL_DEVICE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PHYSICAL_DEVICE_TYPE_END_RANGE :: END_RANGE;
        VK_PHYSICAL_DEVICE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_PHYSICAL_DEVICE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceType :: enum u32 {
        OTHER :: 0;
        INTEGRATED_GPU :: 1;
        DISCRETE_GPU :: 2;
        VIRTUAL_GPU :: 3;
        CPU :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_PHYSICAL_DEVICE_TYPE_OTHER :: OTHER;
        VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU :: INTEGRATED_GPU;
        VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU :: DISCRETE_GPU;
        VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU :: VIRTUAL_GPU;
        VK_PHYSICAL_DEVICE_TYPE_CPU :: CPU;
        VK_PHYSICAL_DEVICE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PHYSICAL_DEVICE_TYPE_END_RANGE :: END_RANGE;
        VK_PHYSICAL_DEVICE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_PHYSICAL_DEVICE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceVariablePointerFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES;
        pNext: *void;
        variablePointersStorageBuffer: VkBool32;
        variablePointers: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceVariablePointerFeatures :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES;
        pNext: *void;
        variablePointersStorageBuffer: VkBool32;
        variablePointers: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
        pNext: *void;
        vertexAttributeInstanceRateDivisor: VkBool32;
        vertexAttributeInstanceRateZeroDivisor: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceVertexAttributeDivisorFeaturesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
        pNext: *void;
        vertexAttributeInstanceRateDivisor: VkBool32;
        vertexAttributeInstanceRateZeroDivisor: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceVertexAttributeDivisorPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT;
        pNext: *void;
        maxVertexAttribDivisor: u32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceVertexAttributeDivisorPropertiesEXT :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT;
        pNext: *void;
        maxVertexAttribDivisor: u32;
    }

}

#if OS == .WINDOWS {
    VkPhysicalDeviceVulkanMemoryModelFeaturesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR;
        pNext: *void;
        vulkanMemoryModel: VkBool32;
        vulkanMemoryModelDeviceScope: VkBool32;
    }

}

#if OS == .LINUX {
    VkPhysicalDeviceVulkanMemoryModelFeaturesKHR :: struct {
        sType: VkStructureType = .PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR;
        pNext: *void;
        vulkanMemoryModel: VkBool32;
        vulkanMemoryModelDeviceScope: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPipeline_T :: struct {}

}

#if OS == .LINUX {
    VkPipeline_T :: struct {}

}

#if OS == .WINDOWS {
    VkPipelineBindPoint :: enum u32 {
        GRAPHICS :: 0;
        COMPUTE :: 1;
        RAYTRACING_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_PIPELINE_BIND_POINT_GRAPHICS :: GRAPHICS;
        VK_PIPELINE_BIND_POINT_COMPUTE :: COMPUTE;
        VK_PIPELINE_BIND_POINT_RAYTRACING_NVX :: RAYTRACING_NVX;
        VK_PIPELINE_BIND_POINT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PIPELINE_BIND_POINT_END_RANGE :: END_RANGE;
        VK_PIPELINE_BIND_POINT_RANGE_SIZE :: RANGE_SIZE;
        VK_PIPELINE_BIND_POINT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPipelineBindPoint :: enum u32 {
        GRAPHICS :: 0;
        COMPUTE :: 1;
        RAYTRACING_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_PIPELINE_BIND_POINT_GRAPHICS :: GRAPHICS;
        VK_PIPELINE_BIND_POINT_COMPUTE :: COMPUTE;
        VK_PIPELINE_BIND_POINT_RAYTRACING_NVX :: RAYTRACING_NVX;
        VK_PIPELINE_BIND_POINT_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PIPELINE_BIND_POINT_END_RANGE :: END_RANGE;
        VK_PIPELINE_BIND_POINT_RANGE_SIZE :: RANGE_SIZE;
        VK_PIPELINE_BIND_POINT_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPipelineCache_T :: struct {}

}

#if OS == .LINUX {
    VkPipelineCache_T :: struct {}

}

#if OS == .WINDOWS {
    VkPipelineCacheCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_CACHE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCacheCreateFlags;
        initialDataSize: u64;
        pInitialData: *void;
    }

}

#if OS == .LINUX {
    VkPipelineCacheCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_CACHE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineCacheCreateFlags;
        initialDataSize: u64;
        pInitialData: *void;
    }

}

#if OS == .WINDOWS {
    VkPipelineCacheHeaderVersion :: enum u32 {
        ONE :: 1;
        BEGIN_RANGE :: 1;
        END_RANGE :: 1;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_PIPELINE_CACHE_HEADER_VERSION_ONE :: ONE;
        VK_PIPELINE_CACHE_HEADER_VERSION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PIPELINE_CACHE_HEADER_VERSION_END_RANGE :: END_RANGE;
        VK_PIPELINE_CACHE_HEADER_VERSION_RANGE_SIZE :: RANGE_SIZE;
        VK_PIPELINE_CACHE_HEADER_VERSION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPipelineCacheHeaderVersion :: enum u32 {
        ONE :: 1;
        BEGIN_RANGE :: 1;
        END_RANGE :: 1;
        RANGE_SIZE :: 1;
        MAX_ENUM :: 2147483647;

        VK_PIPELINE_CACHE_HEADER_VERSION_ONE :: ONE;
        VK_PIPELINE_CACHE_HEADER_VERSION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PIPELINE_CACHE_HEADER_VERSION_END_RANGE :: END_RANGE;
        VK_PIPELINE_CACHE_HEADER_VERSION_RANGE_SIZE :: RANGE_SIZE;
        VK_PIPELINE_CACHE_HEADER_VERSION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPipelineColorBlendAdvancedStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT;
        pNext: *void;
        srcPremultiplied: VkBool32;
        dstPremultiplied: VkBool32;
        blendOverlap: VkBlendOverlapEXT;
    }

}

#if OS == .LINUX {
    VkPipelineColorBlendAdvancedStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT;
        pNext: *void;
        srcPremultiplied: VkBool32;
        dstPremultiplied: VkBool32;
        blendOverlap: VkBlendOverlapEXT;
    }

}

#if OS == .WINDOWS {
    VkPipelineColorBlendAttachmentState :: struct {
        blendEnable: VkBool32;
        srcColorBlendFactor: VkBlendFactor;
        dstColorBlendFactor: VkBlendFactor;
        colorBlendOp: VkBlendOp;
        srcAlphaBlendFactor: VkBlendFactor;
        dstAlphaBlendFactor: VkBlendFactor;
        alphaBlendOp: VkBlendOp;
        colorWriteMask: VkColorComponentFlags;
    }

}

#if OS == .LINUX {
    VkPipelineColorBlendAttachmentState :: struct {
        blendEnable: VkBool32;
        srcColorBlendFactor: VkBlendFactor;
        dstColorBlendFactor: VkBlendFactor;
        colorBlendOp: VkBlendOp;
        srcAlphaBlendFactor: VkBlendFactor;
        dstAlphaBlendFactor: VkBlendFactor;
        alphaBlendOp: VkBlendOp;
        colorWriteMask: VkColorComponentFlags;
    }

}

#if OS == .WINDOWS {
    VkPipelineColorBlendStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineColorBlendStateCreateFlags;
        logicOpEnable: VkBool32;
        logicOp: VkLogicOp;
        attachmentCount: u32;
        pAttachments: *VkPipelineColorBlendAttachmentState;
        blendConstants: [4] float;
    }

}

#if OS == .LINUX {
    VkPipelineColorBlendStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineColorBlendStateCreateFlags;
        logicOpEnable: VkBool32;
        logicOp: VkLogicOp;
        attachmentCount: u32;
        pAttachments: *VkPipelineColorBlendAttachmentState;
        blendConstants: [4] float;
    }

}

#if OS == .WINDOWS {
    VkPipelineCoverageModulationStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineCoverageModulationStateCreateFlagsNV;
        coverageModulationMode: VkCoverageModulationModeNV;
        coverageModulationTableEnable: VkBool32;
        coverageModulationTableCount: u32;
        pCoverageModulationTable: *float;
    }

}

#if OS == .LINUX {
    VkPipelineCoverageModulationStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineCoverageModulationStateCreateFlagsNV;
        coverageModulationMode: VkCoverageModulationModeNV;
        coverageModulationTableEnable: VkBool32;
        coverageModulationTableCount: u32;
        pCoverageModulationTable: *float;
    }

}

#if OS == .WINDOWS {
    VkPipelineCoverageToColorStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineCoverageToColorStateCreateFlagsNV;
        coverageToColorEnable: VkBool32;
        coverageToColorLocation: u32;
    }

}

#if OS == .LINUX {
    VkPipelineCoverageToColorStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineCoverageToColorStateCreateFlagsNV;
        coverageToColorEnable: VkBool32;
        coverageToColorLocation: u32;
    }

}

#if OS == .WINDOWS {
    VkPipelineCreateFlagBits :: enum_flags u32 {
        DISABLE_OPTIMIZATION_BIT :: 0x1;
        ALLOW_DERIVATIVES_BIT :: 0x2;
        DERIVATIVE_BIT :: 0x4;
        VIEW_INDEX_FROM_DEVICE_INDEX_BIT :: 0x8;
        DISPATCH_BASE :: 0x10;
        DEFER_COMPILE_BIT_NVX :: 0x20;
        VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR :: 0x8;
        DISPATCH_BASE_KHR :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT :: DISABLE_OPTIMIZATION_BIT;
        VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT :: ALLOW_DERIVATIVES_BIT;
        VK_PIPELINE_CREATE_DERIVATIVE_BIT :: DERIVATIVE_BIT;
        VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT :: VIEW_INDEX_FROM_DEVICE_INDEX_BIT;
        VK_PIPELINE_CREATE_DISPATCH_BASE :: DISPATCH_BASE;
        VK_PIPELINE_CREATE_DEFER_COMPILE_BIT_NVX :: DEFER_COMPILE_BIT_NVX;
        VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR :: VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR;
        VK_PIPELINE_CREATE_DISPATCH_BASE_KHR :: DISPATCH_BASE_KHR;
        VK_PIPELINE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPipelineCreateFlagBits :: enum_flags u32 {
        DISABLE_OPTIMIZATION_BIT :: 0x1;
        ALLOW_DERIVATIVES_BIT :: 0x2;
        DERIVATIVE_BIT :: 0x4;
        VIEW_INDEX_FROM_DEVICE_INDEX_BIT :: 0x8;
        DISPATCH_BASE :: 0x10;
        DEFER_COMPILE_BIT_NVX :: 0x20;
        VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR :: 0x8;
        DISPATCH_BASE_KHR :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PIPELINE_CREATE_DISABLE_OPTIMIZATION_BIT :: DISABLE_OPTIMIZATION_BIT;
        VK_PIPELINE_CREATE_ALLOW_DERIVATIVES_BIT :: ALLOW_DERIVATIVES_BIT;
        VK_PIPELINE_CREATE_DERIVATIVE_BIT :: DERIVATIVE_BIT;
        VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT :: VIEW_INDEX_FROM_DEVICE_INDEX_BIT;
        VK_PIPELINE_CREATE_DISPATCH_BASE :: DISPATCH_BASE;
        VK_PIPELINE_CREATE_DEFER_COMPILE_BIT_NVX :: DEFER_COMPILE_BIT_NVX;
        VK_PIPELINE_CREATE_VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR :: VIEW_INDEX_FROM_DEVICE_INDEX_BIT_KHR;
        VK_PIPELINE_CREATE_DISPATCH_BASE_KHR :: DISPATCH_BASE_KHR;
        VK_PIPELINE_CREATE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPipelineDepthStencilStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineDepthStencilStateCreateFlags;
        depthTestEnable: VkBool32;
        depthWriteEnable: VkBool32;
        depthCompareOp: VkCompareOp;
        depthBoundsTestEnable: VkBool32;
        stencilTestEnable: VkBool32;
        front: VkStencilOpState;
        back: VkStencilOpState;
        minDepthBounds: float;
        maxDepthBounds: float;
    }

}

#if OS == .LINUX {
    VkPipelineDepthStencilStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineDepthStencilStateCreateFlags;
        depthTestEnable: VkBool32;
        depthWriteEnable: VkBool32;
        depthCompareOp: VkCompareOp;
        depthBoundsTestEnable: VkBool32;
        stencilTestEnable: VkBool32;
        front: VkStencilOpState;
        back: VkStencilOpState;
        minDepthBounds: float;
        maxDepthBounds: float;
    }

}

#if OS == .WINDOWS {
    VkPipelineDiscardRectangleStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkPipelineDiscardRectangleStateCreateFlagsEXT;
        discardRectangleMode: VkDiscardRectangleModeEXT;
        discardRectangleCount: u32;
        pDiscardRectangles: *VkRect2D;
    }

}

#if OS == .LINUX {
    VkPipelineDiscardRectangleStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkPipelineDiscardRectangleStateCreateFlagsEXT;
        discardRectangleMode: VkDiscardRectangleModeEXT;
        discardRectangleCount: u32;
        pDiscardRectangles: *VkRect2D;
    }

}

#if OS == .WINDOWS {
    VkPipelineDynamicStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_DYNAMIC_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineDynamicStateCreateFlags;
        dynamicStateCount: u32;
        pDynamicStates: *VkDynamicState;
    }

}

#if OS == .LINUX {
    VkPipelineDynamicStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_DYNAMIC_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineDynamicStateCreateFlags;
        dynamicStateCount: u32;
        pDynamicStates: *VkDynamicState;
    }

}

#if OS == .WINDOWS {
    VkPipelineInputAssemblyStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineInputAssemblyStateCreateFlags;
        topology: VkPrimitiveTopology;
        primitiveRestartEnable: VkBool32;
    }

}

#if OS == .LINUX {
    VkPipelineInputAssemblyStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineInputAssemblyStateCreateFlags;
        topology: VkPrimitiveTopology;
        primitiveRestartEnable: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPipelineLayout_T :: struct {}

}

#if OS == .LINUX {
    VkPipelineLayout_T :: struct {}

}

#if OS == .WINDOWS {
    VkPipelineLayoutCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_LAYOUT_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineLayoutCreateFlags;
        setLayoutCount: u32;
        pSetLayouts: *VkDescriptorSetLayout;
        pushConstantRangeCount: u32;
        pPushConstantRanges: *VkPushConstantRange;
    }

}

#if OS == .LINUX {
    VkPipelineLayoutCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_LAYOUT_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineLayoutCreateFlags;
        setLayoutCount: u32;
        pSetLayouts: *VkDescriptorSetLayout;
        pushConstantRangeCount: u32;
        pPushConstantRanges: *VkPushConstantRange;
    }

}

#if OS == .WINDOWS {
    VkPipelineMultisampleStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineMultisampleStateCreateFlags;
        rasterizationSamples: VkSampleCountFlagBits;
        sampleShadingEnable: VkBool32;
        minSampleShading: float;
        pSampleMask: *VkSampleMask;
        alphaToCoverageEnable: VkBool32;
        alphaToOneEnable: VkBool32;
    }

}

#if OS == .LINUX {
    VkPipelineMultisampleStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineMultisampleStateCreateFlags;
        rasterizationSamples: VkSampleCountFlagBits;
        sampleShadingEnable: VkBool32;
        minSampleShading: float;
        pSampleMask: *VkSampleMask;
        alphaToCoverageEnable: VkBool32;
        alphaToOneEnable: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPipelineRasterizationConservativeStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkPipelineRasterizationConservativeStateCreateFlagsEXT;
        conservativeRasterizationMode: VkConservativeRasterizationModeEXT;
        extraPrimitiveOverestimationSize: float;
    }

}

#if OS == .LINUX {
    VkPipelineRasterizationConservativeStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkPipelineRasterizationConservativeStateCreateFlagsEXT;
        conservativeRasterizationMode: VkConservativeRasterizationModeEXT;
        extraPrimitiveOverestimationSize: float;
    }

}

#if OS == .WINDOWS {
    VkPipelineRasterizationStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineRasterizationStateCreateFlags;
        depthClampEnable: VkBool32;
        rasterizerDiscardEnable: VkBool32;
        polygonMode: VkPolygonMode;
        cullMode: VkCullModeFlags;
        frontFace: VkFrontFace;
        depthBiasEnable: VkBool32;
        depthBiasConstantFactor: float;
        depthBiasClamp: float;
        depthBiasSlopeFactor: float;
        lineWidth: float;
    }

}

#if OS == .LINUX {
    VkPipelineRasterizationStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineRasterizationStateCreateFlags;
        depthClampEnable: VkBool32;
        rasterizerDiscardEnable: VkBool32;
        polygonMode: VkPolygonMode;
        cullMode: VkCullModeFlags;
        frontFace: VkFrontFace;
        depthBiasEnable: VkBool32;
        depthBiasConstantFactor: float;
        depthBiasClamp: float;
        depthBiasSlopeFactor: float;
        lineWidth: float;
    }

}

#if OS == .WINDOWS {
    VkPipelineRasterizationStateRasterizationOrderAMD :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD;
        pNext: *void;
        rasterizationOrder: VkRasterizationOrderAMD;
    }

}

#if OS == .LINUX {
    VkPipelineRasterizationStateRasterizationOrderAMD :: struct {
        sType: VkStructureType = .PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD;
        pNext: *void;
        rasterizationOrder: VkRasterizationOrderAMD;
    }

}

#if OS == .WINDOWS {
    VkPipelineRepresentativeFragmentTestStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV;
        pNext: *void;
        representativeFragmentTestEnable: VkBool32;
    }

}

#if OS == .LINUX {
    VkPipelineRepresentativeFragmentTestStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV;
        pNext: *void;
        representativeFragmentTestEnable: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPipelineSampleLocationsStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT;
        pNext: *void;
        sampleLocationsEnable: VkBool32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .LINUX {
    VkPipelineSampleLocationsStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT;
        pNext: *void;
        sampleLocationsEnable: VkBool32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .WINDOWS {
    VkPipelineShaderStageCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_SHADER_STAGE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineShaderStageCreateFlags;
        stage: VkShaderStageFlagBits;
        module: VkShaderModule;
        pName: *u8;
        pSpecializationInfo: *VkSpecializationInfo;
    }

}

#if OS == .LINUX {
    VkPipelineShaderStageCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_SHADER_STAGE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineShaderStageCreateFlags;
        stage: VkShaderStageFlagBits;
        module: VkShaderModule;
        pName: *u8;
        pSpecializationInfo: *VkSpecializationInfo;
    }

}

#if OS == .WINDOWS {
    VkPipelineStageFlagBits :: enum_flags u32 {
        TOP_OF_PIPE_BIT :: 0x1;
        DRAW_INDIRECT_BIT :: 0x2;
        VERTEX_INPUT_BIT :: 0x4;
        VERTEX_SHADER_BIT :: 0x8;
        TESSELLATION_CONTROL_SHADER_BIT :: 0x10;
        TESSELLATION_EVALUATION_SHADER_BIT :: 0x20;
        GEOMETRY_SHADER_BIT :: 0x40;
        FRAGMENT_SHADER_BIT :: 0x80;
        EARLY_FRAGMENT_TESTS_BIT :: 0x100;
        LATE_FRAGMENT_TESTS_BIT :: 0x200;
        COLOR_ATTACHMENT_OUTPUT_BIT :: 0x400;
        COMPUTE_SHADER_BIT :: 0x800;
        TRANSFER_BIT :: 0x1000;
        BOTTOM_OF_PIPE_BIT :: 0x2000;
        HOST_BIT :: 0x4000;
        ALL_GRAPHICS_BIT :: 0x8000;
        ALL_COMMANDS_BIT :: 0x10000;
        CONDITIONAL_RENDERING_BIT_EXT :: 0x40000;
        COMMAND_PROCESS_BIT_NVX :: 0x20000;
        SHADING_RATE_IMAGE_BIT_NV :: 0x400000;
        RAYTRACING_BIT_NVX :: 0x200000;
        TASK_SHADER_BIT_NV :: 0x80000;
        MESH_SHADER_BIT_NV :: 0x100000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT :: TOP_OF_PIPE_BIT;
        VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT :: DRAW_INDIRECT_BIT;
        VK_PIPELINE_STAGE_VERTEX_INPUT_BIT :: VERTEX_INPUT_BIT;
        VK_PIPELINE_STAGE_VERTEX_SHADER_BIT :: VERTEX_SHADER_BIT;
        VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT :: TESSELLATION_CONTROL_SHADER_BIT;
        VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT :: TESSELLATION_EVALUATION_SHADER_BIT;
        VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT :: GEOMETRY_SHADER_BIT;
        VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT :: FRAGMENT_SHADER_BIT;
        VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT :: EARLY_FRAGMENT_TESTS_BIT;
        VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT :: LATE_FRAGMENT_TESTS_BIT;
        VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT :: COLOR_ATTACHMENT_OUTPUT_BIT;
        VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT :: COMPUTE_SHADER_BIT;
        VK_PIPELINE_STAGE_TRANSFER_BIT :: TRANSFER_BIT;
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT :: BOTTOM_OF_PIPE_BIT;
        VK_PIPELINE_STAGE_HOST_BIT :: HOST_BIT;
        VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT :: ALL_GRAPHICS_BIT;
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT :: ALL_COMMANDS_BIT;
        VK_PIPELINE_STAGE_CONDITIONAL_RENDERING_BIT_EXT :: CONDITIONAL_RENDERING_BIT_EXT;
        VK_PIPELINE_STAGE_COMMAND_PROCESS_BIT_NVX :: COMMAND_PROCESS_BIT_NVX;
        VK_PIPELINE_STAGE_SHADING_RATE_IMAGE_BIT_NV :: SHADING_RATE_IMAGE_BIT_NV;
        VK_PIPELINE_STAGE_RAYTRACING_BIT_NVX :: RAYTRACING_BIT_NVX;
        VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV :: TASK_SHADER_BIT_NV;
        VK_PIPELINE_STAGE_MESH_SHADER_BIT_NV :: MESH_SHADER_BIT_NV;
        VK_PIPELINE_STAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPipelineStageFlagBits :: enum_flags u32 {
        TOP_OF_PIPE_BIT :: 0x1;
        DRAW_INDIRECT_BIT :: 0x2;
        VERTEX_INPUT_BIT :: 0x4;
        VERTEX_SHADER_BIT :: 0x8;
        TESSELLATION_CONTROL_SHADER_BIT :: 0x10;
        TESSELLATION_EVALUATION_SHADER_BIT :: 0x20;
        GEOMETRY_SHADER_BIT :: 0x40;
        FRAGMENT_SHADER_BIT :: 0x80;
        EARLY_FRAGMENT_TESTS_BIT :: 0x100;
        LATE_FRAGMENT_TESTS_BIT :: 0x200;
        COLOR_ATTACHMENT_OUTPUT_BIT :: 0x400;
        COMPUTE_SHADER_BIT :: 0x800;
        TRANSFER_BIT :: 0x1000;
        BOTTOM_OF_PIPE_BIT :: 0x2000;
        HOST_BIT :: 0x4000;
        ALL_GRAPHICS_BIT :: 0x8000;
        ALL_COMMANDS_BIT :: 0x10000;
        CONDITIONAL_RENDERING_BIT_EXT :: 0x40000;
        COMMAND_PROCESS_BIT_NVX :: 0x20000;
        SHADING_RATE_IMAGE_BIT_NV :: 0x400000;
        RAYTRACING_BIT_NVX :: 0x200000;
        TASK_SHADER_BIT_NV :: 0x80000;
        MESH_SHADER_BIT_NV :: 0x100000;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT :: TOP_OF_PIPE_BIT;
        VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT :: DRAW_INDIRECT_BIT;
        VK_PIPELINE_STAGE_VERTEX_INPUT_BIT :: VERTEX_INPUT_BIT;
        VK_PIPELINE_STAGE_VERTEX_SHADER_BIT :: VERTEX_SHADER_BIT;
        VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT :: TESSELLATION_CONTROL_SHADER_BIT;
        VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT :: TESSELLATION_EVALUATION_SHADER_BIT;
        VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT :: GEOMETRY_SHADER_BIT;
        VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT :: FRAGMENT_SHADER_BIT;
        VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT :: EARLY_FRAGMENT_TESTS_BIT;
        VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT :: LATE_FRAGMENT_TESTS_BIT;
        VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT :: COLOR_ATTACHMENT_OUTPUT_BIT;
        VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT :: COMPUTE_SHADER_BIT;
        VK_PIPELINE_STAGE_TRANSFER_BIT :: TRANSFER_BIT;
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT :: BOTTOM_OF_PIPE_BIT;
        VK_PIPELINE_STAGE_HOST_BIT :: HOST_BIT;
        VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT :: ALL_GRAPHICS_BIT;
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT :: ALL_COMMANDS_BIT;
        VK_PIPELINE_STAGE_CONDITIONAL_RENDERING_BIT_EXT :: CONDITIONAL_RENDERING_BIT_EXT;
        VK_PIPELINE_STAGE_COMMAND_PROCESS_BIT_NVX :: COMMAND_PROCESS_BIT_NVX;
        VK_PIPELINE_STAGE_SHADING_RATE_IMAGE_BIT_NV :: SHADING_RATE_IMAGE_BIT_NV;
        VK_PIPELINE_STAGE_RAYTRACING_BIT_NVX :: RAYTRACING_BIT_NVX;
        VK_PIPELINE_STAGE_TASK_SHADER_BIT_NV :: TASK_SHADER_BIT_NV;
        VK_PIPELINE_STAGE_MESH_SHADER_BIT_NV :: MESH_SHADER_BIT_NV;
        VK_PIPELINE_STAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPipelineTessellationDomainOriginStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO;
        pNext: *void;
        domainOrigin: VkTessellationDomainOrigin;
    }

}

#if OS == .LINUX {
    VkPipelineTessellationDomainOriginStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO;
        pNext: *void;
        domainOrigin: VkTessellationDomainOrigin;
    }

}

#if OS == .WINDOWS {
    VkPipelineTessellationStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_TESSELLATION_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineTessellationStateCreateFlags;
        patchControlPoints: u32;
    }

}

#if OS == .LINUX {
    VkPipelineTessellationStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_TESSELLATION_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineTessellationStateCreateFlags;
        patchControlPoints: u32;
    }

}

#if OS == .WINDOWS {
    VkPipelineVertexInputDivisorStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT;
        pNext: *void;
        vertexBindingDivisorCount: u32;
        pVertexBindingDivisors: *VkVertexInputBindingDivisorDescriptionEXT;
    }

}

#if OS == .LINUX {
    VkPipelineVertexInputDivisorStateCreateInfoEXT :: struct {
        sType: VkStructureType = .PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT;
        pNext: *void;
        vertexBindingDivisorCount: u32;
        pVertexBindingDivisors: *VkVertexInputBindingDivisorDescriptionEXT;
    }

}

#if OS == .WINDOWS {
    VkPipelineVertexInputStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineVertexInputStateCreateFlags;
        vertexBindingDescriptionCount: u32;
        pVertexBindingDescriptions: *VkVertexInputBindingDescription;
        vertexAttributeDescriptionCount: u32;
        pVertexAttributeDescriptions: *VkVertexInputAttributeDescription;
    }

}

#if OS == .LINUX {
    VkPipelineVertexInputStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineVertexInputStateCreateFlags;
        vertexBindingDescriptionCount: u32;
        pVertexBindingDescriptions: *VkVertexInputBindingDescription;
        vertexAttributeDescriptionCount: u32;
        pVertexAttributeDescriptions: *VkVertexInputAttributeDescription;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportCoarseSampleOrderStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV;
        pNext: *void;
        sampleOrderType: VkCoarseSampleOrderTypeNV;
        customSampleOrderCount: u32;
        pCustomSampleOrders: *VkCoarseSampleOrderCustomNV;
    }

}

#if OS == .LINUX {
    VkPipelineViewportCoarseSampleOrderStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV;
        pNext: *void;
        sampleOrderType: VkCoarseSampleOrderTypeNV;
        customSampleOrderCount: u32;
        pCustomSampleOrders: *VkCoarseSampleOrderCustomNV;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportExclusiveScissorStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV;
        pNext: *void;
        exclusiveScissorCount: u32;
        pExclusiveScissors: *VkRect2D;
    }

}

#if OS == .LINUX {
    VkPipelineViewportExclusiveScissorStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV;
        pNext: *void;
        exclusiveScissorCount: u32;
        pExclusiveScissors: *VkRect2D;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportShadingRateImageStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV;
        pNext: *void;
        shadingRateImageEnable: VkBool32;
        viewportCount: u32;
        pShadingRatePalettes: *VkShadingRatePaletteNV;
    }

}

#if OS == .LINUX {
    VkPipelineViewportShadingRateImageStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV;
        pNext: *void;
        shadingRateImageEnable: VkBool32;
        viewportCount: u32;
        pShadingRatePalettes: *VkShadingRatePaletteNV;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineViewportStateCreateFlags;
        viewportCount: u32;
        pViewports: *VkViewport;
        scissorCount: u32;
        pScissors: *VkRect2D;
    }

}

#if OS == .LINUX {
    VkPipelineViewportStateCreateInfo :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_STATE_CREATE_INFO;
        pNext: *void;
        flags: VkPipelineViewportStateCreateFlags;
        viewportCount: u32;
        pViewports: *VkViewport;
        scissorCount: u32;
        pScissors: *VkRect2D;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportSwizzleStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineViewportSwizzleStateCreateFlagsNV;
        viewportCount: u32;
        pViewportSwizzles: *VkViewportSwizzleNV;
    }

}

#if OS == .LINUX {
    VkPipelineViewportSwizzleStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV;
        pNext: *void;
        flags: VkPipelineViewportSwizzleStateCreateFlagsNV;
        viewportCount: u32;
        pViewportSwizzles: *VkViewportSwizzleNV;
    }

}

#if OS == .WINDOWS {
    VkPipelineViewportWScalingStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV;
        pNext: *void;
        viewportWScalingEnable: VkBool32;
        viewportCount: u32;
        pViewportWScalings: *VkViewportWScalingNV;
    }

}

#if OS == .LINUX {
    VkPipelineViewportWScalingStateCreateInfoNV :: struct {
        sType: VkStructureType = .PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV;
        pNext: *void;
        viewportWScalingEnable: VkBool32;
        viewportCount: u32;
        pViewportWScalings: *VkViewportWScalingNV;
    }

}

#if OS == .WINDOWS {
    VkPointClippingBehavior :: enum u32 {
        ALL_CLIP_PLANES :: 0;
        USER_CLIP_PLANES_ONLY :: 1;
        ALL_CLIP_PLANES_KHR :: 0;
        USER_CLIP_PLANES_ONLY_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_POINT_CLIPPING_BEHAVIOR_ALL_CLIP_PLANES :: ALL_CLIP_PLANES;
        VK_POINT_CLIPPING_BEHAVIOR_USER_CLIP_PLANES_ONLY :: USER_CLIP_PLANES_ONLY;
        VK_POINT_CLIPPING_BEHAVIOR_ALL_CLIP_PLANES_KHR :: ALL_CLIP_PLANES_KHR;
        VK_POINT_CLIPPING_BEHAVIOR_USER_CLIP_PLANES_ONLY_KHR :: USER_CLIP_PLANES_ONLY_KHR;
        VK_POINT_CLIPPING_BEHAVIOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_POINT_CLIPPING_BEHAVIOR_END_RANGE :: END_RANGE;
        VK_POINT_CLIPPING_BEHAVIOR_RANGE_SIZE :: RANGE_SIZE;
        VK_POINT_CLIPPING_BEHAVIOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPointClippingBehavior :: enum u32 {
        ALL_CLIP_PLANES :: 0;
        USER_CLIP_PLANES_ONLY :: 1;
        ALL_CLIP_PLANES_KHR :: 0;
        USER_CLIP_PLANES_ONLY_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_POINT_CLIPPING_BEHAVIOR_ALL_CLIP_PLANES :: ALL_CLIP_PLANES;
        VK_POINT_CLIPPING_BEHAVIOR_USER_CLIP_PLANES_ONLY :: USER_CLIP_PLANES_ONLY;
        VK_POINT_CLIPPING_BEHAVIOR_ALL_CLIP_PLANES_KHR :: ALL_CLIP_PLANES_KHR;
        VK_POINT_CLIPPING_BEHAVIOR_USER_CLIP_PLANES_ONLY_KHR :: USER_CLIP_PLANES_ONLY_KHR;
        VK_POINT_CLIPPING_BEHAVIOR_BEGIN_RANGE :: BEGIN_RANGE;
        VK_POINT_CLIPPING_BEHAVIOR_END_RANGE :: END_RANGE;
        VK_POINT_CLIPPING_BEHAVIOR_RANGE_SIZE :: RANGE_SIZE;
        VK_POINT_CLIPPING_BEHAVIOR_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPolygonMode :: enum u32 {
        FILL :: 0;
        LINE :: 1;
        POINT :: 2;
        FILL_RECTANGLE_NV :: 1000153000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_POLYGON_MODE_FILL :: FILL;
        VK_POLYGON_MODE_LINE :: LINE;
        VK_POLYGON_MODE_POINT :: POINT;
        VK_POLYGON_MODE_FILL_RECTANGLE_NV :: FILL_RECTANGLE_NV;
        VK_POLYGON_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_POLYGON_MODE_END_RANGE :: END_RANGE;
        VK_POLYGON_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_POLYGON_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPolygonMode :: enum u32 {
        FILL :: 0;
        LINE :: 1;
        POINT :: 2;
        FILL_RECTANGLE_NV :: 1000153000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_POLYGON_MODE_FILL :: FILL;
        VK_POLYGON_MODE_LINE :: LINE;
        VK_POLYGON_MODE_POINT :: POINT;
        VK_POLYGON_MODE_FILL_RECTANGLE_NV :: FILL_RECTANGLE_NV;
        VK_POLYGON_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_POLYGON_MODE_END_RANGE :: END_RANGE;
        VK_POLYGON_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_POLYGON_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkPresentInfoKHR :: struct {
        sType: VkStructureType = .PRESENT_INFO_KHR;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        swapchainCount: u32;
        pSwapchains: *VkSwapchainKHR;
        pImageIndices: *u32;
        pResults: *VkResult;
    }

}

#if OS == .LINUX {
    VkPresentInfoKHR :: struct {
        sType: VkStructureType = .PRESENT_INFO_KHR;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        swapchainCount: u32;
        pSwapchains: *VkSwapchainKHR;
        pImageIndices: *u32;
        pResults: *VkResult;
    }

}

#if OS == .WINDOWS {
    VkPresentModeKHR :: enum u32 {
        IMMEDIATE_KHR :: 0;
        MAILBOX_KHR :: 1;
        FIFO_KHR :: 2;
        FIFO_RELAXED_KHR :: 3;
        SHARED_DEMAND_REFRESH_KHR :: 1000111000;
        SHARED_CONTINUOUS_REFRESH_KHR :: 1000111001;
        BEGIN_RANGE_KHR :: 0;
        END_RANGE_KHR :: 3;
        RANGE_SIZE_KHR :: 4;
        MAX_ENUM_KHR :: 2147483647;

        VK_PRESENT_MODE_IMMEDIATE_KHR :: IMMEDIATE_KHR;
        VK_PRESENT_MODE_MAILBOX_KHR :: MAILBOX_KHR;
        VK_PRESENT_MODE_FIFO_KHR :: FIFO_KHR;
        VK_PRESENT_MODE_FIFO_RELAXED_KHR :: FIFO_RELAXED_KHR;
        VK_PRESENT_MODE_SHARED_DEMAND_REFRESH_KHR :: SHARED_DEMAND_REFRESH_KHR;
        VK_PRESENT_MODE_SHARED_CONTINUOUS_REFRESH_KHR :: SHARED_CONTINUOUS_REFRESH_KHR;
        VK_PRESENT_MODE_BEGIN_RANGE_KHR :: BEGIN_RANGE_KHR;
        VK_PRESENT_MODE_END_RANGE_KHR :: END_RANGE_KHR;
        VK_PRESENT_MODE_RANGE_SIZE_KHR :: RANGE_SIZE_KHR;
        VK_PRESENT_MODE_MAX_ENUM_KHR :: MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkPresentModeKHR :: enum u32 {
        IMMEDIATE_KHR :: 0;
        MAILBOX_KHR :: 1;
        FIFO_KHR :: 2;
        FIFO_RELAXED_KHR :: 3;
        SHARED_DEMAND_REFRESH_KHR :: 1000111000;
        SHARED_CONTINUOUS_REFRESH_KHR :: 1000111001;
        BEGIN_RANGE_KHR :: 0;
        END_RANGE_KHR :: 3;
        RANGE_SIZE_KHR :: 4;
        MAX_ENUM_KHR :: 2147483647;

        VK_PRESENT_MODE_IMMEDIATE_KHR :: IMMEDIATE_KHR;
        VK_PRESENT_MODE_MAILBOX_KHR :: MAILBOX_KHR;
        VK_PRESENT_MODE_FIFO_KHR :: FIFO_KHR;
        VK_PRESENT_MODE_FIFO_RELAXED_KHR :: FIFO_RELAXED_KHR;
        VK_PRESENT_MODE_SHARED_DEMAND_REFRESH_KHR :: SHARED_DEMAND_REFRESH_KHR;
        VK_PRESENT_MODE_SHARED_CONTINUOUS_REFRESH_KHR :: SHARED_CONTINUOUS_REFRESH_KHR;
        VK_PRESENT_MODE_BEGIN_RANGE_KHR :: BEGIN_RANGE_KHR;
        VK_PRESENT_MODE_END_RANGE_KHR :: END_RANGE_KHR;
        VK_PRESENT_MODE_RANGE_SIZE_KHR :: RANGE_SIZE_KHR;
        VK_PRESENT_MODE_MAX_ENUM_KHR :: MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkPresentRegionKHR :: struct {
        rectangleCount: u32;
        pRectangles: *VkRectLayerKHR;
    }

}

#if OS == .LINUX {
    VkPresentRegionKHR :: struct {
        rectangleCount: u32;
        pRectangles: *VkRectLayerKHR;
    }

}

#if OS == .WINDOWS {
    VkPresentRegionsKHR :: struct {
        sType: VkStructureType = .PRESENT_REGIONS_KHR;
        pNext: *void;
        swapchainCount: u32;
        pRegions: *VkPresentRegionKHR;
    }

}

#if OS == .LINUX {
    VkPresentRegionsKHR :: struct {
        sType: VkStructureType = .PRESENT_REGIONS_KHR;
        pNext: *void;
        swapchainCount: u32;
        pRegions: *VkPresentRegionKHR;
    }

}

#if OS == .WINDOWS {
    VkPresentTimeGOOGLE :: struct {
        presentID: u32;
        desiredPresentTime: u64;
    }

}

#if OS == .LINUX {
    VkPresentTimeGOOGLE :: struct {
        presentID: u32;
        desiredPresentTime: u64;
    }

}

#if OS == .WINDOWS {
    VkPresentTimesInfoGOOGLE :: struct {
        sType: VkStructureType = .PRESENT_TIMES_INFO_GOOGLE;
        pNext: *void;
        swapchainCount: u32;
        pTimes: *VkPresentTimeGOOGLE;
    }

}

#if OS == .LINUX {
    VkPresentTimesInfoGOOGLE :: struct {
        sType: VkStructureType = .PRESENT_TIMES_INFO_GOOGLE;
        pNext: *void;
        swapchainCount: u32;
        pTimes: *VkPresentTimeGOOGLE;
    }

}

#if OS == .WINDOWS {
    VkPrimitiveTopology :: enum u32 {
        POINT_LIST :: 0;
        LINE_LIST :: 1;
        LINE_STRIP :: 2;
        TRIANGLE_LIST :: 3;
        TRIANGLE_STRIP :: 4;
        TRIANGLE_FAN :: 5;
        LINE_LIST_WITH_ADJACENCY :: 6;
        LINE_STRIP_WITH_ADJACENCY :: 7;
        TRIANGLE_LIST_WITH_ADJACENCY :: 8;
        TRIANGLE_STRIP_WITH_ADJACENCY :: 9;
        PATCH_LIST :: 10;
        BEGIN_RANGE :: 0;
        END_RANGE :: 10;
        RANGE_SIZE :: 11;
        MAX_ENUM :: 2147483647;

        VK_PRIMITIVE_TOPOLOGY_POINT_LIST :: POINT_LIST;
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST :: LINE_LIST;
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP :: LINE_STRIP;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST :: TRIANGLE_LIST;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP :: TRIANGLE_STRIP;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN :: TRIANGLE_FAN;
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY :: LINE_LIST_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY :: LINE_STRIP_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY :: TRIANGLE_LIST_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY :: TRIANGLE_STRIP_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_PATCH_LIST :: PATCH_LIST;
        VK_PRIMITIVE_TOPOLOGY_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PRIMITIVE_TOPOLOGY_END_RANGE :: END_RANGE;
        VK_PRIMITIVE_TOPOLOGY_RANGE_SIZE :: RANGE_SIZE;
        VK_PRIMITIVE_TOPOLOGY_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkPrimitiveTopology :: enum u32 {
        POINT_LIST :: 0;
        LINE_LIST :: 1;
        LINE_STRIP :: 2;
        TRIANGLE_LIST :: 3;
        TRIANGLE_STRIP :: 4;
        TRIANGLE_FAN :: 5;
        LINE_LIST_WITH_ADJACENCY :: 6;
        LINE_STRIP_WITH_ADJACENCY :: 7;
        TRIANGLE_LIST_WITH_ADJACENCY :: 8;
        TRIANGLE_STRIP_WITH_ADJACENCY :: 9;
        PATCH_LIST :: 10;
        BEGIN_RANGE :: 0;
        END_RANGE :: 10;
        RANGE_SIZE :: 11;
        MAX_ENUM :: 2147483647;

        VK_PRIMITIVE_TOPOLOGY_POINT_LIST :: POINT_LIST;
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST :: LINE_LIST;
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP :: LINE_STRIP;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST :: TRIANGLE_LIST;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP :: TRIANGLE_STRIP;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN :: TRIANGLE_FAN;
        VK_PRIMITIVE_TOPOLOGY_LINE_LIST_WITH_ADJACENCY :: LINE_LIST_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_LINE_STRIP_WITH_ADJACENCY :: LINE_STRIP_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST_WITH_ADJACENCY :: TRIANGLE_LIST_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP_WITH_ADJACENCY :: TRIANGLE_STRIP_WITH_ADJACENCY;
        VK_PRIMITIVE_TOPOLOGY_PATCH_LIST :: PATCH_LIST;
        VK_PRIMITIVE_TOPOLOGY_BEGIN_RANGE :: BEGIN_RANGE;
        VK_PRIMITIVE_TOPOLOGY_END_RANGE :: END_RANGE;
        VK_PRIMITIVE_TOPOLOGY_RANGE_SIZE :: RANGE_SIZE;
        VK_PRIMITIVE_TOPOLOGY_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkProtectedSubmitInfo :: struct {
        sType: VkStructureType = .PROTECTED_SUBMIT_INFO;
        pNext: *void;
        protectedSubmit: VkBool32;
    }

}

#if OS == .LINUX {
    VkProtectedSubmitInfo :: struct {
        sType: VkStructureType = .PROTECTED_SUBMIT_INFO;
        pNext: *void;
        protectedSubmit: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkPushConstantRange :: struct {
        stageFlags: VkShaderStageFlags;
        offset: u32;
        size: u32;
    }

}

#if OS == .LINUX {
    VkPushConstantRange :: struct {
        stageFlags: VkShaderStageFlags;
        offset: u32;
        size: u32;
    }

}

#if OS == .WINDOWS {
    VkQueryControlFlagBits :: enum_flags u32 {
        PRECISE_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_CONTROL_PRECISE_BIT :: PRECISE_BIT;
        VK_QUERY_CONTROL_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkQueryControlFlagBits :: enum_flags u32 {
        PRECISE_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_CONTROL_PRECISE_BIT :: PRECISE_BIT;
        VK_QUERY_CONTROL_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkQueryPipelineStatisticFlagBits :: enum_flags u32 {
        INPUT_ASSEMBLY_VERTICES_BIT :: 0x1;
        INPUT_ASSEMBLY_PRIMITIVES_BIT :: 0x2;
        VERTEX_SHADER_INVOCATIONS_BIT :: 0x4;
        GEOMETRY_SHADER_INVOCATIONS_BIT :: 0x8;
        GEOMETRY_SHADER_PRIMITIVES_BIT :: 0x10;
        CLIPPING_INVOCATIONS_BIT :: 0x20;
        CLIPPING_PRIMITIVES_BIT :: 0x40;
        FRAGMENT_SHADER_INVOCATIONS_BIT :: 0x80;
        TESSELLATION_CONTROL_SHADER_PATCHES_BIT :: 0x100;
        TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT :: 0x200;
        COMPUTE_SHADER_INVOCATIONS_BIT :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT :: INPUT_ASSEMBLY_VERTICES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT :: INPUT_ASSEMBLY_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT :: VERTEX_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT :: GEOMETRY_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT :: GEOMETRY_SHADER_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT :: CLIPPING_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT :: CLIPPING_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT :: FRAGMENT_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT :: TESSELLATION_CONTROL_SHADER_PATCHES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT :: TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT :: COMPUTE_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkQueryPipelineStatisticFlagBits :: enum_flags u32 {
        INPUT_ASSEMBLY_VERTICES_BIT :: 0x1;
        INPUT_ASSEMBLY_PRIMITIVES_BIT :: 0x2;
        VERTEX_SHADER_INVOCATIONS_BIT :: 0x4;
        GEOMETRY_SHADER_INVOCATIONS_BIT :: 0x8;
        GEOMETRY_SHADER_PRIMITIVES_BIT :: 0x10;
        CLIPPING_INVOCATIONS_BIT :: 0x20;
        CLIPPING_PRIMITIVES_BIT :: 0x40;
        FRAGMENT_SHADER_INVOCATIONS_BIT :: 0x80;
        TESSELLATION_CONTROL_SHADER_PATCHES_BIT :: 0x100;
        TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT :: 0x200;
        COMPUTE_SHADER_INVOCATIONS_BIT :: 0x400;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT :: INPUT_ASSEMBLY_VERTICES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT :: INPUT_ASSEMBLY_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT :: VERTEX_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT :: GEOMETRY_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT :: GEOMETRY_SHADER_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT :: CLIPPING_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT :: CLIPPING_PRIMITIVES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT :: FRAGMENT_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT :: TESSELLATION_CONTROL_SHADER_PATCHES_BIT;
        VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT :: TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT :: COMPUTE_SHADER_INVOCATIONS_BIT;
        VK_QUERY_PIPELINE_STATISTIC_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkQueryPool_T :: struct {}

}

#if OS == .LINUX {
    VkQueryPool_T :: struct {}

}

#if OS == .WINDOWS {
    VkQueryPoolCreateInfo :: struct {
        sType: VkStructureType = .QUERY_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkQueryPoolCreateFlags;
        queryType: VkQueryType;
        queryCount: u32;
        pipelineStatistics: VkQueryPipelineStatisticFlags;
    }

}

#if OS == .LINUX {
    VkQueryPoolCreateInfo :: struct {
        sType: VkStructureType = .QUERY_POOL_CREATE_INFO;
        pNext: *void;
        flags: VkQueryPoolCreateFlags;
        queryType: VkQueryType;
        queryCount: u32;
        pipelineStatistics: VkQueryPipelineStatisticFlags;
    }

}

#if OS == .WINDOWS {
    VkQueryResultFlagBits :: enum_flags u32 {
        _64_BIT :: 0x1;
        WAIT_BIT :: 0x2;
        WITH_AVAILABILITY_BIT :: 0x4;
        PARTIAL_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_RESULT_64_BIT :: _64_BIT;
        VK_QUERY_RESULT_WAIT_BIT :: WAIT_BIT;
        VK_QUERY_RESULT_WITH_AVAILABILITY_BIT :: WITH_AVAILABILITY_BIT;
        VK_QUERY_RESULT_PARTIAL_BIT :: PARTIAL_BIT;
        VK_QUERY_RESULT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkQueryResultFlagBits :: enum_flags u32 {
        _64_BIT :: 0x1;
        WAIT_BIT :: 0x2;
        WITH_AVAILABILITY_BIT :: 0x4;
        PARTIAL_BIT :: 0x8;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUERY_RESULT_64_BIT :: _64_BIT;
        VK_QUERY_RESULT_WAIT_BIT :: WAIT_BIT;
        VK_QUERY_RESULT_WITH_AVAILABILITY_BIT :: WITH_AVAILABILITY_BIT;
        VK_QUERY_RESULT_PARTIAL_BIT :: PARTIAL_BIT;
        VK_QUERY_RESULT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkQueryType :: enum u32 {
        OCCLUSION :: 0;
        PIPELINE_STATISTICS :: 1;
        TIMESTAMP :: 2;
        COMPACTED_SIZE_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_QUERY_TYPE_OCCLUSION :: OCCLUSION;
        VK_QUERY_TYPE_PIPELINE_STATISTICS :: PIPELINE_STATISTICS;
        VK_QUERY_TYPE_TIMESTAMP :: TIMESTAMP;
        VK_QUERY_TYPE_COMPACTED_SIZE_NVX :: COMPACTED_SIZE_NVX;
        VK_QUERY_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_QUERY_TYPE_END_RANGE :: END_RANGE;
        VK_QUERY_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_QUERY_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkQueryType :: enum u32 {
        OCCLUSION :: 0;
        PIPELINE_STATISTICS :: 1;
        TIMESTAMP :: 2;
        COMPACTED_SIZE_NVX :: 1000165000;
        BEGIN_RANGE :: 0;
        END_RANGE :: 2;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_QUERY_TYPE_OCCLUSION :: OCCLUSION;
        VK_QUERY_TYPE_PIPELINE_STATISTICS :: PIPELINE_STATISTICS;
        VK_QUERY_TYPE_TIMESTAMP :: TIMESTAMP;
        VK_QUERY_TYPE_COMPACTED_SIZE_NVX :: COMPACTED_SIZE_NVX;
        VK_QUERY_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_QUERY_TYPE_END_RANGE :: END_RANGE;
        VK_QUERY_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_QUERY_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkQueue_T :: struct {}

}

#if OS == .LINUX {
    VkQueue_T :: struct {}

}

#if OS == .WINDOWS {
    VkQueueFamilyCheckpointPropertiesNV :: struct {
        sType: VkStructureType = .QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV;
        pNext: *void;
        checkpointExecutionStageMask: VkPipelineStageFlags;
    }

}

#if OS == .LINUX {
    VkQueueFamilyCheckpointPropertiesNV :: struct {
        sType: VkStructureType = .QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV;
        pNext: *void;
        checkpointExecutionStageMask: VkPipelineStageFlags;
    }

}

#if OS == .WINDOWS {
    VkQueueFamilyProperties :: struct {
        queueFlags: VkQueueFlags;
        queueCount: u32;
        timestampValidBits: u32;
        minImageTransferGranularity: VkExtent3D;
    }

}

#if OS == .LINUX {
    VkQueueFamilyProperties :: struct {
        queueFlags: VkQueueFlags;
        queueCount: u32;
        timestampValidBits: u32;
        minImageTransferGranularity: VkExtent3D;
    }

}

#if OS == .WINDOWS {
    VkQueueFamilyProperties2 :: struct {
        sType: VkStructureType = .QUEUE_FAMILY_PROPERTIES_2;
        pNext: *void;
        queueFamilyProperties: VkQueueFamilyProperties;
    }

}

#if OS == .LINUX {
    VkQueueFamilyProperties2 :: struct {
        sType: VkStructureType = .QUEUE_FAMILY_PROPERTIES_2;
        pNext: *void;
        queueFamilyProperties: VkQueueFamilyProperties;
    }

}

#if OS == .WINDOWS {
    VkQueueFlagBits :: enum_flags u32 {
        GRAPHICS_BIT :: 0x1;
        COMPUTE_BIT :: 0x2;
        TRANSFER_BIT :: 0x4;
        SPARSE_BINDING_BIT :: 0x8;
        PROTECTED_BIT :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUEUE_GRAPHICS_BIT :: GRAPHICS_BIT;
        VK_QUEUE_COMPUTE_BIT :: COMPUTE_BIT;
        VK_QUEUE_TRANSFER_BIT :: TRANSFER_BIT;
        VK_QUEUE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_QUEUE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_QUEUE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkQueueFlagBits :: enum_flags u32 {
        GRAPHICS_BIT :: 0x1;
        COMPUTE_BIT :: 0x2;
        TRANSFER_BIT :: 0x4;
        SPARSE_BINDING_BIT :: 0x8;
        PROTECTED_BIT :: 0x10;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_QUEUE_GRAPHICS_BIT :: GRAPHICS_BIT;
        VK_QUEUE_COMPUTE_BIT :: COMPUTE_BIT;
        VK_QUEUE_TRANSFER_BIT :: TRANSFER_BIT;
        VK_QUEUE_SPARSE_BINDING_BIT :: SPARSE_BINDING_BIT;
        VK_QUEUE_PROTECTED_BIT :: PROTECTED_BIT;
        VK_QUEUE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkQueueGlobalPriorityEXT :: enum u32 {
        LOW_EXT :: 128;
        MEDIUM_EXT :: 256;
        HIGH_EXT :: 512;
        REALTIME_EXT :: 1024;
        BEGIN_RANGE_EXT :: 128;
        END_RANGE_EXT :: 1024;
        RANGE_SIZE_EXT :: 897;
        MAX_ENUM_EXT :: 2147483647;

        VK_QUEUE_GLOBAL_PRIORITY_LOW_EXT :: LOW_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_MEDIUM_EXT :: MEDIUM_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_HIGH_EXT :: HIGH_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_REALTIME_EXT :: REALTIME_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_END_RANGE_EXT :: END_RANGE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkQueueGlobalPriorityEXT :: enum u32 {
        LOW_EXT :: 128;
        MEDIUM_EXT :: 256;
        HIGH_EXT :: 512;
        REALTIME_EXT :: 1024;
        BEGIN_RANGE_EXT :: 128;
        END_RANGE_EXT :: 1024;
        RANGE_SIZE_EXT :: 897;
        MAX_ENUM_EXT :: 2147483647;

        VK_QUEUE_GLOBAL_PRIORITY_LOW_EXT :: LOW_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_MEDIUM_EXT :: MEDIUM_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_HIGH_EXT :: HIGH_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_REALTIME_EXT :: REALTIME_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_END_RANGE_EXT :: END_RANGE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_QUEUE_GLOBAL_PRIORITY_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkRasterizationOrderAMD :: enum u32 {
        STRICT_AMD :: 0;
        RELAXED_AMD :: 1;
        BEGIN_RANGE_AMD :: 0;
        END_RANGE_AMD :: 1;
        RANGE_SIZE_AMD :: 2;
        MAX_ENUM_AMD :: 2147483647;

        VK_RASTERIZATION_ORDER_STRICT_AMD :: STRICT_AMD;
        VK_RASTERIZATION_ORDER_RELAXED_AMD :: RELAXED_AMD;
        VK_RASTERIZATION_ORDER_BEGIN_RANGE_AMD :: BEGIN_RANGE_AMD;
        VK_RASTERIZATION_ORDER_END_RANGE_AMD :: END_RANGE_AMD;
        VK_RASTERIZATION_ORDER_RANGE_SIZE_AMD :: RANGE_SIZE_AMD;
        VK_RASTERIZATION_ORDER_MAX_ENUM_AMD :: MAX_ENUM_AMD;
    }

}

#if OS == .LINUX {
    VkRasterizationOrderAMD :: enum u32 {
        STRICT_AMD :: 0;
        RELAXED_AMD :: 1;
        BEGIN_RANGE_AMD :: 0;
        END_RANGE_AMD :: 1;
        RANGE_SIZE_AMD :: 2;
        MAX_ENUM_AMD :: 2147483647;

        VK_RASTERIZATION_ORDER_STRICT_AMD :: STRICT_AMD;
        VK_RASTERIZATION_ORDER_RELAXED_AMD :: RELAXED_AMD;
        VK_RASTERIZATION_ORDER_BEGIN_RANGE_AMD :: BEGIN_RANGE_AMD;
        VK_RASTERIZATION_ORDER_END_RANGE_AMD :: END_RANGE_AMD;
        VK_RASTERIZATION_ORDER_RANGE_SIZE_AMD :: RANGE_SIZE_AMD;
        VK_RASTERIZATION_ORDER_MAX_ENUM_AMD :: MAX_ENUM_AMD;
    }

}

#if OS == .WINDOWS {
    VkRaytracingPipelineCreateInfoNVX :: struct {
        sType: VkStructureType = .RAYTRACING_PIPELINE_CREATE_INFO_NVX;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stageCount: u32;
        pStages: *VkPipelineShaderStageCreateInfo;
        pGroupNumbers: *u32;
        maxRecursionDepth: u32;
        layout: VkPipelineLayout;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .LINUX {
    VkRaytracingPipelineCreateInfoNVX :: struct {
        sType: VkStructureType = .RAYTRACING_PIPELINE_CREATE_INFO_NVX;
        pNext: *void;
        flags: VkPipelineCreateFlags;
        stageCount: u32;
        pStages: *VkPipelineShaderStageCreateInfo;
        pGroupNumbers: *u32;
        maxRecursionDepth: u32;
        layout: VkPipelineLayout;
        basePipelineHandle: VkPipeline;
        basePipelineIndex: s32;
    }

}

#if OS == .WINDOWS {
    VkRect2D :: struct {
        offset: VkOffset2D;
        extent: VkExtent2D;
    }

}

#if OS == .LINUX {
    VkRect2D :: struct {
        offset: VkOffset2D;
        extent: VkExtent2D;
    }

}

#if OS == .WINDOWS {
    VkRectLayerKHR :: struct {
        offset: VkOffset2D;
        extent: VkExtent2D;
        layer: u32;
    }

}

#if OS == .LINUX {
    VkRectLayerKHR :: struct {
        offset: VkOffset2D;
        extent: VkExtent2D;
        layer: u32;
    }

}

#if OS == .WINDOWS {
    VkRefreshCycleDurationGOOGLE :: struct {
        refreshDuration: u64;
    }

}

#if OS == .LINUX {
    VkRefreshCycleDurationGOOGLE :: struct {
        refreshDuration: u64;
    }

}

#if OS == .WINDOWS {
    VkRenderPass_T :: struct {}

}

#if OS == .LINUX {
    VkRenderPass_T :: struct {}

}

#if OS == .WINDOWS {
    VkRenderPassBeginInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_BEGIN_INFO;
        pNext: *void;
        renderPass: VkRenderPass;
        framebuffer: VkFramebuffer;
        renderArea: VkRect2D;
        clearValueCount: u32;
        pClearValues: *VkClearValue;
    }

}

#if OS == .LINUX {
    VkRenderPassBeginInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_BEGIN_INFO;
        pNext: *void;
        renderPass: VkRenderPass;
        framebuffer: VkFramebuffer;
        renderArea: VkRect2D;
        clearValueCount: u32;
        pClearValues: *VkClearValue;
    }

}

#if OS == .WINDOWS {
    VkRenderPassCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_CREATE_INFO;
        pNext: *void;
        flags: VkRenderPassCreateFlags;
        attachmentCount: u32;
        pAttachments: *VkAttachmentDescription;
        subpassCount: u32;
        pSubpasses: *VkSubpassDescription;
        dependencyCount: u32;
        pDependencies: *VkSubpassDependency;
    }

}

#if OS == .LINUX {
    VkRenderPassCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_CREATE_INFO;
        pNext: *void;
        flags: VkRenderPassCreateFlags;
        attachmentCount: u32;
        pAttachments: *VkAttachmentDescription;
        subpassCount: u32;
        pSubpasses: *VkSubpassDescription;
        dependencyCount: u32;
        pDependencies: *VkSubpassDependency;
    }

}

#if OS == .WINDOWS {
    VkRenderPassCreateInfo2KHR :: struct {
        sType: VkStructureType = .RENDER_PASS_CREATE_INFO_2_KHR;
        pNext: *void;
        flags: VkRenderPassCreateFlags;
        attachmentCount: u32;
        pAttachments: *VkAttachmentDescription2KHR;
        subpassCount: u32;
        pSubpasses: *VkSubpassDescription2KHR;
        dependencyCount: u32;
        pDependencies: *VkSubpassDependency2KHR;
        correlatedViewMaskCount: u32;
        pCorrelatedViewMasks: *u32;
    }

}

#if OS == .LINUX {
    VkRenderPassCreateInfo2KHR :: struct {
        sType: VkStructureType = .RENDER_PASS_CREATE_INFO_2_KHR;
        pNext: *void;
        flags: VkRenderPassCreateFlags;
        attachmentCount: u32;
        pAttachments: *VkAttachmentDescription2KHR;
        subpassCount: u32;
        pSubpasses: *VkSubpassDescription2KHR;
        dependencyCount: u32;
        pDependencies: *VkSubpassDependency2KHR;
        correlatedViewMaskCount: u32;
        pCorrelatedViewMasks: *u32;
    }

}

#if OS == .WINDOWS {
    VkRenderPassInputAttachmentAspectCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO;
        pNext: *void;
        aspectReferenceCount: u32;
        pAspectReferences: *VkInputAttachmentAspectReference;
    }

}

#if OS == .LINUX {
    VkRenderPassInputAttachmentAspectCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO;
        pNext: *void;
        aspectReferenceCount: u32;
        pAspectReferences: *VkInputAttachmentAspectReference;
    }

}

#if OS == .WINDOWS {
    VkRenderPassMultiviewCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_MULTIVIEW_CREATE_INFO;
        pNext: *void;
        subpassCount: u32;
        pViewMasks: *u32;
        dependencyCount: u32;
        pViewOffsets: *s32;
        correlationMaskCount: u32;
        pCorrelationMasks: *u32;
    }

}

#if OS == .LINUX {
    VkRenderPassMultiviewCreateInfo :: struct {
        sType: VkStructureType = .RENDER_PASS_MULTIVIEW_CREATE_INFO;
        pNext: *void;
        subpassCount: u32;
        pViewMasks: *u32;
        dependencyCount: u32;
        pViewOffsets: *s32;
        correlationMaskCount: u32;
        pCorrelationMasks: *u32;
    }

}

#if OS == .WINDOWS {
    VkRenderPassSampleLocationsBeginInfoEXT :: struct {
        sType: VkStructureType = .RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT;
        pNext: *void;
        attachmentInitialSampleLocationsCount: u32;
        pAttachmentInitialSampleLocations: *VkAttachmentSampleLocationsEXT;
        postSubpassSampleLocationsCount: u32;
        pPostSubpassSampleLocations: *VkSubpassSampleLocationsEXT;
    }

}

#if OS == .LINUX {
    VkRenderPassSampleLocationsBeginInfoEXT :: struct {
        sType: VkStructureType = .RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT;
        pNext: *void;
        attachmentInitialSampleLocationsCount: u32;
        pAttachmentInitialSampleLocations: *VkAttachmentSampleLocationsEXT;
        postSubpassSampleLocationsCount: u32;
        pPostSubpassSampleLocations: *VkSubpassSampleLocationsEXT;
    }

}

#if OS == .WINDOWS {
    VkResult :: enum s32 {
        SUCCESS :: 0;
        NOT_READY :: 1;
        TIMEOUT :: 2;
        EVENT_SET :: 3;
        EVENT_RESET :: 4;
        INCOMPLETE :: 5;
        ERROR_OUT_OF_HOST_MEMORY :: -1;
        ERROR_OUT_OF_DEVICE_MEMORY :: -2;
        ERROR_INITIALIZATION_FAILED :: -3;
        ERROR_DEVICE_LOST :: -4;
        ERROR_MEMORY_MAP_FAILED :: -5;
        ERROR_LAYER_NOT_PRESENT :: -6;
        ERROR_EXTENSION_NOT_PRESENT :: -7;
        ERROR_FEATURE_NOT_PRESENT :: -8;
        ERROR_INCOMPATIBLE_DRIVER :: -9;
        ERROR_TOO_MANY_OBJECTS :: -10;
        ERROR_FORMAT_NOT_SUPPORTED :: -11;
        ERROR_FRAGMENTED_POOL :: -12;
        ERROR_OUT_OF_POOL_MEMORY :: -1000069000;
        ERROR_INVALID_EXTERNAL_HANDLE :: -1000072003;
        ERROR_SURFACE_LOST_KHR :: -1000000000;
        ERROR_NATIVE_WINDOW_IN_USE_KHR :: -1000000001;
        SUBOPTIMAL_KHR :: 1000001003;
        ERROR_OUT_OF_DATE_KHR :: -1000001004;
        ERROR_INCOMPATIBLE_DISPLAY_KHR :: -1000003001;
        ERROR_VALIDATION_FAILED_EXT :: -1000011001;
        ERROR_INVALID_SHADER_NV :: -1000012000;
        ERROR_FRAGMENTATION_EXT :: -1000161000;
        ERROR_NOT_PERMITTED_EXT :: -1000174001;
        ERROR_OUT_OF_POOL_MEMORY_KHR :: -1000069000;
        ERROR_INVALID_EXTERNAL_HANDLE_KHR :: -1000072003;
        RESULT_BEGIN_RANGE :: -12;
        RESULT_END_RANGE :: 5;
        RESULT_RANGE_SIZE :: 18;
        RESULT_MAX_ENUM :: 2147483647;

        VK_SUCCESS :: SUCCESS;
        VK_NOT_READY :: NOT_READY;
        VK_TIMEOUT :: TIMEOUT;
        VK_EVENT_SET :: EVENT_SET;
        VK_EVENT_RESET :: EVENT_RESET;
        VK_INCOMPLETE :: INCOMPLETE;
        VK_ERROR_OUT_OF_HOST_MEMORY :: ERROR_OUT_OF_HOST_MEMORY;
        VK_ERROR_OUT_OF_DEVICE_MEMORY :: ERROR_OUT_OF_DEVICE_MEMORY;
        VK_ERROR_INITIALIZATION_FAILED :: ERROR_INITIALIZATION_FAILED;
        VK_ERROR_DEVICE_LOST :: ERROR_DEVICE_LOST;
        VK_ERROR_MEMORY_MAP_FAILED :: ERROR_MEMORY_MAP_FAILED;
        VK_ERROR_LAYER_NOT_PRESENT :: ERROR_LAYER_NOT_PRESENT;
        VK_ERROR_EXTENSION_NOT_PRESENT :: ERROR_EXTENSION_NOT_PRESENT;
        VK_ERROR_FEATURE_NOT_PRESENT :: ERROR_FEATURE_NOT_PRESENT;
        VK_ERROR_INCOMPATIBLE_DRIVER :: ERROR_INCOMPATIBLE_DRIVER;
        VK_ERROR_TOO_MANY_OBJECTS :: ERROR_TOO_MANY_OBJECTS;
        VK_ERROR_FORMAT_NOT_SUPPORTED :: ERROR_FORMAT_NOT_SUPPORTED;
        VK_ERROR_FRAGMENTED_POOL :: ERROR_FRAGMENTED_POOL;
        VK_ERROR_OUT_OF_POOL_MEMORY :: ERROR_OUT_OF_POOL_MEMORY;
        VK_ERROR_INVALID_EXTERNAL_HANDLE :: ERROR_INVALID_EXTERNAL_HANDLE;
        VK_ERROR_SURFACE_LOST_KHR :: ERROR_SURFACE_LOST_KHR;
        VK_ERROR_NATIVE_WINDOW_IN_USE_KHR :: ERROR_NATIVE_WINDOW_IN_USE_KHR;
        VK_SUBOPTIMAL_KHR :: SUBOPTIMAL_KHR;
        VK_ERROR_OUT_OF_DATE_KHR :: ERROR_OUT_OF_DATE_KHR;
        VK_ERROR_INCOMPATIBLE_DISPLAY_KHR :: ERROR_INCOMPATIBLE_DISPLAY_KHR;
        VK_ERROR_VALIDATION_FAILED_EXT :: ERROR_VALIDATION_FAILED_EXT;
        VK_ERROR_INVALID_SHADER_NV :: ERROR_INVALID_SHADER_NV;
        VK_ERROR_FRAGMENTATION_EXT :: ERROR_FRAGMENTATION_EXT;
        VK_ERROR_NOT_PERMITTED_EXT :: ERROR_NOT_PERMITTED_EXT;
        VK_ERROR_OUT_OF_POOL_MEMORY_KHR :: ERROR_OUT_OF_POOL_MEMORY_KHR;
        VK_ERROR_INVALID_EXTERNAL_HANDLE_KHR :: ERROR_INVALID_EXTERNAL_HANDLE_KHR;
        VK_RESULT_BEGIN_RANGE :: RESULT_BEGIN_RANGE;
        VK_RESULT_END_RANGE :: RESULT_END_RANGE;
        VK_RESULT_RANGE_SIZE :: RESULT_RANGE_SIZE;
        VK_RESULT_MAX_ENUM :: RESULT_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkResult :: enum s32 {
        SUCCESS :: 0;
        NOT_READY :: 1;
        TIMEOUT :: 2;
        EVENT_SET :: 3;
        EVENT_RESET :: 4;
        INCOMPLETE :: 5;
        ERROR_OUT_OF_HOST_MEMORY :: -1;
        ERROR_OUT_OF_DEVICE_MEMORY :: -2;
        ERROR_INITIALIZATION_FAILED :: -3;
        ERROR_DEVICE_LOST :: -4;
        ERROR_MEMORY_MAP_FAILED :: -5;
        ERROR_LAYER_NOT_PRESENT :: -6;
        ERROR_EXTENSION_NOT_PRESENT :: -7;
        ERROR_FEATURE_NOT_PRESENT :: -8;
        ERROR_INCOMPATIBLE_DRIVER :: -9;
        ERROR_TOO_MANY_OBJECTS :: -10;
        ERROR_FORMAT_NOT_SUPPORTED :: -11;
        ERROR_FRAGMENTED_POOL :: -12;
        ERROR_OUT_OF_POOL_MEMORY :: -1000069000;
        ERROR_INVALID_EXTERNAL_HANDLE :: -1000072003;
        ERROR_SURFACE_LOST_KHR :: -1000000000;
        ERROR_NATIVE_WINDOW_IN_USE_KHR :: -1000000001;
        SUBOPTIMAL_KHR :: 1000001003;
        ERROR_OUT_OF_DATE_KHR :: -1000001004;
        ERROR_INCOMPATIBLE_DISPLAY_KHR :: -1000003001;
        ERROR_VALIDATION_FAILED_EXT :: -1000011001;
        ERROR_INVALID_SHADER_NV :: -1000012000;
        ERROR_FRAGMENTATION_EXT :: -1000161000;
        ERROR_NOT_PERMITTED_EXT :: -1000174001;
        ERROR_OUT_OF_POOL_MEMORY_KHR :: -1000069000;
        ERROR_INVALID_EXTERNAL_HANDLE_KHR :: -1000072003;
        RESULT_BEGIN_RANGE :: -12;
        RESULT_END_RANGE :: 5;
        RESULT_RANGE_SIZE :: 18;
        RESULT_MAX_ENUM :: 2147483647;

        VK_SUCCESS :: SUCCESS;
        VK_NOT_READY :: NOT_READY;
        VK_TIMEOUT :: TIMEOUT;
        VK_EVENT_SET :: EVENT_SET;
        VK_EVENT_RESET :: EVENT_RESET;
        VK_INCOMPLETE :: INCOMPLETE;
        VK_ERROR_OUT_OF_HOST_MEMORY :: ERROR_OUT_OF_HOST_MEMORY;
        VK_ERROR_OUT_OF_DEVICE_MEMORY :: ERROR_OUT_OF_DEVICE_MEMORY;
        VK_ERROR_INITIALIZATION_FAILED :: ERROR_INITIALIZATION_FAILED;
        VK_ERROR_DEVICE_LOST :: ERROR_DEVICE_LOST;
        VK_ERROR_MEMORY_MAP_FAILED :: ERROR_MEMORY_MAP_FAILED;
        VK_ERROR_LAYER_NOT_PRESENT :: ERROR_LAYER_NOT_PRESENT;
        VK_ERROR_EXTENSION_NOT_PRESENT :: ERROR_EXTENSION_NOT_PRESENT;
        VK_ERROR_FEATURE_NOT_PRESENT :: ERROR_FEATURE_NOT_PRESENT;
        VK_ERROR_INCOMPATIBLE_DRIVER :: ERROR_INCOMPATIBLE_DRIVER;
        VK_ERROR_TOO_MANY_OBJECTS :: ERROR_TOO_MANY_OBJECTS;
        VK_ERROR_FORMAT_NOT_SUPPORTED :: ERROR_FORMAT_NOT_SUPPORTED;
        VK_ERROR_FRAGMENTED_POOL :: ERROR_FRAGMENTED_POOL;
        VK_ERROR_OUT_OF_POOL_MEMORY :: ERROR_OUT_OF_POOL_MEMORY;
        VK_ERROR_INVALID_EXTERNAL_HANDLE :: ERROR_INVALID_EXTERNAL_HANDLE;
        VK_ERROR_SURFACE_LOST_KHR :: ERROR_SURFACE_LOST_KHR;
        VK_ERROR_NATIVE_WINDOW_IN_USE_KHR :: ERROR_NATIVE_WINDOW_IN_USE_KHR;
        VK_SUBOPTIMAL_KHR :: SUBOPTIMAL_KHR;
        VK_ERROR_OUT_OF_DATE_KHR :: ERROR_OUT_OF_DATE_KHR;
        VK_ERROR_INCOMPATIBLE_DISPLAY_KHR :: ERROR_INCOMPATIBLE_DISPLAY_KHR;
        VK_ERROR_VALIDATION_FAILED_EXT :: ERROR_VALIDATION_FAILED_EXT;
        VK_ERROR_INVALID_SHADER_NV :: ERROR_INVALID_SHADER_NV;
        VK_ERROR_FRAGMENTATION_EXT :: ERROR_FRAGMENTATION_EXT;
        VK_ERROR_NOT_PERMITTED_EXT :: ERROR_NOT_PERMITTED_EXT;
        VK_ERROR_OUT_OF_POOL_MEMORY_KHR :: ERROR_OUT_OF_POOL_MEMORY_KHR;
        VK_ERROR_INVALID_EXTERNAL_HANDLE_KHR :: ERROR_INVALID_EXTERNAL_HANDLE_KHR;
        VK_RESULT_BEGIN_RANGE :: RESULT_BEGIN_RANGE;
        VK_RESULT_END_RANGE :: RESULT_END_RANGE;
        VK_RESULT_RANGE_SIZE :: RESULT_RANGE_SIZE;
        VK_RESULT_MAX_ENUM :: RESULT_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSampleCountFlagBits :: enum_flags u32 {
        _1_BIT :: 0x1;
        _2_BIT :: 0x2;
        _4_BIT :: 0x4;
        _8_BIT :: 0x8;
        _16_BIT :: 0x10;
        _32_BIT :: 0x20;
        _64_BIT :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SAMPLE_COUNT_1_BIT :: _1_BIT;
        VK_SAMPLE_COUNT_2_BIT :: _2_BIT;
        VK_SAMPLE_COUNT_4_BIT :: _4_BIT;
        VK_SAMPLE_COUNT_8_BIT :: _8_BIT;
        VK_SAMPLE_COUNT_16_BIT :: _16_BIT;
        VK_SAMPLE_COUNT_32_BIT :: _32_BIT;
        VK_SAMPLE_COUNT_64_BIT :: _64_BIT;
        VK_SAMPLE_COUNT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSampleCountFlagBits :: enum_flags u32 {
        _1_BIT :: 0x1;
        _2_BIT :: 0x2;
        _4_BIT :: 0x4;
        _8_BIT :: 0x8;
        _16_BIT :: 0x10;
        _32_BIT :: 0x20;
        _64_BIT :: 0x40;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SAMPLE_COUNT_1_BIT :: _1_BIT;
        VK_SAMPLE_COUNT_2_BIT :: _2_BIT;
        VK_SAMPLE_COUNT_4_BIT :: _4_BIT;
        VK_SAMPLE_COUNT_8_BIT :: _8_BIT;
        VK_SAMPLE_COUNT_16_BIT :: _16_BIT;
        VK_SAMPLE_COUNT_32_BIT :: _32_BIT;
        VK_SAMPLE_COUNT_64_BIT :: _64_BIT;
        VK_SAMPLE_COUNT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSampleLocationEXT :: struct {
        x: float;
        y: float;
    }

}

#if OS == .LINUX {
    VkSampleLocationEXT :: struct {
        x: float;
        y: float;
    }

}

#if OS == .WINDOWS {
    VkSampleLocationsInfoEXT :: struct {
        sType: VkStructureType = .SAMPLE_LOCATIONS_INFO_EXT;
        pNext: *void;
        sampleLocationsPerPixel: VkSampleCountFlagBits;
        sampleLocationGridSize: VkExtent2D;
        sampleLocationsCount: u32;
        pSampleLocations: *VkSampleLocationEXT;
    }

}

#if OS == .LINUX {
    VkSampleLocationsInfoEXT :: struct {
        sType: VkStructureType = .SAMPLE_LOCATIONS_INFO_EXT;
        pNext: *void;
        sampleLocationsPerPixel: VkSampleCountFlagBits;
        sampleLocationGridSize: VkExtent2D;
        sampleLocationsCount: u32;
        pSampleLocations: *VkSampleLocationEXT;
    }

}

#if OS == .WINDOWS {
    VkSampler_T :: struct {}

}

#if OS == .LINUX {
    VkSampler_T :: struct {}

}

#if OS == .WINDOWS {
    VkSamplerAddressMode :: enum u32 {
        REPEAT :: 0;
        MIRRORED_REPEAT :: 1;
        CLAMP_TO_EDGE :: 2;
        CLAMP_TO_BORDER :: 3;
        MIRROR_CLAMP_TO_EDGE :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 3;
        RANGE_SIZE :: 4;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_ADDRESS_MODE_REPEAT :: REPEAT;
        VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT :: MIRRORED_REPEAT;
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE :: CLAMP_TO_EDGE;
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER :: CLAMP_TO_BORDER;
        VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE :: MIRROR_CLAMP_TO_EDGE;
        VK_SAMPLER_ADDRESS_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_ADDRESS_MODE_END_RANGE :: END_RANGE;
        VK_SAMPLER_ADDRESS_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_ADDRESS_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSamplerAddressMode :: enum u32 {
        REPEAT :: 0;
        MIRRORED_REPEAT :: 1;
        CLAMP_TO_EDGE :: 2;
        CLAMP_TO_BORDER :: 3;
        MIRROR_CLAMP_TO_EDGE :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 3;
        RANGE_SIZE :: 4;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_ADDRESS_MODE_REPEAT :: REPEAT;
        VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT :: MIRRORED_REPEAT;
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE :: CLAMP_TO_EDGE;
        VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER :: CLAMP_TO_BORDER;
        VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE :: MIRROR_CLAMP_TO_EDGE;
        VK_SAMPLER_ADDRESS_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_ADDRESS_MODE_END_RANGE :: END_RANGE;
        VK_SAMPLER_ADDRESS_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_ADDRESS_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSamplerCreateInfo :: struct {
        sType: VkStructureType = .SAMPLER_CREATE_INFO;
        pNext: *void;
        flags: VkSamplerCreateFlags;
        magFilter: VkFilter;
        minFilter: VkFilter;
        mipmapMode: VkSamplerMipmapMode;
        addressModeU: VkSamplerAddressMode;
        addressModeV: VkSamplerAddressMode;
        addressModeW: VkSamplerAddressMode;
        mipLodBias: float;
        anisotropyEnable: VkBool32;
        maxAnisotropy: float;
        compareEnable: VkBool32;
        compareOp: VkCompareOp;
        minLod: float;
        maxLod: float;
        borderColor: VkBorderColor;
        unnormalizedCoordinates: VkBool32;
    }

}

#if OS == .LINUX {
    VkSamplerCreateInfo :: struct {
        sType: VkStructureType = .SAMPLER_CREATE_INFO;
        pNext: *void;
        flags: VkSamplerCreateFlags;
        magFilter: VkFilter;
        minFilter: VkFilter;
        mipmapMode: VkSamplerMipmapMode;
        addressModeU: VkSamplerAddressMode;
        addressModeV: VkSamplerAddressMode;
        addressModeW: VkSamplerAddressMode;
        mipLodBias: float;
        anisotropyEnable: VkBool32;
        maxAnisotropy: float;
        compareEnable: VkBool32;
        compareOp: VkCompareOp;
        minLod: float;
        maxLod: float;
        borderColor: VkBorderColor;
        unnormalizedCoordinates: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkSamplerMipmapMode :: enum u32 {
        NEAREST :: 0;
        LINEAR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_MIPMAP_MODE_NEAREST :: NEAREST;
        VK_SAMPLER_MIPMAP_MODE_LINEAR :: LINEAR;
        VK_SAMPLER_MIPMAP_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_MIPMAP_MODE_END_RANGE :: END_RANGE;
        VK_SAMPLER_MIPMAP_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_MIPMAP_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSamplerMipmapMode :: enum u32 {
        NEAREST :: 0;
        LINEAR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_MIPMAP_MODE_NEAREST :: NEAREST;
        VK_SAMPLER_MIPMAP_MODE_LINEAR :: LINEAR;
        VK_SAMPLER_MIPMAP_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_MIPMAP_MODE_END_RANGE :: END_RANGE;
        VK_SAMPLER_MIPMAP_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_MIPMAP_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSamplerReductionModeCreateInfoEXT :: struct {
        sType: VkStructureType = .SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT;
        pNext: *void;
        reductionMode: VkSamplerReductionModeEXT;
    }

}

#if OS == .LINUX {
    VkSamplerReductionModeCreateInfoEXT :: struct {
        sType: VkStructureType = .SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT;
        pNext: *void;
        reductionMode: VkSamplerReductionModeEXT;
    }

}

#if OS == .WINDOWS {
    VkSamplerReductionModeEXT :: enum u32 {
        WEIGHTED_AVERAGE_EXT :: 0;
        MIN_EXT :: 1;
        MAX_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_SAMPLER_REDUCTION_MODE_WEIGHTED_AVERAGE_EXT :: WEIGHTED_AVERAGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_MIN_EXT :: MIN_EXT;
        VK_SAMPLER_REDUCTION_MODE_MAX_EXT :: MAX_EXT;
        VK_SAMPLER_REDUCTION_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_SAMPLER_REDUCTION_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkSamplerReductionModeEXT :: enum u32 {
        WEIGHTED_AVERAGE_EXT :: 0;
        MIN_EXT :: 1;
        MAX_EXT :: 2;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 2;
        RANGE_SIZE_EXT :: 3;
        MAX_ENUM_EXT :: 2147483647;

        VK_SAMPLER_REDUCTION_MODE_WEIGHTED_AVERAGE_EXT :: WEIGHTED_AVERAGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_MIN_EXT :: MIN_EXT;
        VK_SAMPLER_REDUCTION_MODE_MAX_EXT :: MAX_EXT;
        VK_SAMPLER_REDUCTION_MODE_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_END_RANGE_EXT :: END_RANGE_EXT;
        VK_SAMPLER_REDUCTION_MODE_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_SAMPLER_REDUCTION_MODE_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversion_T :: struct {}

}

#if OS == .LINUX {
    VkSamplerYcbcrConversion_T :: struct {}

}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionCreateInfo :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_CREATE_INFO;
        pNext: *void;
        format: VkFormat;
        ycbcrModel: VkSamplerYcbcrModelConversion;
        ycbcrRange: VkSamplerYcbcrRange;
        components: VkComponentMapping;
        xChromaOffset: VkChromaLocation;
        yChromaOffset: VkChromaLocation;
        chromaFilter: VkFilter;
        forceExplicitReconstruction: VkBool32;
    }

}

#if OS == .LINUX {
    VkSamplerYcbcrConversionCreateInfo :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_CREATE_INFO;
        pNext: *void;
        format: VkFormat;
        ycbcrModel: VkSamplerYcbcrModelConversion;
        ycbcrRange: VkSamplerYcbcrRange;
        components: VkComponentMapping;
        xChromaOffset: VkChromaLocation;
        yChromaOffset: VkChromaLocation;
        chromaFilter: VkFilter;
        forceExplicitReconstruction: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionImageFormatProperties :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES;
        pNext: *void;
        combinedImageSamplerDescriptorCount: u32;
    }

}

#if OS == .LINUX {
    VkSamplerYcbcrConversionImageFormatProperties :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES;
        pNext: *void;
        combinedImageSamplerDescriptorCount: u32;
    }

}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionInfo :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_INFO;
        pNext: *void;
        conversion: VkSamplerYcbcrConversion;
    }

}

#if OS == .LINUX {
    VkSamplerYcbcrConversionInfo :: struct {
        sType: VkStructureType = .SAMPLER_YCBCR_CONVERSION_INFO;
        pNext: *void;
        conversion: VkSamplerYcbcrConversion;
    }

}

#if OS == .WINDOWS {
    VkSamplerYcbcrModelConversion :: enum u32 {
        RGB_IDENTITY :: 0;
        YCBCR_IDENTITY :: 1;
        YCBCR_709 :: 2;
        YCBCR_601 :: 3;
        YCBCR_2020 :: 4;
        RGB_IDENTITY_KHR :: 0;
        YCBCR_IDENTITY_KHR :: 1;
        YCBCR_709_KHR :: 2;
        YCBCR_601_KHR :: 3;
        YCBCR_2020_KHR :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RGB_IDENTITY :: RGB_IDENTITY;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_IDENTITY :: YCBCR_IDENTITY;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_709 :: YCBCR_709;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_601 :: YCBCR_601;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_2020 :: YCBCR_2020;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RGB_IDENTITY_KHR :: RGB_IDENTITY_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_IDENTITY_KHR :: YCBCR_IDENTITY_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_709_KHR :: YCBCR_709_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_601_KHR :: YCBCR_601_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_2020_KHR :: YCBCR_2020_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_END_RANGE :: END_RANGE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSamplerYcbcrModelConversion :: enum u32 {
        RGB_IDENTITY :: 0;
        YCBCR_IDENTITY :: 1;
        YCBCR_709 :: 2;
        YCBCR_601 :: 3;
        YCBCR_2020 :: 4;
        RGB_IDENTITY_KHR :: 0;
        YCBCR_IDENTITY_KHR :: 1;
        YCBCR_709_KHR :: 2;
        YCBCR_601_KHR :: 3;
        YCBCR_2020_KHR :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RGB_IDENTITY :: RGB_IDENTITY;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_IDENTITY :: YCBCR_IDENTITY;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_709 :: YCBCR_709;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_601 :: YCBCR_601;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_2020 :: YCBCR_2020;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RGB_IDENTITY_KHR :: RGB_IDENTITY_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_IDENTITY_KHR :: YCBCR_IDENTITY_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_709_KHR :: YCBCR_709_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_601_KHR :: YCBCR_601_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_YCBCR_2020_KHR :: YCBCR_2020_KHR;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_END_RANGE :: END_RANGE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_YCBCR_MODEL_CONVERSION_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSamplerYcbcrRange :: enum u32 {
        ITU_FULL :: 0;
        ITU_NARROW :: 1;
        ITU_FULL_KHR :: 0;
        ITU_NARROW_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_YCBCR_RANGE_ITU_FULL :: ITU_FULL;
        VK_SAMPLER_YCBCR_RANGE_ITU_NARROW :: ITU_NARROW;
        VK_SAMPLER_YCBCR_RANGE_ITU_FULL_KHR :: ITU_FULL_KHR;
        VK_SAMPLER_YCBCR_RANGE_ITU_NARROW_KHR :: ITU_NARROW_KHR;
        VK_SAMPLER_YCBCR_RANGE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_YCBCR_RANGE_END_RANGE :: END_RANGE;
        VK_SAMPLER_YCBCR_RANGE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_YCBCR_RANGE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSamplerYcbcrRange :: enum u32 {
        ITU_FULL :: 0;
        ITU_NARROW :: 1;
        ITU_FULL_KHR :: 0;
        ITU_NARROW_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SAMPLER_YCBCR_RANGE_ITU_FULL :: ITU_FULL;
        VK_SAMPLER_YCBCR_RANGE_ITU_NARROW :: ITU_NARROW;
        VK_SAMPLER_YCBCR_RANGE_ITU_FULL_KHR :: ITU_FULL_KHR;
        VK_SAMPLER_YCBCR_RANGE_ITU_NARROW_KHR :: ITU_NARROW_KHR;
        VK_SAMPLER_YCBCR_RANGE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SAMPLER_YCBCR_RANGE_END_RANGE :: END_RANGE;
        VK_SAMPLER_YCBCR_RANGE_RANGE_SIZE :: RANGE_SIZE;
        VK_SAMPLER_YCBCR_RANGE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSemaphore_T :: struct {}

}

#if OS == .LINUX {
    VkSemaphore_T :: struct {}

}

#if OS == .WINDOWS {
    VkSemaphoreCreateInfo :: struct {
        sType: VkStructureType = .SEMAPHORE_CREATE_INFO;
        pNext: *void;
        flags: VkSemaphoreCreateFlags;
    }

}

#if OS == .LINUX {
    VkSemaphoreCreateInfo :: struct {
        sType: VkStructureType = .SEMAPHORE_CREATE_INFO;
        pNext: *void;
        flags: VkSemaphoreCreateFlags;
    }

}

#if OS == .WINDOWS {
    VkSemaphoreGetFdInfoKHR :: struct {
        sType: VkStructureType = .SEMAPHORE_GET_FD_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
    }

}

#if OS == .LINUX {
    VkSemaphoreGetFdInfoKHR :: struct {
        sType: VkStructureType = .SEMAPHORE_GET_FD_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkSemaphoreGetWin32HandleInfoKHR :: struct {
        sType: VkStructureType = .SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR;
        pNext: *void;
        semaphore: VkSemaphore;
        handleType: VkExternalSemaphoreHandleTypeFlagBits;
    }

}

#if OS == .WINDOWS {
    VkSemaphoreImportFlagBits :: enum_flags u32 {
        TEMPORARY_BIT :: 0x1;
        TEMPORARY_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SEMAPHORE_IMPORT_TEMPORARY_BIT :: TEMPORARY_BIT;
        VK_SEMAPHORE_IMPORT_TEMPORARY_BIT_KHR :: TEMPORARY_BIT_KHR;
        VK_SEMAPHORE_IMPORT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSemaphoreImportFlagBits :: enum_flags u32 {
        TEMPORARY_BIT :: 0x1;
        TEMPORARY_BIT_KHR :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SEMAPHORE_IMPORT_TEMPORARY_BIT :: TEMPORARY_BIT;
        VK_SEMAPHORE_IMPORT_TEMPORARY_BIT_KHR :: TEMPORARY_BIT_KHR;
        VK_SEMAPHORE_IMPORT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkShaderInfoTypeAMD :: enum u32 {
        STATISTICS_AMD :: 0;
        BINARY_AMD :: 1;
        DISASSEMBLY_AMD :: 2;
        BEGIN_RANGE_AMD :: 0;
        END_RANGE_AMD :: 2;
        RANGE_SIZE_AMD :: 3;
        MAX_ENUM_AMD :: 2147483647;

        VK_SHADER_INFO_TYPE_STATISTICS_AMD :: STATISTICS_AMD;
        VK_SHADER_INFO_TYPE_BINARY_AMD :: BINARY_AMD;
        VK_SHADER_INFO_TYPE_DISASSEMBLY_AMD :: DISASSEMBLY_AMD;
        VK_SHADER_INFO_TYPE_BEGIN_RANGE_AMD :: BEGIN_RANGE_AMD;
        VK_SHADER_INFO_TYPE_END_RANGE_AMD :: END_RANGE_AMD;
        VK_SHADER_INFO_TYPE_RANGE_SIZE_AMD :: RANGE_SIZE_AMD;
        VK_SHADER_INFO_TYPE_MAX_ENUM_AMD :: MAX_ENUM_AMD;
    }

}

#if OS == .LINUX {
    VkShaderInfoTypeAMD :: enum u32 {
        STATISTICS_AMD :: 0;
        BINARY_AMD :: 1;
        DISASSEMBLY_AMD :: 2;
        BEGIN_RANGE_AMD :: 0;
        END_RANGE_AMD :: 2;
        RANGE_SIZE_AMD :: 3;
        MAX_ENUM_AMD :: 2147483647;

        VK_SHADER_INFO_TYPE_STATISTICS_AMD :: STATISTICS_AMD;
        VK_SHADER_INFO_TYPE_BINARY_AMD :: BINARY_AMD;
        VK_SHADER_INFO_TYPE_DISASSEMBLY_AMD :: DISASSEMBLY_AMD;
        VK_SHADER_INFO_TYPE_BEGIN_RANGE_AMD :: BEGIN_RANGE_AMD;
        VK_SHADER_INFO_TYPE_END_RANGE_AMD :: END_RANGE_AMD;
        VK_SHADER_INFO_TYPE_RANGE_SIZE_AMD :: RANGE_SIZE_AMD;
        VK_SHADER_INFO_TYPE_MAX_ENUM_AMD :: MAX_ENUM_AMD;
    }

}

#if OS == .WINDOWS {
    VkShaderModule_T :: struct {}

}

#if OS == .LINUX {
    VkShaderModule_T :: struct {}

}

#if OS == .WINDOWS {
    VkShaderModuleCreateInfo :: struct {
        sType: VkStructureType = .SHADER_MODULE_CREATE_INFO;
        pNext: *void;
        flags: VkShaderModuleCreateFlags;
        codeSize: u64;
        pCode: *u32;
    }

}

#if OS == .LINUX {
    VkShaderModuleCreateInfo :: struct {
        sType: VkStructureType = .SHADER_MODULE_CREATE_INFO;
        pNext: *void;
        flags: VkShaderModuleCreateFlags;
        codeSize: u64;
        pCode: *u32;
    }

}

#if OS == .WINDOWS {
    VkShaderModuleValidationCacheCreateInfoEXT :: struct {
        sType: VkStructureType = .SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT;
        pNext: *void;
        validationCache: VkValidationCacheEXT;
    }

}

#if OS == .LINUX {
    VkShaderModuleValidationCacheCreateInfoEXT :: struct {
        sType: VkStructureType = .SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT;
        pNext: *void;
        validationCache: VkValidationCacheEXT;
    }

}

#if OS == .WINDOWS {
    VkShaderResourceUsageAMD :: struct {
        numUsedVgprs: u32;
        numUsedSgprs: u32;
        ldsSizePerLocalWorkGroup: u32;
        ldsUsageSizeInBytes: u64;
        scratchMemUsageInBytes: u64;
    }

}

#if OS == .LINUX {
    VkShaderResourceUsageAMD :: struct {
        numUsedVgprs: u32;
        numUsedSgprs: u32;
        ldsSizePerLocalWorkGroup: u32;
        ldsUsageSizeInBytes: u64;
        scratchMemUsageInBytes: u64;
    }

}

#if OS == .WINDOWS {
    VkShaderStageFlagBits :: enum_flags u32 {
        VERTEX_BIT :: 0x1;
        TESSELLATION_CONTROL_BIT :: 0x2;
        TESSELLATION_EVALUATION_BIT :: 0x4;
        GEOMETRY_BIT :: 0x8;
        FRAGMENT_BIT :: 0x10;
        COMPUTE_BIT :: 0x20;
        ALL_GRAPHICS :: 0x1f;
        ALL :: 0x7fffffff;
        RAYGEN_BIT_NVX :: 0x100;
        ANY_HIT_BIT_NVX :: 0x200;
        CLOSEST_HIT_BIT_NVX :: 0x400;
        MISS_BIT_NVX :: 0x800;
        INTERSECTION_BIT_NVX :: 0x1000;
        CALLABLE_BIT_NVX :: 0x2000;
        TASK_BIT_NV :: 0x40;
        MESH_BIT_NV :: 0x80;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SHADER_STAGE_VERTEX_BIT :: VERTEX_BIT;
        VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT :: TESSELLATION_CONTROL_BIT;
        VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT :: TESSELLATION_EVALUATION_BIT;
        VK_SHADER_STAGE_GEOMETRY_BIT :: GEOMETRY_BIT;
        VK_SHADER_STAGE_FRAGMENT_BIT :: FRAGMENT_BIT;
        VK_SHADER_STAGE_COMPUTE_BIT :: COMPUTE_BIT;
        VK_SHADER_STAGE_ALL_GRAPHICS :: ALL_GRAPHICS;
        VK_SHADER_STAGE_ALL :: ALL;
        VK_SHADER_STAGE_RAYGEN_BIT_NVX :: RAYGEN_BIT_NVX;
        VK_SHADER_STAGE_ANY_HIT_BIT_NVX :: ANY_HIT_BIT_NVX;
        VK_SHADER_STAGE_CLOSEST_HIT_BIT_NVX :: CLOSEST_HIT_BIT_NVX;
        VK_SHADER_STAGE_MISS_BIT_NVX :: MISS_BIT_NVX;
        VK_SHADER_STAGE_INTERSECTION_BIT_NVX :: INTERSECTION_BIT_NVX;
        VK_SHADER_STAGE_CALLABLE_BIT_NVX :: CALLABLE_BIT_NVX;
        VK_SHADER_STAGE_TASK_BIT_NV :: TASK_BIT_NV;
        VK_SHADER_STAGE_MESH_BIT_NV :: MESH_BIT_NV;
        VK_SHADER_STAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkShaderStageFlagBits :: enum_flags u32 {
        VERTEX_BIT :: 0x1;
        TESSELLATION_CONTROL_BIT :: 0x2;
        TESSELLATION_EVALUATION_BIT :: 0x4;
        GEOMETRY_BIT :: 0x8;
        FRAGMENT_BIT :: 0x10;
        COMPUTE_BIT :: 0x20;
        ALL_GRAPHICS :: 0x1f;
        ALL :: 0x7fffffff;
        RAYGEN_BIT_NVX :: 0x100;
        ANY_HIT_BIT_NVX :: 0x200;
        CLOSEST_HIT_BIT_NVX :: 0x400;
        MISS_BIT_NVX :: 0x800;
        INTERSECTION_BIT_NVX :: 0x1000;
        CALLABLE_BIT_NVX :: 0x2000;
        TASK_BIT_NV :: 0x40;
        MESH_BIT_NV :: 0x80;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SHADER_STAGE_VERTEX_BIT :: VERTEX_BIT;
        VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT :: TESSELLATION_CONTROL_BIT;
        VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT :: TESSELLATION_EVALUATION_BIT;
        VK_SHADER_STAGE_GEOMETRY_BIT :: GEOMETRY_BIT;
        VK_SHADER_STAGE_FRAGMENT_BIT :: FRAGMENT_BIT;
        VK_SHADER_STAGE_COMPUTE_BIT :: COMPUTE_BIT;
        VK_SHADER_STAGE_ALL_GRAPHICS :: ALL_GRAPHICS;
        VK_SHADER_STAGE_ALL :: ALL;
        VK_SHADER_STAGE_RAYGEN_BIT_NVX :: RAYGEN_BIT_NVX;
        VK_SHADER_STAGE_ANY_HIT_BIT_NVX :: ANY_HIT_BIT_NVX;
        VK_SHADER_STAGE_CLOSEST_HIT_BIT_NVX :: CLOSEST_HIT_BIT_NVX;
        VK_SHADER_STAGE_MISS_BIT_NVX :: MISS_BIT_NVX;
        VK_SHADER_STAGE_INTERSECTION_BIT_NVX :: INTERSECTION_BIT_NVX;
        VK_SHADER_STAGE_CALLABLE_BIT_NVX :: CALLABLE_BIT_NVX;
        VK_SHADER_STAGE_TASK_BIT_NV :: TASK_BIT_NV;
        VK_SHADER_STAGE_MESH_BIT_NV :: MESH_BIT_NV;
        VK_SHADER_STAGE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkShaderStatisticsInfoAMD :: struct {
        shaderStageMask: VkShaderStageFlags;
        resourceUsage: VkShaderResourceUsageAMD;
        numPhysicalVgprs: u32;
        numPhysicalSgprs: u32;
        numAvailableVgprs: u32;
        numAvailableSgprs: u32;
        computeWorkGroupSize: [3] u32;
    }

}

#if OS == .LINUX {
    VkShaderStatisticsInfoAMD :: struct {
        shaderStageMask: VkShaderStageFlags;
        resourceUsage: VkShaderResourceUsageAMD;
        numPhysicalVgprs: u32;
        numPhysicalSgprs: u32;
        numAvailableVgprs: u32;
        numAvailableSgprs: u32;
        computeWorkGroupSize: [3] u32;
    }

}

#if OS == .WINDOWS {
    VkShadingRatePaletteEntryNV :: enum u32 {
        NO_INVOCATIONS_NV :: 0;
        _16_INVOCATIONS_PER_PIXEL_NV :: 1;
        _8_INVOCATIONS_PER_PIXEL_NV :: 2;
        _4_INVOCATIONS_PER_PIXEL_NV :: 3;
        _2_INVOCATIONS_PER_PIXEL_NV :: 4;
        _1_INVOCATION_PER_PIXEL_NV :: 5;
        _1_INVOCATION_PER_2X1_PIXELS_NV :: 6;
        _1_INVOCATION_PER_1X2_PIXELS_NV :: 7;
        _1_INVOCATION_PER_2X2_PIXELS_NV :: 8;
        _1_INVOCATION_PER_4X2_PIXELS_NV :: 9;
        _1_INVOCATION_PER_2X4_PIXELS_NV :: 10;
        _1_INVOCATION_PER_4X4_PIXELS_NV :: 11;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 11;
        RANGE_SIZE_NV :: 12;
        MAX_ENUM_NV :: 2147483647;

        VK_SHADING_RATE_PALETTE_ENTRY_NO_INVOCATIONS_NV :: NO_INVOCATIONS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_16_INVOCATIONS_PER_PIXEL_NV :: _16_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_8_INVOCATIONS_PER_PIXEL_NV :: _8_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_4_INVOCATIONS_PER_PIXEL_NV :: _4_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_2_INVOCATIONS_PER_PIXEL_NV :: _2_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_PIXEL_NV :: _1_INVOCATION_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X1_PIXELS_NV :: _1_INVOCATION_PER_2X1_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_1X2_PIXELS_NV :: _1_INVOCATION_PER_1X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X2_PIXELS_NV :: _1_INVOCATION_PER_2X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_4X2_PIXELS_NV :: _1_INVOCATION_PER_4X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X4_PIXELS_NV :: _1_INVOCATION_PER_2X4_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_4X4_PIXELS_NV :: _1_INVOCATION_PER_4X4_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_END_RANGE_NV :: END_RANGE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkShadingRatePaletteEntryNV :: enum u32 {
        NO_INVOCATIONS_NV :: 0;
        _16_INVOCATIONS_PER_PIXEL_NV :: 1;
        _8_INVOCATIONS_PER_PIXEL_NV :: 2;
        _4_INVOCATIONS_PER_PIXEL_NV :: 3;
        _2_INVOCATIONS_PER_PIXEL_NV :: 4;
        _1_INVOCATION_PER_PIXEL_NV :: 5;
        _1_INVOCATION_PER_2X1_PIXELS_NV :: 6;
        _1_INVOCATION_PER_1X2_PIXELS_NV :: 7;
        _1_INVOCATION_PER_2X2_PIXELS_NV :: 8;
        _1_INVOCATION_PER_4X2_PIXELS_NV :: 9;
        _1_INVOCATION_PER_2X4_PIXELS_NV :: 10;
        _1_INVOCATION_PER_4X4_PIXELS_NV :: 11;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 11;
        RANGE_SIZE_NV :: 12;
        MAX_ENUM_NV :: 2147483647;

        VK_SHADING_RATE_PALETTE_ENTRY_NO_INVOCATIONS_NV :: NO_INVOCATIONS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_16_INVOCATIONS_PER_PIXEL_NV :: _16_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_8_INVOCATIONS_PER_PIXEL_NV :: _8_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_4_INVOCATIONS_PER_PIXEL_NV :: _4_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_2_INVOCATIONS_PER_PIXEL_NV :: _2_INVOCATIONS_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_PIXEL_NV :: _1_INVOCATION_PER_PIXEL_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X1_PIXELS_NV :: _1_INVOCATION_PER_2X1_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_1X2_PIXELS_NV :: _1_INVOCATION_PER_1X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X2_PIXELS_NV :: _1_INVOCATION_PER_2X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_4X2_PIXELS_NV :: _1_INVOCATION_PER_4X2_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_2X4_PIXELS_NV :: _1_INVOCATION_PER_2X4_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_1_INVOCATION_PER_4X4_PIXELS_NV :: _1_INVOCATION_PER_4X4_PIXELS_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_END_RANGE_NV :: END_RANGE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_SHADING_RATE_PALETTE_ENTRY_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkShadingRatePaletteNV :: struct {
        shadingRatePaletteEntryCount: u32;
        pShadingRatePaletteEntries: *VkShadingRatePaletteEntryNV;
    }

}

#if OS == .LINUX {
    VkShadingRatePaletteNV :: struct {
        shadingRatePaletteEntryCount: u32;
        pShadingRatePaletteEntries: *VkShadingRatePaletteEntryNV;
    }

}

#if OS == .WINDOWS {
    VkSharedPresentSurfaceCapabilitiesKHR :: struct {
        sType: VkStructureType = .SHARED_PRESENT_SURFACE_CAPABILITIES_KHR;
        pNext: *void;
        sharedPresentSupportedUsageFlags: VkImageUsageFlags;
    }

}

#if OS == .LINUX {
    VkSharedPresentSurfaceCapabilitiesKHR :: struct {
        sType: VkStructureType = .SHARED_PRESENT_SURFACE_CAPABILITIES_KHR;
        pNext: *void;
        sharedPresentSupportedUsageFlags: VkImageUsageFlags;
    }

}

#if OS == .WINDOWS {
    VkSharingMode :: enum u32 {
        EXCLUSIVE :: 0;
        CONCURRENT :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SHARING_MODE_EXCLUSIVE :: EXCLUSIVE;
        VK_SHARING_MODE_CONCURRENT :: CONCURRENT;
        VK_SHARING_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SHARING_MODE_END_RANGE :: END_RANGE;
        VK_SHARING_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SHARING_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSharingMode :: enum u32 {
        EXCLUSIVE :: 0;
        CONCURRENT :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SHARING_MODE_EXCLUSIVE :: EXCLUSIVE;
        VK_SHARING_MODE_CONCURRENT :: CONCURRENT;
        VK_SHARING_MODE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SHARING_MODE_END_RANGE :: END_RANGE;
        VK_SHARING_MODE_RANGE_SIZE :: RANGE_SIZE;
        VK_SHARING_MODE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSparseBufferMemoryBindInfo :: struct {
        buffer: VkBuffer;
        bindCount: u32;
        pBinds: *VkSparseMemoryBind;
    }

}

#if OS == .LINUX {
    VkSparseBufferMemoryBindInfo :: struct {
        buffer: VkBuffer;
        bindCount: u32;
        pBinds: *VkSparseMemoryBind;
    }

}

#if OS == .WINDOWS {
    VkSparseImageFormatFlagBits :: enum_flags u32 {
        SINGLE_MIPTAIL_BIT :: 0x1;
        ALIGNED_MIP_SIZE_BIT :: 0x2;
        NONSTANDARD_BLOCK_SIZE_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SPARSE_IMAGE_FORMAT_SINGLE_MIPTAIL_BIT :: SINGLE_MIPTAIL_BIT;
        VK_SPARSE_IMAGE_FORMAT_ALIGNED_MIP_SIZE_BIT :: ALIGNED_MIP_SIZE_BIT;
        VK_SPARSE_IMAGE_FORMAT_NONSTANDARD_BLOCK_SIZE_BIT :: NONSTANDARD_BLOCK_SIZE_BIT;
        VK_SPARSE_IMAGE_FORMAT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSparseImageFormatFlagBits :: enum_flags u32 {
        SINGLE_MIPTAIL_BIT :: 0x1;
        ALIGNED_MIP_SIZE_BIT :: 0x2;
        NONSTANDARD_BLOCK_SIZE_BIT :: 0x4;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SPARSE_IMAGE_FORMAT_SINGLE_MIPTAIL_BIT :: SINGLE_MIPTAIL_BIT;
        VK_SPARSE_IMAGE_FORMAT_ALIGNED_MIP_SIZE_BIT :: ALIGNED_MIP_SIZE_BIT;
        VK_SPARSE_IMAGE_FORMAT_NONSTANDARD_BLOCK_SIZE_BIT :: NONSTANDARD_BLOCK_SIZE_BIT;
        VK_SPARSE_IMAGE_FORMAT_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSparseImageFormatProperties :: struct {
        aspectMask: VkImageAspectFlags;
        imageGranularity: VkExtent3D;
        flags: VkSparseImageFormatFlags;
    }

}

#if OS == .LINUX {
    VkSparseImageFormatProperties :: struct {
        aspectMask: VkImageAspectFlags;
        imageGranularity: VkExtent3D;
        flags: VkSparseImageFormatFlags;
    }

}

#if OS == .WINDOWS {
    VkSparseImageFormatProperties2 :: struct {
        sType: VkStructureType = .SPARSE_IMAGE_FORMAT_PROPERTIES_2;
        pNext: *void;
        properties: VkSparseImageFormatProperties;
    }

}

#if OS == .LINUX {
    VkSparseImageFormatProperties2 :: struct {
        sType: VkStructureType = .SPARSE_IMAGE_FORMAT_PROPERTIES_2;
        pNext: *void;
        properties: VkSparseImageFormatProperties;
    }

}

#if OS == .WINDOWS {
    VkSparseImageMemoryBind :: struct {
        subresource: VkImageSubresource;
        offset: VkOffset3D;
        extent: VkExtent3D;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        flags: VkSparseMemoryBindFlags;
    }

}

#if OS == .LINUX {
    VkSparseImageMemoryBind :: struct {
        subresource: VkImageSubresource;
        offset: VkOffset3D;
        extent: VkExtent3D;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        flags: VkSparseMemoryBindFlags;
    }

}

#if OS == .WINDOWS {
    VkSparseImageMemoryBindInfo :: struct {
        image: VkImage;
        bindCount: u32;
        pBinds: *VkSparseImageMemoryBind;
    }

}

#if OS == .LINUX {
    VkSparseImageMemoryBindInfo :: struct {
        image: VkImage;
        bindCount: u32;
        pBinds: *VkSparseImageMemoryBind;
    }

}

#if OS == .WINDOWS {
    VkSparseImageMemoryRequirements :: struct {
        formatProperties: VkSparseImageFormatProperties;
        imageMipTailFirstLod: u32;
        imageMipTailSize: VkDeviceSize;
        imageMipTailOffset: VkDeviceSize;
        imageMipTailStride: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkSparseImageMemoryRequirements :: struct {
        formatProperties: VkSparseImageFormatProperties;
        imageMipTailFirstLod: u32;
        imageMipTailSize: VkDeviceSize;
        imageMipTailOffset: VkDeviceSize;
        imageMipTailStride: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkSparseImageMemoryRequirements2 :: struct {
        sType: VkStructureType = .SPARSE_IMAGE_MEMORY_REQUIREMENTS_2;
        pNext: *void;
        memoryRequirements: VkSparseImageMemoryRequirements;
    }

}

#if OS == .LINUX {
    VkSparseImageMemoryRequirements2 :: struct {
        sType: VkStructureType = .SPARSE_IMAGE_MEMORY_REQUIREMENTS_2;
        pNext: *void;
        memoryRequirements: VkSparseImageMemoryRequirements;
    }

}

#if OS == .WINDOWS {
    VkSparseImageOpaqueMemoryBindInfo :: struct {
        image: VkImage;
        bindCount: u32;
        pBinds: *VkSparseMemoryBind;
    }

}

#if OS == .LINUX {
    VkSparseImageOpaqueMemoryBindInfo :: struct {
        image: VkImage;
        bindCount: u32;
        pBinds: *VkSparseMemoryBind;
    }

}

#if OS == .WINDOWS {
    VkSparseMemoryBind :: struct {
        resourceOffset: VkDeviceSize;
        size: VkDeviceSize;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        flags: VkSparseMemoryBindFlags;
    }

}

#if OS == .LINUX {
    VkSparseMemoryBind :: struct {
        resourceOffset: VkDeviceSize;
        size: VkDeviceSize;
        memory: VkDeviceMemory;
        memoryOffset: VkDeviceSize;
        flags: VkSparseMemoryBindFlags;
    }

}

#if OS == .WINDOWS {
    VkSparseMemoryBindFlagBits :: enum_flags u32 {
        METADATA_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SPARSE_MEMORY_BIND_METADATA_BIT :: METADATA_BIT;
        VK_SPARSE_MEMORY_BIND_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSparseMemoryBindFlagBits :: enum_flags u32 {
        METADATA_BIT :: 0x1;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SPARSE_MEMORY_BIND_METADATA_BIT :: METADATA_BIT;
        VK_SPARSE_MEMORY_BIND_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSpecializationInfo :: struct {
        mapEntryCount: u32;
        pMapEntries: *VkSpecializationMapEntry;
        dataSize: u64;
        pData: *void;
    }

}

#if OS == .LINUX {
    VkSpecializationInfo :: struct {
        mapEntryCount: u32;
        pMapEntries: *VkSpecializationMapEntry;
        dataSize: u64;
        pData: *void;
    }

}

#if OS == .WINDOWS {
    VkSpecializationMapEntry :: struct {
        constantID: u32;
        offset: u32;
        size: u64;
    }

}

#if OS == .LINUX {
    VkSpecializationMapEntry :: struct {
        constantID: u32;
        offset: u32;
        size: u64;
    }

}

#if OS == .WINDOWS {
    VkStencilFaceFlagBits :: enum_flags u32 {
        FACE_FRONT_BIT :: 0x1;
        FACE_BACK_BIT :: 0x2;
        FRONT_AND_BACK :: 0x3;
        FACE_FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_STENCIL_FACE_FRONT_BIT :: FACE_FRONT_BIT;
        VK_STENCIL_FACE_BACK_BIT :: FACE_BACK_BIT;
        VK_STENCIL_FRONT_AND_BACK :: FRONT_AND_BACK;
        VK_STENCIL_FACE_FLAG_BITS_MAX_ENUM :: FACE_FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkStencilFaceFlagBits :: enum_flags u32 {
        FACE_FRONT_BIT :: 0x1;
        FACE_BACK_BIT :: 0x2;
        FRONT_AND_BACK :: 0x3;
        FACE_FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_STENCIL_FACE_FRONT_BIT :: FACE_FRONT_BIT;
        VK_STENCIL_FACE_BACK_BIT :: FACE_BACK_BIT;
        VK_STENCIL_FRONT_AND_BACK :: FRONT_AND_BACK;
        VK_STENCIL_FACE_FLAG_BITS_MAX_ENUM :: FACE_FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkStencilOp :: enum u32 {
        KEEP :: 0;
        ZERO :: 1;
        REPLACE :: 2;
        INCREMENT_AND_CLAMP :: 3;
        DECREMENT_AND_CLAMP :: 4;
        INVERT :: 5;
        INCREMENT_AND_WRAP :: 6;
        DECREMENT_AND_WRAP :: 7;
        BEGIN_RANGE :: 0;
        END_RANGE :: 7;
        RANGE_SIZE :: 8;
        MAX_ENUM :: 2147483647;

        VK_STENCIL_OP_KEEP :: KEEP;
        VK_STENCIL_OP_ZERO :: ZERO;
        VK_STENCIL_OP_REPLACE :: REPLACE;
        VK_STENCIL_OP_INCREMENT_AND_CLAMP :: INCREMENT_AND_CLAMP;
        VK_STENCIL_OP_DECREMENT_AND_CLAMP :: DECREMENT_AND_CLAMP;
        VK_STENCIL_OP_INVERT :: INVERT;
        VK_STENCIL_OP_INCREMENT_AND_WRAP :: INCREMENT_AND_WRAP;
        VK_STENCIL_OP_DECREMENT_AND_WRAP :: DECREMENT_AND_WRAP;
        VK_STENCIL_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_STENCIL_OP_END_RANGE :: END_RANGE;
        VK_STENCIL_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_STENCIL_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkStencilOp :: enum u32 {
        KEEP :: 0;
        ZERO :: 1;
        REPLACE :: 2;
        INCREMENT_AND_CLAMP :: 3;
        DECREMENT_AND_CLAMP :: 4;
        INVERT :: 5;
        INCREMENT_AND_WRAP :: 6;
        DECREMENT_AND_WRAP :: 7;
        BEGIN_RANGE :: 0;
        END_RANGE :: 7;
        RANGE_SIZE :: 8;
        MAX_ENUM :: 2147483647;

        VK_STENCIL_OP_KEEP :: KEEP;
        VK_STENCIL_OP_ZERO :: ZERO;
        VK_STENCIL_OP_REPLACE :: REPLACE;
        VK_STENCIL_OP_INCREMENT_AND_CLAMP :: INCREMENT_AND_CLAMP;
        VK_STENCIL_OP_DECREMENT_AND_CLAMP :: DECREMENT_AND_CLAMP;
        VK_STENCIL_OP_INVERT :: INVERT;
        VK_STENCIL_OP_INCREMENT_AND_WRAP :: INCREMENT_AND_WRAP;
        VK_STENCIL_OP_DECREMENT_AND_WRAP :: DECREMENT_AND_WRAP;
        VK_STENCIL_OP_BEGIN_RANGE :: BEGIN_RANGE;
        VK_STENCIL_OP_END_RANGE :: END_RANGE;
        VK_STENCIL_OP_RANGE_SIZE :: RANGE_SIZE;
        VK_STENCIL_OP_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkStencilOpState :: struct {
        failOp: VkStencilOp;
        passOp: VkStencilOp;
        depthFailOp: VkStencilOp;
        compareOp: VkCompareOp;
        compareMask: u32;
        writeMask: u32;
        reference: u32;
    }

}

#if OS == .LINUX {
    VkStencilOpState :: struct {
        failOp: VkStencilOp;
        passOp: VkStencilOp;
        depthFailOp: VkStencilOp;
        compareOp: VkCompareOp;
        compareMask: u32;
        writeMask: u32;
        reference: u32;
    }

}

#if OS == .WINDOWS {
    VkStructureType :: enum u32 {
        APPLICATION_INFO :: 0;
        INSTANCE_CREATE_INFO :: 1;
        DEVICE_QUEUE_CREATE_INFO :: 2;
        DEVICE_CREATE_INFO :: 3;
        SUBMIT_INFO :: 4;
        MEMORY_ALLOCATE_INFO :: 5;
        MAPPED_MEMORY_RANGE :: 6;
        BIND_SPARSE_INFO :: 7;
        FENCE_CREATE_INFO :: 8;
        SEMAPHORE_CREATE_INFO :: 9;
        EVENT_CREATE_INFO :: 10;
        QUERY_POOL_CREATE_INFO :: 11;
        BUFFER_CREATE_INFO :: 12;
        BUFFER_VIEW_CREATE_INFO :: 13;
        IMAGE_CREATE_INFO :: 14;
        IMAGE_VIEW_CREATE_INFO :: 15;
        SHADER_MODULE_CREATE_INFO :: 16;
        PIPELINE_CACHE_CREATE_INFO :: 17;
        PIPELINE_SHADER_STAGE_CREATE_INFO :: 18;
        PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO :: 19;
        PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO :: 20;
        PIPELINE_TESSELLATION_STATE_CREATE_INFO :: 21;
        PIPELINE_VIEWPORT_STATE_CREATE_INFO :: 22;
        PIPELINE_RASTERIZATION_STATE_CREATE_INFO :: 23;
        PIPELINE_MULTISAMPLE_STATE_CREATE_INFO :: 24;
        PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO :: 25;
        PIPELINE_COLOR_BLEND_STATE_CREATE_INFO :: 26;
        PIPELINE_DYNAMIC_STATE_CREATE_INFO :: 27;
        GRAPHICS_PIPELINE_CREATE_INFO :: 28;
        COMPUTE_PIPELINE_CREATE_INFO :: 29;
        PIPELINE_LAYOUT_CREATE_INFO :: 30;
        SAMPLER_CREATE_INFO :: 31;
        DESCRIPTOR_SET_LAYOUT_CREATE_INFO :: 32;
        DESCRIPTOR_POOL_CREATE_INFO :: 33;
        DESCRIPTOR_SET_ALLOCATE_INFO :: 34;
        WRITE_DESCRIPTOR_SET :: 35;
        COPY_DESCRIPTOR_SET :: 36;
        FRAMEBUFFER_CREATE_INFO :: 37;
        RENDER_PASS_CREATE_INFO :: 38;
        COMMAND_POOL_CREATE_INFO :: 39;
        COMMAND_BUFFER_ALLOCATE_INFO :: 40;
        COMMAND_BUFFER_INHERITANCE_INFO :: 41;
        COMMAND_BUFFER_BEGIN_INFO :: 42;
        RENDER_PASS_BEGIN_INFO :: 43;
        BUFFER_MEMORY_BARRIER :: 44;
        IMAGE_MEMORY_BARRIER :: 45;
        MEMORY_BARRIER :: 46;
        LOADER_INSTANCE_CREATE_INFO :: 47;
        LOADER_DEVICE_CREATE_INFO :: 48;
        PHYSICAL_DEVICE_SUBGROUP_PROPERTIES :: 1000094000;
        BIND_BUFFER_MEMORY_INFO :: 1000157000;
        BIND_IMAGE_MEMORY_INFO :: 1000157001;
        PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES :: 1000083000;
        MEMORY_DEDICATED_REQUIREMENTS :: 1000127000;
        MEMORY_DEDICATED_ALLOCATE_INFO :: 1000127001;
        MEMORY_ALLOCATE_FLAGS_INFO :: 1000060000;
        DEVICE_GROUP_RENDER_PASS_BEGIN_INFO :: 1000060003;
        DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO :: 1000060004;
        DEVICE_GROUP_SUBMIT_INFO :: 1000060005;
        DEVICE_GROUP_BIND_SPARSE_INFO :: 1000060006;
        BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO :: 1000060013;
        BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO :: 1000060014;
        PHYSICAL_DEVICE_GROUP_PROPERTIES :: 1000070000;
        DEVICE_GROUP_DEVICE_CREATE_INFO :: 1000070001;
        BUFFER_MEMORY_REQUIREMENTS_INFO_2 :: 1000146000;
        IMAGE_MEMORY_REQUIREMENTS_INFO_2 :: 1000146001;
        IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2 :: 1000146002;
        MEMORY_REQUIREMENTS_2 :: 1000146003;
        SPARSE_IMAGE_MEMORY_REQUIREMENTS_2 :: 1000146004;
        PHYSICAL_DEVICE_FEATURES_2 :: 1000059000;
        PHYSICAL_DEVICE_PROPERTIES_2 :: 1000059001;
        FORMAT_PROPERTIES_2 :: 1000059002;
        IMAGE_FORMAT_PROPERTIES_2 :: 1000059003;
        PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2 :: 1000059004;
        QUEUE_FAMILY_PROPERTIES_2 :: 1000059005;
        PHYSICAL_DEVICE_MEMORY_PROPERTIES_2 :: 1000059006;
        SPARSE_IMAGE_FORMAT_PROPERTIES_2 :: 1000059007;
        PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2 :: 1000059008;
        PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES :: 1000117000;
        RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO :: 1000117001;
        IMAGE_VIEW_USAGE_CREATE_INFO :: 1000117002;
        PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO :: 1000117003;
        RENDER_PASS_MULTIVIEW_CREATE_INFO :: 1000053000;
        PHYSICAL_DEVICE_MULTIVIEW_FEATURES :: 1000053001;
        PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES :: 1000053002;
        PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES :: 1000120000;
        PROTECTED_SUBMIT_INFO :: 1000145000;
        PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES :: 1000145001;
        PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES :: 1000145002;
        DEVICE_QUEUE_INFO_2 :: 1000145003;
        SAMPLER_YCBCR_CONVERSION_CREATE_INFO :: 1000156000;
        SAMPLER_YCBCR_CONVERSION_INFO :: 1000156001;
        BIND_IMAGE_PLANE_MEMORY_INFO :: 1000156002;
        IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO :: 1000156003;
        PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES :: 1000156004;
        SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES :: 1000156005;
        DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO :: 1000085000;
        PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO :: 1000071000;
        EXTERNAL_IMAGE_FORMAT_PROPERTIES :: 1000071001;
        PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO :: 1000071002;
        EXTERNAL_BUFFER_PROPERTIES :: 1000071003;
        PHYSICAL_DEVICE_ID_PROPERTIES :: 1000071004;
        EXTERNAL_MEMORY_BUFFER_CREATE_INFO :: 1000072000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO :: 1000072001;
        EXPORT_MEMORY_ALLOCATE_INFO :: 1000072002;
        PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO :: 1000112000;
        EXTERNAL_FENCE_PROPERTIES :: 1000112001;
        EXPORT_FENCE_CREATE_INFO :: 1000113000;
        EXPORT_SEMAPHORE_CREATE_INFO :: 1000077000;
        PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO :: 1000076000;
        EXTERNAL_SEMAPHORE_PROPERTIES :: 1000076001;
        PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES :: 1000168000;
        DESCRIPTOR_SET_LAYOUT_SUPPORT :: 1000168001;
        PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES :: 1000063000;
        SWAPCHAIN_CREATE_INFO_KHR :: 1000001000;
        PRESENT_INFO_KHR :: 1000001001;
        DEVICE_GROUP_PRESENT_CAPABILITIES_KHR :: 1000060007;
        IMAGE_SWAPCHAIN_CREATE_INFO_KHR :: 1000060008;
        BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR :: 1000060009;
        ACQUIRE_NEXT_IMAGE_INFO_KHR :: 1000060010;
        DEVICE_GROUP_PRESENT_INFO_KHR :: 1000060011;
        DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR :: 1000060012;
        DISPLAY_MODE_CREATE_INFO_KHR :: 1000002000;
        DISPLAY_SURFACE_CREATE_INFO_KHR :: 1000002001;
        DISPLAY_PRESENT_INFO_KHR :: 1000003000;
        XLIB_SURFACE_CREATE_INFO_KHR :: 1000004000;
        XCB_SURFACE_CREATE_INFO_KHR :: 1000005000;
        WAYLAND_SURFACE_CREATE_INFO_KHR :: 1000006000;
        MIR_SURFACE_CREATE_INFO_KHR :: 1000007000;
        ANDROID_SURFACE_CREATE_INFO_KHR :: 1000008000;
        WIN32_SURFACE_CREATE_INFO_KHR :: 1000009000;
        DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT :: 1000011000;
        PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD :: 1000018000;
        DEBUG_MARKER_OBJECT_NAME_INFO_EXT :: 1000022000;
        DEBUG_MARKER_OBJECT_TAG_INFO_EXT :: 1000022001;
        DEBUG_MARKER_MARKER_INFO_EXT :: 1000022002;
        DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV :: 1000026000;
        DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV :: 1000026001;
        DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV :: 1000026002;
        TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD :: 1000041000;
        PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV :: 1000050000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV :: 1000056000;
        EXPORT_MEMORY_ALLOCATE_INFO_NV :: 1000056001;
        IMPORT_MEMORY_WIN32_HANDLE_INFO_NV :: 1000057000;
        EXPORT_MEMORY_WIN32_HANDLE_INFO_NV :: 1000057001;
        WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV :: 1000058000;
        VALIDATION_FLAGS_EXT :: 1000061000;
        VI_SURFACE_CREATE_INFO_NN :: 1000062000;
        IMAGE_VIEW_ASTC_DECODE_MODE_EXT :: 1000067000;
        PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT :: 1000067001;
        IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: 1000073000;
        EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: 1000073001;
        MEMORY_WIN32_HANDLE_PROPERTIES_KHR :: 1000073002;
        MEMORY_GET_WIN32_HANDLE_INFO_KHR :: 1000073003;
        IMPORT_MEMORY_FD_INFO_KHR :: 1000074000;
        MEMORY_FD_PROPERTIES_KHR :: 1000074001;
        MEMORY_GET_FD_INFO_KHR :: 1000074002;
        WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR :: 1000075000;
        IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: 1000078000;
        EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: 1000078001;
        D3D12_FENCE_SUBMIT_INFO_KHR :: 1000078002;
        SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR :: 1000078003;
        IMPORT_SEMAPHORE_FD_INFO_KHR :: 1000079000;
        SEMAPHORE_GET_FD_INFO_KHR :: 1000079001;
        PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR :: 1000080000;
        COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT :: 1000081000;
        PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT :: 1000081001;
        CONDITIONAL_RENDERING_BEGIN_INFO_EXT :: 1000081002;
        PRESENT_REGIONS_KHR :: 1000084000;
        OBJECT_TABLE_CREATE_INFO_NVX :: 1000086000;
        INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX :: 1000086001;
        CMD_PROCESS_COMMANDS_INFO_NVX :: 1000086002;
        CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX :: 1000086003;
        DEVICE_GENERATED_COMMANDS_LIMITS_NVX :: 1000086004;
        DEVICE_GENERATED_COMMANDS_FEATURES_NVX :: 1000086005;
        PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV :: 1000087000;
        SURFACE_CAPABILITIES_2_EXT :: 1000090000;
        DISPLAY_POWER_INFO_EXT :: 1000091000;
        DEVICE_EVENT_INFO_EXT :: 1000091001;
        DISPLAY_EVENT_INFO_EXT :: 1000091002;
        SWAPCHAIN_COUNTER_CREATE_INFO_EXT :: 1000091003;
        PRESENT_TIMES_INFO_GOOGLE :: 1000092000;
        PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX :: 1000097000;
        PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV :: 1000098000;
        PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT :: 1000099000;
        PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT :: 1000099001;
        PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT :: 1000101000;
        PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT :: 1000101001;
        HDR_METADATA_EXT :: 1000105000;
        ATTACHMENT_DESCRIPTION_2_KHR :: 1000109000;
        ATTACHMENT_REFERENCE_2_KHR :: 1000109001;
        SUBPASS_DESCRIPTION_2_KHR :: 1000109002;
        SUBPASS_DEPENDENCY_2_KHR :: 1000109003;
        RENDER_PASS_CREATE_INFO_2_KHR :: 1000109004;
        SUBPASS_BEGIN_INFO_KHR :: 1000109005;
        SUBPASS_END_INFO_KHR :: 1000109006;
        SHARED_PRESENT_SURFACE_CAPABILITIES_KHR :: 1000111000;
        IMPORT_FENCE_WIN32_HANDLE_INFO_KHR :: 1000114000;
        EXPORT_FENCE_WIN32_HANDLE_INFO_KHR :: 1000114001;
        FENCE_GET_WIN32_HANDLE_INFO_KHR :: 1000114002;
        IMPORT_FENCE_FD_INFO_KHR :: 1000115000;
        FENCE_GET_FD_INFO_KHR :: 1000115001;
        PHYSICAL_DEVICE_SURFACE_INFO_2_KHR :: 1000119000;
        SURFACE_CAPABILITIES_2_KHR :: 1000119001;
        SURFACE_FORMAT_2_KHR :: 1000119002;
        DISPLAY_PROPERTIES_2_KHR :: 1000121000;
        DISPLAY_PLANE_PROPERTIES_2_KHR :: 1000121001;
        DISPLAY_MODE_PROPERTIES_2_KHR :: 1000121002;
        DISPLAY_PLANE_INFO_2_KHR :: 1000121003;
        DISPLAY_PLANE_CAPABILITIES_2_KHR :: 1000121004;
        IOS_SURFACE_CREATE_INFO_MVK :: 1000122000;
        MACOS_SURFACE_CREATE_INFO_MVK :: 1000123000;
        DEBUG_UTILS_OBJECT_NAME_INFO_EXT :: 1000128000;
        DEBUG_UTILS_OBJECT_TAG_INFO_EXT :: 1000128001;
        DEBUG_UTILS_LABEL_EXT :: 1000128002;
        DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT :: 1000128003;
        DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT :: 1000128004;
        ANDROID_HARDWARE_BUFFER_USAGE_ANDROID :: 1000129000;
        ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID :: 1000129001;
        ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID :: 1000129002;
        IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: 1000129003;
        MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: 1000129004;
        EXTERNAL_FORMAT_ANDROID :: 1000129005;
        PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT :: 1000130000;
        SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT :: 1000130001;
        PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT :: 1000138000;
        PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT :: 1000138001;
        WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT :: 1000138002;
        DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT :: 1000138003;
        SAMPLE_LOCATIONS_INFO_EXT :: 1000143000;
        RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT :: 1000143001;
        PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT :: 1000143002;
        PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT :: 1000143003;
        MULTISAMPLE_PROPERTIES_EXT :: 1000143004;
        IMAGE_FORMAT_LIST_CREATE_INFO_KHR :: 1000147000;
        PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT :: 1000148000;
        PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT :: 1000148001;
        PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT :: 1000148002;
        PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV :: 1000149000;
        PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV :: 1000152000;
        VALIDATION_CACHE_CREATE_INFO_EXT :: 1000160000;
        SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT :: 1000160001;
        DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT :: 1000161000;
        PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT :: 1000161001;
        PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT :: 1000161002;
        DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT :: 1000161003;
        DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT :: 1000161004;
        PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV :: 1000164000;
        PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV :: 1000164001;
        PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV :: 1000164002;
        PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV :: 1000164005;
        RAYTRACING_PIPELINE_CREATE_INFO_NVX :: 1000165000;
        ACCELERATION_STRUCTURE_CREATE_INFO_NVX :: 1000165001;
        GEOMETRY_INSTANCE_NVX :: 1000165002;
        GEOMETRY_NVX :: 1000165003;
        GEOMETRY_TRIANGLES_NVX :: 1000165004;
        GEOMETRY_AABB_NVX :: 1000165005;
        BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX :: 1000165006;
        DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX :: 1000165007;
        ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX :: 1000165008;
        PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX :: 1000165009;
        HIT_SHADER_MODULE_CREATE_INFO_NVX :: 1000165010;
        PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV :: 1000166000;
        PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV :: 1000166001;
        DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT :: 1000174000;
        PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR :: 1000177000;
        IMPORT_MEMORY_HOST_POINTER_INFO_EXT :: 1000178000;
        MEMORY_HOST_POINTER_PROPERTIES_EXT :: 1000178001;
        PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT :: 1000178002;
        PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD :: 1000185000;
        PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT :: 1000190000;
        PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT :: 1000190001;
        PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT :: 1000190002;
        PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV :: 1000201000;
        PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV :: 1000202000;
        PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV :: 1000202001;
        PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV :: 1000203000;
        PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV :: 1000204000;
        PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV :: 1000205000;
        PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV :: 1000205002;
        CHECKPOINT_DATA_NV :: 1000206000;
        QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV :: 1000206001;
        PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR :: 1000211000;
        DEBUG_REPORT_CREATE_INFO_EXT :: 1000011000;
        RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR :: 1000053000;
        PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR :: 1000053001;
        PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR :: 1000053002;
        PHYSICAL_DEVICE_FEATURES_2_KHR :: 1000059000;
        PHYSICAL_DEVICE_PROPERTIES_2_KHR :: 1000059001;
        FORMAT_PROPERTIES_2_KHR :: 1000059002;
        IMAGE_FORMAT_PROPERTIES_2_KHR :: 1000059003;
        PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR :: 1000059004;
        QUEUE_FAMILY_PROPERTIES_2_KHR :: 1000059005;
        PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR :: 1000059006;
        SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR :: 1000059007;
        PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR :: 1000059008;
        MEMORY_ALLOCATE_FLAGS_INFO_KHR :: 1000060000;
        DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR :: 1000060003;
        DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR :: 1000060004;
        DEVICE_GROUP_SUBMIT_INFO_KHR :: 1000060005;
        DEVICE_GROUP_BIND_SPARSE_INFO_KHR :: 1000060006;
        BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR :: 1000060013;
        BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR :: 1000060014;
        PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR :: 1000070000;
        DEVICE_GROUP_DEVICE_CREATE_INFO_KHR :: 1000070001;
        PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR :: 1000071000;
        EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR :: 1000071001;
        PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR :: 1000071002;
        EXTERNAL_BUFFER_PROPERTIES_KHR :: 1000071003;
        PHYSICAL_DEVICE_ID_PROPERTIES_KHR :: 1000071004;
        EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR :: 1000072000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR :: 1000072001;
        EXPORT_MEMORY_ALLOCATE_INFO_KHR :: 1000072002;
        PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR :: 1000076000;
        EXTERNAL_SEMAPHORE_PROPERTIES_KHR :: 1000076001;
        EXPORT_SEMAPHORE_CREATE_INFO_KHR :: 1000077000;
        PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR :: 1000083000;
        DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR :: 1000085000;
        SURFACE_CAPABILITIES2_EXT :: 1000090000;
        PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR :: 1000112000;
        EXTERNAL_FENCE_PROPERTIES_KHR :: 1000112001;
        EXPORT_FENCE_CREATE_INFO_KHR :: 1000113000;
        PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR :: 1000117000;
        RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR :: 1000117001;
        IMAGE_VIEW_USAGE_CREATE_INFO_KHR :: 1000117002;
        PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR :: 1000117003;
        PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR :: 1000120000;
        MEMORY_DEDICATED_REQUIREMENTS_KHR :: 1000127000;
        MEMORY_DEDICATED_ALLOCATE_INFO_KHR :: 1000127001;
        BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146000;
        IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146001;
        IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146002;
        MEMORY_REQUIREMENTS_2_KHR :: 1000146003;
        SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR :: 1000146004;
        SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR :: 1000156000;
        SAMPLER_YCBCR_CONVERSION_INFO_KHR :: 1000156001;
        BIND_IMAGE_PLANE_MEMORY_INFO_KHR :: 1000156002;
        IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR :: 1000156003;
        PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR :: 1000156004;
        SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR :: 1000156005;
        BIND_BUFFER_MEMORY_INFO_KHR :: 1000157000;
        BIND_IMAGE_MEMORY_INFO_KHR :: 1000157001;
        PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR :: 1000168000;
        DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR :: 1000168001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 48;
        RANGE_SIZE :: 49;
        MAX_ENUM :: 2147483647;

        VK_STRUCTURE_TYPE_APPLICATION_INFO :: APPLICATION_INFO;
        VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO :: INSTANCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO :: DEVICE_QUEUE_CREATE_INFO;
        VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO :: DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_SUBMIT_INFO :: SUBMIT_INFO;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO :: MEMORY_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE :: MAPPED_MEMORY_RANGE;
        VK_STRUCTURE_TYPE_BIND_SPARSE_INFO :: BIND_SPARSE_INFO;
        VK_STRUCTURE_TYPE_FENCE_CREATE_INFO :: FENCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO :: SEMAPHORE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EVENT_CREATE_INFO :: EVENT_CREATE_INFO;
        VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO :: QUERY_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO :: BUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO :: BUFFER_VIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO :: IMAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO :: IMAGE_VIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO :: SHADER_MODULE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO :: PIPELINE_CACHE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO :: PIPELINE_SHADER_STAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO :: PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO :: PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO :: PIPELINE_TESSELLATION_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO :: PIPELINE_VIEWPORT_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO :: PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO :: PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO :: PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO :: PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO :: PIPELINE_DYNAMIC_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO :: GRAPHICS_PIPELINE_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO :: COMPUTE_PIPELINE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO :: PIPELINE_LAYOUT_CREATE_INFO;
        VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO :: SAMPLER_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO :: DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO :: DESCRIPTOR_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO :: DESCRIPTOR_SET_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET :: WRITE_DESCRIPTOR_SET;
        VK_STRUCTURE_TYPE_COPY_DESCRIPTOR_SET :: COPY_DESCRIPTOR_SET;
        VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO :: FRAMEBUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO :: RENDER_PASS_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO :: COMMAND_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO :: COMMAND_BUFFER_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO :: COMMAND_BUFFER_INHERITANCE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO :: COMMAND_BUFFER_BEGIN_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO :: RENDER_PASS_BEGIN_INFO;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER :: BUFFER_MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER :: IMAGE_MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_MEMORY_BARRIER :: MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO :: LOADER_INSTANCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO :: LOADER_DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_PROPERTIES :: PHYSICAL_DEVICE_SUBGROUP_PROPERTIES;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO :: BIND_BUFFER_MEMORY_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO :: BIND_IMAGE_MEMORY_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES :: PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS :: MEMORY_DEDICATED_REQUIREMENTS;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO :: MEMORY_DEDICATED_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO :: MEMORY_ALLOCATE_FLAGS_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO :: DEVICE_GROUP_RENDER_PASS_BEGIN_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO :: DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO :: DEVICE_GROUP_SUBMIT_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO :: DEVICE_GROUP_BIND_SPARSE_INFO;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO :: BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO :: BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES :: PHYSICAL_DEVICE_GROUP_PROPERTIES;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO :: DEVICE_GROUP_DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2 :: BUFFER_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2 :: IMAGE_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2 :: IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2 :: MEMORY_REQUIREMENTS_2;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_MEMORY_REQUIREMENTS_2 :: SPARSE_IMAGE_MEMORY_REQUIREMENTS_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2 :: PHYSICAL_DEVICE_FEATURES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2 :: PHYSICAL_DEVICE_PROPERTIES_2;
        VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2 :: FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2 :: IMAGE_FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2 :: PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2 :: QUEUE_FAMILY_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2 :: PHYSICAL_DEVICE_MEMORY_PROPERTIES_2;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2 :: SPARSE_IMAGE_FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2 :: PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES :: PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES;
        VK_STRUCTURE_TYPE_RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO :: RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO :: IMAGE_VIEW_USAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO :: PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO :: RENDER_PASS_MULTIVIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES :: PHYSICAL_DEVICE_MULTIVIEW_FEATURES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES :: PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES :: PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES;
        VK_STRUCTURE_TYPE_PROTECTED_SUBMIT_INFO :: PROTECTED_SUBMIT_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES :: PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES :: PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2 :: DEVICE_QUEUE_INFO_2;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_CREATE_INFO :: SAMPLER_YCBCR_CONVERSION_CREATE_INFO;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_INFO :: SAMPLER_YCBCR_CONVERSION_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO :: BIND_IMAGE_PLANE_MEMORY_INFO;
        VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO :: IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES :: PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES :: SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES;
        VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO :: DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO :: PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES :: EXTERNAL_IMAGE_FORMAT_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO :: PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES :: EXTERNAL_BUFFER_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES :: PHYSICAL_DEVICE_ID_PROPERTIES;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO :: EXTERNAL_MEMORY_BUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO :: EXPORT_MEMORY_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO :: PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_FENCE_PROPERTIES :: EXTERNAL_FENCE_PROPERTIES;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_CREATE_INFO :: EXPORT_FENCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO :: EXPORT_SEMAPHORE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO :: PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES :: EXTERNAL_SEMAPHORE_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES :: PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_SUPPORT :: DESCRIPTOR_SET_LAYOUT_SUPPORT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES :: PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES;
        VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR :: SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PRESENT_INFO_KHR :: PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_CAPABILITIES_KHR :: DEVICE_GROUP_PRESENT_CAPABILITIES_KHR;
        VK_STRUCTURE_TYPE_IMAGE_SWAPCHAIN_CREATE_INFO_KHR :: IMAGE_SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR :: BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR;
        VK_STRUCTURE_TYPE_ACQUIRE_NEXT_IMAGE_INFO_KHR :: ACQUIRE_NEXT_IMAGE_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_INFO_KHR :: DEVICE_GROUP_PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR :: DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_MODE_CREATE_INFO_KHR :: DISPLAY_MODE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_SURFACE_CREATE_INFO_KHR :: DISPLAY_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PRESENT_INFO_KHR :: DISPLAY_PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR :: XLIB_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR :: XCB_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR :: WAYLAND_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_MIR_SURFACE_CREATE_INFO_KHR :: MIR_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_ANDROID_SURFACE_CREATE_INFO_KHR :: ANDROID_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR :: WIN32_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT :: DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD :: PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_NAME_INFO_EXT :: DEBUG_MARKER_OBJECT_NAME_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_TAG_INFO_EXT :: DEBUG_MARKER_OBJECT_TAG_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_MARKER_INFO_EXT :: DEBUG_MARKER_MARKER_INFO_EXT;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV :: DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV :: DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV :: DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV;
        VK_STRUCTURE_TYPE_TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD :: TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV :: PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_NV :: EXPORT_MEMORY_ALLOCATE_INFO_NV;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_NV :: IMPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_NV :: EXPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV :: WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV;
        VK_STRUCTURE_TYPE_VALIDATION_FLAGS_EXT :: VALIDATION_FLAGS_EXT;
        VK_STRUCTURE_TYPE_VI_SURFACE_CREATE_INFO_NN :: VI_SURFACE_CREATE_INFO_NN;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_ASTC_DECODE_MODE_EXT :: IMAGE_VIEW_ASTC_DECODE_MODE_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT :: PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_MEMORY_WIN32_HANDLE_PROPERTIES_KHR :: MEMORY_WIN32_HANDLE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_GET_WIN32_HANDLE_INFO_KHR :: MEMORY_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_FD_INFO_KHR :: IMPORT_MEMORY_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_MEMORY_FD_PROPERTIES_KHR :: MEMORY_FD_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_GET_FD_INFO_KHR :: MEMORY_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR :: WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_D3D12_FENCE_SUBMIT_INFO_KHR :: D3D12_FENCE_SUBMIT_INFO_KHR;
        VK_STRUCTURE_TYPE_SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR :: SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_FD_INFO_KHR :: IMPORT_SEMAPHORE_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_SEMAPHORE_GET_FD_INFO_KHR :: SEMAPHORE_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR :: PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT :: COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT :: PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT;
        VK_STRUCTURE_TYPE_CONDITIONAL_RENDERING_BEGIN_INFO_EXT :: CONDITIONAL_RENDERING_BEGIN_INFO_EXT;
        VK_STRUCTURE_TYPE_PRESENT_REGIONS_KHR :: PRESENT_REGIONS_KHR;
        VK_STRUCTURE_TYPE_OBJECT_TABLE_CREATE_INFO_NVX :: OBJECT_TABLE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX :: INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_CMD_PROCESS_COMMANDS_INFO_NVX :: CMD_PROCESS_COMMANDS_INFO_NVX;
        VK_STRUCTURE_TYPE_CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX :: CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX;
        VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_LIMITS_NVX :: DEVICE_GENERATED_COMMANDS_LIMITS_NVX;
        VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_FEATURES_NVX :: DEVICE_GENERATED_COMMANDS_FEATURES_NVX;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_EXT :: SURFACE_CAPABILITIES_2_EXT;
        VK_STRUCTURE_TYPE_DISPLAY_POWER_INFO_EXT :: DISPLAY_POWER_INFO_EXT;
        VK_STRUCTURE_TYPE_DEVICE_EVENT_INFO_EXT :: DEVICE_EVENT_INFO_EXT;
        VK_STRUCTURE_TYPE_DISPLAY_EVENT_INFO_EXT :: DISPLAY_EVENT_INFO_EXT;
        VK_STRUCTURE_TYPE_SWAPCHAIN_COUNTER_CREATE_INFO_EXT :: SWAPCHAIN_COUNTER_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PRESENT_TIMES_INFO_GOOGLE :: PRESENT_TIMES_INFO_GOOGLE;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX :: PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT :: PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT :: PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT :: PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT :: PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_HDR_METADATA_EXT :: HDR_METADATA_EXT;
        VK_STRUCTURE_TYPE_ATTACHMENT_DESCRIPTION_2_KHR :: ATTACHMENT_DESCRIPTION_2_KHR;
        VK_STRUCTURE_TYPE_ATTACHMENT_REFERENCE_2_KHR :: ATTACHMENT_REFERENCE_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_DESCRIPTION_2_KHR :: SUBPASS_DESCRIPTION_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_DEPENDENCY_2_KHR :: SUBPASS_DEPENDENCY_2_KHR;
        VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO_2_KHR :: RENDER_PASS_CREATE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_BEGIN_INFO_KHR :: SUBPASS_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_END_INFO_KHR :: SUBPASS_END_INFO_KHR;
        VK_STRUCTURE_TYPE_SHARED_PRESENT_SURFACE_CAPABILITIES_KHR :: SHARED_PRESENT_SURFACE_CAPABILITIES_KHR;
        VK_STRUCTURE_TYPE_IMPORT_FENCE_WIN32_HANDLE_INFO_KHR :: IMPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_WIN32_HANDLE_INFO_KHR :: EXPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_FENCE_GET_WIN32_HANDLE_INFO_KHR :: FENCE_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_FENCE_FD_INFO_KHR :: IMPORT_FENCE_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_FENCE_GET_FD_INFO_KHR :: FENCE_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR :: PHYSICAL_DEVICE_SURFACE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR :: SURFACE_CAPABILITIES_2_KHR;
        VK_STRUCTURE_TYPE_SURFACE_FORMAT_2_KHR :: SURFACE_FORMAT_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PROPERTIES_2_KHR :: DISPLAY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_PROPERTIES_2_KHR :: DISPLAY_PLANE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_MODE_PROPERTIES_2_KHR :: DISPLAY_MODE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_INFO_2_KHR :: DISPLAY_PLANE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_CAPABILITIES_2_KHR :: DISPLAY_PLANE_CAPABILITIES_2_KHR;
        VK_STRUCTURE_TYPE_IOS_SURFACE_CREATE_INFO_MVK :: IOS_SURFACE_CREATE_INFO_MVK;
        VK_STRUCTURE_TYPE_MACOS_SURFACE_CREATE_INFO_MVK :: MACOS_SURFACE_CREATE_INFO_MVK;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_OBJECT_NAME_INFO_EXT :: DEBUG_UTILS_OBJECT_NAME_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_OBJECT_TAG_INFO_EXT :: DEBUG_UTILS_OBJECT_TAG_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_LABEL_EXT :: DEBUG_UTILS_LABEL_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT :: DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT :: DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_USAGE_ANDROID :: ANDROID_HARDWARE_BUFFER_USAGE_ANDROID;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID :: ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID :: ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID;
        VK_STRUCTURE_TYPE_IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID;
        VK_STRUCTURE_TYPE_MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID;
        VK_STRUCTURE_TYPE_EXTERNAL_FORMAT_ANDROID :: EXTERNAL_FORMAT_ANDROID;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT :: PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT :: SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT :: PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT :: PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT :: WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT :: DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_SAMPLE_LOCATIONS_INFO_EXT :: SAMPLE_LOCATIONS_INFO_EXT;
        VK_STRUCTURE_TYPE_RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT :: RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT :: PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT :: PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_MULTISAMPLE_PROPERTIES_EXT :: MULTISAMPLE_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_LIST_CREATE_INFO_KHR :: IMAGE_FORMAT_LIST_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT :: PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT :: PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT :: PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV :: PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV :: PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_VALIDATION_CACHE_CREATE_INFO_EXT :: VALIDATION_CACHE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT :: SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT :: DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT :: PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT :: PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT :: DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT :: DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV :: PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV :: PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_RAYTRACING_PIPELINE_CREATE_INFO_NVX :: RAYTRACING_PIPELINE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_ACCELERATION_STRUCTURE_CREATE_INFO_NVX :: ACCELERATION_STRUCTURE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_INSTANCE_NVX :: GEOMETRY_INSTANCE_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_NVX :: GEOMETRY_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_TRIANGLES_NVX :: GEOMETRY_TRIANGLES_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_AABB_NVX :: GEOMETRY_AABB_NVX;
        VK_STRUCTURE_TYPE_BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX :: BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX;
        VK_STRUCTURE_TYPE_DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX :: DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX;
        VK_STRUCTURE_TYPE_ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX :: ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX :: PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX;
        VK_STRUCTURE_TYPE_HIT_SHADER_MODULE_CREATE_INFO_NVX :: HIT_SHADER_MODULE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV :: PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV :: PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT :: DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR :: PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_HOST_POINTER_INFO_EXT :: IMPORT_MEMORY_HOST_POINTER_INFO_EXT;
        VK_STRUCTURE_TYPE_MEMORY_HOST_POINTER_PROPERTIES_EXT :: MEMORY_HOST_POINTER_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT :: PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD :: PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT :: PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT :: PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT :: PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV :: PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV :: PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV :: PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV :: PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV :: PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV :: PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV;
        VK_STRUCTURE_TYPE_CHECKPOINT_DATA_NV :: CHECKPOINT_DATA_NV;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV :: QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR :: PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR;
        VK_STRUCTURE_TYPE_DEBUG_REPORT_CREATE_INFO_EXT :: DEBUG_REPORT_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR :: RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR :: PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR :: PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2_KHR :: PHYSICAL_DEVICE_FEATURES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2_KHR :: PHYSICAL_DEVICE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2_KHR :: FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2_KHR :: IMAGE_FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR :: PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2_KHR :: QUEUE_FAMILY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR :: PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR :: SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR :: PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO_KHR :: MEMORY_ALLOCATE_FLAGS_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR :: DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR :: DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO_KHR :: DEVICE_GROUP_SUBMIT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO_KHR :: DEVICE_GROUP_BIND_SPARSE_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR :: BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR :: BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR :: PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO_KHR :: DEVICE_GROUP_DEVICE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR :: EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES_KHR :: EXTERNAL_BUFFER_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES_KHR :: PHYSICAL_DEVICE_ID_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR :: EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_KHR :: EXPORT_MEMORY_ALLOCATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES_KHR :: EXTERNAL_SEMAPHORE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO_KHR :: EXPORT_SEMAPHORE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR :: PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR;
        VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR :: DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES2_EXT :: SURFACE_CAPABILITIES2_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_FENCE_PROPERTIES_KHR :: EXTERNAL_FENCE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_CREATE_INFO_KHR :: EXPORT_FENCE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR :: PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR :: RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR :: IMAGE_VIEW_USAGE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR :: PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR :: PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS_KHR :: MEMORY_DEDICATED_REQUIREMENTS_KHR;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO_KHR :: MEMORY_DEDICATED_ALLOCATE_INFO_KHR;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR :: BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR :: IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR :: IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2_KHR :: MEMORY_REQUIREMENTS_2_KHR;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR :: SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR :: SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_INFO_KHR :: SAMPLER_YCBCR_CONVERSION_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO_KHR :: BIND_IMAGE_PLANE_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR :: IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR :: PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR :: SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO_KHR :: BIND_BUFFER_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO_KHR :: BIND_IMAGE_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR :: PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR :: DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR;
        VK_STRUCTURE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_STRUCTURE_TYPE_END_RANGE :: END_RANGE;
        VK_STRUCTURE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_STRUCTURE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkStructureType :: enum u32 {
        APPLICATION_INFO :: 0;
        INSTANCE_CREATE_INFO :: 1;
        DEVICE_QUEUE_CREATE_INFO :: 2;
        DEVICE_CREATE_INFO :: 3;
        SUBMIT_INFO :: 4;
        MEMORY_ALLOCATE_INFO :: 5;
        MAPPED_MEMORY_RANGE :: 6;
        BIND_SPARSE_INFO :: 7;
        FENCE_CREATE_INFO :: 8;
        SEMAPHORE_CREATE_INFO :: 9;
        EVENT_CREATE_INFO :: 10;
        QUERY_POOL_CREATE_INFO :: 11;
        BUFFER_CREATE_INFO :: 12;
        BUFFER_VIEW_CREATE_INFO :: 13;
        IMAGE_CREATE_INFO :: 14;
        IMAGE_VIEW_CREATE_INFO :: 15;
        SHADER_MODULE_CREATE_INFO :: 16;
        PIPELINE_CACHE_CREATE_INFO :: 17;
        PIPELINE_SHADER_STAGE_CREATE_INFO :: 18;
        PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO :: 19;
        PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO :: 20;
        PIPELINE_TESSELLATION_STATE_CREATE_INFO :: 21;
        PIPELINE_VIEWPORT_STATE_CREATE_INFO :: 22;
        PIPELINE_RASTERIZATION_STATE_CREATE_INFO :: 23;
        PIPELINE_MULTISAMPLE_STATE_CREATE_INFO :: 24;
        PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO :: 25;
        PIPELINE_COLOR_BLEND_STATE_CREATE_INFO :: 26;
        PIPELINE_DYNAMIC_STATE_CREATE_INFO :: 27;
        GRAPHICS_PIPELINE_CREATE_INFO :: 28;
        COMPUTE_PIPELINE_CREATE_INFO :: 29;
        PIPELINE_LAYOUT_CREATE_INFO :: 30;
        SAMPLER_CREATE_INFO :: 31;
        DESCRIPTOR_SET_LAYOUT_CREATE_INFO :: 32;
        DESCRIPTOR_POOL_CREATE_INFO :: 33;
        DESCRIPTOR_SET_ALLOCATE_INFO :: 34;
        WRITE_DESCRIPTOR_SET :: 35;
        COPY_DESCRIPTOR_SET :: 36;
        FRAMEBUFFER_CREATE_INFO :: 37;
        RENDER_PASS_CREATE_INFO :: 38;
        COMMAND_POOL_CREATE_INFO :: 39;
        COMMAND_BUFFER_ALLOCATE_INFO :: 40;
        COMMAND_BUFFER_INHERITANCE_INFO :: 41;
        COMMAND_BUFFER_BEGIN_INFO :: 42;
        RENDER_PASS_BEGIN_INFO :: 43;
        BUFFER_MEMORY_BARRIER :: 44;
        IMAGE_MEMORY_BARRIER :: 45;
        MEMORY_BARRIER :: 46;
        LOADER_INSTANCE_CREATE_INFO :: 47;
        LOADER_DEVICE_CREATE_INFO :: 48;
        PHYSICAL_DEVICE_SUBGROUP_PROPERTIES :: 1000094000;
        BIND_BUFFER_MEMORY_INFO :: 1000157000;
        BIND_IMAGE_MEMORY_INFO :: 1000157001;
        PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES :: 1000083000;
        MEMORY_DEDICATED_REQUIREMENTS :: 1000127000;
        MEMORY_DEDICATED_ALLOCATE_INFO :: 1000127001;
        MEMORY_ALLOCATE_FLAGS_INFO :: 1000060000;
        DEVICE_GROUP_RENDER_PASS_BEGIN_INFO :: 1000060003;
        DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO :: 1000060004;
        DEVICE_GROUP_SUBMIT_INFO :: 1000060005;
        DEVICE_GROUP_BIND_SPARSE_INFO :: 1000060006;
        BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO :: 1000060013;
        BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO :: 1000060014;
        PHYSICAL_DEVICE_GROUP_PROPERTIES :: 1000070000;
        DEVICE_GROUP_DEVICE_CREATE_INFO :: 1000070001;
        BUFFER_MEMORY_REQUIREMENTS_INFO_2 :: 1000146000;
        IMAGE_MEMORY_REQUIREMENTS_INFO_2 :: 1000146001;
        IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2 :: 1000146002;
        MEMORY_REQUIREMENTS_2 :: 1000146003;
        SPARSE_IMAGE_MEMORY_REQUIREMENTS_2 :: 1000146004;
        PHYSICAL_DEVICE_FEATURES_2 :: 1000059000;
        PHYSICAL_DEVICE_PROPERTIES_2 :: 1000059001;
        FORMAT_PROPERTIES_2 :: 1000059002;
        IMAGE_FORMAT_PROPERTIES_2 :: 1000059003;
        PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2 :: 1000059004;
        QUEUE_FAMILY_PROPERTIES_2 :: 1000059005;
        PHYSICAL_DEVICE_MEMORY_PROPERTIES_2 :: 1000059006;
        SPARSE_IMAGE_FORMAT_PROPERTIES_2 :: 1000059007;
        PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2 :: 1000059008;
        PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES :: 1000117000;
        RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO :: 1000117001;
        IMAGE_VIEW_USAGE_CREATE_INFO :: 1000117002;
        PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO :: 1000117003;
        RENDER_PASS_MULTIVIEW_CREATE_INFO :: 1000053000;
        PHYSICAL_DEVICE_MULTIVIEW_FEATURES :: 1000053001;
        PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES :: 1000053002;
        PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES :: 1000120000;
        PROTECTED_SUBMIT_INFO :: 1000145000;
        PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES :: 1000145001;
        PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES :: 1000145002;
        DEVICE_QUEUE_INFO_2 :: 1000145003;
        SAMPLER_YCBCR_CONVERSION_CREATE_INFO :: 1000156000;
        SAMPLER_YCBCR_CONVERSION_INFO :: 1000156001;
        BIND_IMAGE_PLANE_MEMORY_INFO :: 1000156002;
        IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO :: 1000156003;
        PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES :: 1000156004;
        SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES :: 1000156005;
        DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO :: 1000085000;
        PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO :: 1000071000;
        EXTERNAL_IMAGE_FORMAT_PROPERTIES :: 1000071001;
        PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO :: 1000071002;
        EXTERNAL_BUFFER_PROPERTIES :: 1000071003;
        PHYSICAL_DEVICE_ID_PROPERTIES :: 1000071004;
        EXTERNAL_MEMORY_BUFFER_CREATE_INFO :: 1000072000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO :: 1000072001;
        EXPORT_MEMORY_ALLOCATE_INFO :: 1000072002;
        PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO :: 1000112000;
        EXTERNAL_FENCE_PROPERTIES :: 1000112001;
        EXPORT_FENCE_CREATE_INFO :: 1000113000;
        EXPORT_SEMAPHORE_CREATE_INFO :: 1000077000;
        PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO :: 1000076000;
        EXTERNAL_SEMAPHORE_PROPERTIES :: 1000076001;
        PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES :: 1000168000;
        DESCRIPTOR_SET_LAYOUT_SUPPORT :: 1000168001;
        PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES :: 1000063000;
        SWAPCHAIN_CREATE_INFO_KHR :: 1000001000;
        PRESENT_INFO_KHR :: 1000001001;
        DEVICE_GROUP_PRESENT_CAPABILITIES_KHR :: 1000060007;
        IMAGE_SWAPCHAIN_CREATE_INFO_KHR :: 1000060008;
        BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR :: 1000060009;
        ACQUIRE_NEXT_IMAGE_INFO_KHR :: 1000060010;
        DEVICE_GROUP_PRESENT_INFO_KHR :: 1000060011;
        DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR :: 1000060012;
        DISPLAY_MODE_CREATE_INFO_KHR :: 1000002000;
        DISPLAY_SURFACE_CREATE_INFO_KHR :: 1000002001;
        DISPLAY_PRESENT_INFO_KHR :: 1000003000;
        XLIB_SURFACE_CREATE_INFO_KHR :: 1000004000;
        XCB_SURFACE_CREATE_INFO_KHR :: 1000005000;
        WAYLAND_SURFACE_CREATE_INFO_KHR :: 1000006000;
        MIR_SURFACE_CREATE_INFO_KHR :: 1000007000;
        ANDROID_SURFACE_CREATE_INFO_KHR :: 1000008000;
        WIN32_SURFACE_CREATE_INFO_KHR :: 1000009000;
        DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT :: 1000011000;
        PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD :: 1000018000;
        DEBUG_MARKER_OBJECT_NAME_INFO_EXT :: 1000022000;
        DEBUG_MARKER_OBJECT_TAG_INFO_EXT :: 1000022001;
        DEBUG_MARKER_MARKER_INFO_EXT :: 1000022002;
        DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV :: 1000026000;
        DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV :: 1000026001;
        DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV :: 1000026002;
        TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD :: 1000041000;
        PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV :: 1000050000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV :: 1000056000;
        EXPORT_MEMORY_ALLOCATE_INFO_NV :: 1000056001;
        IMPORT_MEMORY_WIN32_HANDLE_INFO_NV :: 1000057000;
        EXPORT_MEMORY_WIN32_HANDLE_INFO_NV :: 1000057001;
        WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV :: 1000058000;
        VALIDATION_FLAGS_EXT :: 1000061000;
        VI_SURFACE_CREATE_INFO_NN :: 1000062000;
        IMAGE_VIEW_ASTC_DECODE_MODE_EXT :: 1000067000;
        PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT :: 1000067001;
        IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: 1000073000;
        EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: 1000073001;
        MEMORY_WIN32_HANDLE_PROPERTIES_KHR :: 1000073002;
        MEMORY_GET_WIN32_HANDLE_INFO_KHR :: 1000073003;
        IMPORT_MEMORY_FD_INFO_KHR :: 1000074000;
        MEMORY_FD_PROPERTIES_KHR :: 1000074001;
        MEMORY_GET_FD_INFO_KHR :: 1000074002;
        WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR :: 1000075000;
        IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: 1000078000;
        EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: 1000078001;
        D3D12_FENCE_SUBMIT_INFO_KHR :: 1000078002;
        SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR :: 1000078003;
        IMPORT_SEMAPHORE_FD_INFO_KHR :: 1000079000;
        SEMAPHORE_GET_FD_INFO_KHR :: 1000079001;
        PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR :: 1000080000;
        COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT :: 1000081000;
        PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT :: 1000081001;
        CONDITIONAL_RENDERING_BEGIN_INFO_EXT :: 1000081002;
        PRESENT_REGIONS_KHR :: 1000084000;
        OBJECT_TABLE_CREATE_INFO_NVX :: 1000086000;
        INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX :: 1000086001;
        CMD_PROCESS_COMMANDS_INFO_NVX :: 1000086002;
        CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX :: 1000086003;
        DEVICE_GENERATED_COMMANDS_LIMITS_NVX :: 1000086004;
        DEVICE_GENERATED_COMMANDS_FEATURES_NVX :: 1000086005;
        PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV :: 1000087000;
        SURFACE_CAPABILITIES_2_EXT :: 1000090000;
        DISPLAY_POWER_INFO_EXT :: 1000091000;
        DEVICE_EVENT_INFO_EXT :: 1000091001;
        DISPLAY_EVENT_INFO_EXT :: 1000091002;
        SWAPCHAIN_COUNTER_CREATE_INFO_EXT :: 1000091003;
        PRESENT_TIMES_INFO_GOOGLE :: 1000092000;
        PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX :: 1000097000;
        PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV :: 1000098000;
        PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT :: 1000099000;
        PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT :: 1000099001;
        PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT :: 1000101000;
        PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT :: 1000101001;
        HDR_METADATA_EXT :: 1000105000;
        ATTACHMENT_DESCRIPTION_2_KHR :: 1000109000;
        ATTACHMENT_REFERENCE_2_KHR :: 1000109001;
        SUBPASS_DESCRIPTION_2_KHR :: 1000109002;
        SUBPASS_DEPENDENCY_2_KHR :: 1000109003;
        RENDER_PASS_CREATE_INFO_2_KHR :: 1000109004;
        SUBPASS_BEGIN_INFO_KHR :: 1000109005;
        SUBPASS_END_INFO_KHR :: 1000109006;
        SHARED_PRESENT_SURFACE_CAPABILITIES_KHR :: 1000111000;
        IMPORT_FENCE_WIN32_HANDLE_INFO_KHR :: 1000114000;
        EXPORT_FENCE_WIN32_HANDLE_INFO_KHR :: 1000114001;
        FENCE_GET_WIN32_HANDLE_INFO_KHR :: 1000114002;
        IMPORT_FENCE_FD_INFO_KHR :: 1000115000;
        FENCE_GET_FD_INFO_KHR :: 1000115001;
        PHYSICAL_DEVICE_SURFACE_INFO_2_KHR :: 1000119000;
        SURFACE_CAPABILITIES_2_KHR :: 1000119001;
        SURFACE_FORMAT_2_KHR :: 1000119002;
        DISPLAY_PROPERTIES_2_KHR :: 1000121000;
        DISPLAY_PLANE_PROPERTIES_2_KHR :: 1000121001;
        DISPLAY_MODE_PROPERTIES_2_KHR :: 1000121002;
        DISPLAY_PLANE_INFO_2_KHR :: 1000121003;
        DISPLAY_PLANE_CAPABILITIES_2_KHR :: 1000121004;
        IOS_SURFACE_CREATE_INFO_MVK :: 1000122000;
        MACOS_SURFACE_CREATE_INFO_MVK :: 1000123000;
        DEBUG_UTILS_OBJECT_NAME_INFO_EXT :: 1000128000;
        DEBUG_UTILS_OBJECT_TAG_INFO_EXT :: 1000128001;
        DEBUG_UTILS_LABEL_EXT :: 1000128002;
        DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT :: 1000128003;
        DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT :: 1000128004;
        ANDROID_HARDWARE_BUFFER_USAGE_ANDROID :: 1000129000;
        ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID :: 1000129001;
        ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID :: 1000129002;
        IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: 1000129003;
        MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: 1000129004;
        EXTERNAL_FORMAT_ANDROID :: 1000129005;
        PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT :: 1000130000;
        SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT :: 1000130001;
        PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT :: 1000138000;
        PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT :: 1000138001;
        WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT :: 1000138002;
        DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT :: 1000138003;
        SAMPLE_LOCATIONS_INFO_EXT :: 1000143000;
        RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT :: 1000143001;
        PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT :: 1000143002;
        PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT :: 1000143003;
        MULTISAMPLE_PROPERTIES_EXT :: 1000143004;
        IMAGE_FORMAT_LIST_CREATE_INFO_KHR :: 1000147000;
        PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT :: 1000148000;
        PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT :: 1000148001;
        PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT :: 1000148002;
        PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV :: 1000149000;
        PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV :: 1000152000;
        VALIDATION_CACHE_CREATE_INFO_EXT :: 1000160000;
        SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT :: 1000160001;
        DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT :: 1000161000;
        PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT :: 1000161001;
        PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT :: 1000161002;
        DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT :: 1000161003;
        DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT :: 1000161004;
        PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV :: 1000164000;
        PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV :: 1000164001;
        PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV :: 1000164002;
        PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV :: 1000164005;
        RAYTRACING_PIPELINE_CREATE_INFO_NVX :: 1000165000;
        ACCELERATION_STRUCTURE_CREATE_INFO_NVX :: 1000165001;
        GEOMETRY_INSTANCE_NVX :: 1000165002;
        GEOMETRY_NVX :: 1000165003;
        GEOMETRY_TRIANGLES_NVX :: 1000165004;
        GEOMETRY_AABB_NVX :: 1000165005;
        BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX :: 1000165006;
        DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX :: 1000165007;
        ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX :: 1000165008;
        PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX :: 1000165009;
        HIT_SHADER_MODULE_CREATE_INFO_NVX :: 1000165010;
        PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV :: 1000166000;
        PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV :: 1000166001;
        DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT :: 1000174000;
        PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR :: 1000177000;
        IMPORT_MEMORY_HOST_POINTER_INFO_EXT :: 1000178000;
        MEMORY_HOST_POINTER_PROPERTIES_EXT :: 1000178001;
        PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT :: 1000178002;
        PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD :: 1000185000;
        PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT :: 1000190000;
        PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT :: 1000190001;
        PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT :: 1000190002;
        PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV :: 1000201000;
        PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV :: 1000202000;
        PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV :: 1000202001;
        PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV :: 1000203000;
        PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV :: 1000204000;
        PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV :: 1000205000;
        PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV :: 1000205002;
        CHECKPOINT_DATA_NV :: 1000206000;
        QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV :: 1000206001;
        PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR :: 1000211000;
        DEBUG_REPORT_CREATE_INFO_EXT :: 1000011000;
        RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR :: 1000053000;
        PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR :: 1000053001;
        PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR :: 1000053002;
        PHYSICAL_DEVICE_FEATURES_2_KHR :: 1000059000;
        PHYSICAL_DEVICE_PROPERTIES_2_KHR :: 1000059001;
        FORMAT_PROPERTIES_2_KHR :: 1000059002;
        IMAGE_FORMAT_PROPERTIES_2_KHR :: 1000059003;
        PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR :: 1000059004;
        QUEUE_FAMILY_PROPERTIES_2_KHR :: 1000059005;
        PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR :: 1000059006;
        SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR :: 1000059007;
        PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR :: 1000059008;
        MEMORY_ALLOCATE_FLAGS_INFO_KHR :: 1000060000;
        DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR :: 1000060003;
        DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR :: 1000060004;
        DEVICE_GROUP_SUBMIT_INFO_KHR :: 1000060005;
        DEVICE_GROUP_BIND_SPARSE_INFO_KHR :: 1000060006;
        BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR :: 1000060013;
        BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR :: 1000060014;
        PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR :: 1000070000;
        DEVICE_GROUP_DEVICE_CREATE_INFO_KHR :: 1000070001;
        PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR :: 1000071000;
        EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR :: 1000071001;
        PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR :: 1000071002;
        EXTERNAL_BUFFER_PROPERTIES_KHR :: 1000071003;
        PHYSICAL_DEVICE_ID_PROPERTIES_KHR :: 1000071004;
        EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR :: 1000072000;
        EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR :: 1000072001;
        EXPORT_MEMORY_ALLOCATE_INFO_KHR :: 1000072002;
        PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR :: 1000076000;
        EXTERNAL_SEMAPHORE_PROPERTIES_KHR :: 1000076001;
        EXPORT_SEMAPHORE_CREATE_INFO_KHR :: 1000077000;
        PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR :: 1000083000;
        DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR :: 1000085000;
        SURFACE_CAPABILITIES2_EXT :: 1000090000;
        PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR :: 1000112000;
        EXTERNAL_FENCE_PROPERTIES_KHR :: 1000112001;
        EXPORT_FENCE_CREATE_INFO_KHR :: 1000113000;
        PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR :: 1000117000;
        RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR :: 1000117001;
        IMAGE_VIEW_USAGE_CREATE_INFO_KHR :: 1000117002;
        PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR :: 1000117003;
        PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR :: 1000120000;
        MEMORY_DEDICATED_REQUIREMENTS_KHR :: 1000127000;
        MEMORY_DEDICATED_ALLOCATE_INFO_KHR :: 1000127001;
        BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146000;
        IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146001;
        IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR :: 1000146002;
        MEMORY_REQUIREMENTS_2_KHR :: 1000146003;
        SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR :: 1000146004;
        SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR :: 1000156000;
        SAMPLER_YCBCR_CONVERSION_INFO_KHR :: 1000156001;
        BIND_IMAGE_PLANE_MEMORY_INFO_KHR :: 1000156002;
        IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR :: 1000156003;
        PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR :: 1000156004;
        SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR :: 1000156005;
        BIND_BUFFER_MEMORY_INFO_KHR :: 1000157000;
        BIND_IMAGE_MEMORY_INFO_KHR :: 1000157001;
        PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR :: 1000168000;
        DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR :: 1000168001;
        BEGIN_RANGE :: 0;
        END_RANGE :: 48;
        RANGE_SIZE :: 49;
        MAX_ENUM :: 2147483647;

        VK_STRUCTURE_TYPE_APPLICATION_INFO :: APPLICATION_INFO;
        VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO :: INSTANCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO :: DEVICE_QUEUE_CREATE_INFO;
        VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO :: DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_SUBMIT_INFO :: SUBMIT_INFO;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO :: MEMORY_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_MAPPED_MEMORY_RANGE :: MAPPED_MEMORY_RANGE;
        VK_STRUCTURE_TYPE_BIND_SPARSE_INFO :: BIND_SPARSE_INFO;
        VK_STRUCTURE_TYPE_FENCE_CREATE_INFO :: FENCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO :: SEMAPHORE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EVENT_CREATE_INFO :: EVENT_CREATE_INFO;
        VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO :: QUERY_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO :: BUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_VIEW_CREATE_INFO :: BUFFER_VIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO :: IMAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO :: IMAGE_VIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO :: SHADER_MODULE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO :: PIPELINE_CACHE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO :: PIPELINE_SHADER_STAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO :: PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO :: PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO :: PIPELINE_TESSELLATION_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO :: PIPELINE_VIEWPORT_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO :: PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO :: PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO :: PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO :: PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO :: PIPELINE_DYNAMIC_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO :: GRAPHICS_PIPELINE_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO :: COMPUTE_PIPELINE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO :: PIPELINE_LAYOUT_CREATE_INFO;
        VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO :: SAMPLER_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO :: DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO :: DESCRIPTOR_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO :: DESCRIPTOR_SET_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET :: WRITE_DESCRIPTOR_SET;
        VK_STRUCTURE_TYPE_COPY_DESCRIPTOR_SET :: COPY_DESCRIPTOR_SET;
        VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO :: FRAMEBUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO :: RENDER_PASS_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO :: COMMAND_POOL_CREATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO :: COMMAND_BUFFER_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO :: COMMAND_BUFFER_INHERITANCE_INFO;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO :: COMMAND_BUFFER_BEGIN_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO :: RENDER_PASS_BEGIN_INFO;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER :: BUFFER_MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER :: IMAGE_MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_MEMORY_BARRIER :: MEMORY_BARRIER;
        VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO :: LOADER_INSTANCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO :: LOADER_DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_PROPERTIES :: PHYSICAL_DEVICE_SUBGROUP_PROPERTIES;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO :: BIND_BUFFER_MEMORY_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO :: BIND_IMAGE_MEMORY_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES :: PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS :: MEMORY_DEDICATED_REQUIREMENTS;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO :: MEMORY_DEDICATED_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO :: MEMORY_ALLOCATE_FLAGS_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO :: DEVICE_GROUP_RENDER_PASS_BEGIN_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO :: DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO :: DEVICE_GROUP_SUBMIT_INFO;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO :: DEVICE_GROUP_BIND_SPARSE_INFO;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO :: BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO :: BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES :: PHYSICAL_DEVICE_GROUP_PROPERTIES;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO :: DEVICE_GROUP_DEVICE_CREATE_INFO;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2 :: BUFFER_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2 :: IMAGE_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2 :: IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2;
        VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2 :: MEMORY_REQUIREMENTS_2;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_MEMORY_REQUIREMENTS_2 :: SPARSE_IMAGE_MEMORY_REQUIREMENTS_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2 :: PHYSICAL_DEVICE_FEATURES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2 :: PHYSICAL_DEVICE_PROPERTIES_2;
        VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2 :: FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2 :: IMAGE_FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2 :: PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2 :: QUEUE_FAMILY_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2 :: PHYSICAL_DEVICE_MEMORY_PROPERTIES_2;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2 :: SPARSE_IMAGE_FORMAT_PROPERTIES_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2 :: PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES :: PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES;
        VK_STRUCTURE_TYPE_RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO :: RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO :: IMAGE_VIEW_USAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO :: PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO :: RENDER_PASS_MULTIVIEW_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES :: PHYSICAL_DEVICE_MULTIVIEW_FEATURES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES :: PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES :: PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES;
        VK_STRUCTURE_TYPE_PROTECTED_SUBMIT_INFO :: PROTECTED_SUBMIT_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES :: PHYSICAL_DEVICE_PROTECTED_MEMORY_FEATURES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES :: PHYSICAL_DEVICE_PROTECTED_MEMORY_PROPERTIES;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_INFO_2 :: DEVICE_QUEUE_INFO_2;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_CREATE_INFO :: SAMPLER_YCBCR_CONVERSION_CREATE_INFO;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_INFO :: SAMPLER_YCBCR_CONVERSION_INFO;
        VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO :: BIND_IMAGE_PLANE_MEMORY_INFO;
        VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO :: IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES :: PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES :: SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES;
        VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO :: DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO :: PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES :: EXTERNAL_IMAGE_FORMAT_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO :: PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES :: EXTERNAL_BUFFER_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES :: PHYSICAL_DEVICE_ID_PROPERTIES;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO :: EXTERNAL_MEMORY_BUFFER_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO :: EXPORT_MEMORY_ALLOCATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO :: PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_FENCE_PROPERTIES :: EXTERNAL_FENCE_PROPERTIES;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_CREATE_INFO :: EXPORT_FENCE_CREATE_INFO;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO :: EXPORT_SEMAPHORE_CREATE_INFO;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO :: PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO;
        VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES :: EXTERNAL_SEMAPHORE_PROPERTIES;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES :: PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_SUPPORT :: DESCRIPTOR_SET_LAYOUT_SUPPORT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES :: PHYSICAL_DEVICE_SHADER_DRAW_PARAMETER_FEATURES;
        VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR :: SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PRESENT_INFO_KHR :: PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_CAPABILITIES_KHR :: DEVICE_GROUP_PRESENT_CAPABILITIES_KHR;
        VK_STRUCTURE_TYPE_IMAGE_SWAPCHAIN_CREATE_INFO_KHR :: IMAGE_SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR :: BIND_IMAGE_MEMORY_SWAPCHAIN_INFO_KHR;
        VK_STRUCTURE_TYPE_ACQUIRE_NEXT_IMAGE_INFO_KHR :: ACQUIRE_NEXT_IMAGE_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_PRESENT_INFO_KHR :: DEVICE_GROUP_PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR :: DEVICE_GROUP_SWAPCHAIN_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_MODE_CREATE_INFO_KHR :: DISPLAY_MODE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_SURFACE_CREATE_INFO_KHR :: DISPLAY_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PRESENT_INFO_KHR :: DISPLAY_PRESENT_INFO_KHR;
        VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR :: XLIB_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR :: XCB_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR :: WAYLAND_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_MIR_SURFACE_CREATE_INFO_KHR :: MIR_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_ANDROID_SURFACE_CREATE_INFO_KHR :: ANDROID_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR :: WIN32_SURFACE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT :: DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD :: PIPELINE_RASTERIZATION_STATE_RASTERIZATION_ORDER_AMD;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_NAME_INFO_EXT :: DEBUG_MARKER_OBJECT_NAME_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_OBJECT_TAG_INFO_EXT :: DEBUG_MARKER_OBJECT_TAG_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_MARKER_MARKER_INFO_EXT :: DEBUG_MARKER_MARKER_INFO_EXT;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV :: DEDICATED_ALLOCATION_IMAGE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV :: DEDICATED_ALLOCATION_BUFFER_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV :: DEDICATED_ALLOCATION_MEMORY_ALLOCATE_INFO_NV;
        VK_STRUCTURE_TYPE_TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD :: TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV :: PHYSICAL_DEVICE_CORNER_SAMPLED_IMAGE_FEATURES_NV;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_NV :: EXPORT_MEMORY_ALLOCATE_INFO_NV;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_NV :: IMPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_NV :: EXPORT_MEMORY_WIN32_HANDLE_INFO_NV;
        VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV :: WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV;
        VK_STRUCTURE_TYPE_VALIDATION_FLAGS_EXT :: VALIDATION_FLAGS_EXT;
        VK_STRUCTURE_TYPE_VI_SURFACE_CREATE_INFO_NN :: VI_SURFACE_CREATE_INFO_NN;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_ASTC_DECODE_MODE_EXT :: IMAGE_VIEW_ASTC_DECODE_MODE_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT :: PHYSICAL_DEVICE_ASTC_DECODE_FEATURES_EXT;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: IMPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR :: EXPORT_MEMORY_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_MEMORY_WIN32_HANDLE_PROPERTIES_KHR :: MEMORY_WIN32_HANDLE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_GET_WIN32_HANDLE_INFO_KHR :: MEMORY_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_FD_INFO_KHR :: IMPORT_MEMORY_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_MEMORY_FD_PROPERTIES_KHR :: MEMORY_FD_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_GET_FD_INFO_KHR :: MEMORY_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR :: WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: IMPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR :: EXPORT_SEMAPHORE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_D3D12_FENCE_SUBMIT_INFO_KHR :: D3D12_FENCE_SUBMIT_INFO_KHR;
        VK_STRUCTURE_TYPE_SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR :: SEMAPHORE_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_SEMAPHORE_FD_INFO_KHR :: IMPORT_SEMAPHORE_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_SEMAPHORE_GET_FD_INFO_KHR :: SEMAPHORE_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR :: PHYSICAL_DEVICE_PUSH_DESCRIPTOR_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT :: COMMAND_BUFFER_INHERITANCE_CONDITIONAL_RENDERING_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT :: PHYSICAL_DEVICE_CONDITIONAL_RENDERING_FEATURES_EXT;
        VK_STRUCTURE_TYPE_CONDITIONAL_RENDERING_BEGIN_INFO_EXT :: CONDITIONAL_RENDERING_BEGIN_INFO_EXT;
        VK_STRUCTURE_TYPE_PRESENT_REGIONS_KHR :: PRESENT_REGIONS_KHR;
        VK_STRUCTURE_TYPE_OBJECT_TABLE_CREATE_INFO_NVX :: OBJECT_TABLE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX :: INDIRECT_COMMANDS_LAYOUT_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_CMD_PROCESS_COMMANDS_INFO_NVX :: CMD_PROCESS_COMMANDS_INFO_NVX;
        VK_STRUCTURE_TYPE_CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX :: CMD_RESERVE_SPACE_FOR_COMMANDS_INFO_NVX;
        VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_LIMITS_NVX :: DEVICE_GENERATED_COMMANDS_LIMITS_NVX;
        VK_STRUCTURE_TYPE_DEVICE_GENERATED_COMMANDS_FEATURES_NVX :: DEVICE_GENERATED_COMMANDS_FEATURES_NVX;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_W_SCALING_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_EXT :: SURFACE_CAPABILITIES_2_EXT;
        VK_STRUCTURE_TYPE_DISPLAY_POWER_INFO_EXT :: DISPLAY_POWER_INFO_EXT;
        VK_STRUCTURE_TYPE_DEVICE_EVENT_INFO_EXT :: DEVICE_EVENT_INFO_EXT;
        VK_STRUCTURE_TYPE_DISPLAY_EVENT_INFO_EXT :: DISPLAY_EVENT_INFO_EXT;
        VK_STRUCTURE_TYPE_SWAPCHAIN_COUNTER_CREATE_INFO_EXT :: SWAPCHAIN_COUNTER_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PRESENT_TIMES_INFO_GOOGLE :: PRESENT_TIMES_INFO_GOOGLE;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX :: PHYSICAL_DEVICE_MULTIVIEW_PER_VIEW_ATTRIBUTES_PROPERTIES_NVX;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_SWIZZLE_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT :: PHYSICAL_DEVICE_DISCARD_RECTANGLE_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT :: PIPELINE_DISCARD_RECTANGLE_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT :: PHYSICAL_DEVICE_CONSERVATIVE_RASTERIZATION_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT :: PIPELINE_RASTERIZATION_CONSERVATIVE_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_HDR_METADATA_EXT :: HDR_METADATA_EXT;
        VK_STRUCTURE_TYPE_ATTACHMENT_DESCRIPTION_2_KHR :: ATTACHMENT_DESCRIPTION_2_KHR;
        VK_STRUCTURE_TYPE_ATTACHMENT_REFERENCE_2_KHR :: ATTACHMENT_REFERENCE_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_DESCRIPTION_2_KHR :: SUBPASS_DESCRIPTION_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_DEPENDENCY_2_KHR :: SUBPASS_DEPENDENCY_2_KHR;
        VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO_2_KHR :: RENDER_PASS_CREATE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_BEGIN_INFO_KHR :: SUBPASS_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_SUBPASS_END_INFO_KHR :: SUBPASS_END_INFO_KHR;
        VK_STRUCTURE_TYPE_SHARED_PRESENT_SURFACE_CAPABILITIES_KHR :: SHARED_PRESENT_SURFACE_CAPABILITIES_KHR;
        VK_STRUCTURE_TYPE_IMPORT_FENCE_WIN32_HANDLE_INFO_KHR :: IMPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_WIN32_HANDLE_INFO_KHR :: EXPORT_FENCE_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_FENCE_GET_WIN32_HANDLE_INFO_KHR :: FENCE_GET_WIN32_HANDLE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMPORT_FENCE_FD_INFO_KHR :: IMPORT_FENCE_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_FENCE_GET_FD_INFO_KHR :: FENCE_GET_FD_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR :: PHYSICAL_DEVICE_SURFACE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR :: SURFACE_CAPABILITIES_2_KHR;
        VK_STRUCTURE_TYPE_SURFACE_FORMAT_2_KHR :: SURFACE_FORMAT_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PROPERTIES_2_KHR :: DISPLAY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_PROPERTIES_2_KHR :: DISPLAY_PLANE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_MODE_PROPERTIES_2_KHR :: DISPLAY_MODE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_INFO_2_KHR :: DISPLAY_PLANE_INFO_2_KHR;
        VK_STRUCTURE_TYPE_DISPLAY_PLANE_CAPABILITIES_2_KHR :: DISPLAY_PLANE_CAPABILITIES_2_KHR;
        VK_STRUCTURE_TYPE_IOS_SURFACE_CREATE_INFO_MVK :: IOS_SURFACE_CREATE_INFO_MVK;
        VK_STRUCTURE_TYPE_MACOS_SURFACE_CREATE_INFO_MVK :: MACOS_SURFACE_CREATE_INFO_MVK;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_OBJECT_NAME_INFO_EXT :: DEBUG_UTILS_OBJECT_NAME_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_OBJECT_TAG_INFO_EXT :: DEBUG_UTILS_OBJECT_TAG_INFO_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_LABEL_EXT :: DEBUG_UTILS_LABEL_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT :: DEBUG_UTILS_MESSENGER_CALLBACK_DATA_EXT;
        VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT :: DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_USAGE_ANDROID :: ANDROID_HARDWARE_BUFFER_USAGE_ANDROID;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID :: ANDROID_HARDWARE_BUFFER_PROPERTIES_ANDROID;
        VK_STRUCTURE_TYPE_ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID :: ANDROID_HARDWARE_BUFFER_FORMAT_PROPERTIES_ANDROID;
        VK_STRUCTURE_TYPE_IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: IMPORT_ANDROID_HARDWARE_BUFFER_INFO_ANDROID;
        VK_STRUCTURE_TYPE_MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID :: MEMORY_GET_ANDROID_HARDWARE_BUFFER_INFO_ANDROID;
        VK_STRUCTURE_TYPE_EXTERNAL_FORMAT_ANDROID :: EXTERNAL_FORMAT_ANDROID;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT :: PHYSICAL_DEVICE_SAMPLER_FILTER_MINMAX_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT :: SAMPLER_REDUCTION_MODE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT :: PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT :: PHYSICAL_DEVICE_INLINE_UNIFORM_BLOCK_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT :: WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT :: DESCRIPTOR_POOL_INLINE_UNIFORM_BLOCK_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_SAMPLE_LOCATIONS_INFO_EXT :: SAMPLE_LOCATIONS_INFO_EXT;
        VK_STRUCTURE_TYPE_RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT :: RENDER_PASS_SAMPLE_LOCATIONS_BEGIN_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT :: PIPELINE_SAMPLE_LOCATIONS_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT :: PHYSICAL_DEVICE_SAMPLE_LOCATIONS_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_MULTISAMPLE_PROPERTIES_EXT :: MULTISAMPLE_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_LIST_CREATE_INFO_KHR :: IMAGE_FORMAT_LIST_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT :: PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT :: PHYSICAL_DEVICE_BLEND_OPERATION_ADVANCED_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT :: PIPELINE_COLOR_BLEND_ADVANCED_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV :: PIPELINE_COVERAGE_TO_COLOR_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV :: PIPELINE_COVERAGE_MODULATION_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_VALIDATION_CACHE_CREATE_INFO_EXT :: VALIDATION_CACHE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT :: SHADER_MODULE_VALIDATION_CACHE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT :: DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT :: PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT :: PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT :: DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO_EXT;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT :: DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_LAYOUT_SUPPORT_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_SHADING_RATE_IMAGE_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV :: PHYSICAL_DEVICE_SHADING_RATE_IMAGE_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV :: PHYSICAL_DEVICE_SHADING_RATE_IMAGE_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_COARSE_SAMPLE_ORDER_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_RAYTRACING_PIPELINE_CREATE_INFO_NVX :: RAYTRACING_PIPELINE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_ACCELERATION_STRUCTURE_CREATE_INFO_NVX :: ACCELERATION_STRUCTURE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_INSTANCE_NVX :: GEOMETRY_INSTANCE_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_NVX :: GEOMETRY_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_TRIANGLES_NVX :: GEOMETRY_TRIANGLES_NVX;
        VK_STRUCTURE_TYPE_GEOMETRY_AABB_NVX :: GEOMETRY_AABB_NVX;
        VK_STRUCTURE_TYPE_BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX :: BIND_ACCELERATION_STRUCTURE_MEMORY_INFO_NVX;
        VK_STRUCTURE_TYPE_DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX :: DESCRIPTOR_ACCELERATION_STRUCTURE_INFO_NVX;
        VK_STRUCTURE_TYPE_ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX :: ACCELERATION_STRUCTURE_MEMORY_REQUIREMENTS_INFO_NVX;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX :: PHYSICAL_DEVICE_RAYTRACING_PROPERTIES_NVX;
        VK_STRUCTURE_TYPE_HIT_SHADER_MODULE_CREATE_INFO_NVX :: HIT_SHADER_MODULE_CREATE_INFO_NVX;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV :: PHYSICAL_DEVICE_REPRESENTATIVE_FRAGMENT_TEST_FEATURES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV :: PIPELINE_REPRESENTATIVE_FRAGMENT_TEST_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT :: DEVICE_QUEUE_GLOBAL_PRIORITY_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR :: PHYSICAL_DEVICE_8BIT_STORAGE_FEATURES_KHR;
        VK_STRUCTURE_TYPE_IMPORT_MEMORY_HOST_POINTER_INFO_EXT :: IMPORT_MEMORY_HOST_POINTER_INFO_EXT;
        VK_STRUCTURE_TYPE_MEMORY_HOST_POINTER_PROPERTIES_EXT :: MEMORY_HOST_POINTER_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT :: PHYSICAL_DEVICE_EXTERNAL_MEMORY_HOST_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD :: PHYSICAL_DEVICE_SHADER_CORE_PROPERTIES_AMD;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT :: PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_PROPERTIES_EXT;
        VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT :: PIPELINE_VERTEX_INPUT_DIVISOR_STATE_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT :: PHYSICAL_DEVICE_VERTEX_ATTRIBUTE_DIVISOR_FEATURES_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV :: PHYSICAL_DEVICE_COMPUTE_SHADER_DERIVATIVES_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV :: PHYSICAL_DEVICE_MESH_SHADER_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV :: PHYSICAL_DEVICE_MESH_SHADER_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV :: PHYSICAL_DEVICE_FRAGMENT_SHADER_BARYCENTRIC_FEATURES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV :: PHYSICAL_DEVICE_SHADER_IMAGE_FOOTPRINT_FEATURES_NV;
        VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV :: PIPELINE_VIEWPORT_EXCLUSIVE_SCISSOR_STATE_CREATE_INFO_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV :: PHYSICAL_DEVICE_EXCLUSIVE_SCISSOR_FEATURES_NV;
        VK_STRUCTURE_TYPE_CHECKPOINT_DATA_NV :: CHECKPOINT_DATA_NV;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV :: QUEUE_FAMILY_CHECKPOINT_PROPERTIES_NV;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR :: PHYSICAL_DEVICE_VULKAN_MEMORY_MODEL_FEATURES_KHR;
        VK_STRUCTURE_TYPE_DEBUG_REPORT_CREATE_INFO_EXT :: DEBUG_REPORT_CREATE_INFO_EXT;
        VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR :: RENDER_PASS_MULTIVIEW_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR :: PHYSICAL_DEVICE_MULTIVIEW_FEATURES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR :: PHYSICAL_DEVICE_MULTIVIEW_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2_KHR :: PHYSICAL_DEVICE_FEATURES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2_KHR :: PHYSICAL_DEVICE_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2_KHR :: FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_FORMAT_PROPERTIES_2_KHR :: IMAGE_FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR :: PHYSICAL_DEVICE_IMAGE_FORMAT_INFO_2_KHR;
        VK_STRUCTURE_TYPE_QUEUE_FAMILY_PROPERTIES_2_KHR :: QUEUE_FAMILY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR :: PHYSICAL_DEVICE_MEMORY_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR :: SPARSE_IMAGE_FORMAT_PROPERTIES_2_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR :: PHYSICAL_DEVICE_SPARSE_IMAGE_FORMAT_INFO_2_KHR;
        VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_FLAGS_INFO_KHR :: MEMORY_ALLOCATE_FLAGS_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR :: DEVICE_GROUP_RENDER_PASS_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR :: DEVICE_GROUP_COMMAND_BUFFER_BEGIN_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_SUBMIT_INFO_KHR :: DEVICE_GROUP_SUBMIT_INFO_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_BIND_SPARSE_INFO_KHR :: DEVICE_GROUP_BIND_SPARSE_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR :: BIND_BUFFER_MEMORY_DEVICE_GROUP_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR :: BIND_IMAGE_MEMORY_DEVICE_GROUP_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR :: PHYSICAL_DEVICE_GROUP_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_DEVICE_GROUP_DEVICE_CREATE_INFO_KHR :: DEVICE_GROUP_DEVICE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_IMAGE_FORMAT_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR :: EXTERNAL_IMAGE_FORMAT_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_BUFFER_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_BUFFER_PROPERTIES_KHR :: EXTERNAL_BUFFER_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_ID_PROPERTIES_KHR :: PHYSICAL_DEVICE_ID_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR :: EXTERNAL_MEMORY_BUFFER_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR :: EXTERNAL_MEMORY_IMAGE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXPORT_MEMORY_ALLOCATE_INFO_KHR :: EXPORT_MEMORY_ALLOCATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_SEMAPHORE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_SEMAPHORE_PROPERTIES_KHR :: EXTERNAL_SEMAPHORE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXPORT_SEMAPHORE_CREATE_INFO_KHR :: EXPORT_SEMAPHORE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR :: PHYSICAL_DEVICE_16BIT_STORAGE_FEATURES_KHR;
        VK_STRUCTURE_TYPE_DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR :: DESCRIPTOR_UPDATE_TEMPLATE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES2_EXT :: SURFACE_CAPABILITIES2_EXT;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR :: PHYSICAL_DEVICE_EXTERNAL_FENCE_INFO_KHR;
        VK_STRUCTURE_TYPE_EXTERNAL_FENCE_PROPERTIES_KHR :: EXTERNAL_FENCE_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_EXPORT_FENCE_CREATE_INFO_KHR :: EXPORT_FENCE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR :: PHYSICAL_DEVICE_POINT_CLIPPING_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR :: RENDER_PASS_INPUT_ATTACHMENT_ASPECT_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_IMAGE_VIEW_USAGE_CREATE_INFO_KHR :: IMAGE_VIEW_USAGE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR :: PIPELINE_TESSELLATION_DOMAIN_ORIGIN_STATE_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR :: PHYSICAL_DEVICE_VARIABLE_POINTER_FEATURES_KHR;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_REQUIREMENTS_KHR :: MEMORY_DEDICATED_REQUIREMENTS_KHR;
        VK_STRUCTURE_TYPE_MEMORY_DEDICATED_ALLOCATE_INFO_KHR :: MEMORY_DEDICATED_ALLOCATE_INFO_KHR;
        VK_STRUCTURE_TYPE_BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR :: BUFFER_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR :: IMAGE_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR :: IMAGE_SPARSE_MEMORY_REQUIREMENTS_INFO_2_KHR;
        VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2_KHR :: MEMORY_REQUIREMENTS_2_KHR;
        VK_STRUCTURE_TYPE_SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR :: SPARSE_IMAGE_MEMORY_REQUIREMENTS_2_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR :: SAMPLER_YCBCR_CONVERSION_CREATE_INFO_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_INFO_KHR :: SAMPLER_YCBCR_CONVERSION_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_PLANE_MEMORY_INFO_KHR :: BIND_IMAGE_PLANE_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR :: IMAGE_PLANE_MEMORY_REQUIREMENTS_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR :: PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES_KHR;
        VK_STRUCTURE_TYPE_SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR :: SAMPLER_YCBCR_CONVERSION_IMAGE_FORMAT_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_BIND_BUFFER_MEMORY_INFO_KHR :: BIND_BUFFER_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_BIND_IMAGE_MEMORY_INFO_KHR :: BIND_IMAGE_MEMORY_INFO_KHR;
        VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR :: PHYSICAL_DEVICE_MAINTENANCE_3_PROPERTIES_KHR;
        VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR :: DESCRIPTOR_SET_LAYOUT_SUPPORT_KHR;
        VK_STRUCTURE_TYPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_STRUCTURE_TYPE_END_RANGE :: END_RANGE;
        VK_STRUCTURE_TYPE_RANGE_SIZE :: RANGE_SIZE;
        VK_STRUCTURE_TYPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSubgroupFeatureFlagBits :: enum_flags u32 {
        BASIC_BIT :: 0x1;
        VOTE_BIT :: 0x2;
        ARITHMETIC_BIT :: 0x4;
        BALLOT_BIT :: 0x8;
        SHUFFLE_BIT :: 0x10;
        SHUFFLE_RELATIVE_BIT :: 0x20;
        CLUSTERED_BIT :: 0x40;
        QUAD_BIT :: 0x80;
        PARTITIONED_BIT_NV :: 0x100;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SUBGROUP_FEATURE_BASIC_BIT :: BASIC_BIT;
        VK_SUBGROUP_FEATURE_VOTE_BIT :: VOTE_BIT;
        VK_SUBGROUP_FEATURE_ARITHMETIC_BIT :: ARITHMETIC_BIT;
        VK_SUBGROUP_FEATURE_BALLOT_BIT :: BALLOT_BIT;
        VK_SUBGROUP_FEATURE_SHUFFLE_BIT :: SHUFFLE_BIT;
        VK_SUBGROUP_FEATURE_SHUFFLE_RELATIVE_BIT :: SHUFFLE_RELATIVE_BIT;
        VK_SUBGROUP_FEATURE_CLUSTERED_BIT :: CLUSTERED_BIT;
        VK_SUBGROUP_FEATURE_QUAD_BIT :: QUAD_BIT;
        VK_SUBGROUP_FEATURE_PARTITIONED_BIT_NV :: PARTITIONED_BIT_NV;
        VK_SUBGROUP_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSubgroupFeatureFlagBits :: enum_flags u32 {
        BASIC_BIT :: 0x1;
        VOTE_BIT :: 0x2;
        ARITHMETIC_BIT :: 0x4;
        BALLOT_BIT :: 0x8;
        SHUFFLE_BIT :: 0x10;
        SHUFFLE_RELATIVE_BIT :: 0x20;
        CLUSTERED_BIT :: 0x40;
        QUAD_BIT :: 0x80;
        PARTITIONED_BIT_NV :: 0x100;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SUBGROUP_FEATURE_BASIC_BIT :: BASIC_BIT;
        VK_SUBGROUP_FEATURE_VOTE_BIT :: VOTE_BIT;
        VK_SUBGROUP_FEATURE_ARITHMETIC_BIT :: ARITHMETIC_BIT;
        VK_SUBGROUP_FEATURE_BALLOT_BIT :: BALLOT_BIT;
        VK_SUBGROUP_FEATURE_SHUFFLE_BIT :: SHUFFLE_BIT;
        VK_SUBGROUP_FEATURE_SHUFFLE_RELATIVE_BIT :: SHUFFLE_RELATIVE_BIT;
        VK_SUBGROUP_FEATURE_CLUSTERED_BIT :: CLUSTERED_BIT;
        VK_SUBGROUP_FEATURE_QUAD_BIT :: QUAD_BIT;
        VK_SUBGROUP_FEATURE_PARTITIONED_BIT_NV :: PARTITIONED_BIT_NV;
        VK_SUBGROUP_FEATURE_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSubmitInfo :: struct {
        sType: VkStructureType = .SUBMIT_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        pWaitDstStageMask: *VkPipelineStageFlags;
        commandBufferCount: u32;
        pCommandBuffers: *VkCommandBuffer;
        signalSemaphoreCount: u32;
        pSignalSemaphores: *VkSemaphore;
    }

}

#if OS == .LINUX {
    VkSubmitInfo :: struct {
        sType: VkStructureType = .SUBMIT_INFO;
        pNext: *void;
        waitSemaphoreCount: u32;
        pWaitSemaphores: *VkSemaphore;
        pWaitDstStageMask: *VkPipelineStageFlags;
        commandBufferCount: u32;
        pCommandBuffers: *VkCommandBuffer;
        signalSemaphoreCount: u32;
        pSignalSemaphores: *VkSemaphore;
    }

}

#if OS == .WINDOWS {
    VkSubpassBeginInfoKHR :: struct {
        sType: VkStructureType = .SUBPASS_BEGIN_INFO_KHR;
        pNext: *void;
        contents: VkSubpassContents;
    }

}

#if OS == .LINUX {
    VkSubpassBeginInfoKHR :: struct {
        sType: VkStructureType = .SUBPASS_BEGIN_INFO_KHR;
        pNext: *void;
        contents: VkSubpassContents;
    }

}

#if OS == .WINDOWS {
    VkSubpassContents :: enum u32 {
        INLINE :: 0;
        SECONDARY_COMMAND_BUFFERS :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SUBPASS_CONTENTS_INLINE :: INLINE;
        VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS :: SECONDARY_COMMAND_BUFFERS;
        VK_SUBPASS_CONTENTS_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SUBPASS_CONTENTS_END_RANGE :: END_RANGE;
        VK_SUBPASS_CONTENTS_RANGE_SIZE :: RANGE_SIZE;
        VK_SUBPASS_CONTENTS_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSubpassContents :: enum u32 {
        INLINE :: 0;
        SECONDARY_COMMAND_BUFFERS :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_SUBPASS_CONTENTS_INLINE :: INLINE;
        VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS :: SECONDARY_COMMAND_BUFFERS;
        VK_SUBPASS_CONTENTS_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SUBPASS_CONTENTS_END_RANGE :: END_RANGE;
        VK_SUBPASS_CONTENTS_RANGE_SIZE :: RANGE_SIZE;
        VK_SUBPASS_CONTENTS_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSubpassDependency :: struct {
        srcSubpass: u32;
        dstSubpass: u32;
        srcStageMask: VkPipelineStageFlags;
        dstStageMask: VkPipelineStageFlags;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        dependencyFlags: VkDependencyFlags;
    }

}

#if OS == .LINUX {
    VkSubpassDependency :: struct {
        srcSubpass: u32;
        dstSubpass: u32;
        srcStageMask: VkPipelineStageFlags;
        dstStageMask: VkPipelineStageFlags;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        dependencyFlags: VkDependencyFlags;
    }

}

#if OS == .WINDOWS {
    VkSubpassDependency2KHR :: struct {
        sType: VkStructureType = .SUBPASS_DEPENDENCY_2_KHR;
        pNext: *void;
        srcSubpass: u32;
        dstSubpass: u32;
        srcStageMask: VkPipelineStageFlags;
        dstStageMask: VkPipelineStageFlags;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        dependencyFlags: VkDependencyFlags;
        viewOffset: s32;
    }

}

#if OS == .LINUX {
    VkSubpassDependency2KHR :: struct {
        sType: VkStructureType = .SUBPASS_DEPENDENCY_2_KHR;
        pNext: *void;
        srcSubpass: u32;
        dstSubpass: u32;
        srcStageMask: VkPipelineStageFlags;
        dstStageMask: VkPipelineStageFlags;
        srcAccessMask: VkAccessFlags;
        dstAccessMask: VkAccessFlags;
        dependencyFlags: VkDependencyFlags;
        viewOffset: s32;
    }

}

#if OS == .WINDOWS {
    VkSubpassDescription :: struct {
        flags: VkSubpassDescriptionFlags;
        pipelineBindPoint: VkPipelineBindPoint;
        inputAttachmentCount: u32;
        pInputAttachments: *VkAttachmentReference;
        colorAttachmentCount: u32;
        pColorAttachments: *VkAttachmentReference;
        pResolveAttachments: *VkAttachmentReference;
        pDepthStencilAttachment: *VkAttachmentReference;
        preserveAttachmentCount: u32;
        pPreserveAttachments: *u32;
    }

}

#if OS == .LINUX {
    VkSubpassDescription :: struct {
        flags: VkSubpassDescriptionFlags;
        pipelineBindPoint: VkPipelineBindPoint;
        inputAttachmentCount: u32;
        pInputAttachments: *VkAttachmentReference;
        colorAttachmentCount: u32;
        pColorAttachments: *VkAttachmentReference;
        pResolveAttachments: *VkAttachmentReference;
        pDepthStencilAttachment: *VkAttachmentReference;
        preserveAttachmentCount: u32;
        pPreserveAttachments: *u32;
    }

}

#if OS == .WINDOWS {
    VkSubpassDescription2KHR :: struct {
        sType: VkStructureType = .SUBPASS_DESCRIPTION_2_KHR;
        pNext: *void;
        flags: VkSubpassDescriptionFlags;
        pipelineBindPoint: VkPipelineBindPoint;
        viewMask: u32;
        inputAttachmentCount: u32;
        pInputAttachments: *VkAttachmentReference2KHR;
        colorAttachmentCount: u32;
        pColorAttachments: *VkAttachmentReference2KHR;
        pResolveAttachments: *VkAttachmentReference2KHR;
        pDepthStencilAttachment: *VkAttachmentReference2KHR;
        preserveAttachmentCount: u32;
        pPreserveAttachments: *u32;
    }

}

#if OS == .LINUX {
    VkSubpassDescription2KHR :: struct {
        sType: VkStructureType = .SUBPASS_DESCRIPTION_2_KHR;
        pNext: *void;
        flags: VkSubpassDescriptionFlags;
        pipelineBindPoint: VkPipelineBindPoint;
        viewMask: u32;
        inputAttachmentCount: u32;
        pInputAttachments: *VkAttachmentReference2KHR;
        colorAttachmentCount: u32;
        pColorAttachments: *VkAttachmentReference2KHR;
        pResolveAttachments: *VkAttachmentReference2KHR;
        pDepthStencilAttachment: *VkAttachmentReference2KHR;
        preserveAttachmentCount: u32;
        pPreserveAttachments: *u32;
    }

}

#if OS == .WINDOWS {
    VkSubpassDescriptionFlagBits :: enum_flags u32 {
        PER_VIEW_ATTRIBUTES_BIT_NVX :: 0x1;
        PER_VIEW_POSITION_X_ONLY_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SUBPASS_DESCRIPTION_PER_VIEW_ATTRIBUTES_BIT_NVX :: PER_VIEW_ATTRIBUTES_BIT_NVX;
        VK_SUBPASS_DESCRIPTION_PER_VIEW_POSITION_X_ONLY_BIT_NVX :: PER_VIEW_POSITION_X_ONLY_BIT_NVX;
        VK_SUBPASS_DESCRIPTION_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSubpassDescriptionFlagBits :: enum_flags u32 {
        PER_VIEW_ATTRIBUTES_BIT_NVX :: 0x1;
        PER_VIEW_POSITION_X_ONLY_BIT_NVX :: 0x2;
        FLAG_BITS_MAX_ENUM :: 0x7fffffff;

        VK_SUBPASS_DESCRIPTION_PER_VIEW_ATTRIBUTES_BIT_NVX :: PER_VIEW_ATTRIBUTES_BIT_NVX;
        VK_SUBPASS_DESCRIPTION_PER_VIEW_POSITION_X_ONLY_BIT_NVX :: PER_VIEW_POSITION_X_ONLY_BIT_NVX;
        VK_SUBPASS_DESCRIPTION_FLAG_BITS_MAX_ENUM :: FLAG_BITS_MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkSubpassEndInfoKHR :: struct {
        sType: VkStructureType = .SUBPASS_END_INFO_KHR;
        pNext: *void;
    }

}

#if OS == .LINUX {
    VkSubpassEndInfoKHR :: struct {
        sType: VkStructureType = .SUBPASS_END_INFO_KHR;
        pNext: *void;
    }

}

#if OS == .WINDOWS {
    VkSubpassSampleLocationsEXT :: struct {
        subpassIndex: u32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .LINUX {
    VkSubpassSampleLocationsEXT :: struct {
        subpassIndex: u32;
        sampleLocationsInfo: VkSampleLocationsInfoEXT;
    }

}

#if OS == .WINDOWS {
    VkSubresourceLayout :: struct {
        offset: VkDeviceSize;
        size: VkDeviceSize;
        rowPitch: VkDeviceSize;
        arrayPitch: VkDeviceSize;
        depthPitch: VkDeviceSize;
    }

}

#if OS == .LINUX {
    VkSubresourceLayout :: struct {
        offset: VkDeviceSize;
        size: VkDeviceSize;
        rowPitch: VkDeviceSize;
        arrayPitch: VkDeviceSize;
        depthPitch: VkDeviceSize;
    }

}

#if OS == .WINDOWS {
    VkSurfaceCapabilities2EXT :: struct {
        sType: VkStructureType = .SURFACE_CAPABILITIES_2_EXT;
        pNext: *void;
        minImageCount: u32;
        maxImageCount: u32;
        currentExtent: VkExtent2D;
        minImageExtent: VkExtent2D;
        maxImageExtent: VkExtent2D;
        maxImageArrayLayers: u32;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        currentTransform: VkSurfaceTransformFlagBitsKHR;
        supportedCompositeAlpha: VkCompositeAlphaFlagsKHR;
        supportedUsageFlags: VkImageUsageFlags;
        supportedSurfaceCounters: VkSurfaceCounterFlagsEXT;
    }

}

#if OS == .LINUX {
    VkSurfaceCapabilities2EXT :: struct {
        sType: VkStructureType = .SURFACE_CAPABILITIES_2_EXT;
        pNext: *void;
        minImageCount: u32;
        maxImageCount: u32;
        currentExtent: VkExtent2D;
        minImageExtent: VkExtent2D;
        maxImageExtent: VkExtent2D;
        maxImageArrayLayers: u32;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        currentTransform: VkSurfaceTransformFlagBitsKHR;
        supportedCompositeAlpha: VkCompositeAlphaFlagsKHR;
        supportedUsageFlags: VkImageUsageFlags;
        supportedSurfaceCounters: VkSurfaceCounterFlagsEXT;
    }

}

#if OS == .WINDOWS {
    VkSurfaceCapabilities2KHR :: struct {
        sType: VkStructureType = .SURFACE_CAPABILITIES_2_KHR;
        pNext: *void;
        surfaceCapabilities: VkSurfaceCapabilitiesKHR;
    }

}

#if OS == .LINUX {
    VkSurfaceCapabilities2KHR :: struct {
        sType: VkStructureType = .SURFACE_CAPABILITIES_2_KHR;
        pNext: *void;
        surfaceCapabilities: VkSurfaceCapabilitiesKHR;
    }

}

#if OS == .WINDOWS {
    VkSurfaceCapabilitiesKHR :: struct {
        minImageCount: u32;
        maxImageCount: u32;
        currentExtent: VkExtent2D;
        minImageExtent: VkExtent2D;
        maxImageExtent: VkExtent2D;
        maxImageArrayLayers: u32;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        currentTransform: VkSurfaceTransformFlagBitsKHR;
        supportedCompositeAlpha: VkCompositeAlphaFlagsKHR;
        supportedUsageFlags: VkImageUsageFlags;
    }

}

#if OS == .LINUX {
    VkSurfaceCapabilitiesKHR :: struct {
        minImageCount: u32;
        maxImageCount: u32;
        currentExtent: VkExtent2D;
        minImageExtent: VkExtent2D;
        maxImageExtent: VkExtent2D;
        maxImageArrayLayers: u32;
        supportedTransforms: VkSurfaceTransformFlagsKHR;
        currentTransform: VkSurfaceTransformFlagBitsKHR;
        supportedCompositeAlpha: VkCompositeAlphaFlagsKHR;
        supportedUsageFlags: VkImageUsageFlags;
    }

}

#if OS == .WINDOWS {
    VkSurfaceCounterFlagBitsEXT :: enum_flags u32 {
        VBLANK_EXT :: 0x1;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_SURFACE_COUNTER_VBLANK_EXT :: VBLANK_EXT;
        VK_SURFACE_COUNTER_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkSurfaceCounterFlagBitsEXT :: enum_flags u32 {
        VBLANK_EXT :: 0x1;
        FLAG_BITS_MAX_ENUM_EXT :: 0x7fffffff;

        VK_SURFACE_COUNTER_VBLANK_EXT :: VBLANK_EXT;
        VK_SURFACE_COUNTER_FLAG_BITS_MAX_ENUM_EXT :: FLAG_BITS_MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkSurfaceFormat2KHR :: struct {
        sType: VkStructureType = .SURFACE_FORMAT_2_KHR;
        pNext: *void;
        surfaceFormat: VkSurfaceFormatKHR;
    }

}

#if OS == .LINUX {
    VkSurfaceFormat2KHR :: struct {
        sType: VkStructureType = .SURFACE_FORMAT_2_KHR;
        pNext: *void;
        surfaceFormat: VkSurfaceFormatKHR;
    }

}

#if OS == .WINDOWS {
    VkSurfaceFormatKHR :: struct {
        format: VkFormat;
        colorSpace: VkColorSpaceKHR;
    }

}

#if OS == .LINUX {
    VkSurfaceFormatKHR :: struct {
        format: VkFormat;
        colorSpace: VkColorSpaceKHR;
    }

}

#if OS == .WINDOWS {
    VkSurfaceKHR_T :: struct {}

}

#if OS == .LINUX {
    VkSurfaceKHR_T :: struct {}

}

#if OS == .WINDOWS {
    VkSurfaceTransformFlagBitsKHR :: enum_flags u32 {
        IDENTITY_BIT_KHR :: 0x1;
        ROTATE_90_BIT_KHR :: 0x2;
        ROTATE_180_BIT_KHR :: 0x4;
        ROTATE_270_BIT_KHR :: 0x8;
        HORIZONTAL_MIRROR_BIT_KHR :: 0x10;
        HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR :: 0x20;
        HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR :: 0x40;
        HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR :: 0x80;
        INHERIT_BIT_KHR :: 0x100;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR :: IDENTITY_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_90_BIT_KHR :: ROTATE_90_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_180_BIT_KHR :: ROTATE_180_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_270_BIT_KHR :: ROTATE_270_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_BIT_KHR :: HORIZONTAL_MIRROR_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR;
        VK_SURFACE_TRANSFORM_INHERIT_BIT_KHR :: INHERIT_BIT_KHR;
        VK_SURFACE_TRANSFORM_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkSurfaceTransformFlagBitsKHR :: enum_flags u32 {
        IDENTITY_BIT_KHR :: 0x1;
        ROTATE_90_BIT_KHR :: 0x2;
        ROTATE_180_BIT_KHR :: 0x4;
        ROTATE_270_BIT_KHR :: 0x8;
        HORIZONTAL_MIRROR_BIT_KHR :: 0x10;
        HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR :: 0x20;
        HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR :: 0x40;
        HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR :: 0x80;
        INHERIT_BIT_KHR :: 0x100;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR :: IDENTITY_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_90_BIT_KHR :: ROTATE_90_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_180_BIT_KHR :: ROTATE_180_BIT_KHR;
        VK_SURFACE_TRANSFORM_ROTATE_270_BIT_KHR :: ROTATE_270_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_BIT_KHR :: HORIZONTAL_MIRROR_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_90_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_180_BIT_KHR;
        VK_SURFACE_TRANSFORM_HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR :: HORIZONTAL_MIRROR_ROTATE_270_BIT_KHR;
        VK_SURFACE_TRANSFORM_INHERIT_BIT_KHR :: INHERIT_BIT_KHR;
        VK_SURFACE_TRANSFORM_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkSwapchainCounterCreateInfoEXT :: struct {
        sType: VkStructureType = .SWAPCHAIN_COUNTER_CREATE_INFO_EXT;
        pNext: *void;
        surfaceCounters: VkSurfaceCounterFlagsEXT;
    }

}

#if OS == .LINUX {
    VkSwapchainCounterCreateInfoEXT :: struct {
        sType: VkStructureType = .SWAPCHAIN_COUNTER_CREATE_INFO_EXT;
        pNext: *void;
        surfaceCounters: VkSurfaceCounterFlagsEXT;
    }

}

#if OS == .WINDOWS {
    VkSwapchainCreateFlagBitsKHR :: enum_flags u32 {
        SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: 0x1;
        PROTECTED_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_SWAPCHAIN_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR;
        VK_SWAPCHAIN_CREATE_PROTECTED_BIT_KHR :: PROTECTED_BIT_KHR;
        VK_SWAPCHAIN_CREATE_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .LINUX {
    VkSwapchainCreateFlagBitsKHR :: enum_flags u32 {
        SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: 0x1;
        PROTECTED_BIT_KHR :: 0x2;
        FLAG_BITS_MAX_ENUM_KHR :: 0x7fffffff;

        VK_SWAPCHAIN_CREATE_SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR :: SPLIT_INSTANCE_BIND_REGIONS_BIT_KHR;
        VK_SWAPCHAIN_CREATE_PROTECTED_BIT_KHR :: PROTECTED_BIT_KHR;
        VK_SWAPCHAIN_CREATE_FLAG_BITS_MAX_ENUM_KHR :: FLAG_BITS_MAX_ENUM_KHR;
    }

}

#if OS == .WINDOWS {
    VkSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkSwapchainCreateFlagsKHR;
        surface: VkSurfaceKHR;
        minImageCount: u32;
        imageFormat: VkFormat;
        imageColorSpace: VkColorSpaceKHR;
        imageExtent: VkExtent2D;
        imageArrayLayers: u32;
        imageUsage: VkImageUsageFlags;
        imageSharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
        preTransform: VkSurfaceTransformFlagBitsKHR;
        compositeAlpha: VkCompositeAlphaFlagBitsKHR;
        presentMode: VkPresentModeKHR;
        clipped: VkBool32;
        oldSwapchain: VkSwapchainKHR;
    }

}

#if OS == .LINUX {
    VkSwapchainCreateInfoKHR :: struct {
        sType: VkStructureType = .SWAPCHAIN_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkSwapchainCreateFlagsKHR;
        surface: VkSurfaceKHR;
        minImageCount: u32;
        imageFormat: VkFormat;
        imageColorSpace: VkColorSpaceKHR;
        imageExtent: VkExtent2D;
        imageArrayLayers: u32;
        imageUsage: VkImageUsageFlags;
        imageSharingMode: VkSharingMode;
        queueFamilyIndexCount: u32;
        pQueueFamilyIndices: *u32;
        preTransform: VkSurfaceTransformFlagBitsKHR;
        compositeAlpha: VkCompositeAlphaFlagBitsKHR;
        presentMode: VkPresentModeKHR;
        clipped: VkBool32;
        oldSwapchain: VkSwapchainKHR;
    }

}

#if OS == .WINDOWS {
    VkSwapchainKHR_T :: struct {}

}

#if OS == .LINUX {
    VkSwapchainKHR_T :: struct {}

}

#if OS == .WINDOWS {
    VkSystemAllocationScope :: enum u32 {
        COMMAND :: 0;
        OBJECT :: 1;
        CACHE :: 2;
        DEVICE :: 3;
        INSTANCE :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_SYSTEM_ALLOCATION_SCOPE_COMMAND :: COMMAND;
        VK_SYSTEM_ALLOCATION_SCOPE_OBJECT :: OBJECT;
        VK_SYSTEM_ALLOCATION_SCOPE_CACHE :: CACHE;
        VK_SYSTEM_ALLOCATION_SCOPE_DEVICE :: DEVICE;
        VK_SYSTEM_ALLOCATION_SCOPE_INSTANCE :: INSTANCE;
        VK_SYSTEM_ALLOCATION_SCOPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SYSTEM_ALLOCATION_SCOPE_END_RANGE :: END_RANGE;
        VK_SYSTEM_ALLOCATION_SCOPE_RANGE_SIZE :: RANGE_SIZE;
        VK_SYSTEM_ALLOCATION_SCOPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkSystemAllocationScope :: enum u32 {
        COMMAND :: 0;
        OBJECT :: 1;
        CACHE :: 2;
        DEVICE :: 3;
        INSTANCE :: 4;
        BEGIN_RANGE :: 0;
        END_RANGE :: 4;
        RANGE_SIZE :: 5;
        MAX_ENUM :: 2147483647;

        VK_SYSTEM_ALLOCATION_SCOPE_COMMAND :: COMMAND;
        VK_SYSTEM_ALLOCATION_SCOPE_OBJECT :: OBJECT;
        VK_SYSTEM_ALLOCATION_SCOPE_CACHE :: CACHE;
        VK_SYSTEM_ALLOCATION_SCOPE_DEVICE :: DEVICE;
        VK_SYSTEM_ALLOCATION_SCOPE_INSTANCE :: INSTANCE;
        VK_SYSTEM_ALLOCATION_SCOPE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_SYSTEM_ALLOCATION_SCOPE_END_RANGE :: END_RANGE;
        VK_SYSTEM_ALLOCATION_SCOPE_RANGE_SIZE :: RANGE_SIZE;
        VK_SYSTEM_ALLOCATION_SCOPE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkTessellationDomainOrigin :: enum u32 {
        UPPER_LEFT :: 0;
        LOWER_LEFT :: 1;
        UPPER_LEFT_KHR :: 0;
        LOWER_LEFT_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_TESSELLATION_DOMAIN_ORIGIN_UPPER_LEFT :: UPPER_LEFT;
        VK_TESSELLATION_DOMAIN_ORIGIN_LOWER_LEFT :: LOWER_LEFT;
        VK_TESSELLATION_DOMAIN_ORIGIN_UPPER_LEFT_KHR :: UPPER_LEFT_KHR;
        VK_TESSELLATION_DOMAIN_ORIGIN_LOWER_LEFT_KHR :: LOWER_LEFT_KHR;
        VK_TESSELLATION_DOMAIN_ORIGIN_BEGIN_RANGE :: BEGIN_RANGE;
        VK_TESSELLATION_DOMAIN_ORIGIN_END_RANGE :: END_RANGE;
        VK_TESSELLATION_DOMAIN_ORIGIN_RANGE_SIZE :: RANGE_SIZE;
        VK_TESSELLATION_DOMAIN_ORIGIN_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkTessellationDomainOrigin :: enum u32 {
        UPPER_LEFT :: 0;
        LOWER_LEFT :: 1;
        UPPER_LEFT_KHR :: 0;
        LOWER_LEFT_KHR :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_TESSELLATION_DOMAIN_ORIGIN_UPPER_LEFT :: UPPER_LEFT;
        VK_TESSELLATION_DOMAIN_ORIGIN_LOWER_LEFT :: LOWER_LEFT;
        VK_TESSELLATION_DOMAIN_ORIGIN_UPPER_LEFT_KHR :: UPPER_LEFT_KHR;
        VK_TESSELLATION_DOMAIN_ORIGIN_LOWER_LEFT_KHR :: LOWER_LEFT_KHR;
        VK_TESSELLATION_DOMAIN_ORIGIN_BEGIN_RANGE :: BEGIN_RANGE;
        VK_TESSELLATION_DOMAIN_ORIGIN_END_RANGE :: END_RANGE;
        VK_TESSELLATION_DOMAIN_ORIGIN_RANGE_SIZE :: RANGE_SIZE;
        VK_TESSELLATION_DOMAIN_ORIGIN_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkTextureLODGatherFormatPropertiesAMD :: struct {
        sType: VkStructureType = .TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD;
        pNext: *void;
        supportsTextureGatherLODBiasAMD: VkBool32;
    }

}

#if OS == .LINUX {
    VkTextureLODGatherFormatPropertiesAMD :: struct {
        sType: VkStructureType = .TEXTURE_LOD_GATHER_FORMAT_PROPERTIES_AMD;
        pNext: *void;
        supportsTextureGatherLODBiasAMD: VkBool32;
    }

}

#if OS == .WINDOWS {
    VkValidationCacheCreateInfoEXT :: struct {
        sType: VkStructureType = .VALIDATION_CACHE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkValidationCacheCreateFlagsEXT;
        initialDataSize: u64;
        pInitialData: *void;
    }

}

#if OS == .LINUX {
    VkValidationCacheCreateInfoEXT :: struct {
        sType: VkStructureType = .VALIDATION_CACHE_CREATE_INFO_EXT;
        pNext: *void;
        flags: VkValidationCacheCreateFlagsEXT;
        initialDataSize: u64;
        pInitialData: *void;
    }

}

#if OS == .WINDOWS {
    VkValidationCacheEXT_T :: struct {}

}

#if OS == .LINUX {
    VkValidationCacheEXT_T :: struct {}

}

#if OS == .WINDOWS {
    VkValidationCacheHeaderVersionEXT :: enum u32 {
        ONE_EXT :: 1;
        BEGIN_RANGE_EXT :: 1;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_VALIDATION_CACHE_HEADER_VERSION_ONE_EXT :: ONE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_END_RANGE_EXT :: END_RANGE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkValidationCacheHeaderVersionEXT :: enum u32 {
        ONE_EXT :: 1;
        BEGIN_RANGE_EXT :: 1;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 1;
        MAX_ENUM_EXT :: 2147483647;

        VK_VALIDATION_CACHE_HEADER_VERSION_ONE_EXT :: ONE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_END_RANGE_EXT :: END_RANGE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_VALIDATION_CACHE_HEADER_VERSION_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkValidationCheckEXT :: enum u32 {
        ALL_EXT :: 0;
        SHADERS_EXT :: 1;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 2;
        MAX_ENUM_EXT :: 2147483647;

        VK_VALIDATION_CHECK_ALL_EXT :: ALL_EXT;
        VK_VALIDATION_CHECK_SHADERS_EXT :: SHADERS_EXT;
        VK_VALIDATION_CHECK_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_VALIDATION_CHECK_END_RANGE_EXT :: END_RANGE_EXT;
        VK_VALIDATION_CHECK_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_VALIDATION_CHECK_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .LINUX {
    VkValidationCheckEXT :: enum u32 {
        ALL_EXT :: 0;
        SHADERS_EXT :: 1;
        BEGIN_RANGE_EXT :: 0;
        END_RANGE_EXT :: 1;
        RANGE_SIZE_EXT :: 2;
        MAX_ENUM_EXT :: 2147483647;

        VK_VALIDATION_CHECK_ALL_EXT :: ALL_EXT;
        VK_VALIDATION_CHECK_SHADERS_EXT :: SHADERS_EXT;
        VK_VALIDATION_CHECK_BEGIN_RANGE_EXT :: BEGIN_RANGE_EXT;
        VK_VALIDATION_CHECK_END_RANGE_EXT :: END_RANGE_EXT;
        VK_VALIDATION_CHECK_RANGE_SIZE_EXT :: RANGE_SIZE_EXT;
        VK_VALIDATION_CHECK_MAX_ENUM_EXT :: MAX_ENUM_EXT;
    }

}

#if OS == .WINDOWS {
    VkValidationFlagsEXT :: struct {
        sType: VkStructureType = .VALIDATION_FLAGS_EXT;
        pNext: *void;
        disabledValidationCheckCount: u32;
        pDisabledValidationChecks: *VkValidationCheckEXT;
    }

}

#if OS == .LINUX {
    VkValidationFlagsEXT :: struct {
        sType: VkStructureType = .VALIDATION_FLAGS_EXT;
        pNext: *void;
        disabledValidationCheckCount: u32;
        pDisabledValidationChecks: *VkValidationCheckEXT;
    }

}

#if OS == .WINDOWS {
    VkVendorId :: enum u32 {
        VIV :: 65537;
        VSI :: 65538;
        KAZAN :: 65539;
        BEGIN_RANGE :: 65537;
        END_RANGE :: 65539;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_VENDOR_ID_VIV :: VIV;
        VK_VENDOR_ID_VSI :: VSI;
        VK_VENDOR_ID_KAZAN :: KAZAN;
        VK_VENDOR_ID_BEGIN_RANGE :: BEGIN_RANGE;
        VK_VENDOR_ID_END_RANGE :: END_RANGE;
        VK_VENDOR_ID_RANGE_SIZE :: RANGE_SIZE;
        VK_VENDOR_ID_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkVendorId :: enum u32 {
        VIV :: 65537;
        VSI :: 65538;
        KAZAN :: 65539;
        BEGIN_RANGE :: 65537;
        END_RANGE :: 65539;
        RANGE_SIZE :: 3;
        MAX_ENUM :: 2147483647;

        VK_VENDOR_ID_VIV :: VIV;
        VK_VENDOR_ID_VSI :: VSI;
        VK_VENDOR_ID_KAZAN :: KAZAN;
        VK_VENDOR_ID_BEGIN_RANGE :: BEGIN_RANGE;
        VK_VENDOR_ID_END_RANGE :: END_RANGE;
        VK_VENDOR_ID_RANGE_SIZE :: RANGE_SIZE;
        VK_VENDOR_ID_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkVertexInputAttributeDescription :: struct {
        location: u32;
        binding: u32;
        format: VkFormat;
        offset: u32;
    }

}

#if OS == .LINUX {
    VkVertexInputAttributeDescription :: struct {
        location: u32;
        binding: u32;
        format: VkFormat;
        offset: u32;
    }

}

#if OS == .WINDOWS {
    VkVertexInputBindingDescription :: struct {
        binding: u32;
        stride: u32;
        inputRate: VkVertexInputRate;
    }

}

#if OS == .LINUX {
    VkVertexInputBindingDescription :: struct {
        binding: u32;
        stride: u32;
        inputRate: VkVertexInputRate;
    }

}

#if OS == .WINDOWS {
    VkVertexInputBindingDivisorDescriptionEXT :: struct {
        binding: u32;
        divisor: u32;
    }

}

#if OS == .LINUX {
    VkVertexInputBindingDivisorDescriptionEXT :: struct {
        binding: u32;
        divisor: u32;
    }

}

#if OS == .WINDOWS {
    VkVertexInputRate :: enum u32 {
        VERTEX :: 0;
        INSTANCE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_VERTEX_INPUT_RATE_VERTEX :: VERTEX;
        VK_VERTEX_INPUT_RATE_INSTANCE :: INSTANCE;
        VK_VERTEX_INPUT_RATE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_VERTEX_INPUT_RATE_END_RANGE :: END_RANGE;
        VK_VERTEX_INPUT_RATE_RANGE_SIZE :: RANGE_SIZE;
        VK_VERTEX_INPUT_RATE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .LINUX {
    VkVertexInputRate :: enum u32 {
        VERTEX :: 0;
        INSTANCE :: 1;
        BEGIN_RANGE :: 0;
        END_RANGE :: 1;
        RANGE_SIZE :: 2;
        MAX_ENUM :: 2147483647;

        VK_VERTEX_INPUT_RATE_VERTEX :: VERTEX;
        VK_VERTEX_INPUT_RATE_INSTANCE :: INSTANCE;
        VK_VERTEX_INPUT_RATE_BEGIN_RANGE :: BEGIN_RANGE;
        VK_VERTEX_INPUT_RATE_END_RANGE :: END_RANGE;
        VK_VERTEX_INPUT_RATE_RANGE_SIZE :: RANGE_SIZE;
        VK_VERTEX_INPUT_RATE_MAX_ENUM :: MAX_ENUM;
    }

}

#if OS == .WINDOWS {
    VkViewport :: struct {
        x: float;
        y: float;
        width: float;
        height: float;
        minDepth: float;
        maxDepth: float;
    }

}

#if OS == .LINUX {
    VkViewport :: struct {
        x: float;
        y: float;
        width: float;
        height: float;
        minDepth: float;
        maxDepth: float;
    }

}

#if OS == .WINDOWS {
    VkViewportCoordinateSwizzleNV :: enum u32 {
        POSITIVE_X_NV :: 0;
        NEGATIVE_X_NV :: 1;
        POSITIVE_Y_NV :: 2;
        NEGATIVE_Y_NV :: 3;
        POSITIVE_Z_NV :: 4;
        NEGATIVE_Z_NV :: 5;
        POSITIVE_W_NV :: 6;
        NEGATIVE_W_NV :: 7;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 7;
        RANGE_SIZE_NV :: 8;
        MAX_ENUM_NV :: 2147483647;

        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_X_NV :: POSITIVE_X_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_X_NV :: NEGATIVE_X_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Y_NV :: POSITIVE_Y_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Y_NV :: NEGATIVE_Y_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Z_NV :: POSITIVE_Z_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Z_NV :: NEGATIVE_Z_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_W_NV :: POSITIVE_W_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_W_NV :: NEGATIVE_W_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_END_RANGE_NV :: END_RANGE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .LINUX {
    VkViewportCoordinateSwizzleNV :: enum u32 {
        POSITIVE_X_NV :: 0;
        NEGATIVE_X_NV :: 1;
        POSITIVE_Y_NV :: 2;
        NEGATIVE_Y_NV :: 3;
        POSITIVE_Z_NV :: 4;
        NEGATIVE_Z_NV :: 5;
        POSITIVE_W_NV :: 6;
        NEGATIVE_W_NV :: 7;
        BEGIN_RANGE_NV :: 0;
        END_RANGE_NV :: 7;
        RANGE_SIZE_NV :: 8;
        MAX_ENUM_NV :: 2147483647;

        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_X_NV :: POSITIVE_X_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_X_NV :: NEGATIVE_X_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Y_NV :: POSITIVE_Y_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Y_NV :: NEGATIVE_Y_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_Z_NV :: POSITIVE_Z_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_Z_NV :: NEGATIVE_Z_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_POSITIVE_W_NV :: POSITIVE_W_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_NEGATIVE_W_NV :: NEGATIVE_W_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_BEGIN_RANGE_NV :: BEGIN_RANGE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_END_RANGE_NV :: END_RANGE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_RANGE_SIZE_NV :: RANGE_SIZE_NV;
        VK_VIEWPORT_COORDINATE_SWIZZLE_MAX_ENUM_NV :: MAX_ENUM_NV;
    }

}

#if OS == .WINDOWS {
    VkViewportSwizzleNV :: struct {
        x: VkViewportCoordinateSwizzleNV;
        y: VkViewportCoordinateSwizzleNV;
        z: VkViewportCoordinateSwizzleNV;
        w: VkViewportCoordinateSwizzleNV;
    }

}

#if OS == .LINUX {
    VkViewportSwizzleNV :: struct {
        x: VkViewportCoordinateSwizzleNV;
        y: VkViewportCoordinateSwizzleNV;
        z: VkViewportCoordinateSwizzleNV;
        w: VkViewportCoordinateSwizzleNV;
    }

}

#if OS == .WINDOWS {
    VkViewportWScalingNV :: struct {
        xcoeff: float;
        ycoeff: float;
    }

}

#if OS == .LINUX {
    VkViewportWScalingNV :: struct {
        xcoeff: float;
        ycoeff: float;
    }

}

#if OS == .WINDOWS {
    VkWin32KeyedMutexAcquireReleaseInfoKHR :: struct {
        sType: VkStructureType = .WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_KHR;
        pNext: *void;
        acquireCount: u32;
        pAcquireSyncs: *VkDeviceMemory;
        pAcquireKeys: *u64;
        pAcquireTimeouts: *u32;
        releaseCount: u32;
        pReleaseSyncs: *VkDeviceMemory;
        pReleaseKeys: *u64;
    }

}

#if OS == .WINDOWS {
    VkWin32KeyedMutexAcquireReleaseInfoNV :: struct {
        sType: VkStructureType = .WIN32_KEYED_MUTEX_ACQUIRE_RELEASE_INFO_NV;
        pNext: *void;
        acquireCount: u32;
        pAcquireSyncs: *VkDeviceMemory;
        pAcquireKeys: *u64;
        pAcquireTimeoutMilliseconds: *u32;
        releaseCount: u32;
        pReleaseSyncs: *VkDeviceMemory;
        pReleaseKeys: *u64;
    }

}

#if OS == .WINDOWS {
    VkWin32SurfaceCreateInfoKHR :: struct {
        sType: VkStructureType = .WIN32_SURFACE_CREATE_INFO_KHR;
        pNext: *void;
        flags: VkWin32SurfaceCreateFlagsKHR;
        hinstance: HINSTANCE;
        hwnd: HWND;
    }

}

#if OS == .WINDOWS {
    VkWriteDescriptorSet :: struct {
        sType: VkStructureType = .WRITE_DESCRIPTOR_SET;
        pNext: *void;
        dstSet: VkDescriptorSet;
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
        descriptorType: VkDescriptorType;
        pImageInfo: *VkDescriptorImageInfo;
        pBufferInfo: *VkDescriptorBufferInfo;
        pTexelBufferView: *VkBufferView;
    }

}

#if OS == .LINUX {
    VkWriteDescriptorSet :: struct {
        sType: VkStructureType = .WRITE_DESCRIPTOR_SET;
        pNext: *void;
        dstSet: VkDescriptorSet;
        dstBinding: u32;
        dstArrayElement: u32;
        descriptorCount: u32;
        descriptorType: VkDescriptorType;
        pImageInfo: *VkDescriptorImageInfo;
        pBufferInfo: *VkDescriptorBufferInfo;
        pTexelBufferView: *VkBufferView;
    }

}

#if OS == .WINDOWS {
    VkWriteDescriptorSetInlineUniformBlockEXT :: struct {
        sType: VkStructureType = .WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT;
        pNext: *void;
        dataSize: u32;
        pData: *void;
    }

}

#if OS == .LINUX {
    VkWriteDescriptorSetInlineUniformBlockEXT :: struct {
        sType: VkStructureType = .WRITE_DESCRIPTOR_SET_INLINE_UNIFORM_BLOCK_EXT;
        pNext: *void;
        dataSize: u32;
        pData: *void;
    }

}

#if OS == .WINDOWS {
    VkXYColorEXT :: struct {
        x: float;
        y: float;
    }

}

#if OS == .LINUX {
    VkXYColorEXT :: struct {
        x: float;
        y: float;
    }

}

// Callback and opaque types

#if OS == .WINDOWS {
    PFN_vkAcquireNextImage2KHR :: #type (device: VkDevice, pAcquireInfo: *VkAcquireNextImageInfoKHR, pImageIndex: *u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkAcquireNextImage2KHR :: #type (device: VkDevice, pAcquireInfo: *VkAcquireNextImageInfoKHR, pImageIndex: *u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkAcquireNextImageKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, timeout: u64, semaphore: VkSemaphore, fence: VkFence, pImageIndex: *u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkAcquireNextImageKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, timeout: u64, semaphore: VkSemaphore, fence: VkFence, pImageIndex: *u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkAllocateCommandBuffers :: #type (device: VkDevice, pAllocateInfo: *VkCommandBufferAllocateInfo, pCommandBuffers: *VkCommandBuffer) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkAllocateCommandBuffers :: #type (device: VkDevice, pAllocateInfo: *VkCommandBufferAllocateInfo, pCommandBuffers: *VkCommandBuffer) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkAllocateDescriptorSets :: #type (device: VkDevice, pAllocateInfo: *VkDescriptorSetAllocateInfo, pDescriptorSets: *VkDescriptorSet) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkAllocateDescriptorSets :: #type (device: VkDevice, pAllocateInfo: *VkDescriptorSetAllocateInfo, pDescriptorSets: *VkDescriptorSet) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkAllocateMemory :: #type (device: VkDevice, pAllocateInfo: *VkMemoryAllocateInfo, pAllocator: *VkAllocationCallbacks, pMemory: *VkDeviceMemory) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkAllocateMemory :: #type (device: VkDevice, pAllocateInfo: *VkMemoryAllocateInfo, pAllocator: *VkAllocationCallbacks, pMemory: *VkDeviceMemory) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkAllocationFunction :: #type (pUserData: *void, size: u64, alignment: u64, allocationScope: VkSystemAllocationScope) -> *void #c_call;
}

#if OS == .LINUX {
    PFN_vkAllocationFunction :: #type (pUserData: *void, size: u64, alignment: u64, allocationScope: VkSystemAllocationScope) -> *void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBeginCommandBuffer :: #type (commandBuffer: VkCommandBuffer, pBeginInfo: *VkCommandBufferBeginInfo) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBeginCommandBuffer :: #type (commandBuffer: VkCommandBuffer, pBeginInfo: *VkCommandBufferBeginInfo) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindAccelerationStructureMemoryNVX :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindAccelerationStructureMemoryInfoNVX) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindAccelerationStructureMemoryNVX :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindAccelerationStructureMemoryInfoNVX) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindBufferMemory :: #type (device: VkDevice, buffer: VkBuffer, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindBufferMemory :: #type (device: VkDevice, buffer: VkBuffer, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindBufferMemory2 :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindBufferMemory2 :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindBufferMemory2KHR :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindBufferMemory2KHR :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindImageMemory :: #type (device: VkDevice, image: VkImage, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindImageMemory :: #type (device: VkDevice, image: VkImage, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindImageMemory2 :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindImageMemory2 :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkBindImageMemory2KHR :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkBindImageMemory2KHR :: #type (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBeginConditionalRenderingEXT :: #type (commandBuffer: VkCommandBuffer, pConditionalRenderingBegin: *VkConditionalRenderingBeginInfoEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBeginConditionalRenderingEXT :: #type (commandBuffer: VkCommandBuffer, pConditionalRenderingBegin: *VkConditionalRenderingBeginInfoEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBeginDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBeginDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBeginQuery :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32, flags: VkQueryControlFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBeginQuery :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32, flags: VkQueryControlFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBeginRenderPass :: #type (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, contents: VkSubpassContents) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBeginRenderPass :: #type (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, contents: VkSubpassContents) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBeginRenderPass2KHR :: #type (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, pSubpassBeginInfo: *VkSubpassBeginInfoKHR) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBeginRenderPass2KHR :: #type (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, pSubpassBeginInfo: *VkSubpassBeginInfoKHR) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBindDescriptorSets :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, firstSet: u32, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet, dynamicOffsetCount: u32, pDynamicOffsets: *u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBindDescriptorSets :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, firstSet: u32, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet, dynamicOffsetCount: u32, pDynamicOffsets: *u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBindIndexBuffer :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, indexType: VkIndexType) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBindIndexBuffer :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, indexType: VkIndexType) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBindPipeline :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, pipeline: VkPipeline) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBindPipeline :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, pipeline: VkPipeline) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBindShadingRateImageNV :: #type (commandBuffer: VkCommandBuffer, imageView: VkImageView, imageLayout: VkImageLayout) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBindShadingRateImageNV :: #type (commandBuffer: VkCommandBuffer, imageView: VkImageView, imageLayout: VkImageLayout) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBindVertexBuffers :: #type (commandBuffer: VkCommandBuffer, firstBinding: u32, bindingCount: u32, pBuffers: *VkBuffer, pOffsets: *VkDeviceSize) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBindVertexBuffers :: #type (commandBuffer: VkCommandBuffer, firstBinding: u32, bindingCount: u32, pBuffers: *VkBuffer, pOffsets: *VkDeviceSize) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBlitImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageBlit, filter: VkFilter) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBlitImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageBlit, filter: VkFilter) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdBuildAccelerationStructureNVX :: #type (cmdBuf: VkCommandBuffer, type: VkAccelerationStructureTypeNVX, instanceCount: u32, instanceData: VkBuffer, instanceOffset: VkDeviceSize, geometryCount: u32, pGeometries: *VkGeometryNVX, flags: VkBuildAccelerationStructureFlagsNVX, update: VkBool32, dst: VkAccelerationStructureNVX, src: VkAccelerationStructureNVX, scratch: VkBuffer, scratchOffset: VkDeviceSize) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdBuildAccelerationStructureNVX :: #type (cmdBuf: VkCommandBuffer, type: VkAccelerationStructureTypeNVX, instanceCount: u32, instanceData: VkBuffer, instanceOffset: VkDeviceSize, geometryCount: u32, pGeometries: *VkGeometryNVX, flags: VkBuildAccelerationStructureFlagsNVX, update: VkBool32, dst: VkAccelerationStructureNVX, src: VkAccelerationStructureNVX, scratch: VkBuffer, scratchOffset: VkDeviceSize) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdClearAttachments :: #type (commandBuffer: VkCommandBuffer, attachmentCount: u32, pAttachments: *VkClearAttachment, rectCount: u32, pRects: *VkClearRect) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdClearAttachments :: #type (commandBuffer: VkCommandBuffer, attachmentCount: u32, pAttachments: *VkClearAttachment, rectCount: u32, pRects: *VkClearRect) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdClearColorImage :: #type (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pColor: *VkClearColorValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdClearColorImage :: #type (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pColor: *VkClearColorValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdClearDepthStencilImage :: #type (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pDepthStencil: *VkClearDepthStencilValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdClearDepthStencilImage :: #type (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pDepthStencil: *VkClearDepthStencilValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyAccelerationStructureNVX :: #type (cmdBuf: VkCommandBuffer, dst: VkAccelerationStructureNVX, src: VkAccelerationStructureNVX, mode: VkCopyAccelerationStructureModeNVX) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyAccelerationStructureNVX :: #type (cmdBuf: VkCommandBuffer, dst: VkAccelerationStructureNVX, src: VkAccelerationStructureNVX, mode: VkCopyAccelerationStructureModeNVX) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyBuffer :: #type (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferCopy) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyBuffer :: #type (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferCopy) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyBufferToImage :: #type (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyBufferToImage :: #type (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageCopy) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageCopy) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyImageToBuffer :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyImageToBuffer :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdCopyQueryPoolResults :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, stride: VkDeviceSize, flags: VkQueryResultFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdCopyQueryPoolResults :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, stride: VkDeviceSize, flags: VkQueryResultFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDebugMarkerBeginEXT :: #type (commandBuffer: VkCommandBuffer, pMarkerInfo: *VkDebugMarkerMarkerInfoEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDebugMarkerBeginEXT :: #type (commandBuffer: VkCommandBuffer, pMarkerInfo: *VkDebugMarkerMarkerInfoEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDebugMarkerEndEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDebugMarkerEndEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDebugMarkerInsertEXT :: #type (commandBuffer: VkCommandBuffer, pMarkerInfo: *VkDebugMarkerMarkerInfoEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDebugMarkerInsertEXT :: #type (commandBuffer: VkCommandBuffer, pMarkerInfo: *VkDebugMarkerMarkerInfoEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDispatch :: #type (commandBuffer: VkCommandBuffer, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDispatch :: #type (commandBuffer: VkCommandBuffer, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDispatchBase :: #type (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDispatchBase :: #type (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDispatchBaseKHR :: #type (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDispatchBaseKHR :: #type (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDispatchIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDispatchIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDraw :: #type (commandBuffer: VkCommandBuffer, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDraw :: #type (commandBuffer: VkCommandBuffer, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndexed :: #type (commandBuffer: VkCommandBuffer, indexCount: u32, instanceCount: u32, firstIndex: u32, vertexOffset: s32, firstInstance: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndexed :: #type (commandBuffer: VkCommandBuffer, indexCount: u32, instanceCount: u32, firstIndex: u32, vertexOffset: s32, firstInstance: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndexedIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndexedIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndexedIndirectCountAMD :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndexedIndirectCountAMD :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndexedIndirectCountKHR :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndexedIndirectCountKHR :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndirect :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndirectCountAMD :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndirectCountAMD :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawIndirectCountKHR :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawIndirectCountKHR :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawMeshTasksIndirectCountNV :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawMeshTasksIndirectCountNV :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, countBuffer: VkBuffer, countBufferOffset: VkDeviceSize, maxDrawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawMeshTasksIndirectNV :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawMeshTasksIndirectNV :: #type (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdDrawMeshTasksNV :: #type (commandBuffer: VkCommandBuffer, taskCount: u32, firstTask: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdDrawMeshTasksNV :: #type (commandBuffer: VkCommandBuffer, taskCount: u32, firstTask: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdEndConditionalRenderingEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdEndConditionalRenderingEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdEndDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdEndDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdEndQuery :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdEndQuery :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdEndRenderPass :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdEndRenderPass :: #type (commandBuffer: VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdEndRenderPass2KHR :: #type (commandBuffer: VkCommandBuffer, pSubpassEndInfo: *VkSubpassEndInfoKHR) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdEndRenderPass2KHR :: #type (commandBuffer: VkCommandBuffer, pSubpassEndInfo: *VkSubpassEndInfoKHR) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdExecuteCommands :: #type (commandBuffer: VkCommandBuffer, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdExecuteCommands :: #type (commandBuffer: VkCommandBuffer, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdFillBuffer :: #type (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, size: VkDeviceSize, data: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdFillBuffer :: #type (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, size: VkDeviceSize, data: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdInsertDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdInsertDebugUtilsLabelEXT :: #type (commandBuffer: VkCommandBuffer, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdNextSubpass :: #type (commandBuffer: VkCommandBuffer, contents: VkSubpassContents) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdNextSubpass :: #type (commandBuffer: VkCommandBuffer, contents: VkSubpassContents) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdNextSubpass2KHR :: #type (commandBuffer: VkCommandBuffer, pSubpassBeginInfo: *VkSubpassBeginInfoKHR, pSubpassEndInfo: *VkSubpassEndInfoKHR) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdNextSubpass2KHR :: #type (commandBuffer: VkCommandBuffer, pSubpassBeginInfo: *VkSubpassBeginInfoKHR, pSubpassEndInfo: *VkSubpassEndInfoKHR) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdPipelineBarrier :: #type (commandBuffer: VkCommandBuffer, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, dependencyFlags: VkDependencyFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdPipelineBarrier :: #type (commandBuffer: VkCommandBuffer, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, dependencyFlags: VkDependencyFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdProcessCommandsNVX :: #type (commandBuffer: VkCommandBuffer, pProcessCommandsInfo: *VkCmdProcessCommandsInfoNVX) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdProcessCommandsNVX :: #type (commandBuffer: VkCommandBuffer, pProcessCommandsInfo: *VkCmdProcessCommandsInfoNVX) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdPushConstants :: #type (commandBuffer: VkCommandBuffer, layout: VkPipelineLayout, stageFlags: VkShaderStageFlags, offset: u32, size: u32, pValues: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdPushConstants :: #type (commandBuffer: VkCommandBuffer, layout: VkPipelineLayout, stageFlags: VkShaderStageFlags, offset: u32, size: u32, pValues: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdPushDescriptorSetKHR :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, set: u32, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdPushDescriptorSetKHR :: #type (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, set: u32, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdPushDescriptorSetWithTemplateKHR :: #type (commandBuffer: VkCommandBuffer, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, layout: VkPipelineLayout, set: u32, pData: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdPushDescriptorSetWithTemplateKHR :: #type (commandBuffer: VkCommandBuffer, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, layout: VkPipelineLayout, set: u32, pData: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdReserveSpaceForCommandsNVX :: #type (commandBuffer: VkCommandBuffer, pReserveSpaceInfo: *VkCmdReserveSpaceForCommandsInfoNVX) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdReserveSpaceForCommandsNVX :: #type (commandBuffer: VkCommandBuffer, pReserveSpaceInfo: *VkCmdReserveSpaceForCommandsInfoNVX) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdResetEvent :: #type (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdResetEvent :: #type (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdResetQueryPool :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdResetQueryPool :: #type (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdResolveImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageResolve) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdResolveImage :: #type (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageResolve) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetBlendConstants :: #type (commandBuffer: VkCommandBuffer, blendConstants: *[4] float) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetBlendConstants :: #type (commandBuffer: VkCommandBuffer, blendConstants: *[4] float) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetCheckpointNV :: #type (commandBuffer: VkCommandBuffer, pCheckpointMarker: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetCheckpointNV :: #type (commandBuffer: VkCommandBuffer, pCheckpointMarker: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetCoarseSampleOrderNV :: #type (commandBuffer: VkCommandBuffer, sampleOrderType: VkCoarseSampleOrderTypeNV, customSampleOrderCount: u32, pCustomSampleOrders: *VkCoarseSampleOrderCustomNV) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetCoarseSampleOrderNV :: #type (commandBuffer: VkCommandBuffer, sampleOrderType: VkCoarseSampleOrderTypeNV, customSampleOrderCount: u32, pCustomSampleOrders: *VkCoarseSampleOrderCustomNV) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetDepthBias :: #type (commandBuffer: VkCommandBuffer, depthBiasConstantFactor: float, depthBiasClamp: float, depthBiasSlopeFactor: float) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetDepthBias :: #type (commandBuffer: VkCommandBuffer, depthBiasConstantFactor: float, depthBiasClamp: float, depthBiasSlopeFactor: float) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetDepthBounds :: #type (commandBuffer: VkCommandBuffer, minDepthBounds: float, maxDepthBounds: float) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetDepthBounds :: #type (commandBuffer: VkCommandBuffer, minDepthBounds: float, maxDepthBounds: float) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetDeviceMask :: #type (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetDeviceMask :: #type (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetDeviceMaskKHR :: #type (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetDeviceMaskKHR :: #type (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetDiscardRectangleEXT :: #type (commandBuffer: VkCommandBuffer, firstDiscardRectangle: u32, discardRectangleCount: u32, pDiscardRectangles: *VkRect2D) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetDiscardRectangleEXT :: #type (commandBuffer: VkCommandBuffer, firstDiscardRectangle: u32, discardRectangleCount: u32, pDiscardRectangles: *VkRect2D) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetEvent :: #type (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetEvent :: #type (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetExclusiveScissorNV :: #type (commandBuffer: VkCommandBuffer, firstExclusiveScissor: u32, exclusiveScissorCount: u32, pExclusiveScissors: *VkRect2D) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetExclusiveScissorNV :: #type (commandBuffer: VkCommandBuffer, firstExclusiveScissor: u32, exclusiveScissorCount: u32, pExclusiveScissors: *VkRect2D) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetLineWidth :: #type (commandBuffer: VkCommandBuffer, lineWidth: float) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetLineWidth :: #type (commandBuffer: VkCommandBuffer, lineWidth: float) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetSampleLocationsEXT :: #type (commandBuffer: VkCommandBuffer, pSampleLocationsInfo: *VkSampleLocationsInfoEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetSampleLocationsEXT :: #type (commandBuffer: VkCommandBuffer, pSampleLocationsInfo: *VkSampleLocationsInfoEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetScissor :: #type (commandBuffer: VkCommandBuffer, firstScissor: u32, scissorCount: u32, pScissors: *VkRect2D) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetScissor :: #type (commandBuffer: VkCommandBuffer, firstScissor: u32, scissorCount: u32, pScissors: *VkRect2D) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetStencilCompareMask :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, compareMask: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetStencilCompareMask :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, compareMask: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetStencilReference :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, reference: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetStencilReference :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, reference: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetStencilWriteMask :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, writeMask: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetStencilWriteMask :: #type (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, writeMask: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetViewport :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewports: *VkViewport) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetViewport :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewports: *VkViewport) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetViewportShadingRatePaletteNV :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pShadingRatePalettes: *VkShadingRatePaletteNV) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetViewportShadingRatePaletteNV :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pShadingRatePalettes: *VkShadingRatePaletteNV) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdSetViewportWScalingNV :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewportWScalings: *VkViewportWScalingNV) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdSetViewportWScalingNV :: #type (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewportWScalings: *VkViewportWScalingNV) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdTraceRaysNVX :: #type (cmdBuf: VkCommandBuffer, raygenShaderBindingTableBuffer: VkBuffer, raygenShaderBindingOffset: VkDeviceSize, missShaderBindingTableBuffer: VkBuffer, missShaderBindingOffset: VkDeviceSize, missShaderBindingStride: VkDeviceSize, hitShaderBindingTableBuffer: VkBuffer, hitShaderBindingOffset: VkDeviceSize, hitShaderBindingStride: VkDeviceSize, width: u32, height: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdTraceRaysNVX :: #type (cmdBuf: VkCommandBuffer, raygenShaderBindingTableBuffer: VkBuffer, raygenShaderBindingOffset: VkDeviceSize, missShaderBindingTableBuffer: VkBuffer, missShaderBindingOffset: VkDeviceSize, missShaderBindingStride: VkDeviceSize, hitShaderBindingTableBuffer: VkBuffer, hitShaderBindingOffset: VkDeviceSize, hitShaderBindingStride: VkDeviceSize, width: u32, height: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdUpdateBuffer :: #type (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, dataSize: VkDeviceSize, pData: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdUpdateBuffer :: #type (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, dataSize: VkDeviceSize, pData: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdWaitEvents :: #type (commandBuffer: VkCommandBuffer, eventCount: u32, pEvents: *VkEvent, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdWaitEvents :: #type (commandBuffer: VkCommandBuffer, eventCount: u32, pEvents: *VkEvent, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdWriteAccelerationStructurePropertiesNVX :: #type (cmdBuf: VkCommandBuffer, accelerationStructure: VkAccelerationStructureNVX, queryType: VkQueryType, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdWriteAccelerationStructurePropertiesNVX :: #type (cmdBuf: VkCommandBuffer, accelerationStructure: VkAccelerationStructureNVX, queryType: VkQueryType, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdWriteBufferMarkerAMD :: #type (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, marker: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdWriteBufferMarkerAMD :: #type (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, marker: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCmdWriteTimestamp :: #type (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkCmdWriteTimestamp :: #type (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, queryPool: VkQueryPool, query: u32) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCompileDeferredNVX :: #type (device: VkDevice, pipeline: VkPipeline, shader: u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCompileDeferredNVX :: #type (device: VkDevice, pipeline: VkPipeline, shader: u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateAccelerationStructureNVX :: #type (device: VkDevice, pCreateInfo: *VkAccelerationStructureCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pAccelerationStructure: *VkAccelerationStructureNVX) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateAccelerationStructureNVX :: #type (device: VkDevice, pCreateInfo: *VkAccelerationStructureCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pAccelerationStructure: *VkAccelerationStructureNVX) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateBuffer :: #type (device: VkDevice, pCreateInfo: *VkBufferCreateInfo, pAllocator: *VkAllocationCallbacks, pBuffer: *VkBuffer) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateBuffer :: #type (device: VkDevice, pCreateInfo: *VkBufferCreateInfo, pAllocator: *VkAllocationCallbacks, pBuffer: *VkBuffer) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateBufferView :: #type (device: VkDevice, pCreateInfo: *VkBufferViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkBufferView) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateBufferView :: #type (device: VkDevice, pCreateInfo: *VkBufferViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkBufferView) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateCommandPool :: #type (device: VkDevice, pCreateInfo: *VkCommandPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pCommandPool: *VkCommandPool) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateCommandPool :: #type (device: VkDevice, pCreateInfo: *VkCommandPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pCommandPool: *VkCommandPool) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateComputePipelines :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkComputePipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateComputePipelines :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkComputePipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDebugReportCallbackEXT :: #type (instance: VkInstance, pCreateInfo: *VkDebugReportCallbackCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pCallback: *VkDebugReportCallbackEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDebugReportCallbackEXT :: #type (instance: VkInstance, pCreateInfo: *VkDebugReportCallbackCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pCallback: *VkDebugReportCallbackEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDebugUtilsMessengerEXT :: #type (instance: VkInstance, pCreateInfo: *VkDebugUtilsMessengerCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pMessenger: *VkDebugUtilsMessengerEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDebugUtilsMessengerEXT :: #type (instance: VkInstance, pCreateInfo: *VkDebugUtilsMessengerCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pMessenger: *VkDebugUtilsMessengerEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDescriptorPool :: #type (device: VkDevice, pCreateInfo: *VkDescriptorPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorPool: *VkDescriptorPool) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDescriptorPool :: #type (device: VkDevice, pCreateInfo: *VkDescriptorPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorPool: *VkDescriptorPool) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDescriptorSetLayout :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pSetLayout: *VkDescriptorSetLayout) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDescriptorSetLayout :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pSetLayout: *VkDescriptorSetLayout) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDescriptorUpdateTemplate :: #type (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDescriptorUpdateTemplate :: #type (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDescriptorUpdateTemplateKHR :: #type (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDescriptorUpdateTemplateKHR :: #type (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDevice :: #type (physicalDevice: VkPhysicalDevice, pCreateInfo: *VkDeviceCreateInfo, pAllocator: *VkAllocationCallbacks, pDevice: *VkDevice) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDevice :: #type (physicalDevice: VkPhysicalDevice, pCreateInfo: *VkDeviceCreateInfo, pAllocator: *VkAllocationCallbacks, pDevice: *VkDevice) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDisplayModeKHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pCreateInfo: *VkDisplayModeCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pMode: *VkDisplayModeKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDisplayModeKHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pCreateInfo: *VkDisplayModeCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pMode: *VkDisplayModeKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateDisplayPlaneSurfaceKHR :: #type (instance: VkInstance, pCreateInfo: *VkDisplaySurfaceCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSurface: *VkSurfaceKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateDisplayPlaneSurfaceKHR :: #type (instance: VkInstance, pCreateInfo: *VkDisplaySurfaceCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSurface: *VkSurfaceKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateEvent :: #type (device: VkDevice, pCreateInfo: *VkEventCreateInfo, pAllocator: *VkAllocationCallbacks, pEvent: *VkEvent) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateEvent :: #type (device: VkDevice, pCreateInfo: *VkEventCreateInfo, pAllocator: *VkAllocationCallbacks, pEvent: *VkEvent) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateFence :: #type (device: VkDevice, pCreateInfo: *VkFenceCreateInfo, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateFence :: #type (device: VkDevice, pCreateInfo: *VkFenceCreateInfo, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateFramebuffer :: #type (device: VkDevice, pCreateInfo: *VkFramebufferCreateInfo, pAllocator: *VkAllocationCallbacks, pFramebuffer: *VkFramebuffer) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateFramebuffer :: #type (device: VkDevice, pCreateInfo: *VkFramebufferCreateInfo, pAllocator: *VkAllocationCallbacks, pFramebuffer: *VkFramebuffer) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateGraphicsPipelines :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkGraphicsPipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateGraphicsPipelines :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkGraphicsPipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateImage :: #type (device: VkDevice, pCreateInfo: *VkImageCreateInfo, pAllocator: *VkAllocationCallbacks, pImage: *VkImage) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateImage :: #type (device: VkDevice, pCreateInfo: *VkImageCreateInfo, pAllocator: *VkAllocationCallbacks, pImage: *VkImage) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateImageView :: #type (device: VkDevice, pCreateInfo: *VkImageViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkImageView) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateImageView :: #type (device: VkDevice, pCreateInfo: *VkImageViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkImageView) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateIndirectCommandsLayoutNVX :: #type (device: VkDevice, pCreateInfo: *VkIndirectCommandsLayoutCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pIndirectCommandsLayout: *VkIndirectCommandsLayoutNVX) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateIndirectCommandsLayoutNVX :: #type (device: VkDevice, pCreateInfo: *VkIndirectCommandsLayoutCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pIndirectCommandsLayout: *VkIndirectCommandsLayoutNVX) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateInstance :: #type (pCreateInfo: *VkInstanceCreateInfo, pAllocator: *VkAllocationCallbacks, pInstance: *VkInstance) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateInstance :: #type (pCreateInfo: *VkInstanceCreateInfo, pAllocator: *VkAllocationCallbacks, pInstance: *VkInstance) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateObjectTableNVX :: #type (device: VkDevice, pCreateInfo: *VkObjectTableCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pObjectTable: *VkObjectTableNVX) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateObjectTableNVX :: #type (device: VkDevice, pCreateInfo: *VkObjectTableCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pObjectTable: *VkObjectTableNVX) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreatePipelineCache :: #type (device: VkDevice, pCreateInfo: *VkPipelineCacheCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineCache: *VkPipelineCache) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreatePipelineCache :: #type (device: VkDevice, pCreateInfo: *VkPipelineCacheCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineCache: *VkPipelineCache) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreatePipelineLayout :: #type (device: VkDevice, pCreateInfo: *VkPipelineLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineLayout: *VkPipelineLayout) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreatePipelineLayout :: #type (device: VkDevice, pCreateInfo: *VkPipelineLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineLayout: *VkPipelineLayout) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateQueryPool :: #type (device: VkDevice, pCreateInfo: *VkQueryPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pQueryPool: *VkQueryPool) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateQueryPool :: #type (device: VkDevice, pCreateInfo: *VkQueryPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pQueryPool: *VkQueryPool) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateRaytracingPipelinesNVX :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkRaytracingPipelineCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateRaytracingPipelinesNVX :: #type (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkRaytracingPipelineCreateInfoNVX, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateRenderPass :: #type (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateRenderPass :: #type (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateRenderPass2KHR :: #type (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo2KHR, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateRenderPass2KHR :: #type (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo2KHR, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSampler :: #type (device: VkDevice, pCreateInfo: *VkSamplerCreateInfo, pAllocator: *VkAllocationCallbacks, pSampler: *VkSampler) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSampler :: #type (device: VkDevice, pCreateInfo: *VkSamplerCreateInfo, pAllocator: *VkAllocationCallbacks, pSampler: *VkSampler) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSamplerYcbcrConversion :: #type (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSamplerYcbcrConversion :: #type (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSamplerYcbcrConversionKHR :: #type (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSamplerYcbcrConversionKHR :: #type (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSemaphore :: #type (device: VkDevice, pCreateInfo: *VkSemaphoreCreateInfo, pAllocator: *VkAllocationCallbacks, pSemaphore: *VkSemaphore) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSemaphore :: #type (device: VkDevice, pCreateInfo: *VkSemaphoreCreateInfo, pAllocator: *VkAllocationCallbacks, pSemaphore: *VkSemaphore) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateShaderModule :: #type (device: VkDevice, pCreateInfo: *VkShaderModuleCreateInfo, pAllocator: *VkAllocationCallbacks, pShaderModule: *VkShaderModule) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateShaderModule :: #type (device: VkDevice, pCreateInfo: *VkShaderModuleCreateInfo, pAllocator: *VkAllocationCallbacks, pShaderModule: *VkShaderModule) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSharedSwapchainsKHR :: #type (device: VkDevice, swapchainCount: u32, pCreateInfos: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchains: *VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSharedSwapchainsKHR :: #type (device: VkDevice, swapchainCount: u32, pCreateInfos: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchains: *VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateSwapchainKHR :: #type (device: VkDevice, pCreateInfo: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchain: *VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateSwapchainKHR :: #type (device: VkDevice, pCreateInfo: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchain: *VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateValidationCacheEXT :: #type (device: VkDevice, pCreateInfo: *VkValidationCacheCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pValidationCache: *VkValidationCacheEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkCreateValidationCacheEXT :: #type (device: VkDevice, pCreateInfo: *VkValidationCacheCreateInfoEXT, pAllocator: *VkAllocationCallbacks, pValidationCache: *VkValidationCacheEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkCreateWin32SurfaceKHR :: #type (a0: VkInstance, a1: *VkWin32SurfaceCreateInfoKHR, a2: *VkAllocationCallbacks, a3: *VkSurfaceKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDebugMarkerSetObjectNameEXT :: #type (device: VkDevice, pNameInfo: *VkDebugMarkerObjectNameInfoEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkDebugMarkerSetObjectNameEXT :: #type (device: VkDevice, pNameInfo: *VkDebugMarkerObjectNameInfoEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDebugMarkerSetObjectTagEXT :: #type (device: VkDevice, pTagInfo: *VkDebugMarkerObjectTagInfoEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkDebugMarkerSetObjectTagEXT :: #type (device: VkDevice, pTagInfo: *VkDebugMarkerObjectTagInfoEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDebugReportCallbackEXT :: #type (flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: u64, messageCode: s32, pLayerPrefix: *u8, pMessage: *u8, pUserData: *void) -> VkBool32 #c_call;
}

#if OS == .LINUX {
    PFN_vkDebugReportCallbackEXT :: #type (flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: u64, messageCode: s32, pLayerPrefix: *u8, pMessage: *u8, pUserData: *void) -> VkBool32 #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDebugReportMessageEXT :: #type (instance: VkInstance, flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: u64, messageCode: s32, pLayerPrefix: *u8, pMessage: *u8) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDebugReportMessageEXT :: #type (instance: VkInstance, flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: u64, messageCode: s32, pLayerPrefix: *u8, pMessage: *u8) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDebugUtilsMessengerCallbackEXT :: #type (messageSeverity: VkDebugUtilsMessageSeverityFlagBitsEXT, messageType: VkDebugUtilsMessageTypeFlagsEXT, pCallbackData: *VkDebugUtilsMessengerCallbackDataEXT, pUserData: *void) -> VkBool32 #c_call;
}

#if OS == .LINUX {
    PFN_vkDebugUtilsMessengerCallbackEXT :: #type (messageSeverity: VkDebugUtilsMessageSeverityFlagBitsEXT, messageType: VkDebugUtilsMessageTypeFlagsEXT, pCallbackData: *VkDebugUtilsMessengerCallbackDataEXT, pUserData: *void) -> VkBool32 #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyAccelerationStructureNVX :: #type (device: VkDevice, accelerationStructure: VkAccelerationStructureNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyAccelerationStructureNVX :: #type (device: VkDevice, accelerationStructure: VkAccelerationStructureNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyBuffer :: #type (device: VkDevice, buffer: VkBuffer, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyBuffer :: #type (device: VkDevice, buffer: VkBuffer, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyBufferView :: #type (device: VkDevice, bufferView: VkBufferView, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyBufferView :: #type (device: VkDevice, bufferView: VkBufferView, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDebugReportCallbackEXT :: #type (instance: VkInstance, callback: VkDebugReportCallbackEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDebugReportCallbackEXT :: #type (instance: VkInstance, callback: VkDebugReportCallbackEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDebugUtilsMessengerEXT :: #type (instance: VkInstance, messenger: VkDebugUtilsMessengerEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDebugUtilsMessengerEXT :: #type (instance: VkInstance, messenger: VkDebugUtilsMessengerEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDescriptorPool :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDescriptorPool :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDescriptorSetLayout :: #type (device: VkDevice, descriptorSetLayout: VkDescriptorSetLayout, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDescriptorSetLayout :: #type (device: VkDevice, descriptorSetLayout: VkDescriptorSetLayout, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDescriptorUpdateTemplate :: #type (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDescriptorUpdateTemplate :: #type (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDescriptorUpdateTemplateKHR :: #type (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDescriptorUpdateTemplateKHR :: #type (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyDevice :: #type (device: VkDevice, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyDevice :: #type (device: VkDevice, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyEvent :: #type (device: VkDevice, event: VkEvent, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyEvent :: #type (device: VkDevice, event: VkEvent, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyFence :: #type (device: VkDevice, fence: VkFence, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyFence :: #type (device: VkDevice, fence: VkFence, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyFramebuffer :: #type (device: VkDevice, framebuffer: VkFramebuffer, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyFramebuffer :: #type (device: VkDevice, framebuffer: VkFramebuffer, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyImage :: #type (device: VkDevice, image: VkImage, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyImage :: #type (device: VkDevice, image: VkImage, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyImageView :: #type (device: VkDevice, imageView: VkImageView, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyImageView :: #type (device: VkDevice, imageView: VkImageView, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyIndirectCommandsLayoutNVX :: #type (device: VkDevice, indirectCommandsLayout: VkIndirectCommandsLayoutNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyIndirectCommandsLayoutNVX :: #type (device: VkDevice, indirectCommandsLayout: VkIndirectCommandsLayoutNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyInstance :: #type (instance: VkInstance, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyInstance :: #type (instance: VkInstance, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyObjectTableNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyObjectTableNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyPipeline :: #type (device: VkDevice, pipeline: VkPipeline, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyPipeline :: #type (device: VkDevice, pipeline: VkPipeline, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyPipelineCache :: #type (device: VkDevice, pipelineCache: VkPipelineCache, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyPipelineCache :: #type (device: VkDevice, pipelineCache: VkPipelineCache, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyPipelineLayout :: #type (device: VkDevice, pipelineLayout: VkPipelineLayout, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyPipelineLayout :: #type (device: VkDevice, pipelineLayout: VkPipelineLayout, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyQueryPool :: #type (device: VkDevice, queryPool: VkQueryPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyQueryPool :: #type (device: VkDevice, queryPool: VkQueryPool, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyRenderPass :: #type (device: VkDevice, renderPass: VkRenderPass, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyRenderPass :: #type (device: VkDevice, renderPass: VkRenderPass, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySampler :: #type (device: VkDevice, sampler: VkSampler, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySampler :: #type (device: VkDevice, sampler: VkSampler, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySamplerYcbcrConversion :: #type (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySamplerYcbcrConversion :: #type (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySamplerYcbcrConversionKHR :: #type (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySamplerYcbcrConversionKHR :: #type (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySemaphore :: #type (device: VkDevice, semaphore: VkSemaphore, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySemaphore :: #type (device: VkDevice, semaphore: VkSemaphore, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyShaderModule :: #type (device: VkDevice, shaderModule: VkShaderModule, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyShaderModule :: #type (device: VkDevice, shaderModule: VkShaderModule, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySurfaceKHR :: #type (instance: VkInstance, surface: VkSurfaceKHR, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySurfaceKHR :: #type (instance: VkInstance, surface: VkSurfaceKHR, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroySwapchainKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroySwapchainKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDestroyValidationCacheEXT :: #type (device: VkDevice, validationCache: VkValidationCacheEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkDestroyValidationCacheEXT :: #type (device: VkDevice, validationCache: VkValidationCacheEXT, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDeviceWaitIdle :: #type (device: VkDevice) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkDeviceWaitIdle :: #type (device: VkDevice) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkDisplayPowerControlEXT :: #type (device: VkDevice, display: VkDisplayKHR, pDisplayPowerInfo: *VkDisplayPowerInfoEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkDisplayPowerControlEXT :: #type (device: VkDevice, display: VkDisplayKHR, pDisplayPowerInfo: *VkDisplayPowerInfoEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEndCommandBuffer :: #type (commandBuffer: VkCommandBuffer) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEndCommandBuffer :: #type (commandBuffer: VkCommandBuffer) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumerateDeviceExtensionProperties :: #type (physicalDevice: VkPhysicalDevice, pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumerateDeviceExtensionProperties :: #type (physicalDevice: VkPhysicalDevice, pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumerateDeviceLayerProperties :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumerateDeviceLayerProperties :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumerateInstanceExtensionProperties :: #type (pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumerateInstanceExtensionProperties :: #type (pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumerateInstanceLayerProperties :: #type (pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumerateInstanceLayerProperties :: #type (pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumerateInstanceVersion :: #type (pApiVersion: *u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumerateInstanceVersion :: #type (pApiVersion: *u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumeratePhysicalDeviceGroups :: #type (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumeratePhysicalDeviceGroups :: #type (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumeratePhysicalDeviceGroupsKHR :: #type (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumeratePhysicalDeviceGroupsKHR :: #type (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkEnumeratePhysicalDevices :: #type (instance: VkInstance, pPhysicalDeviceCount: *u32, pPhysicalDevices: *VkPhysicalDevice) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkEnumeratePhysicalDevices :: #type (instance: VkInstance, pPhysicalDeviceCount: *u32, pPhysicalDevices: *VkPhysicalDevice) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkFlushMappedMemoryRanges :: #type (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkFlushMappedMemoryRanges :: #type (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkFreeCommandBuffers :: #type (device: VkDevice, commandPool: VkCommandPool, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkFreeCommandBuffers :: #type (device: VkDevice, commandPool: VkCommandPool, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkFreeDescriptorSets :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkFreeDescriptorSets :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkFreeFunction :: #type (pUserData: *void, pMemory: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkFreeFunction :: #type (pUserData: *void, pMemory: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkFreeMemory :: #type (device: VkDevice, memory: VkDeviceMemory, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkFreeMemory :: #type (device: VkDevice, memory: VkDeviceMemory, pAllocator: *VkAllocationCallbacks) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetAccelerationStructureHandleNVX :: #type (device: VkDevice, accelerationStructure: VkAccelerationStructureNVX, dataSize: u64, pData: *void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetAccelerationStructureHandleNVX :: #type (device: VkDevice, accelerationStructure: VkAccelerationStructureNVX, dataSize: u64, pData: *void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetAccelerationStructureMemoryRequirementsNVX :: #type (device: VkDevice, pInfo: *VkAccelerationStructureMemoryRequirementsInfoNVX, pMemoryRequirements: *VkMemoryRequirements2KHR) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetAccelerationStructureMemoryRequirementsNVX :: #type (device: VkDevice, pInfo: *VkAccelerationStructureMemoryRequirementsInfoNVX, pMemoryRequirements: *VkMemoryRequirements2KHR) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetAccelerationStructureScratchMemoryRequirementsNVX :: #type (device: VkDevice, pInfo: *VkAccelerationStructureMemoryRequirementsInfoNVX, pMemoryRequirements: *VkMemoryRequirements2KHR) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetAccelerationStructureScratchMemoryRequirementsNVX :: #type (device: VkDevice, pInfo: *VkAccelerationStructureMemoryRequirementsInfoNVX, pMemoryRequirements: *VkMemoryRequirements2KHR) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetBufferMemoryRequirements :: #type (device: VkDevice, buffer: VkBuffer, pMemoryRequirements: *VkMemoryRequirements) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetBufferMemoryRequirements :: #type (device: VkDevice, buffer: VkBuffer, pMemoryRequirements: *VkMemoryRequirements) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetBufferMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetBufferMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetBufferMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetBufferMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDescriptorSetLayoutSupport :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDescriptorSetLayoutSupport :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDescriptorSetLayoutSupportKHR :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDescriptorSetLayoutSupportKHR :: #type (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceGroupPeerMemoryFeatures :: #type (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceGroupPeerMemoryFeatures :: #type (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceGroupPeerMemoryFeaturesKHR :: #type (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceGroupPeerMemoryFeaturesKHR :: #type (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceGroupPresentCapabilitiesKHR :: #type (device: VkDevice, pDeviceGroupPresentCapabilities: *VkDeviceGroupPresentCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceGroupPresentCapabilitiesKHR :: #type (device: VkDevice, pDeviceGroupPresentCapabilities: *VkDeviceGroupPresentCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceGroupSurfacePresentModesKHR :: #type (device: VkDevice, surface: VkSurfaceKHR, pModes: *VkDeviceGroupPresentModeFlagsKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceGroupSurfacePresentModesKHR :: #type (device: VkDevice, surface: VkSurfaceKHR, pModes: *VkDeviceGroupPresentModeFlagsKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceMemoryCommitment :: #type (device: VkDevice, memory: VkDeviceMemory, pCommittedMemoryInBytes: *VkDeviceSize) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceMemoryCommitment :: #type (device: VkDevice, memory: VkDeviceMemory, pCommittedMemoryInBytes: *VkDeviceSize) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceProcAddr :: #type (device: VkDevice, pName: *u8) -> PFN_vkVoidFunction #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceProcAddr :: #type (device: VkDevice, pName: *u8) -> PFN_vkVoidFunction #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceQueue :: #type (device: VkDevice, queueFamilyIndex: u32, queueIndex: u32, pQueue: *VkQueue) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceQueue :: #type (device: VkDevice, queueFamilyIndex: u32, queueIndex: u32, pQueue: *VkQueue) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDeviceQueue2 :: #type (device: VkDevice, pQueueInfo: *VkDeviceQueueInfo2, pQueue: *VkQueue) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDeviceQueue2 :: #type (device: VkDevice, pQueueInfo: *VkDeviceQueueInfo2, pQueue: *VkQueue) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDisplayModeProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModeProperties2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDisplayModeProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModeProperties2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDisplayModePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModePropertiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDisplayModePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModePropertiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDisplayPlaneCapabilities2KHR :: #type (physicalDevice: VkPhysicalDevice, pDisplayPlaneInfo: *VkDisplayPlaneInfo2KHR, pCapabilities: *VkDisplayPlaneCapabilities2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDisplayPlaneCapabilities2KHR :: #type (physicalDevice: VkPhysicalDevice, pDisplayPlaneInfo: *VkDisplayPlaneInfo2KHR, pCapabilities: *VkDisplayPlaneCapabilities2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDisplayPlaneCapabilitiesKHR :: #type (physicalDevice: VkPhysicalDevice, mode: VkDisplayModeKHR, planeIndex: u32, pCapabilities: *VkDisplayPlaneCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDisplayPlaneCapabilitiesKHR :: #type (physicalDevice: VkPhysicalDevice, mode: VkDisplayModeKHR, planeIndex: u32, pCapabilities: *VkDisplayPlaneCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetDisplayPlaneSupportedDisplaysKHR :: #type (physicalDevice: VkPhysicalDevice, planeIndex: u32, pDisplayCount: *u32, pDisplays: *VkDisplayKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetDisplayPlaneSupportedDisplaysKHR :: #type (physicalDevice: VkPhysicalDevice, planeIndex: u32, pDisplayCount: *u32, pDisplays: *VkDisplayKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetEventStatus :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetEventStatus :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetFenceFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkFenceGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetFenceFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkFenceGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetFenceStatus :: #type (device: VkDevice, fence: VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetFenceStatus :: #type (device: VkDevice, fence: VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetFenceWin32HandleKHR :: #type (a0: VkDevice, a1: *VkFenceGetWin32HandleInfoKHR, a2: *HANDLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageMemoryRequirements :: #type (device: VkDevice, image: VkImage, pMemoryRequirements: *VkMemoryRequirements) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageMemoryRequirements :: #type (device: VkDevice, image: VkImage, pMemoryRequirements: *VkMemoryRequirements) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageSparseMemoryRequirements :: #type (device: VkDevice, image: VkImage, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageSparseMemoryRequirements :: #type (device: VkDevice, image: VkImage, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageSparseMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageSparseMemoryRequirements2 :: #type (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageSparseMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageSparseMemoryRequirements2KHR :: #type (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetImageSubresourceLayout :: #type (device: VkDevice, image: VkImage, pSubresource: *VkImageSubresource, pLayout: *VkSubresourceLayout) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetImageSubresourceLayout :: #type (device: VkDevice, image: VkImage, pSubresource: *VkImageSubresource, pLayout: *VkSubresourceLayout) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetInstanceProcAddr :: #type (instance: VkInstance, pName: *u8) -> PFN_vkVoidFunction #c_call;
}

#if OS == .LINUX {
    PFN_vkGetInstanceProcAddr :: #type (instance: VkInstance, pName: *u8) -> PFN_vkVoidFunction #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkMemoryGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetMemoryFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkMemoryGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryFdPropertiesKHR :: #type (device: VkDevice, handleType: VkExternalMemoryHandleTypeFlagBits, fd: s32, pMemoryFdProperties: *VkMemoryFdPropertiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetMemoryFdPropertiesKHR :: #type (device: VkDevice, handleType: VkExternalMemoryHandleTypeFlagBits, fd: s32, pMemoryFdProperties: *VkMemoryFdPropertiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryHostPointerPropertiesEXT :: #type (device: VkDevice, handleType: VkExternalMemoryHandleTypeFlagBits, pHostPointer: *void, pMemoryHostPointerProperties: *VkMemoryHostPointerPropertiesEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetMemoryHostPointerPropertiesEXT :: #type (device: VkDevice, handleType: VkExternalMemoryHandleTypeFlagBits, pHostPointer: *void, pMemoryHostPointerProperties: *VkMemoryHostPointerPropertiesEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryWin32HandleKHR :: #type (a0: VkDevice, a1: *VkMemoryGetWin32HandleInfoKHR, a2: *HANDLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryWin32HandleNV :: #type (a0: VkDevice, a1: VkDeviceMemory, a2: VkExternalMemoryHandleTypeFlagsNV, a3: *HANDLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetMemoryWin32HandlePropertiesKHR :: #type (a0: VkDevice, a1: VkExternalMemoryHandleTypeFlagBits, a2: HANDLE, a3: *VkMemoryWin32HandlePropertiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPastPresentationTimingGOOGLE :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pPresentationTimingCount: *u32, pPresentationTimings: *VkPastPresentationTimingGOOGLE) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPastPresentationTimingGOOGLE :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pPresentationTimingCount: *u32, pPresentationTimings: *VkPastPresentationTimingGOOGLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlaneProperties2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlaneProperties2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlanePropertiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlanePropertiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceDisplayProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayProperties2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceDisplayProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayProperties2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceDisplayPropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPropertiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceDisplayPropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPropertiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalBufferProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalBufferProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalBufferPropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalFenceProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalFenceProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalFencePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalFencePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, externalHandleType: VkExternalMemoryHandleTypeFlagsNV, pExternalImageFormatProperties: *VkExternalImageFormatPropertiesNV) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalImageFormatPropertiesNV :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, externalHandleType: VkExternalMemoryHandleTypeFlagsNV, pExternalImageFormatProperties: *VkExternalImageFormatPropertiesNV) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalSemaphoreProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalSemaphoreProperties :: #type (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceExternalSemaphorePropertiesKHR :: #type (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFeatures :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFeatures :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFeatures2 :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFeatures2 :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFeatures2KHR :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFeatures2KHR :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceGeneratedCommandsPropertiesNVX :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkDeviceGeneratedCommandsFeaturesNVX, pLimits: *VkDeviceGeneratedCommandsLimitsNVX) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceGeneratedCommandsPropertiesNVX :: #type (physicalDevice: VkPhysicalDevice, pFeatures: *VkDeviceGeneratedCommandsFeaturesNVX, pLimits: *VkDeviceGeneratedCommandsLimitsNVX) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceImageFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, pImageFormatProperties: *VkImageFormatProperties) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceImageFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, pImageFormatProperties: *VkImageFormatProperties) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceImageFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceImageFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceImageFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceImageFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceMemoryProperties :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceMemoryProperties :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceMemoryProperties2 :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceMemoryProperties2 :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceMemoryProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceMemoryProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT :: #type (physicalDevice: VkPhysicalDevice, samples: VkSampleCountFlagBits, pMultisampleProperties: *VkMultisamplePropertiesEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceMultisamplePropertiesEXT :: #type (physicalDevice: VkPhysicalDevice, samples: VkSampleCountFlagBits, pMultisampleProperties: *VkMultisamplePropertiesEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDevicePresentRectanglesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pRectCount: *u32, pRects: *VkRect2D) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDevicePresentRectanglesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pRectCount: *u32, pRects: *VkRect2D) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceProperties :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceProperties :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceProperties2 :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceProperties2 :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties2 :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties2 :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties :: #type (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties2 :: #type (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR :: #type (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilities2EXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceCapabilities2EXT :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilities2EXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR :: #type (physicalDevice: VkPhysicalDevice, pSurfaceInfo: *VkPhysicalDeviceSurfaceInfo2KHR, pSurfaceCapabilities: *VkSurfaceCapabilities2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceCapabilities2KHR :: #type (physicalDevice: VkPhysicalDevice, pSurfaceInfo: *VkPhysicalDeviceSurfaceInfo2KHR, pSurfaceCapabilities: *VkSurfaceCapabilities2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilitiesKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceFormats2KHR :: #type (physicalDevice: VkPhysicalDevice, pSurfaceInfo: *VkPhysicalDeviceSurfaceInfo2KHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormat2KHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceFormats2KHR :: #type (physicalDevice: VkPhysicalDevice, pSurfaceInfo: *VkPhysicalDeviceSurfaceInfo2KHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormat2KHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceFormatsKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormatKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceFormatsKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormatKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfacePresentModesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *u32, pPresentModes: *VkPresentModeKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfacePresentModesKHR :: #type (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *u32, pPresentModes: *VkPresentModeKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceSurfaceSupportKHR :: #type (physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *VkBool32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPhysicalDeviceSurfaceSupportKHR :: #type (physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *VkBool32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPhysicalDeviceWin32PresentationSupportKHR :: #type (a0: VkPhysicalDevice, a1: u32) -> VkBool32 #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetPipelineCacheData :: #type (device: VkDevice, pipelineCache: VkPipelineCache, pDataSize: *u64, pData: *void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetPipelineCacheData :: #type (device: VkDevice, pipelineCache: VkPipelineCache, pDataSize: *u64, pData: *void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetQueryPoolResults :: #type (device: VkDevice, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dataSize: u64, pData: *void, stride: VkDeviceSize, flags: VkQueryResultFlags) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetQueryPoolResults :: #type (device: VkDevice, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dataSize: u64, pData: *void, stride: VkDeviceSize, flags: VkQueryResultFlags) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetQueueCheckpointDataNV :: #type (queue: VkQueue, pCheckpointDataCount: *u32, pCheckpointData: *VkCheckpointDataNV) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetQueueCheckpointDataNV :: #type (queue: VkQueue, pCheckpointDataCount: *u32, pCheckpointData: *VkCheckpointDataNV) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetRaytracingShaderHandlesNVX :: #type (device: VkDevice, pipeline: VkPipeline, firstGroup: u32, groupCount: u32, dataSize: u64, pData: *void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetRaytracingShaderHandlesNVX :: #type (device: VkDevice, pipeline: VkPipeline, firstGroup: u32, groupCount: u32, dataSize: u64, pData: *void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetRefreshCycleDurationGOOGLE :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pDisplayTimingProperties: *VkRefreshCycleDurationGOOGLE) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetRefreshCycleDurationGOOGLE :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pDisplayTimingProperties: *VkRefreshCycleDurationGOOGLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetRenderAreaGranularity :: #type (device: VkDevice, renderPass: VkRenderPass, pGranularity: *VkExtent2D) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkGetRenderAreaGranularity :: #type (device: VkDevice, renderPass: VkRenderPass, pGranularity: *VkExtent2D) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetSemaphoreFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkSemaphoreGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetSemaphoreFdKHR :: #type (device: VkDevice, pGetFdInfo: *VkSemaphoreGetFdInfoKHR, pFd: *s32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetSemaphoreWin32HandleKHR :: #type (a0: VkDevice, a1: *VkSemaphoreGetWin32HandleInfoKHR, a2: *HANDLE) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetShaderInfoAMD :: #type (device: VkDevice, pipeline: VkPipeline, shaderStage: VkShaderStageFlagBits, infoType: VkShaderInfoTypeAMD, pInfoSize: *u64, pInfo: *void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetShaderInfoAMD :: #type (device: VkDevice, pipeline: VkPipeline, shaderStage: VkShaderStageFlagBits, infoType: VkShaderInfoTypeAMD, pInfoSize: *u64, pInfo: *void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetSwapchainCounterEXT :: #type (device: VkDevice, swapchain: VkSwapchainKHR, counter: VkSurfaceCounterFlagBitsEXT, pCounterValue: *u64) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetSwapchainCounterEXT :: #type (device: VkDevice, swapchain: VkSwapchainKHR, counter: VkSurfaceCounterFlagBitsEXT, pCounterValue: *u64) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetSwapchainImagesKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pSwapchainImageCount: *u32, pSwapchainImages: *VkImage) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetSwapchainImagesKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR, pSwapchainImageCount: *u32, pSwapchainImages: *VkImage) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetSwapchainStatusKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetSwapchainStatusKHR :: #type (device: VkDevice, swapchain: VkSwapchainKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkGetValidationCacheDataEXT :: #type (device: VkDevice, validationCache: VkValidationCacheEXT, pDataSize: *u64, pData: *void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkGetValidationCacheDataEXT :: #type (device: VkDevice, validationCache: VkValidationCacheEXT, pDataSize: *u64, pData: *void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkImportFenceFdKHR :: #type (device: VkDevice, pImportFenceFdInfo: *VkImportFenceFdInfoKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkImportFenceFdKHR :: #type (device: VkDevice, pImportFenceFdInfo: *VkImportFenceFdInfoKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkImportFenceWin32HandleKHR :: #type (a0: VkDevice, a1: *VkImportFenceWin32HandleInfoKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkImportSemaphoreFdKHR :: #type (device: VkDevice, pImportSemaphoreFdInfo: *VkImportSemaphoreFdInfoKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkImportSemaphoreFdKHR :: #type (device: VkDevice, pImportSemaphoreFdInfo: *VkImportSemaphoreFdInfoKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkImportSemaphoreWin32HandleKHR :: #type (a0: VkDevice, a1: *VkImportSemaphoreWin32HandleInfoKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkInternalAllocationNotification :: #type (pUserData: *void, size: u64, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkInternalAllocationNotification :: #type (pUserData: *void, size: u64, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkInternalFreeNotification :: #type (pUserData: *void, size: u64, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkInternalFreeNotification :: #type (pUserData: *void, size: u64, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkInvalidateMappedMemoryRanges :: #type (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkInvalidateMappedMemoryRanges :: #type (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkMapMemory :: #type (device: VkDevice, memory: VkDeviceMemory, offset: VkDeviceSize, size: VkDeviceSize, flags: VkMemoryMapFlags, ppData: **void) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkMapMemory :: #type (device: VkDevice, memory: VkDeviceMemory, offset: VkDeviceSize, size: VkDeviceSize, flags: VkMemoryMapFlags, ppData: **void) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkMergePipelineCaches :: #type (device: VkDevice, dstCache: VkPipelineCache, srcCacheCount: u32, pSrcCaches: *VkPipelineCache) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkMergePipelineCaches :: #type (device: VkDevice, dstCache: VkPipelineCache, srcCacheCount: u32, pSrcCaches: *VkPipelineCache) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkMergeValidationCachesEXT :: #type (device: VkDevice, dstCache: VkValidationCacheEXT, srcCacheCount: u32, pSrcCaches: *VkValidationCacheEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkMergeValidationCachesEXT :: #type (device: VkDevice, dstCache: VkValidationCacheEXT, srcCacheCount: u32, pSrcCaches: *VkValidationCacheEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueBeginDebugUtilsLabelEXT :: #type (queue: VkQueue, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueBeginDebugUtilsLabelEXT :: #type (queue: VkQueue, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueBindSparse :: #type (queue: VkQueue, bindInfoCount: u32, pBindInfo: *VkBindSparseInfo, fence: VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueBindSparse :: #type (queue: VkQueue, bindInfoCount: u32, pBindInfo: *VkBindSparseInfo, fence: VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueEndDebugUtilsLabelEXT :: #type (queue: VkQueue) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueEndDebugUtilsLabelEXT :: #type (queue: VkQueue) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueInsertDebugUtilsLabelEXT :: #type (queue: VkQueue, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueInsertDebugUtilsLabelEXT :: #type (queue: VkQueue, pLabelInfo: *VkDebugUtilsLabelEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueuePresentKHR :: #type (queue: VkQueue, pPresentInfo: *VkPresentInfoKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkQueuePresentKHR :: #type (queue: VkQueue, pPresentInfo: *VkPresentInfoKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueSubmit :: #type (queue: VkQueue, submitCount: u32, pSubmits: *VkSubmitInfo, fence: VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueSubmit :: #type (queue: VkQueue, submitCount: u32, pSubmits: *VkSubmitInfo, fence: VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkQueueWaitIdle :: #type (queue: VkQueue) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkQueueWaitIdle :: #type (queue: VkQueue) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkReallocationFunction :: #type (pUserData: *void, pOriginal: *void, size: u64, alignment: u64, allocationScope: VkSystemAllocationScope) -> *void #c_call;
}

#if OS == .LINUX {
    PFN_vkReallocationFunction :: #type (pUserData: *void, pOriginal: *void, size: u64, alignment: u64, allocationScope: VkSystemAllocationScope) -> *void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkRegisterDeviceEventEXT :: #type (device: VkDevice, pDeviceEventInfo: *VkDeviceEventInfoEXT, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkRegisterDeviceEventEXT :: #type (device: VkDevice, pDeviceEventInfo: *VkDeviceEventInfoEXT, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkRegisterDisplayEventEXT :: #type (device: VkDevice, display: VkDisplayKHR, pDisplayEventInfo: *VkDisplayEventInfoEXT, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkRegisterDisplayEventEXT :: #type (device: VkDevice, display: VkDisplayKHR, pDisplayEventInfo: *VkDisplayEventInfoEXT, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkRegisterObjectsNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, objectCount: u32, ppObjectTableEntries: **VkObjectTableEntryNVX, pObjectIndices: *u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkRegisterObjectsNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, objectCount: u32, ppObjectTableEntries: **VkObjectTableEntryNVX, pObjectIndices: *u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkReleaseDisplayEXT :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkReleaseDisplayEXT :: #type (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkResetCommandBuffer :: #type (commandBuffer: VkCommandBuffer, flags: VkCommandBufferResetFlags) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkResetCommandBuffer :: #type (commandBuffer: VkCommandBuffer, flags: VkCommandBufferResetFlags) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkResetCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolResetFlags) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkResetCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolResetFlags) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkResetDescriptorPool :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, flags: VkDescriptorPoolResetFlags) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkResetDescriptorPool :: #type (device: VkDevice, descriptorPool: VkDescriptorPool, flags: VkDescriptorPoolResetFlags) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkResetEvent :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkResetEvent :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkResetFences :: #type (device: VkDevice, fenceCount: u32, pFences: *VkFence) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkResetFences :: #type (device: VkDevice, fenceCount: u32, pFences: *VkFence) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkSetDebugUtilsObjectNameEXT :: #type (device: VkDevice, pNameInfo: *VkDebugUtilsObjectNameInfoEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkSetDebugUtilsObjectNameEXT :: #type (device: VkDevice, pNameInfo: *VkDebugUtilsObjectNameInfoEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkSetDebugUtilsObjectTagEXT :: #type (device: VkDevice, pTagInfo: *VkDebugUtilsObjectTagInfoEXT) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkSetDebugUtilsObjectTagEXT :: #type (device: VkDevice, pTagInfo: *VkDebugUtilsObjectTagInfoEXT) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkSetEvent :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkSetEvent :: #type (device: VkDevice, event: VkEvent) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkSetHdrMetadataEXT :: #type (device: VkDevice, swapchainCount: u32, pSwapchains: *VkSwapchainKHR, pMetadata: *VkHdrMetadataEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkSetHdrMetadataEXT :: #type (device: VkDevice, swapchainCount: u32, pSwapchains: *VkSwapchainKHR, pMetadata: *VkHdrMetadataEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkSubmitDebugUtilsMessageEXT :: #type (instance: VkInstance, messageSeverity: VkDebugUtilsMessageSeverityFlagBitsEXT, messageTypes: VkDebugUtilsMessageTypeFlagsEXT, pCallbackData: *VkDebugUtilsMessengerCallbackDataEXT) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkSubmitDebugUtilsMessageEXT :: #type (instance: VkInstance, messageSeverity: VkDebugUtilsMessageSeverityFlagBitsEXT, messageTypes: VkDebugUtilsMessageTypeFlagsEXT, pCallbackData: *VkDebugUtilsMessengerCallbackDataEXT) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkTrimCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkTrimCommandPool :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkTrimCommandPoolKHR :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkTrimCommandPoolKHR :: #type (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkUnmapMemory :: #type (device: VkDevice, memory: VkDeviceMemory) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkUnmapMemory :: #type (device: VkDevice, memory: VkDeviceMemory) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkUnregisterObjectsNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, objectCount: u32, pObjectEntryTypes: *VkObjectEntryTypeNVX, pObjectIndices: *u32) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkUnregisterObjectsNVX :: #type (device: VkDevice, objectTable: VkObjectTableNVX, objectCount: u32, pObjectEntryTypes: *VkObjectEntryTypeNVX, pObjectIndices: *u32) -> VkResult #c_call;
}

#if OS == .WINDOWS {
    PFN_vkUpdateDescriptorSets :: #type (device: VkDevice, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet, descriptorCopyCount: u32, pDescriptorCopies: *VkCopyDescriptorSet) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkUpdateDescriptorSets :: #type (device: VkDevice, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet, descriptorCopyCount: u32, pDescriptorCopies: *VkCopyDescriptorSet) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkUpdateDescriptorSetWithTemplate :: #type (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkUpdateDescriptorSetWithTemplate :: #type (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkUpdateDescriptorSetWithTemplateKHR :: #type (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkUpdateDescriptorSetWithTemplateKHR :: #type (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkVoidFunction :: #type () -> void #c_call;
}

#if OS == .LINUX {
    PFN_vkVoidFunction :: #type () -> void #c_call;
}

#if OS == .WINDOWS {
    PFN_vkWaitForFences :: #type (device: VkDevice, fenceCount: u32, pFences: *VkFence, waitAll: VkBool32, timeout: u64) -> VkResult #c_call;
}

#if OS == .LINUX {
    PFN_vkWaitForFences :: #type (device: VkDevice, fenceCount: u32, pFences: *VkFence, waitAll: VkBool32, timeout: u64) -> VkResult #c_call;
}

// Aliases and constants

#if OS == .WINDOWS {
    VK_AMD_buffer_marker :: 1;
}

#if OS == .LINUX {
    VK_AMD_buffer_marker :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_BUFFER_MARKER_EXTENSION_NAME :: "VK_AMD_buffer_marker";
}

#if OS == .LINUX {
    VK_AMD_BUFFER_MARKER_EXTENSION_NAME :: "VK_AMD_buffer_marker";
}

#if OS == .WINDOWS {
    VK_AMD_BUFFER_MARKER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_BUFFER_MARKER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_draw_indirect_count :: 1;
}

#if OS == .LINUX {
    VK_AMD_draw_indirect_count :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_DRAW_INDIRECT_COUNT_EXTENSION_NAME :: "VK_AMD_draw_indirect_count";
}

#if OS == .LINUX {
    VK_AMD_DRAW_INDIRECT_COUNT_EXTENSION_NAME :: "VK_AMD_draw_indirect_count";
}

#if OS == .WINDOWS {
    VK_AMD_DRAW_INDIRECT_COUNT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_DRAW_INDIRECT_COUNT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_gcn_shader :: 1;
}

#if OS == .LINUX {
    VK_AMD_gcn_shader :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_GCN_SHADER_EXTENSION_NAME :: "VK_AMD_gcn_shader";
}

#if OS == .LINUX {
    VK_AMD_GCN_SHADER_EXTENSION_NAME :: "VK_AMD_gcn_shader";
}

#if OS == .WINDOWS {
    VK_AMD_GCN_SHADER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_GCN_SHADER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_gpu_shader_half_float :: 1;
}

#if OS == .LINUX {
    VK_AMD_gpu_shader_half_float :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_GPU_SHADER_HALF_FLOAT_EXTENSION_NAME :: "VK_AMD_gpu_shader_half_float";
}

#if OS == .LINUX {
    VK_AMD_GPU_SHADER_HALF_FLOAT_EXTENSION_NAME :: "VK_AMD_gpu_shader_half_float";
}

#if OS == .WINDOWS {
    VK_AMD_GPU_SHADER_HALF_FLOAT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_GPU_SHADER_HALF_FLOAT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_gpu_shader_int16 :: 1;
}

#if OS == .LINUX {
    VK_AMD_gpu_shader_int16 :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_GPU_SHADER_INT16_EXTENSION_NAME :: "VK_AMD_gpu_shader_int16";
}

#if OS == .LINUX {
    VK_AMD_GPU_SHADER_INT16_EXTENSION_NAME :: "VK_AMD_gpu_shader_int16";
}

#if OS == .WINDOWS {
    VK_AMD_GPU_SHADER_INT16_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_GPU_SHADER_INT16_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_mixed_attachment_samples :: 1;
}

#if OS == .LINUX {
    VK_AMD_mixed_attachment_samples :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_MIXED_ATTACHMENT_SAMPLES_EXTENSION_NAME :: "VK_AMD_mixed_attachment_samples";
}

#if OS == .LINUX {
    VK_AMD_MIXED_ATTACHMENT_SAMPLES_EXTENSION_NAME :: "VK_AMD_mixed_attachment_samples";
}

#if OS == .WINDOWS {
    VK_AMD_MIXED_ATTACHMENT_SAMPLES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_MIXED_ATTACHMENT_SAMPLES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_negative_viewport_height :: 1;
}

#if OS == .LINUX {
    VK_AMD_negative_viewport_height :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_EXTENSION_NAME :: "VK_AMD_negative_viewport_height";
}

#if OS == .LINUX {
    VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_EXTENSION_NAME :: "VK_AMD_negative_viewport_height";
}

#if OS == .WINDOWS {
    VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_NEGATIVE_VIEWPORT_HEIGHT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_rasterization_order :: 1;
}

#if OS == .LINUX {
    VK_AMD_rasterization_order :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_RASTERIZATION_ORDER_EXTENSION_NAME :: "VK_AMD_rasterization_order";
}

#if OS == .LINUX {
    VK_AMD_RASTERIZATION_ORDER_EXTENSION_NAME :: "VK_AMD_rasterization_order";
}

#if OS == .WINDOWS {
    VK_AMD_RASTERIZATION_ORDER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_RASTERIZATION_ORDER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_ballot :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_ballot :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_BALLOT_EXTENSION_NAME :: "VK_AMD_shader_ballot";
}

#if OS == .LINUX {
    VK_AMD_SHADER_BALLOT_EXTENSION_NAME :: "VK_AMD_shader_ballot";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_BALLOT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_BALLOT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_core_properties :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_core_properties :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_CORE_PROPERTIES_EXTENSION_NAME :: "VK_AMD_shader_core_properties";
}

#if OS == .LINUX {
    VK_AMD_SHADER_CORE_PROPERTIES_EXTENSION_NAME :: "VK_AMD_shader_core_properties";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_CORE_PROPERTIES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_CORE_PROPERTIES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_explicit_vertex_parameter :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_explicit_vertex_parameter :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_EXTENSION_NAME :: "VK_AMD_shader_explicit_vertex_parameter";
}

#if OS == .LINUX {
    VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_EXTENSION_NAME :: "VK_AMD_shader_explicit_vertex_parameter";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_EXPLICIT_VERTEX_PARAMETER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_fragment_mask :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_fragment_mask :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_FRAGMENT_MASK_EXTENSION_NAME :: "VK_AMD_shader_fragment_mask";
}

#if OS == .LINUX {
    VK_AMD_SHADER_FRAGMENT_MASK_EXTENSION_NAME :: "VK_AMD_shader_fragment_mask";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_FRAGMENT_MASK_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_FRAGMENT_MASK_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_image_load_store_lod :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_image_load_store_lod :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_IMAGE_LOAD_STORE_LOD_EXTENSION_NAME :: "VK_AMD_shader_image_load_store_lod";
}

#if OS == .LINUX {
    VK_AMD_SHADER_IMAGE_LOAD_STORE_LOD_EXTENSION_NAME :: "VK_AMD_shader_image_load_store_lod";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_IMAGE_LOAD_STORE_LOD_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_IMAGE_LOAD_STORE_LOD_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_info :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_info :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_INFO_EXTENSION_NAME :: "VK_AMD_shader_info";
}

#if OS == .LINUX {
    VK_AMD_SHADER_INFO_EXTENSION_NAME :: "VK_AMD_shader_info";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_INFO_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_INFO_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_shader_trinary_minmax :: 1;
}

#if OS == .LINUX {
    VK_AMD_shader_trinary_minmax :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_TRINARY_MINMAX_EXTENSION_NAME :: "VK_AMD_shader_trinary_minmax";
}

#if OS == .LINUX {
    VK_AMD_SHADER_TRINARY_MINMAX_EXTENSION_NAME :: "VK_AMD_shader_trinary_minmax";
}

#if OS == .WINDOWS {
    VK_AMD_SHADER_TRINARY_MINMAX_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_SHADER_TRINARY_MINMAX_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_texture_gather_bias_lod :: 1;
}

#if OS == .LINUX {
    VK_AMD_texture_gather_bias_lod :: 1;
}

#if OS == .WINDOWS {
    VK_AMD_TEXTURE_GATHER_BIAS_LOD_EXTENSION_NAME :: "VK_AMD_texture_gather_bias_lod";
}

#if OS == .LINUX {
    VK_AMD_TEXTURE_GATHER_BIAS_LOD_EXTENSION_NAME :: "VK_AMD_texture_gather_bias_lod";
}

#if OS == .WINDOWS {
    VK_AMD_TEXTURE_GATHER_BIAS_LOD_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_AMD_TEXTURE_GATHER_BIAS_LOD_SPEC_VERSION :: 1;
}

VK_API_VERSION_1_0 :: #run VK_MAKE_VERSION(1, 0, 0);

VK_API_VERSION_1_1 :: #run VK_MAKE_VERSION(1, 1, 0);

VK_API_VERSION_1_2 :: #run VK_MAKE_VERSION(1, 2, 0);

#if OS == .WINDOWS {
    VK_EXT_astc_decode_mode :: 1;
}

#if OS == .LINUX {
    VK_EXT_astc_decode_mode :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_ASTC_DECODE_MODE_EXTENSION_NAME :: "VK_EXT_astc_decode_mode";
}

#if OS == .LINUX {
    VK_EXT_ASTC_DECODE_MODE_EXTENSION_NAME :: "VK_EXT_astc_decode_mode";
}

#if OS == .WINDOWS {
    VK_EXT_ASTC_DECODE_MODE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_ASTC_DECODE_MODE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_blend_operation_advanced :: 1;
}

#if OS == .LINUX {
    VK_EXT_blend_operation_advanced :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_BLEND_OPERATION_ADVANCED_EXTENSION_NAME :: "VK_EXT_blend_operation_advanced";
}

#if OS == .LINUX {
    VK_EXT_BLEND_OPERATION_ADVANCED_EXTENSION_NAME :: "VK_EXT_blend_operation_advanced";
}

#if OS == .WINDOWS {
    VK_EXT_BLEND_OPERATION_ADVANCED_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_EXT_BLEND_OPERATION_ADVANCED_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_EXT_conditional_rendering :: 1;
}

#if OS == .LINUX {
    VK_EXT_conditional_rendering :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_CONDITIONAL_RENDERING_EXTENSION_NAME :: "VK_EXT_conditional_rendering";
}

#if OS == .LINUX {
    VK_EXT_CONDITIONAL_RENDERING_EXTENSION_NAME :: "VK_EXT_conditional_rendering";
}

#if OS == .WINDOWS {
    VK_EXT_CONDITIONAL_RENDERING_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_CONDITIONAL_RENDERING_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_conservative_rasterization :: 1;
}

#if OS == .LINUX {
    VK_EXT_conservative_rasterization :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_CONSERVATIVE_RASTERIZATION_EXTENSION_NAME :: "VK_EXT_conservative_rasterization";
}

#if OS == .LINUX {
    VK_EXT_CONSERVATIVE_RASTERIZATION_EXTENSION_NAME :: "VK_EXT_conservative_rasterization";
}

#if OS == .WINDOWS {
    VK_EXT_CONSERVATIVE_RASTERIZATION_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_CONSERVATIVE_RASTERIZATION_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_debug_marker :: 1;
}

#if OS == .LINUX {
    VK_EXT_debug_marker :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_MARKER_EXTENSION_NAME :: "VK_EXT_debug_marker";
}

#if OS == .LINUX {
    VK_EXT_DEBUG_MARKER_EXTENSION_NAME :: "VK_EXT_debug_marker";
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_MARKER_SPEC_VERSION :: 4;
}

#if OS == .LINUX {
    VK_EXT_DEBUG_MARKER_SPEC_VERSION :: 4;
}

#if OS == .WINDOWS {
    VK_EXT_debug_report :: 1;
}

#if OS == .LINUX {
    VK_EXT_debug_report :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_REPORT_EXTENSION_NAME :: "VK_EXT_debug_report";
}

#if OS == .LINUX {
    VK_EXT_DEBUG_REPORT_EXTENSION_NAME :: "VK_EXT_debug_report";
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_REPORT_SPEC_VERSION :: 9;
}

#if OS == .LINUX {
    VK_EXT_DEBUG_REPORT_SPEC_VERSION :: 9;
}

#if OS == .WINDOWS {
    VK_EXT_debug_utils :: 1;
}

#if OS == .LINUX {
    VK_EXT_debug_utils :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_UTILS_EXTENSION_NAME :: "VK_EXT_debug_utils";
}

#if OS == .LINUX {
    VK_EXT_DEBUG_UTILS_EXTENSION_NAME :: "VK_EXT_debug_utils";
}

#if OS == .WINDOWS {
    VK_EXT_DEBUG_UTILS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DEBUG_UTILS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_depth_range_unrestricted :: 1;
}

#if OS == .LINUX {
    VK_EXT_depth_range_unrestricted :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DEPTH_RANGE_UNRESTRICTED_EXTENSION_NAME :: "VK_EXT_depth_range_unrestricted";
}

#if OS == .LINUX {
    VK_EXT_DEPTH_RANGE_UNRESTRICTED_EXTENSION_NAME :: "VK_EXT_depth_range_unrestricted";
}

#if OS == .WINDOWS {
    VK_EXT_DEPTH_RANGE_UNRESTRICTED_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DEPTH_RANGE_UNRESTRICTED_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_descriptor_indexing :: 1;
}

#if OS == .LINUX {
    VK_EXT_descriptor_indexing :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME :: "VK_EXT_descriptor_indexing";
}

#if OS == .LINUX {
    VK_EXT_DESCRIPTOR_INDEXING_EXTENSION_NAME :: "VK_EXT_descriptor_indexing";
}

#if OS == .WINDOWS {
    VK_EXT_DESCRIPTOR_INDEXING_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_EXT_DESCRIPTOR_INDEXING_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_EXT_direct_mode_display :: 1;
}

#if OS == .LINUX {
    VK_EXT_direct_mode_display :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DIRECT_MODE_DISPLAY_EXTENSION_NAME :: "VK_EXT_direct_mode_display";
}

#if OS == .LINUX {
    VK_EXT_DIRECT_MODE_DISPLAY_EXTENSION_NAME :: "VK_EXT_direct_mode_display";
}

#if OS == .WINDOWS {
    VK_EXT_DIRECT_MODE_DISPLAY_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DIRECT_MODE_DISPLAY_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_discard_rectangles :: 1;
}

#if OS == .LINUX {
    VK_EXT_discard_rectangles :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DISCARD_RECTANGLES_EXTENSION_NAME :: "VK_EXT_discard_rectangles";
}

#if OS == .LINUX {
    VK_EXT_DISCARD_RECTANGLES_EXTENSION_NAME :: "VK_EXT_discard_rectangles";
}

#if OS == .WINDOWS {
    VK_EXT_DISCARD_RECTANGLES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DISCARD_RECTANGLES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_display_control :: 1;
}

#if OS == .LINUX {
    VK_EXT_display_control :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DISPLAY_CONTROL_EXTENSION_NAME :: "VK_EXT_display_control";
}

#if OS == .LINUX {
    VK_EXT_DISPLAY_CONTROL_EXTENSION_NAME :: "VK_EXT_display_control";
}

#if OS == .WINDOWS {
    VK_EXT_DISPLAY_CONTROL_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DISPLAY_CONTROL_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_display_surface_counter :: 1;
}

#if OS == .LINUX {
    VK_EXT_display_surface_counter :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_DISPLAY_SURFACE_COUNTER_EXTENSION_NAME :: "VK_EXT_display_surface_counter";
}

#if OS == .LINUX {
    VK_EXT_DISPLAY_SURFACE_COUNTER_EXTENSION_NAME :: "VK_EXT_display_surface_counter";
}

#if OS == .WINDOWS {
    VK_EXT_DISPLAY_SURFACE_COUNTER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_DISPLAY_SURFACE_COUNTER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_external_memory_dma_buf :: 1;
}

#if OS == .LINUX {
    VK_EXT_external_memory_dma_buf :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_EXTERNAL_MEMORY_DMA_BUF_EXTENSION_NAME :: "VK_EXT_external_memory_dma_buf";
}

#if OS == .LINUX {
    VK_EXT_EXTERNAL_MEMORY_DMA_BUF_EXTENSION_NAME :: "VK_EXT_external_memory_dma_buf";
}

#if OS == .WINDOWS {
    VK_EXT_EXTERNAL_MEMORY_DMA_BUF_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_EXTERNAL_MEMORY_DMA_BUF_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_external_memory_host :: 1;
}

#if OS == .LINUX {
    VK_EXT_external_memory_host :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_EXTERNAL_MEMORY_HOST_EXTENSION_NAME :: "VK_EXT_external_memory_host";
}

#if OS == .LINUX {
    VK_EXT_EXTERNAL_MEMORY_HOST_EXTENSION_NAME :: "VK_EXT_external_memory_host";
}

#if OS == .WINDOWS {
    VK_EXT_EXTERNAL_MEMORY_HOST_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_EXTERNAL_MEMORY_HOST_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_global_priority :: 1;
}

#if OS == .LINUX {
    VK_EXT_global_priority :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_GLOBAL_PRIORITY_EXTENSION_NAME :: "VK_EXT_global_priority";
}

#if OS == .LINUX {
    VK_EXT_GLOBAL_PRIORITY_EXTENSION_NAME :: "VK_EXT_global_priority";
}

#if OS == .WINDOWS {
    VK_EXT_GLOBAL_PRIORITY_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_EXT_GLOBAL_PRIORITY_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_EXT_hdr_metadata :: 1;
}

#if OS == .LINUX {
    VK_EXT_hdr_metadata :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_HDR_METADATA_EXTENSION_NAME :: "VK_EXT_hdr_metadata";
}

#if OS == .LINUX {
    VK_EXT_HDR_METADATA_EXTENSION_NAME :: "VK_EXT_hdr_metadata";
}

#if OS == .WINDOWS {
    VK_EXT_HDR_METADATA_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_HDR_METADATA_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_inline_uniform_block :: 1;
}

#if OS == .LINUX {
    VK_EXT_inline_uniform_block :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_INLINE_UNIFORM_BLOCK_EXTENSION_NAME :: "VK_EXT_inline_uniform_block";
}

#if OS == .LINUX {
    VK_EXT_INLINE_UNIFORM_BLOCK_EXTENSION_NAME :: "VK_EXT_inline_uniform_block";
}

#if OS == .WINDOWS {
    VK_EXT_INLINE_UNIFORM_BLOCK_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_INLINE_UNIFORM_BLOCK_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_post_depth_coverage :: 1;
}

#if OS == .LINUX {
    VK_EXT_post_depth_coverage :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_POST_DEPTH_COVERAGE_EXTENSION_NAME :: "VK_EXT_post_depth_coverage";
}

#if OS == .LINUX {
    VK_EXT_POST_DEPTH_COVERAGE_EXTENSION_NAME :: "VK_EXT_post_depth_coverage";
}

#if OS == .WINDOWS {
    VK_EXT_POST_DEPTH_COVERAGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_POST_DEPTH_COVERAGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_queue_family_foreign :: 1;
}

#if OS == .LINUX {
    VK_EXT_queue_family_foreign :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_QUEUE_FAMILY_FOREIGN_EXTENSION_NAME :: "VK_EXT_queue_family_foreign";
}

#if OS == .LINUX {
    VK_EXT_QUEUE_FAMILY_FOREIGN_EXTENSION_NAME :: "VK_EXT_queue_family_foreign";
}

#if OS == .WINDOWS {
    VK_EXT_QUEUE_FAMILY_FOREIGN_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_QUEUE_FAMILY_FOREIGN_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_sample_locations :: 1;
}

#if OS == .LINUX {
    VK_EXT_sample_locations :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SAMPLE_LOCATIONS_EXTENSION_NAME :: "VK_EXT_sample_locations";
}

#if OS == .LINUX {
    VK_EXT_SAMPLE_LOCATIONS_EXTENSION_NAME :: "VK_EXT_sample_locations";
}

#if OS == .WINDOWS {
    VK_EXT_SAMPLE_LOCATIONS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SAMPLE_LOCATIONS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_sampler_filter_minmax :: 1;
}

#if OS == .LINUX {
    VK_EXT_sampler_filter_minmax :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SAMPLER_FILTER_MINMAX_EXTENSION_NAME :: "VK_EXT_sampler_filter_minmax";
}

#if OS == .LINUX {
    VK_EXT_SAMPLER_FILTER_MINMAX_EXTENSION_NAME :: "VK_EXT_sampler_filter_minmax";
}

#if OS == .WINDOWS {
    VK_EXT_SAMPLER_FILTER_MINMAX_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SAMPLER_FILTER_MINMAX_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_shader_stencil_export :: 1;
}

#if OS == .LINUX {
    VK_EXT_shader_stencil_export :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_STENCIL_EXPORT_EXTENSION_NAME :: "VK_EXT_shader_stencil_export";
}

#if OS == .LINUX {
    VK_EXT_SHADER_STENCIL_EXPORT_EXTENSION_NAME :: "VK_EXT_shader_stencil_export";
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_STENCIL_EXPORT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SHADER_STENCIL_EXPORT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_shader_subgroup_ballot :: 1;
}

#if OS == .LINUX {
    VK_EXT_shader_subgroup_ballot :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_SUBGROUP_BALLOT_EXTENSION_NAME :: "VK_EXT_shader_subgroup_ballot";
}

#if OS == .LINUX {
    VK_EXT_SHADER_SUBGROUP_BALLOT_EXTENSION_NAME :: "VK_EXT_shader_subgroup_ballot";
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_SUBGROUP_BALLOT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SHADER_SUBGROUP_BALLOT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_shader_subgroup_vote :: 1;
}

#if OS == .LINUX {
    VK_EXT_shader_subgroup_vote :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_SUBGROUP_VOTE_EXTENSION_NAME :: "VK_EXT_shader_subgroup_vote";
}

#if OS == .LINUX {
    VK_EXT_SHADER_SUBGROUP_VOTE_EXTENSION_NAME :: "VK_EXT_shader_subgroup_vote";
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_SUBGROUP_VOTE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SHADER_SUBGROUP_VOTE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_shader_viewport_index_layer :: 1;
}

#if OS == .LINUX {
    VK_EXT_shader_viewport_index_layer :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_VIEWPORT_INDEX_LAYER_EXTENSION_NAME :: "VK_EXT_shader_viewport_index_layer";
}

#if OS == .LINUX {
    VK_EXT_SHADER_VIEWPORT_INDEX_LAYER_EXTENSION_NAME :: "VK_EXT_shader_viewport_index_layer";
}

#if OS == .WINDOWS {
    VK_EXT_SHADER_VIEWPORT_INDEX_LAYER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_SHADER_VIEWPORT_INDEX_LAYER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_SWAPCHAIN_COLOR_SPACE_EXTENSION_NAME :: "VK_EXT_swapchain_colorspace";
}

#if OS == .LINUX {
    VK_EXT_SWAPCHAIN_COLOR_SPACE_EXTENSION_NAME :: "VK_EXT_swapchain_colorspace";
}

#if OS == .WINDOWS {
    VK_EXT_SWAPCHAIN_COLOR_SPACE_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_EXT_SWAPCHAIN_COLOR_SPACE_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_EXT_swapchain_colorspace :: 1;
}

#if OS == .LINUX {
    VK_EXT_swapchain_colorspace :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_validation_cache :: 1;
}

#if OS == .LINUX {
    VK_EXT_validation_cache :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_VALIDATION_CACHE_EXTENSION_NAME :: "VK_EXT_validation_cache";
}

#if OS == .LINUX {
    VK_EXT_VALIDATION_CACHE_EXTENSION_NAME :: "VK_EXT_validation_cache";
}

#if OS == .WINDOWS {
    VK_EXT_VALIDATION_CACHE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_VALIDATION_CACHE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_validation_flags :: 1;
}

#if OS == .LINUX {
    VK_EXT_validation_flags :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_VALIDATION_FLAGS_EXTENSION_NAME :: "VK_EXT_validation_flags";
}

#if OS == .LINUX {
    VK_EXT_VALIDATION_FLAGS_EXTENSION_NAME :: "VK_EXT_validation_flags";
}

#if OS == .WINDOWS {
    VK_EXT_VALIDATION_FLAGS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_EXT_VALIDATION_FLAGS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_vertex_attribute_divisor :: 1;
}

#if OS == .LINUX {
    VK_EXT_vertex_attribute_divisor :: 1;
}

#if OS == .WINDOWS {
    VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_EXTENSION_NAME :: "VK_EXT_vertex_attribute_divisor";
}

#if OS == .LINUX {
    VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_EXTENSION_NAME :: "VK_EXT_vertex_attribute_divisor";
}

#if OS == .WINDOWS {
    VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_EXT_VERTEX_ATTRIBUTE_DIVISOR_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_GOOGLE_display_timing :: 1;
}

#if OS == .LINUX {
    VK_GOOGLE_display_timing :: 1;
}

#if OS == .WINDOWS {
    VK_GOOGLE_DISPLAY_TIMING_EXTENSION_NAME :: "VK_GOOGLE_display_timing";
}

#if OS == .LINUX {
    VK_GOOGLE_DISPLAY_TIMING_EXTENSION_NAME :: "VK_GOOGLE_display_timing";
}

#if OS == .WINDOWS {
    VK_GOOGLE_DISPLAY_TIMING_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_GOOGLE_DISPLAY_TIMING_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_HEADER_VERSION :: 85;
}

#if OS == .LINUX {
    VK_HEADER_VERSION :: 85;
}

#if OS == .WINDOWS {
    VK_IMG_filter_cubic :: 1;
}

#if OS == .LINUX {
    VK_IMG_filter_cubic :: 1;
}

#if OS == .WINDOWS {
    VK_IMG_FILTER_CUBIC_EXTENSION_NAME :: "VK_IMG_filter_cubic";
}

#if OS == .LINUX {
    VK_IMG_FILTER_CUBIC_EXTENSION_NAME :: "VK_IMG_filter_cubic";
}

#if OS == .WINDOWS {
    VK_IMG_FILTER_CUBIC_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_IMG_FILTER_CUBIC_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_IMG_format_pvrtc :: 1;
}

#if OS == .LINUX {
    VK_IMG_format_pvrtc :: 1;
}

#if OS == .WINDOWS {
    VK_IMG_FORMAT_PVRTC_EXTENSION_NAME :: "VK_IMG_format_pvrtc";
}

#if OS == .LINUX {
    VK_IMG_FORMAT_PVRTC_EXTENSION_NAME :: "VK_IMG_format_pvrtc";
}

#if OS == .WINDOWS {
    VK_IMG_FORMAT_PVRTC_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_IMG_FORMAT_PVRTC_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_16bit_storage :: 1;
}

#if OS == .LINUX {
    VK_KHR_16bit_storage :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_16BIT_STORAGE_EXTENSION_NAME :: "VK_KHR_16bit_storage";
}

#if OS == .LINUX {
    VK_KHR_16BIT_STORAGE_EXTENSION_NAME :: "VK_KHR_16bit_storage";
}

#if OS == .WINDOWS {
    VK_KHR_16BIT_STORAGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_16BIT_STORAGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_8bit_storage :: 1;
}

#if OS == .LINUX {
    VK_KHR_8bit_storage :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_8BIT_STORAGE_EXTENSION_NAME :: "VK_KHR_8bit_storage";
}

#if OS == .LINUX {
    VK_KHR_8BIT_STORAGE_EXTENSION_NAME :: "VK_KHR_8bit_storage";
}

#if OS == .WINDOWS {
    VK_KHR_8BIT_STORAGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_8BIT_STORAGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_bind_memory2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_bind_memory2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_BIND_MEMORY_2_EXTENSION_NAME :: "VK_KHR_bind_memory2";
}

#if OS == .LINUX {
    VK_KHR_BIND_MEMORY_2_EXTENSION_NAME :: "VK_KHR_bind_memory2";
}

#if OS == .WINDOWS {
    VK_KHR_BIND_MEMORY_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_BIND_MEMORY_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_create_renderpass2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_create_renderpass2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_CREATE_RENDERPASS_2_EXTENSION_NAME :: "VK_KHR_create_renderpass2";
}

#if OS == .LINUX {
    VK_KHR_CREATE_RENDERPASS_2_EXTENSION_NAME :: "VK_KHR_create_renderpass2";
}

#if OS == .WINDOWS {
    VK_KHR_CREATE_RENDERPASS_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_CREATE_RENDERPASS_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_dedicated_allocation :: 1;
}

#if OS == .LINUX {
    VK_KHR_dedicated_allocation :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DEDICATED_ALLOCATION_EXTENSION_NAME :: "VK_KHR_dedicated_allocation";
}

#if OS == .LINUX {
    VK_KHR_DEDICATED_ALLOCATION_EXTENSION_NAME :: "VK_KHR_dedicated_allocation";
}

#if OS == .WINDOWS {
    VK_KHR_DEDICATED_ALLOCATION_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_KHR_DEDICATED_ALLOCATION_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_KHR_descriptor_update_template :: 1;
}

#if OS == .LINUX {
    VK_KHR_descriptor_update_template :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_EXTENSION_NAME :: "VK_KHR_descriptor_update_template";
}

#if OS == .LINUX {
    VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_EXTENSION_NAME :: "VK_KHR_descriptor_update_template";
}

#if OS == .WINDOWS {
    VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_DESCRIPTOR_UPDATE_TEMPLATE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_device_group :: 1;
}

#if OS == .LINUX {
    VK_KHR_device_group :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_device_group_creation :: 1;
}

#if OS == .LINUX {
    VK_KHR_device_group_creation :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DEVICE_GROUP_CREATION_EXTENSION_NAME :: "VK_KHR_device_group_creation";
}

#if OS == .LINUX {
    VK_KHR_DEVICE_GROUP_CREATION_EXTENSION_NAME :: "VK_KHR_device_group_creation";
}

#if OS == .WINDOWS {
    VK_KHR_DEVICE_GROUP_CREATION_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_DEVICE_GROUP_CREATION_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DEVICE_GROUP_EXTENSION_NAME :: "VK_KHR_device_group";
}

#if OS == .LINUX {
    VK_KHR_DEVICE_GROUP_EXTENSION_NAME :: "VK_KHR_device_group";
}

#if OS == .WINDOWS {
    VK_KHR_DEVICE_GROUP_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_KHR_DEVICE_GROUP_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_KHR_display :: 1;
}

#if OS == .LINUX {
    VK_KHR_display :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DISPLAY_EXTENSION_NAME :: "VK_KHR_display";
}

#if OS == .LINUX {
    VK_KHR_DISPLAY_EXTENSION_NAME :: "VK_KHR_display";
}

#if OS == .WINDOWS {
    VK_KHR_DISPLAY_SPEC_VERSION :: 21;
}

#if OS == .LINUX {
    VK_KHR_DISPLAY_SPEC_VERSION :: 21;
}

#if OS == .WINDOWS {
    VK_KHR_display_swapchain :: 1;
}

#if OS == .LINUX {
    VK_KHR_display_swapchain :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DISPLAY_SWAPCHAIN_EXTENSION_NAME :: "VK_KHR_display_swapchain";
}

#if OS == .LINUX {
    VK_KHR_DISPLAY_SWAPCHAIN_EXTENSION_NAME :: "VK_KHR_display_swapchain";
}

#if OS == .WINDOWS {
    VK_KHR_DISPLAY_SWAPCHAIN_SPEC_VERSION :: 9;
}

#if OS == .LINUX {
    VK_KHR_DISPLAY_SWAPCHAIN_SPEC_VERSION :: 9;
}

#if OS == .WINDOWS {
    VK_KHR_draw_indirect_count :: 1;
}

#if OS == .LINUX {
    VK_KHR_draw_indirect_count :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_DRAW_INDIRECT_COUNT_EXTENSION_NAME :: "VK_KHR_draw_indirect_count";
}

#if OS == .LINUX {
    VK_KHR_DRAW_INDIRECT_COUNT_EXTENSION_NAME :: "VK_KHR_draw_indirect_count";
}

#if OS == .WINDOWS {
    VK_KHR_DRAW_INDIRECT_COUNT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_DRAW_INDIRECT_COUNT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_fence :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_fence :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_fence_capabilities :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_fence_capabilities :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_fence_capabilities";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_fence_capabilities";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_EXTENSION_NAME :: "VK_KHR_external_fence";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_EXTENSION_NAME :: "VK_KHR_external_fence";
}

#if OS == .WINDOWS {
    VK_KHR_external_fence_fd :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_fence_fd :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_FD_EXTENSION_NAME :: "VK_KHR_external_fence_fd";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_FD_EXTENSION_NAME :: "VK_KHR_external_fence_fd";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_FD_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_FD_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_FENCE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_FENCE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_memory :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_memory :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_memory_capabilities :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_memory_capabilities :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_memory_capabilities";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_memory_capabilities";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_EXTENSION_NAME :: "VK_KHR_external_memory";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_EXTENSION_NAME :: "VK_KHR_external_memory";
}

#if OS == .WINDOWS {
    VK_KHR_external_memory_fd :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_memory_fd :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_FD_EXTENSION_NAME :: "VK_KHR_external_memory_fd";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_FD_EXTENSION_NAME :: "VK_KHR_external_memory_fd";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_FD_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_FD_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_MEMORY_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_MEMORY_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_semaphore :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_semaphore :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_external_semaphore_capabilities :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_semaphore_capabilities :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_semaphore_capabilities";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_EXTENSION_NAME :: "VK_KHR_external_semaphore_capabilities";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_EXTENSION_NAME :: "VK_KHR_external_semaphore";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_EXTENSION_NAME :: "VK_KHR_external_semaphore";
}

#if OS == .WINDOWS {
    VK_KHR_external_semaphore_fd :: 1;
}

#if OS == .LINUX {
    VK_KHR_external_semaphore_fd :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_FD_EXTENSION_NAME :: "VK_KHR_external_semaphore_fd";
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_FD_EXTENSION_NAME :: "VK_KHR_external_semaphore_fd";
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_FD_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_FD_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_EXTERNAL_SEMAPHORE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_EXTERNAL_SEMAPHORE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_get_display_properties2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_get_display_properties2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_GET_DISPLAY_PROPERTIES_2_EXTENSION_NAME :: "VK_KHR_get_display_properties2";
}

#if OS == .LINUX {
    VK_KHR_GET_DISPLAY_PROPERTIES_2_EXTENSION_NAME :: "VK_KHR_get_display_properties2";
}

#if OS == .WINDOWS {
    VK_KHR_GET_DISPLAY_PROPERTIES_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_GET_DISPLAY_PROPERTIES_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_get_memory_requirements2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_get_memory_requirements2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_GET_MEMORY_REQUIREMENTS_2_EXTENSION_NAME :: "VK_KHR_get_memory_requirements2";
}

#if OS == .LINUX {
    VK_KHR_GET_MEMORY_REQUIREMENTS_2_EXTENSION_NAME :: "VK_KHR_get_memory_requirements2";
}

#if OS == .WINDOWS {
    VK_KHR_GET_MEMORY_REQUIREMENTS_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_GET_MEMORY_REQUIREMENTS_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_get_physical_device_properties2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_get_physical_device_properties2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME :: "VK_KHR_get_physical_device_properties2";
}

#if OS == .LINUX {
    VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME :: "VK_KHR_get_physical_device_properties2";
}

#if OS == .WINDOWS {
    VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_get_surface_capabilities2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_get_surface_capabilities2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME :: "VK_KHR_get_surface_capabilities2";
}

#if OS == .LINUX {
    VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME :: "VK_KHR_get_surface_capabilities2";
}

#if OS == .WINDOWS {
    VK_KHR_GET_SURFACE_CAPABILITIES_2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_GET_SURFACE_CAPABILITIES_2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_image_format_list :: 1;
}

#if OS == .LINUX {
    VK_KHR_image_format_list :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_IMAGE_FORMAT_LIST_EXTENSION_NAME :: "VK_KHR_image_format_list";
}

#if OS == .LINUX {
    VK_KHR_IMAGE_FORMAT_LIST_EXTENSION_NAME :: "VK_KHR_image_format_list";
}

#if OS == .WINDOWS {
    VK_KHR_IMAGE_FORMAT_LIST_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_IMAGE_FORMAT_LIST_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_incremental_present :: 1;
}

#if OS == .LINUX {
    VK_KHR_incremental_present :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_INCREMENTAL_PRESENT_EXTENSION_NAME :: "VK_KHR_incremental_present";
}

#if OS == .LINUX {
    VK_KHR_INCREMENTAL_PRESENT_EXTENSION_NAME :: "VK_KHR_incremental_present";
}

#if OS == .WINDOWS {
    VK_KHR_INCREMENTAL_PRESENT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_INCREMENTAL_PRESENT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_maintenance1 :: 1;
}

#if OS == .LINUX {
    VK_KHR_maintenance1 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE1_EXTENSION_NAME :: "VK_KHR_maintenance1";
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE1_EXTENSION_NAME :: "VK_KHR_maintenance1";
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE1_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE1_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_KHR_maintenance2 :: 1;
}

#if OS == .LINUX {
    VK_KHR_maintenance2 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE2_EXTENSION_NAME :: "VK_KHR_maintenance2";
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE2_EXTENSION_NAME :: "VK_KHR_maintenance2";
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_maintenance3 :: 1;
}

#if OS == .LINUX {
    VK_KHR_maintenance3 :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE3_EXTENSION_NAME :: "VK_KHR_maintenance3";
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE3_EXTENSION_NAME :: "VK_KHR_maintenance3";
}

#if OS == .WINDOWS {
    VK_KHR_MAINTENANCE3_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_MAINTENANCE3_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_multiview :: 1;
}

#if OS == .LINUX {
    VK_KHR_multiview :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_MULTIVIEW_EXTENSION_NAME :: "VK_KHR_multiview";
}

#if OS == .LINUX {
    VK_KHR_MULTIVIEW_EXTENSION_NAME :: "VK_KHR_multiview";
}

#if OS == .WINDOWS {
    VK_KHR_MULTIVIEW_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_MULTIVIEW_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_push_descriptor :: 1;
}

#if OS == .LINUX {
    VK_KHR_push_descriptor :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_PUSH_DESCRIPTOR_EXTENSION_NAME :: "VK_KHR_push_descriptor";
}

#if OS == .LINUX {
    VK_KHR_PUSH_DESCRIPTOR_EXTENSION_NAME :: "VK_KHR_push_descriptor";
}

#if OS == .WINDOWS {
    VK_KHR_PUSH_DESCRIPTOR_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_KHR_PUSH_DESCRIPTOR_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_KHR_relaxed_block_layout :: 1;
}

#if OS == .LINUX {
    VK_KHR_relaxed_block_layout :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_RELAXED_BLOCK_LAYOUT_EXTENSION_NAME :: "VK_KHR_relaxed_block_layout";
}

#if OS == .LINUX {
    VK_KHR_RELAXED_BLOCK_LAYOUT_EXTENSION_NAME :: "VK_KHR_relaxed_block_layout";
}

#if OS == .WINDOWS {
    VK_KHR_RELAXED_BLOCK_LAYOUT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_RELAXED_BLOCK_LAYOUT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_sampler_mirror_clamp_to_edge :: 1;
}

#if OS == .LINUX {
    VK_KHR_sampler_mirror_clamp_to_edge :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME :: "VK_KHR_sampler_mirror_clamp_to_edge";
}

#if OS == .LINUX {
    VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_EXTENSION_NAME :: "VK_KHR_sampler_mirror_clamp_to_edge";
}

#if OS == .WINDOWS {
    VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_SAMPLER_MIRROR_CLAMP_TO_EDGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_sampler_ycbcr_conversion :: 1;
}

#if OS == .LINUX {
    VK_KHR_sampler_ycbcr_conversion :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SAMPLER_YCBCR_CONVERSION_EXTENSION_NAME :: "VK_KHR_sampler_ycbcr_conversion";
}

#if OS == .LINUX {
    VK_KHR_SAMPLER_YCBCR_CONVERSION_EXTENSION_NAME :: "VK_KHR_sampler_ycbcr_conversion";
}

#if OS == .WINDOWS {
    VK_KHR_SAMPLER_YCBCR_CONVERSION_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_SAMPLER_YCBCR_CONVERSION_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_shader_draw_parameters :: 1;
}

#if OS == .LINUX {
    VK_KHR_shader_draw_parameters :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME :: "VK_KHR_shader_draw_parameters";
}

#if OS == .LINUX {
    VK_KHR_SHADER_DRAW_PARAMETERS_EXTENSION_NAME :: "VK_KHR_shader_draw_parameters";
}

#if OS == .WINDOWS {
    VK_KHR_SHADER_DRAW_PARAMETERS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_SHADER_DRAW_PARAMETERS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_shared_presentable_image :: 1;
}

#if OS == .LINUX {
    VK_KHR_shared_presentable_image :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SHARED_PRESENTABLE_IMAGE_EXTENSION_NAME :: "VK_KHR_shared_presentable_image";
}

#if OS == .LINUX {
    VK_KHR_SHARED_PRESENTABLE_IMAGE_EXTENSION_NAME :: "VK_KHR_shared_presentable_image";
}

#if OS == .WINDOWS {
    VK_KHR_SHARED_PRESENTABLE_IMAGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_SHARED_PRESENTABLE_IMAGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_storage_buffer_storage_class :: 1;
}

#if OS == .LINUX {
    VK_KHR_storage_buffer_storage_class :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_STORAGE_BUFFER_STORAGE_CLASS_EXTENSION_NAME :: "VK_KHR_storage_buffer_storage_class";
}

#if OS == .LINUX {
    VK_KHR_STORAGE_BUFFER_STORAGE_CLASS_EXTENSION_NAME :: "VK_KHR_storage_buffer_storage_class";
}

#if OS == .WINDOWS {
    VK_KHR_STORAGE_BUFFER_STORAGE_CLASS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_STORAGE_BUFFER_STORAGE_CLASS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_surface :: 1;
}

#if OS == .LINUX {
    VK_KHR_surface :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SURFACE_EXTENSION_NAME :: "VK_KHR_surface";
}

#if OS == .LINUX {
    VK_KHR_SURFACE_EXTENSION_NAME :: "VK_KHR_surface";
}

#if OS == .WINDOWS {
    VK_KHR_SURFACE_SPEC_VERSION :: 25;
}

#if OS == .LINUX {
    VK_KHR_SURFACE_SPEC_VERSION :: 25;
}

#if OS == .WINDOWS {
    VK_KHR_swapchain :: 1;
}

#if OS == .LINUX {
    VK_KHR_swapchain :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME :: "VK_KHR_swapchain";
}

#if OS == .LINUX {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME :: "VK_KHR_swapchain";
}

#if OS == .WINDOWS {
    VK_KHR_SWAPCHAIN_SPEC_VERSION :: 70;
}

#if OS == .LINUX {
    VK_KHR_SWAPCHAIN_SPEC_VERSION :: 70;
}

#if OS == .WINDOWS {
    VK_KHR_variable_pointers :: 1;
}

#if OS == .LINUX {
    VK_KHR_variable_pointers :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_VARIABLE_POINTERS_EXTENSION_NAME :: "VK_KHR_variable_pointers";
}

#if OS == .LINUX {
    VK_KHR_VARIABLE_POINTERS_EXTENSION_NAME :: "VK_KHR_variable_pointers";
}

#if OS == .WINDOWS {
    VK_KHR_VARIABLE_POINTERS_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_KHR_VARIABLE_POINTERS_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_vulkan_memory_model :: 1;
}

#if OS == .LINUX {
    VK_KHR_vulkan_memory_model :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_VULKAN_MEMORY_MODEL_EXTENSION_NAME :: "VK_KHR_vulkan_memory_model";
}

#if OS == .LINUX {
    VK_KHR_VULKAN_MEMORY_MODEL_EXTENSION_NAME :: "VK_KHR_vulkan_memory_model";
}

#if OS == .WINDOWS {
    VK_KHR_VULKAN_MEMORY_MODEL_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_KHR_VULKAN_MEMORY_MODEL_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_KHR_win32_surface :: 1;
}

#if OS == .WINDOWS {
    VK_KHR_WIN32_SURFACE_EXTENSION_NAME :: "VK_KHR_win32_surface";
}

#if OS == .WINDOWS {
    VK_KHR_WIN32_SURFACE_SPEC_VERSION :: 5;
}

#if OS == .WINDOWS {
    VK_LOD_CLAMP_NONE :: 1000.0;
}

#if OS == .LINUX {
    VK_LOD_CLAMP_NONE :: 1000.0;
}

#if OS == .WINDOWS {
    VK_LUID_SIZE :: 8;
}

#if OS == .LINUX {
    VK_LUID_SIZE :: 8;
}

#if OS == .WINDOWS {
    VK_LUID_SIZE_KHR :: VK_LUID_SIZE;
}

#if OS == .LINUX {
    VK_LUID_SIZE_KHR :: VK_LUID_SIZE;
}

#if OS == .WINDOWS {
    VK_MAX_DEVICE_GROUP_SIZE :: 32;
}

#if OS == .LINUX {
    VK_MAX_DEVICE_GROUP_SIZE :: 32;
}

#if OS == .WINDOWS {
    VK_MAX_DEVICE_GROUP_SIZE_KHR :: VK_MAX_DEVICE_GROUP_SIZE;
}

#if OS == .LINUX {
    VK_MAX_DEVICE_GROUP_SIZE_KHR :: VK_MAX_DEVICE_GROUP_SIZE;
}

VK_NULL_HANDLE :: null;

#if OS == .WINDOWS {
    VK_NV_clip_space_w_scaling :: 1;
}

#if OS == .LINUX {
    VK_NV_clip_space_w_scaling :: 1;
}

#if OS == .WINDOWS {
    VK_NV_CLIP_SPACE_W_SCALING_EXTENSION_NAME :: "VK_NV_clip_space_w_scaling";
}

#if OS == .LINUX {
    VK_NV_CLIP_SPACE_W_SCALING_EXTENSION_NAME :: "VK_NV_clip_space_w_scaling";
}

#if OS == .WINDOWS {
    VK_NV_CLIP_SPACE_W_SCALING_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_CLIP_SPACE_W_SCALING_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_compute_shader_derivatives :: 1;
}

#if OS == .LINUX {
    VK_NV_compute_shader_derivatives :: 1;
}

#if OS == .WINDOWS {
    VK_NV_COMPUTE_SHADER_DERIVATIVES_EXTENSION_NAME :: "VK_NV_compute_shader_derivatives";
}

#if OS == .LINUX {
    VK_NV_COMPUTE_SHADER_DERIVATIVES_EXTENSION_NAME :: "VK_NV_compute_shader_derivatives";
}

#if OS == .WINDOWS {
    VK_NV_COMPUTE_SHADER_DERIVATIVES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_COMPUTE_SHADER_DERIVATIVES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_corner_sampled_image :: 1;
}

#if OS == .LINUX {
    VK_NV_corner_sampled_image :: 1;
}

#if OS == .WINDOWS {
    VK_NV_CORNER_SAMPLED_IMAGE_EXTENSION_NAME :: "VK_NV_corner_sampled_image";
}

#if OS == .LINUX {
    VK_NV_CORNER_SAMPLED_IMAGE_EXTENSION_NAME :: "VK_NV_corner_sampled_image";
}

#if OS == .WINDOWS {
    VK_NV_CORNER_SAMPLED_IMAGE_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_NV_CORNER_SAMPLED_IMAGE_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_NV_dedicated_allocation :: 1;
}

#if OS == .LINUX {
    VK_NV_dedicated_allocation :: 1;
}

#if OS == .WINDOWS {
    VK_NV_DEDICATED_ALLOCATION_EXTENSION_NAME :: "VK_NV_dedicated_allocation";
}

#if OS == .LINUX {
    VK_NV_DEDICATED_ALLOCATION_EXTENSION_NAME :: "VK_NV_dedicated_allocation";
}

#if OS == .WINDOWS {
    VK_NV_DEDICATED_ALLOCATION_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_DEDICATED_ALLOCATION_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_device_diagnostic_checkpoints :: 1;
}

#if OS == .LINUX {
    VK_NV_device_diagnostic_checkpoints :: 1;
}

#if OS == .WINDOWS {
    VK_NV_DEVICE_DIAGNOSTIC_CHECKPOINTS_EXTENSION_NAME :: "VK_NV_device_diagnostic_checkpoints";
}

#if OS == .LINUX {
    VK_NV_DEVICE_DIAGNOSTIC_CHECKPOINTS_EXTENSION_NAME :: "VK_NV_device_diagnostic_checkpoints";
}

#if OS == .WINDOWS {
    VK_NV_DEVICE_DIAGNOSTIC_CHECKPOINTS_SPEC_VERSION :: 2;
}

#if OS == .LINUX {
    VK_NV_DEVICE_DIAGNOSTIC_CHECKPOINTS_SPEC_VERSION :: 2;
}

#if OS == .WINDOWS {
    VK_NV_external_memory :: 1;
}

#if OS == .LINUX {
    VK_NV_external_memory :: 1;
}

#if OS == .WINDOWS {
    VK_NV_external_memory_capabilities :: 1;
}

#if OS == .LINUX {
    VK_NV_external_memory_capabilities :: 1;
}

#if OS == .WINDOWS {
    VK_NV_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME :: "VK_NV_external_memory_capabilities";
}

#if OS == .LINUX {
    VK_NV_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION_NAME :: "VK_NV_external_memory_capabilities";
}

#if OS == .WINDOWS {
    VK_NV_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_EXTERNAL_MEMORY_CAPABILITIES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_EXTERNAL_MEMORY_EXTENSION_NAME :: "VK_NV_external_memory";
}

#if OS == .LINUX {
    VK_NV_EXTERNAL_MEMORY_EXTENSION_NAME :: "VK_NV_external_memory";
}

#if OS == .WINDOWS {
    VK_NV_EXTERNAL_MEMORY_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_EXTERNAL_MEMORY_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_fill_rectangle :: 1;
}

#if OS == .LINUX {
    VK_NV_fill_rectangle :: 1;
}

#if OS == .WINDOWS {
    VK_NV_FILL_RECTANGLE_EXTENSION_NAME :: "VK_NV_fill_rectangle";
}

#if OS == .LINUX {
    VK_NV_FILL_RECTANGLE_EXTENSION_NAME :: "VK_NV_fill_rectangle";
}

#if OS == .WINDOWS {
    VK_NV_FILL_RECTANGLE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_FILL_RECTANGLE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_fragment_coverage_to_color :: 1;
}

#if OS == .LINUX {
    VK_NV_fragment_coverage_to_color :: 1;
}

#if OS == .WINDOWS {
    VK_NV_FRAGMENT_COVERAGE_TO_COLOR_EXTENSION_NAME :: "VK_NV_fragment_coverage_to_color";
}

#if OS == .LINUX {
    VK_NV_FRAGMENT_COVERAGE_TO_COLOR_EXTENSION_NAME :: "VK_NV_fragment_coverage_to_color";
}

#if OS == .WINDOWS {
    VK_NV_FRAGMENT_COVERAGE_TO_COLOR_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_FRAGMENT_COVERAGE_TO_COLOR_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_fragment_shader_barycentric :: 1;
}

#if OS == .LINUX {
    VK_NV_fragment_shader_barycentric :: 1;
}

#if OS == .WINDOWS {
    VK_NV_FRAGMENT_SHADER_BARYCENTRIC_EXTENSION_NAME :: "VK_NV_fragment_shader_barycentric";
}

#if OS == .LINUX {
    VK_NV_FRAGMENT_SHADER_BARYCENTRIC_EXTENSION_NAME :: "VK_NV_fragment_shader_barycentric";
}

#if OS == .WINDOWS {
    VK_NV_FRAGMENT_SHADER_BARYCENTRIC_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_FRAGMENT_SHADER_BARYCENTRIC_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_framebuffer_mixed_samples :: 1;
}

#if OS == .LINUX {
    VK_NV_framebuffer_mixed_samples :: 1;
}

#if OS == .WINDOWS {
    VK_NV_FRAMEBUFFER_MIXED_SAMPLES_EXTENSION_NAME :: "VK_NV_framebuffer_mixed_samples";
}

#if OS == .LINUX {
    VK_NV_FRAMEBUFFER_MIXED_SAMPLES_EXTENSION_NAME :: "VK_NV_framebuffer_mixed_samples";
}

#if OS == .WINDOWS {
    VK_NV_FRAMEBUFFER_MIXED_SAMPLES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_FRAMEBUFFER_MIXED_SAMPLES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_geometry_shader_passthrough :: 1;
}

#if OS == .LINUX {
    VK_NV_geometry_shader_passthrough :: 1;
}

#if OS == .WINDOWS {
    VK_NV_GEOMETRY_SHADER_PASSTHROUGH_EXTENSION_NAME :: "VK_NV_geometry_shader_passthrough";
}

#if OS == .LINUX {
    VK_NV_GEOMETRY_SHADER_PASSTHROUGH_EXTENSION_NAME :: "VK_NV_geometry_shader_passthrough";
}

#if OS == .WINDOWS {
    VK_NV_GEOMETRY_SHADER_PASSTHROUGH_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_GEOMETRY_SHADER_PASSTHROUGH_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_glsl_shader :: 1;
}

#if OS == .LINUX {
    VK_NV_glsl_shader :: 1;
}

#if OS == .WINDOWS {
    VK_NV_GLSL_SHADER_EXTENSION_NAME :: "VK_NV_glsl_shader";
}

#if OS == .LINUX {
    VK_NV_GLSL_SHADER_EXTENSION_NAME :: "VK_NV_glsl_shader";
}

#if OS == .WINDOWS {
    VK_NV_GLSL_SHADER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_GLSL_SHADER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_mesh_shader :: 1;
}

#if OS == .LINUX {
    VK_NV_mesh_shader :: 1;
}

#if OS == .WINDOWS {
    VK_NV_MESH_SHADER_EXTENSION_NAME :: "VK_NV_mesh_shader";
}

#if OS == .LINUX {
    VK_NV_MESH_SHADER_EXTENSION_NAME :: "VK_NV_mesh_shader";
}

#if OS == .WINDOWS {
    VK_NV_MESH_SHADER_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_MESH_SHADER_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_representative_fragment_test :: 1;
}

#if OS == .LINUX {
    VK_NV_representative_fragment_test :: 1;
}

#if OS == .WINDOWS {
    VK_NV_REPRESENTATIVE_FRAGMENT_TEST_EXTENSION_NAME :: "VK_NV_representative_fragment_test";
}

#if OS == .LINUX {
    VK_NV_REPRESENTATIVE_FRAGMENT_TEST_EXTENSION_NAME :: "VK_NV_representative_fragment_test";
}

#if OS == .WINDOWS {
    VK_NV_REPRESENTATIVE_FRAGMENT_TEST_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_REPRESENTATIVE_FRAGMENT_TEST_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_sample_mask_override_coverage :: 1;
}

#if OS == .LINUX {
    VK_NV_sample_mask_override_coverage :: 1;
}

#if OS == .WINDOWS {
    VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_EXTENSION_NAME :: "VK_NV_sample_mask_override_coverage";
}

#if OS == .LINUX {
    VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_EXTENSION_NAME :: "VK_NV_sample_mask_override_coverage";
}

#if OS == .WINDOWS {
    VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_SAMPLE_MASK_OVERRIDE_COVERAGE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_scissor_exclusive :: 1;
}

#if OS == .LINUX {
    VK_NV_scissor_exclusive :: 1;
}

#if OS == .WINDOWS {
    VK_NV_SCISSOR_EXCLUSIVE_EXTENSION_NAME :: "VK_NV_scissor_exclusive";
}

#if OS == .LINUX {
    VK_NV_SCISSOR_EXCLUSIVE_EXTENSION_NAME :: "VK_NV_scissor_exclusive";
}

#if OS == .WINDOWS {
    VK_NV_SCISSOR_EXCLUSIVE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_SCISSOR_EXCLUSIVE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_shader_image_footprint :: 1;
}

#if OS == .LINUX {
    VK_NV_shader_image_footprint :: 1;
}

#if OS == .WINDOWS {
    VK_NV_SHADER_IMAGE_FOOTPRINT_EXTENSION_NAME :: "VK_NV_shader_image_footprint";
}

#if OS == .LINUX {
    VK_NV_SHADER_IMAGE_FOOTPRINT_EXTENSION_NAME :: "VK_NV_shader_image_footprint";
}

#if OS == .WINDOWS {
    VK_NV_SHADER_IMAGE_FOOTPRINT_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_SHADER_IMAGE_FOOTPRINT_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_shader_subgroup_partitioned :: 1;
}

#if OS == .LINUX {
    VK_NV_shader_subgroup_partitioned :: 1;
}

#if OS == .WINDOWS {
    VK_NV_SHADER_SUBGROUP_PARTITIONED_EXTENSION_NAME :: "VK_NV_shader_subgroup_partitioned";
}

#if OS == .LINUX {
    VK_NV_SHADER_SUBGROUP_PARTITIONED_EXTENSION_NAME :: "VK_NV_shader_subgroup_partitioned";
}

#if OS == .WINDOWS {
    VK_NV_SHADER_SUBGROUP_PARTITIONED_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_SHADER_SUBGROUP_PARTITIONED_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_shading_rate_image :: 1;
}

#if OS == .LINUX {
    VK_NV_shading_rate_image :: 1;
}

#if OS == .WINDOWS {
    VK_NV_SHADING_RATE_IMAGE_EXTENSION_NAME :: "VK_NV_shading_rate_image";
}

#if OS == .LINUX {
    VK_NV_SHADING_RATE_IMAGE_EXTENSION_NAME :: "VK_NV_shading_rate_image";
}

#if OS == .WINDOWS {
    VK_NV_SHADING_RATE_IMAGE_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_NV_SHADING_RATE_IMAGE_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_NV_viewport_array2 :: 1;
}

#if OS == .LINUX {
    VK_NV_viewport_array2 :: 1;
}

#if OS == .WINDOWS {
    VK_NV_VIEWPORT_ARRAY2_EXTENSION_NAME :: "VK_NV_viewport_array2";
}

#if OS == .LINUX {
    VK_NV_VIEWPORT_ARRAY2_EXTENSION_NAME :: "VK_NV_viewport_array2";
}

#if OS == .WINDOWS {
    VK_NV_VIEWPORT_ARRAY2_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_VIEWPORT_ARRAY2_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NV_viewport_swizzle :: 1;
}

#if OS == .LINUX {
    VK_NV_viewport_swizzle :: 1;
}

#if OS == .WINDOWS {
    VK_NV_VIEWPORT_SWIZZLE_EXTENSION_NAME :: "VK_NV_viewport_swizzle";
}

#if OS == .LINUX {
    VK_NV_VIEWPORT_SWIZZLE_EXTENSION_NAME :: "VK_NV_viewport_swizzle";
}

#if OS == .WINDOWS {
    VK_NV_VIEWPORT_SWIZZLE_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NV_VIEWPORT_SWIZZLE_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NVX_device_generated_commands :: 1;
}

#if OS == .LINUX {
    VK_NVX_device_generated_commands :: 1;
}

#if OS == .WINDOWS {
    VK_NVX_DEVICE_GENERATED_COMMANDS_EXTENSION_NAME :: "VK_NVX_device_generated_commands";
}

#if OS == .LINUX {
    VK_NVX_DEVICE_GENERATED_COMMANDS_EXTENSION_NAME :: "VK_NVX_device_generated_commands";
}

#if OS == .WINDOWS {
    VK_NVX_DEVICE_GENERATED_COMMANDS_SPEC_VERSION :: 3;
}

#if OS == .LINUX {
    VK_NVX_DEVICE_GENERATED_COMMANDS_SPEC_VERSION :: 3;
}

#if OS == .WINDOWS {
    VK_NVX_multiview_per_view_attributes :: 1;
}

#if OS == .LINUX {
    VK_NVX_multiview_per_view_attributes :: 1;
}

#if OS == .WINDOWS {
    VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_EXTENSION_NAME :: "VK_NVX_multiview_per_view_attributes";
}

#if OS == .LINUX {
    VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_EXTENSION_NAME :: "VK_NVX_multiview_per_view_attributes";
}

#if OS == .WINDOWS {
    VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NVX_MULTIVIEW_PER_VIEW_ATTRIBUTES_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_NVX_raytracing :: 1;
}

#if OS == .LINUX {
    VK_NVX_raytracing :: 1;
}

#if OS == .WINDOWS {
    VK_NVX_RAYTRACING_EXTENSION_NAME :: "VK_NVX_raytracing";
}

#if OS == .LINUX {
    VK_NVX_RAYTRACING_EXTENSION_NAME :: "VK_NVX_raytracing";
}

#if OS == .WINDOWS {
    VK_NVX_RAYTRACING_SPEC_VERSION :: 1;
}

#if OS == .LINUX {
    VK_NVX_RAYTRACING_SPEC_VERSION :: 1;
}

#if OS == .WINDOWS {
    VK_QUEUE_FAMILY_EXTERNAL :: ~0 - 1;
}

#if OS == .LINUX {
    VK_QUEUE_FAMILY_EXTERNAL :: ~0 - 1;
}

#if OS == .WINDOWS {
    VK_QUEUE_FAMILY_EXTERNAL_KHR :: VK_QUEUE_FAMILY_EXTERNAL;
}

#if OS == .LINUX {
    VK_QUEUE_FAMILY_EXTERNAL_KHR :: VK_QUEUE_FAMILY_EXTERNAL;
}

#if OS == .WINDOWS {
    VK_QUEUE_FAMILY_FOREIGN_EXT :: ~0 - 2;
}

#if OS == .LINUX {
    VK_QUEUE_FAMILY_FOREIGN_EXT :: ~0 - 2;
}

#if OS == .WINDOWS {
    VK_VERSION_1_0 :: 1;
}

#if OS == .LINUX {
    VK_VERSION_1_0 :: 1;
}

#if OS == .WINDOWS {
    VK_VERSION_1_1 :: 1;
}

#if OS == .LINUX {
    VK_VERSION_1_1 :: 1;
}

#if OS == .WINDOWS {
    VkAccelerationStructureNVX :: *VkAccelerationStructureNVX_T;
}

#if OS == .LINUX {
    VkAccelerationStructureNVX :: *VkAccelerationStructureNVX_T;
}

#if OS == .WINDOWS {
    VkAccessFlags :: VkAccessFlagBits;
}

#if OS == .LINUX {
    VkAccessFlags :: VkAccessFlagBits;
}

#if OS == .WINDOWS {
    VkAttachmentDescriptionFlags :: VkAttachmentDescriptionFlagBits;
}

#if OS == .LINUX {
    VkAttachmentDescriptionFlags :: VkAttachmentDescriptionFlagBits;
}

#if OS == .WINDOWS {
    VkBindBufferMemoryDeviceGroupInfoKHR :: VkBindBufferMemoryDeviceGroupInfo;
}

#if OS == .LINUX {
    VkBindBufferMemoryDeviceGroupInfoKHR :: VkBindBufferMemoryDeviceGroupInfo;
}

#if OS == .WINDOWS {
    VkBindBufferMemoryInfoKHR :: VkBindBufferMemoryInfo;
}

#if OS == .LINUX {
    VkBindBufferMemoryInfoKHR :: VkBindBufferMemoryInfo;
}

#if OS == .WINDOWS {
    VkBindImageMemoryDeviceGroupInfoKHR :: VkBindImageMemoryDeviceGroupInfo;
}

#if OS == .LINUX {
    VkBindImageMemoryDeviceGroupInfoKHR :: VkBindImageMemoryDeviceGroupInfo;
}

#if OS == .WINDOWS {
    VkBindImageMemoryInfoKHR :: VkBindImageMemoryInfo;
}

#if OS == .LINUX {
    VkBindImageMemoryInfoKHR :: VkBindImageMemoryInfo;
}

#if OS == .WINDOWS {
    VkBindImagePlaneMemoryInfoKHR :: VkBindImagePlaneMemoryInfo;
}

#if OS == .LINUX {
    VkBindImagePlaneMemoryInfoKHR :: VkBindImagePlaneMemoryInfo;
}

#if OS == .WINDOWS {
    VkBool32 :: u32;
}

#if OS == .LINUX {
    VkBool32 :: u32;
}

#if OS == .WINDOWS {
    VkBuffer :: *VkBuffer_T;
}

#if OS == .LINUX {
    VkBuffer :: *VkBuffer_T;
}

#if OS == .WINDOWS {
    VkBufferCreateFlags :: VkBufferCreateFlagBits;
}

#if OS == .LINUX {
    VkBufferCreateFlags :: VkBufferCreateFlagBits;
}

#if OS == .WINDOWS {
    VkBufferMemoryRequirementsInfo2KHR :: VkBufferMemoryRequirementsInfo2;
}

#if OS == .LINUX {
    VkBufferMemoryRequirementsInfo2KHR :: VkBufferMemoryRequirementsInfo2;
}

#if OS == .WINDOWS {
    VkBufferUsageFlags :: VkBufferUsageFlagBits;
}

#if OS == .LINUX {
    VkBufferUsageFlags :: VkBufferUsageFlagBits;
}

#if OS == .WINDOWS {
    VkBufferView :: *VkBufferView_T;
}

#if OS == .LINUX {
    VkBufferView :: *VkBufferView_T;
}

#if OS == .WINDOWS {
    VkBufferViewCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkBufferViewCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkBuildAccelerationStructureFlagsNVX :: VkBuildAccelerationStructureFlagBitsNVX;
}

#if OS == .LINUX {
    VkBuildAccelerationStructureFlagsNVX :: VkBuildAccelerationStructureFlagBitsNVX;
}

#if OS == .WINDOWS {
    VkChromaLocationKHR :: VkChromaLocation;
}

#if OS == .LINUX {
    VkChromaLocationKHR :: VkChromaLocation;
}

#if OS == .WINDOWS {
    VkColorComponentFlags :: VkColorComponentFlagBits;
}

#if OS == .LINUX {
    VkColorComponentFlags :: VkColorComponentFlagBits;
}

#if OS == .WINDOWS {
    VkCommandBuffer :: *VkCommandBuffer_T;
}

#if OS == .LINUX {
    VkCommandBuffer :: *VkCommandBuffer_T;
}

#if OS == .WINDOWS {
    VkCommandBufferResetFlags :: VkCommandBufferResetFlagBits;
}

#if OS == .LINUX {
    VkCommandBufferResetFlags :: VkCommandBufferResetFlagBits;
}

#if OS == .WINDOWS {
    VkCommandBufferUsageFlags :: VkCommandBufferUsageFlagBits;
}

#if OS == .LINUX {
    VkCommandBufferUsageFlags :: VkCommandBufferUsageFlagBits;
}

#if OS == .WINDOWS {
    VkCommandPool :: *VkCommandPool_T;
}

#if OS == .LINUX {
    VkCommandPool :: *VkCommandPool_T;
}

#if OS == .WINDOWS {
    VkCommandPoolCreateFlags :: VkCommandPoolCreateFlagBits;
}

#if OS == .LINUX {
    VkCommandPoolCreateFlags :: VkCommandPoolCreateFlagBits;
}

#if OS == .WINDOWS {
    VkCommandPoolResetFlags :: VkCommandPoolResetFlagBits;
}

#if OS == .LINUX {
    VkCommandPoolResetFlags :: VkCommandPoolResetFlagBits;
}

#if OS == .WINDOWS {
    VkCommandPoolTrimFlags :: VkFlags;
}

#if OS == .LINUX {
    VkCommandPoolTrimFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkCommandPoolTrimFlagsKHR :: VkCommandPoolTrimFlags;
}

#if OS == .LINUX {
    VkCommandPoolTrimFlagsKHR :: VkCommandPoolTrimFlags;
}

#if OS == .WINDOWS {
    VkCompositeAlphaFlagsKHR :: VkCompositeAlphaFlagBitsKHR;
}

#if OS == .LINUX {
    VkCompositeAlphaFlagsKHR :: VkCompositeAlphaFlagBitsKHR;
}

#if OS == .WINDOWS {
    VkConditionalRenderingFlagsEXT :: VkConditionalRenderingFlagBitsEXT;
}

#if OS == .LINUX {
    VkConditionalRenderingFlagsEXT :: VkConditionalRenderingFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkCullModeFlags :: VkCullModeFlagBits;
}

#if OS == .LINUX {
    VkCullModeFlags :: VkCullModeFlagBits;
}

#if OS == .WINDOWS {
    VkDebugReportCallbackEXT :: *VkDebugReportCallbackEXT_T;
}

#if OS == .LINUX {
    VkDebugReportCallbackEXT :: *VkDebugReportCallbackEXT_T;
}

#if OS == .WINDOWS {
    VkDebugReportFlagsEXT :: VkDebugReportFlagBitsEXT;
}

#if OS == .LINUX {
    VkDebugReportFlagsEXT :: VkDebugReportFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkDebugUtilsMessageSeverityFlagsEXT :: VkDebugUtilsMessageSeverityFlagBitsEXT;
}

#if OS == .LINUX {
    VkDebugUtilsMessageSeverityFlagsEXT :: VkDebugUtilsMessageSeverityFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkDebugUtilsMessageTypeFlagsEXT :: VkDebugUtilsMessageTypeFlagBitsEXT;
}

#if OS == .LINUX {
    VkDebugUtilsMessageTypeFlagsEXT :: VkDebugUtilsMessageTypeFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerCallbackDataFlagsEXT :: VkFlags;
}

#if OS == .LINUX {
    VkDebugUtilsMessengerCallbackDataFlagsEXT :: VkFlags;
}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerCreateFlagsEXT :: VkFlags;
}

#if OS == .LINUX {
    VkDebugUtilsMessengerCreateFlagsEXT :: VkFlags;
}

#if OS == .WINDOWS {
    VkDebugUtilsMessengerEXT :: *VkDebugUtilsMessengerEXT_T;
}

#if OS == .LINUX {
    VkDebugUtilsMessengerEXT :: *VkDebugUtilsMessengerEXT_T;
}

#if OS == .WINDOWS {
    VkDependencyFlags :: VkDependencyFlagBits;
}

#if OS == .LINUX {
    VkDependencyFlags :: VkDependencyFlagBits;
}

#if OS == .WINDOWS {
    VkDescriptorBindingFlagsEXT :: VkDescriptorBindingFlagBitsEXT;
}

#if OS == .LINUX {
    VkDescriptorBindingFlagsEXT :: VkDescriptorBindingFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkDescriptorPool :: *VkDescriptorPool_T;
}

#if OS == .LINUX {
    VkDescriptorPool :: *VkDescriptorPool_T;
}

#if OS == .WINDOWS {
    VkDescriptorPoolCreateFlags :: VkDescriptorPoolCreateFlagBits;
}

#if OS == .LINUX {
    VkDescriptorPoolCreateFlags :: VkDescriptorPoolCreateFlagBits;
}

#if OS == .WINDOWS {
    VkDescriptorPoolResetFlags :: VkFlags;
}

#if OS == .LINUX {
    VkDescriptorPoolResetFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkDescriptorSet :: *VkDescriptorSet_T;
}

#if OS == .LINUX {
    VkDescriptorSet :: *VkDescriptorSet_T;
}

#if OS == .WINDOWS {
    VkDescriptorSetLayout :: *VkDescriptorSetLayout_T;
}

#if OS == .LINUX {
    VkDescriptorSetLayout :: *VkDescriptorSetLayout_T;
}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutCreateFlags :: VkDescriptorSetLayoutCreateFlagBits;
}

#if OS == .LINUX {
    VkDescriptorSetLayoutCreateFlags :: VkDescriptorSetLayoutCreateFlagBits;
}

#if OS == .WINDOWS {
    VkDescriptorSetLayoutSupportKHR :: VkDescriptorSetLayoutSupport;
}

#if OS == .LINUX {
    VkDescriptorSetLayoutSupportKHR :: VkDescriptorSetLayoutSupport;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplate :: *VkDescriptorUpdateTemplate_T;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplate :: *VkDescriptorUpdateTemplate_T;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateCreateFlagsKHR :: VkDescriptorUpdateTemplateCreateFlags;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateCreateFlagsKHR :: VkDescriptorUpdateTemplateCreateFlags;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateCreateInfoKHR :: VkDescriptorUpdateTemplateCreateInfo;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateCreateInfoKHR :: VkDescriptorUpdateTemplateCreateInfo;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateEntryKHR :: VkDescriptorUpdateTemplateEntry;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateEntryKHR :: VkDescriptorUpdateTemplateEntry;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateKHR :: VkDescriptorUpdateTemplate;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateKHR :: VkDescriptorUpdateTemplate;
}

#if OS == .WINDOWS {
    VkDescriptorUpdateTemplateTypeKHR :: VkDescriptorUpdateTemplateType;
}

#if OS == .LINUX {
    VkDescriptorUpdateTemplateTypeKHR :: VkDescriptorUpdateTemplateType;
}

#if OS == .WINDOWS {
    VkDevice :: *VkDevice_T;
}

#if OS == .LINUX {
    VkDevice :: *VkDevice_T;
}

#if OS == .WINDOWS {
    VkDeviceCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkDeviceCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkDeviceGroupBindSparseInfoKHR :: VkDeviceGroupBindSparseInfo;
}

#if OS == .LINUX {
    VkDeviceGroupBindSparseInfoKHR :: VkDeviceGroupBindSparseInfo;
}

#if OS == .WINDOWS {
    VkDeviceGroupCommandBufferBeginInfoKHR :: VkDeviceGroupCommandBufferBeginInfo;
}

#if OS == .LINUX {
    VkDeviceGroupCommandBufferBeginInfoKHR :: VkDeviceGroupCommandBufferBeginInfo;
}

#if OS == .WINDOWS {
    VkDeviceGroupDeviceCreateInfoKHR :: VkDeviceGroupDeviceCreateInfo;
}

#if OS == .LINUX {
    VkDeviceGroupDeviceCreateInfoKHR :: VkDeviceGroupDeviceCreateInfo;
}

#if OS == .WINDOWS {
    VkDeviceGroupPresentModeFlagsKHR :: VkDeviceGroupPresentModeFlagBitsKHR;
}

#if OS == .LINUX {
    VkDeviceGroupPresentModeFlagsKHR :: VkDeviceGroupPresentModeFlagBitsKHR;
}

#if OS == .WINDOWS {
    VkDeviceGroupRenderPassBeginInfoKHR :: VkDeviceGroupRenderPassBeginInfo;
}

#if OS == .LINUX {
    VkDeviceGroupRenderPassBeginInfoKHR :: VkDeviceGroupRenderPassBeginInfo;
}

#if OS == .WINDOWS {
    VkDeviceGroupSubmitInfoKHR :: VkDeviceGroupSubmitInfo;
}

#if OS == .LINUX {
    VkDeviceGroupSubmitInfoKHR :: VkDeviceGroupSubmitInfo;
}

#if OS == .WINDOWS {
    VkDeviceMemory :: *VkDeviceMemory_T;
}

#if OS == .LINUX {
    VkDeviceMemory :: *VkDeviceMemory_T;
}

#if OS == .WINDOWS {
    VkDeviceQueueCreateFlags :: VkDeviceQueueCreateFlagBits;
}

#if OS == .LINUX {
    VkDeviceQueueCreateFlags :: VkDeviceQueueCreateFlagBits;
}

#if OS == .WINDOWS {
    VkDeviceSize :: u64;
}

#if OS == .LINUX {
    VkDeviceSize :: u64;
}

#if OS == .WINDOWS {
    VkDisplayKHR :: *VkDisplayKHR_T;
}

#if OS == .LINUX {
    VkDisplayKHR :: *VkDisplayKHR_T;
}

#if OS == .WINDOWS {
    VkDisplayModeCreateFlagsKHR :: VkFlags;
}

#if OS == .LINUX {
    VkDisplayModeCreateFlagsKHR :: VkFlags;
}

#if OS == .WINDOWS {
    VkDisplayModeKHR :: *VkDisplayModeKHR_T;
}

#if OS == .LINUX {
    VkDisplayModeKHR :: *VkDisplayModeKHR_T;
}

#if OS == .WINDOWS {
    VkDisplayPlaneAlphaFlagsKHR :: VkDisplayPlaneAlphaFlagBitsKHR;
}

#if OS == .LINUX {
    VkDisplayPlaneAlphaFlagsKHR :: VkDisplayPlaneAlphaFlagBitsKHR;
}

#if OS == .WINDOWS {
    VkDisplaySurfaceCreateFlagsKHR :: VkFlags;
}

#if OS == .LINUX {
    VkDisplaySurfaceCreateFlagsKHR :: VkFlags;
}

#if OS == .WINDOWS {
    VkEvent :: *VkEvent_T;
}

#if OS == .LINUX {
    VkEvent :: *VkEvent_T;
}

#if OS == .WINDOWS {
    VkEventCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkEventCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkExportFenceCreateInfoKHR :: VkExportFenceCreateInfo;
}

#if OS == .LINUX {
    VkExportFenceCreateInfoKHR :: VkExportFenceCreateInfo;
}

#if OS == .WINDOWS {
    VkExportMemoryAllocateInfoKHR :: VkExportMemoryAllocateInfo;
}

#if OS == .LINUX {
    VkExportMemoryAllocateInfoKHR :: VkExportMemoryAllocateInfo;
}

#if OS == .WINDOWS {
    VkExportSemaphoreCreateInfoKHR :: VkExportSemaphoreCreateInfo;
}

#if OS == .LINUX {
    VkExportSemaphoreCreateInfoKHR :: VkExportSemaphoreCreateInfo;
}

#if OS == .WINDOWS {
    VkExternalBufferPropertiesKHR :: VkExternalBufferProperties;
}

#if OS == .LINUX {
    VkExternalBufferPropertiesKHR :: VkExternalBufferProperties;
}

#if OS == .WINDOWS {
    VkExternalFenceFeatureFlagBitsKHR :: VkExternalFenceFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalFenceFeatureFlagBitsKHR :: VkExternalFenceFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalFenceFeatureFlags :: VkExternalFenceFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalFenceFeatureFlags :: VkExternalFenceFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalFenceFeatureFlagsKHR :: VkExternalFenceFeatureFlags;
}

#if OS == .LINUX {
    VkExternalFenceFeatureFlagsKHR :: VkExternalFenceFeatureFlags;
}

#if OS == .WINDOWS {
    VkExternalFenceHandleTypeFlagBitsKHR :: VkExternalFenceHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalFenceHandleTypeFlagBitsKHR :: VkExternalFenceHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalFenceHandleTypeFlags :: VkExternalFenceHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalFenceHandleTypeFlags :: VkExternalFenceHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalFenceHandleTypeFlagsKHR :: VkExternalFenceHandleTypeFlags;
}

#if OS == .LINUX {
    VkExternalFenceHandleTypeFlagsKHR :: VkExternalFenceHandleTypeFlags;
}

#if OS == .WINDOWS {
    VkExternalFencePropertiesKHR :: VkExternalFenceProperties;
}

#if OS == .LINUX {
    VkExternalFencePropertiesKHR :: VkExternalFenceProperties;
}

#if OS == .WINDOWS {
    VkExternalImageFormatPropertiesKHR :: VkExternalImageFormatProperties;
}

#if OS == .LINUX {
    VkExternalImageFormatPropertiesKHR :: VkExternalImageFormatProperties;
}

#if OS == .WINDOWS {
    VkExternalMemoryBufferCreateInfoKHR :: VkExternalMemoryBufferCreateInfo;
}

#if OS == .LINUX {
    VkExternalMemoryBufferCreateInfoKHR :: VkExternalMemoryBufferCreateInfo;
}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlagBitsKHR :: VkExternalMemoryFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlagBitsKHR :: VkExternalMemoryFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlags :: VkExternalMemoryFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlags :: VkExternalMemoryFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlagsKHR :: VkExternalMemoryFeatureFlags;
}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlagsKHR :: VkExternalMemoryFeatureFlags;
}

#if OS == .WINDOWS {
    VkExternalMemoryFeatureFlagsNV :: VkExternalMemoryFeatureFlagBitsNV;
}

#if OS == .LINUX {
    VkExternalMemoryFeatureFlagsNV :: VkExternalMemoryFeatureFlagBitsNV;
}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlagBitsKHR :: VkExternalMemoryHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlagBitsKHR :: VkExternalMemoryHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlags :: VkExternalMemoryHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlags :: VkExternalMemoryHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlagsKHR :: VkExternalMemoryHandleTypeFlags;
}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlagsKHR :: VkExternalMemoryHandleTypeFlags;
}

#if OS == .WINDOWS {
    VkExternalMemoryHandleTypeFlagsNV :: VkExternalMemoryHandleTypeFlagBitsNV;
}

#if OS == .LINUX {
    VkExternalMemoryHandleTypeFlagsNV :: VkExternalMemoryHandleTypeFlagBitsNV;
}

#if OS == .WINDOWS {
    VkExternalMemoryImageCreateInfoKHR :: VkExternalMemoryImageCreateInfo;
}

#if OS == .LINUX {
    VkExternalMemoryImageCreateInfoKHR :: VkExternalMemoryImageCreateInfo;
}

#if OS == .WINDOWS {
    VkExternalMemoryPropertiesKHR :: VkExternalMemoryProperties;
}

#if OS == .LINUX {
    VkExternalMemoryPropertiesKHR :: VkExternalMemoryProperties;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreFeatureFlagBitsKHR :: VkExternalSemaphoreFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalSemaphoreFeatureFlagBitsKHR :: VkExternalSemaphoreFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreFeatureFlags :: VkExternalSemaphoreFeatureFlagBits;
}

#if OS == .LINUX {
    VkExternalSemaphoreFeatureFlags :: VkExternalSemaphoreFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreFeatureFlagsKHR :: VkExternalSemaphoreFeatureFlags;
}

#if OS == .LINUX {
    VkExternalSemaphoreFeatureFlagsKHR :: VkExternalSemaphoreFeatureFlags;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreHandleTypeFlagBitsKHR :: VkExternalSemaphoreHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalSemaphoreHandleTypeFlagBitsKHR :: VkExternalSemaphoreHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreHandleTypeFlags :: VkExternalSemaphoreHandleTypeFlagBits;
}

#if OS == .LINUX {
    VkExternalSemaphoreHandleTypeFlags :: VkExternalSemaphoreHandleTypeFlagBits;
}

#if OS == .WINDOWS {
    VkExternalSemaphoreHandleTypeFlagsKHR :: VkExternalSemaphoreHandleTypeFlags;
}

#if OS == .LINUX {
    VkExternalSemaphoreHandleTypeFlagsKHR :: VkExternalSemaphoreHandleTypeFlags;
}

#if OS == .WINDOWS {
    VkExternalSemaphorePropertiesKHR :: VkExternalSemaphoreProperties;
}

#if OS == .LINUX {
    VkExternalSemaphorePropertiesKHR :: VkExternalSemaphoreProperties;
}

#if OS == .WINDOWS {
    VkFence :: *VkFence_T;
}

#if OS == .LINUX {
    VkFence :: *VkFence_T;
}

#if OS == .WINDOWS {
    VkFenceCreateFlags :: VkFenceCreateFlagBits;
}

#if OS == .LINUX {
    VkFenceCreateFlags :: VkFenceCreateFlagBits;
}

#if OS == .WINDOWS {
    VkFenceImportFlagBitsKHR :: VkFenceImportFlagBits;
}

#if OS == .LINUX {
    VkFenceImportFlagBitsKHR :: VkFenceImportFlagBits;
}

#if OS == .WINDOWS {
    VkFenceImportFlags :: VkFenceImportFlagBits;
}

#if OS == .LINUX {
    VkFenceImportFlags :: VkFenceImportFlagBits;
}

#if OS == .WINDOWS {
    VkFenceImportFlagsKHR :: VkFenceImportFlags;
}

#if OS == .LINUX {
    VkFenceImportFlagsKHR :: VkFenceImportFlags;
}

#if OS == .WINDOWS {
    VkFlags :: u32;
}

#if OS == .LINUX {
    VkFlags :: u32;
}

#if OS == .WINDOWS {
    VkFormatFeatureFlags :: VkFormatFeatureFlagBits;
}

#if OS == .LINUX {
    VkFormatFeatureFlags :: VkFormatFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkFormatProperties2KHR :: VkFormatProperties2;
}

#if OS == .LINUX {
    VkFormatProperties2KHR :: VkFormatProperties2;
}

#if OS == .WINDOWS {
    VkFramebuffer :: *VkFramebuffer_T;
}

#if OS == .LINUX {
    VkFramebuffer :: *VkFramebuffer_T;
}

#if OS == .WINDOWS {
    VkFramebufferCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkFramebufferCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkGeometryFlagsNVX :: VkGeometryFlagBitsNVX;
}

#if OS == .LINUX {
    VkGeometryFlagsNVX :: VkGeometryFlagBitsNVX;
}

#if OS == .WINDOWS {
    VkGeometryInstanceFlagsNVX :: VkGeometryInstanceFlagBitsNVX;
}

#if OS == .LINUX {
    VkGeometryInstanceFlagsNVX :: VkGeometryInstanceFlagBitsNVX;
}

#if OS == .WINDOWS {
    VkImage :: *VkImage_T;
}

#if OS == .LINUX {
    VkImage :: *VkImage_T;
}

#if OS == .WINDOWS {
    VkImageAspectFlags :: VkImageAspectFlagBits;
}

#if OS == .LINUX {
    VkImageAspectFlags :: VkImageAspectFlagBits;
}

#if OS == .WINDOWS {
    VkImageCreateFlags :: VkImageCreateFlagBits;
}

#if OS == .LINUX {
    VkImageCreateFlags :: VkImageCreateFlagBits;
}

#if OS == .WINDOWS {
    VkImageFormatProperties2KHR :: VkImageFormatProperties2;
}

#if OS == .LINUX {
    VkImageFormatProperties2KHR :: VkImageFormatProperties2;
}

#if OS == .WINDOWS {
    VkImageMemoryRequirementsInfo2KHR :: VkImageMemoryRequirementsInfo2;
}

#if OS == .LINUX {
    VkImageMemoryRequirementsInfo2KHR :: VkImageMemoryRequirementsInfo2;
}

#if OS == .WINDOWS {
    VkImagePlaneMemoryRequirementsInfoKHR :: VkImagePlaneMemoryRequirementsInfo;
}

#if OS == .LINUX {
    VkImagePlaneMemoryRequirementsInfoKHR :: VkImagePlaneMemoryRequirementsInfo;
}

#if OS == .WINDOWS {
    VkImageSparseMemoryRequirementsInfo2KHR :: VkImageSparseMemoryRequirementsInfo2;
}

#if OS == .LINUX {
    VkImageSparseMemoryRequirementsInfo2KHR :: VkImageSparseMemoryRequirementsInfo2;
}

#if OS == .WINDOWS {
    VkImageUsageFlags :: VkImageUsageFlagBits;
}

#if OS == .LINUX {
    VkImageUsageFlags :: VkImageUsageFlagBits;
}

#if OS == .WINDOWS {
    VkImageView :: *VkImageView_T;
}

#if OS == .LINUX {
    VkImageView :: *VkImageView_T;
}

#if OS == .WINDOWS {
    VkImageViewCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkImageViewCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkImageViewUsageCreateInfoKHR :: VkImageViewUsageCreateInfo;
}

#if OS == .LINUX {
    VkImageViewUsageCreateInfoKHR :: VkImageViewUsageCreateInfo;
}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutNVX :: *VkIndirectCommandsLayoutNVX_T;
}

#if OS == .LINUX {
    VkIndirectCommandsLayoutNVX :: *VkIndirectCommandsLayoutNVX_T;
}

#if OS == .WINDOWS {
    VkIndirectCommandsLayoutUsageFlagsNVX :: VkIndirectCommandsLayoutUsageFlagBitsNVX;
}

#if OS == .LINUX {
    VkIndirectCommandsLayoutUsageFlagsNVX :: VkIndirectCommandsLayoutUsageFlagBitsNVX;
}

#if OS == .WINDOWS {
    VkInputAttachmentAspectReferenceKHR :: VkInputAttachmentAspectReference;
}

#if OS == .LINUX {
    VkInputAttachmentAspectReferenceKHR :: VkInputAttachmentAspectReference;
}

#if OS == .WINDOWS {
    VkInstance :: *VkInstance_T;
}

#if OS == .LINUX {
    VkInstance :: *VkInstance_T;
}

#if OS == .WINDOWS {
    VkInstanceCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkInstanceCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkMemoryAllocateFlagBitsKHR :: VkMemoryAllocateFlagBits;
}

#if OS == .LINUX {
    VkMemoryAllocateFlagBitsKHR :: VkMemoryAllocateFlagBits;
}

#if OS == .WINDOWS {
    VkMemoryAllocateFlags :: VkMemoryAllocateFlagBits;
}

#if OS == .LINUX {
    VkMemoryAllocateFlags :: VkMemoryAllocateFlagBits;
}

#if OS == .WINDOWS {
    VkMemoryAllocateFlagsInfoKHR :: VkMemoryAllocateFlagsInfo;
}

#if OS == .LINUX {
    VkMemoryAllocateFlagsInfoKHR :: VkMemoryAllocateFlagsInfo;
}

#if OS == .WINDOWS {
    VkMemoryAllocateFlagsKHR :: VkMemoryAllocateFlags;
}

#if OS == .LINUX {
    VkMemoryAllocateFlagsKHR :: VkMemoryAllocateFlags;
}

#if OS == .WINDOWS {
    VkMemoryDedicatedAllocateInfoKHR :: VkMemoryDedicatedAllocateInfo;
}

#if OS == .LINUX {
    VkMemoryDedicatedAllocateInfoKHR :: VkMemoryDedicatedAllocateInfo;
}

#if OS == .WINDOWS {
    VkMemoryDedicatedRequirementsKHR :: VkMemoryDedicatedRequirements;
}

#if OS == .LINUX {
    VkMemoryDedicatedRequirementsKHR :: VkMemoryDedicatedRequirements;
}

#if OS == .WINDOWS {
    VkMemoryHeapFlags :: VkMemoryHeapFlagBits;
}

#if OS == .LINUX {
    VkMemoryHeapFlags :: VkMemoryHeapFlagBits;
}

#if OS == .WINDOWS {
    VkMemoryMapFlags :: VkFlags;
}

#if OS == .LINUX {
    VkMemoryMapFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkMemoryPropertyFlags :: VkMemoryPropertyFlagBits;
}

#if OS == .LINUX {
    VkMemoryPropertyFlags :: VkMemoryPropertyFlagBits;
}

#if OS == .WINDOWS {
    VkMemoryRequirements2KHR :: VkMemoryRequirements2;
}

#if OS == .LINUX {
    VkMemoryRequirements2KHR :: VkMemoryRequirements2;
}

#if OS == .WINDOWS {
    VkObjectEntryUsageFlagsNVX :: VkObjectEntryUsageFlagBitsNVX;
}

#if OS == .LINUX {
    VkObjectEntryUsageFlagsNVX :: VkObjectEntryUsageFlagBitsNVX;
}

#if OS == .WINDOWS {
    VkObjectTableNVX :: *VkObjectTableNVX_T;
}

#if OS == .LINUX {
    VkObjectTableNVX :: *VkObjectTableNVX_T;
}

#if OS == .WINDOWS {
    VkPeerMemoryFeatureFlagBitsKHR :: VkPeerMemoryFeatureFlagBits;
}

#if OS == .LINUX {
    VkPeerMemoryFeatureFlagBitsKHR :: VkPeerMemoryFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkPeerMemoryFeatureFlags :: VkPeerMemoryFeatureFlagBits;
}

#if OS == .LINUX {
    VkPeerMemoryFeatureFlags :: VkPeerMemoryFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkPeerMemoryFeatureFlagsKHR :: VkPeerMemoryFeatureFlags;
}

#if OS == .LINUX {
    VkPeerMemoryFeatureFlagsKHR :: VkPeerMemoryFeatureFlags;
}

#if OS == .WINDOWS {
    VkPhysicalDevice :: *VkPhysicalDevice_T;
}

#if OS == .LINUX {
    VkPhysicalDevice :: *VkPhysicalDevice_T;
}

#if OS == .WINDOWS {
    VkPhysicalDevice16BitStorageFeaturesKHR :: VkPhysicalDevice16BitStorageFeatures;
}

#if OS == .LINUX {
    VkPhysicalDevice16BitStorageFeaturesKHR :: VkPhysicalDevice16BitStorageFeatures;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalBufferInfoKHR :: VkPhysicalDeviceExternalBufferInfo;
}

#if OS == .LINUX {
    VkPhysicalDeviceExternalBufferInfoKHR :: VkPhysicalDeviceExternalBufferInfo;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalFenceInfoKHR :: VkPhysicalDeviceExternalFenceInfo;
}

#if OS == .LINUX {
    VkPhysicalDeviceExternalFenceInfoKHR :: VkPhysicalDeviceExternalFenceInfo;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalImageFormatInfoKHR :: VkPhysicalDeviceExternalImageFormatInfo;
}

#if OS == .LINUX {
    VkPhysicalDeviceExternalImageFormatInfoKHR :: VkPhysicalDeviceExternalImageFormatInfo;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceExternalSemaphoreInfoKHR :: VkPhysicalDeviceExternalSemaphoreInfo;
}

#if OS == .LINUX {
    VkPhysicalDeviceExternalSemaphoreInfoKHR :: VkPhysicalDeviceExternalSemaphoreInfo;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceFeatures2KHR :: VkPhysicalDeviceFeatures2;
}

#if OS == .LINUX {
    VkPhysicalDeviceFeatures2KHR :: VkPhysicalDeviceFeatures2;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceGroupPropertiesKHR :: VkPhysicalDeviceGroupProperties;
}

#if OS == .LINUX {
    VkPhysicalDeviceGroupPropertiesKHR :: VkPhysicalDeviceGroupProperties;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceIDPropertiesKHR :: VkPhysicalDeviceIDProperties;
}

#if OS == .LINUX {
    VkPhysicalDeviceIDPropertiesKHR :: VkPhysicalDeviceIDProperties;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceImageFormatInfo2KHR :: VkPhysicalDeviceImageFormatInfo2;
}

#if OS == .LINUX {
    VkPhysicalDeviceImageFormatInfo2KHR :: VkPhysicalDeviceImageFormatInfo2;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceMaintenance3PropertiesKHR :: VkPhysicalDeviceMaintenance3Properties;
}

#if OS == .LINUX {
    VkPhysicalDeviceMaintenance3PropertiesKHR :: VkPhysicalDeviceMaintenance3Properties;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceMemoryProperties2KHR :: VkPhysicalDeviceMemoryProperties2;
}

#if OS == .LINUX {
    VkPhysicalDeviceMemoryProperties2KHR :: VkPhysicalDeviceMemoryProperties2;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceMultiviewFeaturesKHR :: VkPhysicalDeviceMultiviewFeatures;
}

#if OS == .LINUX {
    VkPhysicalDeviceMultiviewFeaturesKHR :: VkPhysicalDeviceMultiviewFeatures;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceMultiviewPropertiesKHR :: VkPhysicalDeviceMultiviewProperties;
}

#if OS == .LINUX {
    VkPhysicalDeviceMultiviewPropertiesKHR :: VkPhysicalDeviceMultiviewProperties;
}

#if OS == .WINDOWS {
    VkPhysicalDevicePointClippingPropertiesKHR :: VkPhysicalDevicePointClippingProperties;
}

#if OS == .LINUX {
    VkPhysicalDevicePointClippingPropertiesKHR :: VkPhysicalDevicePointClippingProperties;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceProperties2KHR :: VkPhysicalDeviceProperties2;
}

#if OS == .LINUX {
    VkPhysicalDeviceProperties2KHR :: VkPhysicalDeviceProperties2;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceSamplerYcbcrConversionFeaturesKHR :: VkPhysicalDeviceSamplerYcbcrConversionFeatures;
}

#if OS == .LINUX {
    VkPhysicalDeviceSamplerYcbcrConversionFeaturesKHR :: VkPhysicalDeviceSamplerYcbcrConversionFeatures;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceSparseImageFormatInfo2KHR :: VkPhysicalDeviceSparseImageFormatInfo2;
}

#if OS == .LINUX {
    VkPhysicalDeviceSparseImageFormatInfo2KHR :: VkPhysicalDeviceSparseImageFormatInfo2;
}

#if OS == .WINDOWS {
    VkPhysicalDeviceVariablePointerFeaturesKHR :: VkPhysicalDeviceVariablePointerFeatures;
}

#if OS == .LINUX {
    VkPhysicalDeviceVariablePointerFeaturesKHR :: VkPhysicalDeviceVariablePointerFeatures;
}

#if OS == .WINDOWS {
    VkPipeline :: *VkPipeline_T;
}

#if OS == .LINUX {
    VkPipeline :: *VkPipeline_T;
}

#if OS == .WINDOWS {
    VkPipelineCache :: *VkPipelineCache_T;
}

#if OS == .LINUX {
    VkPipelineCache :: *VkPipelineCache_T;
}

#if OS == .WINDOWS {
    VkPipelineCacheCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineCacheCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineColorBlendStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineColorBlendStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineCoverageModulationStateCreateFlagsNV :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineCoverageModulationStateCreateFlagsNV :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineCoverageToColorStateCreateFlagsNV :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineCoverageToColorStateCreateFlagsNV :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineCreateFlags :: VkPipelineCreateFlagBits;
}

#if OS == .LINUX {
    VkPipelineCreateFlags :: VkPipelineCreateFlagBits;
}

#if OS == .WINDOWS {
    VkPipelineDepthStencilStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineDepthStencilStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineDiscardRectangleStateCreateFlagsEXT :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineDiscardRectangleStateCreateFlagsEXT :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineDynamicStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineDynamicStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineInputAssemblyStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineInputAssemblyStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineLayout :: *VkPipelineLayout_T;
}

#if OS == .LINUX {
    VkPipelineLayout :: *VkPipelineLayout_T;
}

#if OS == .WINDOWS {
    VkPipelineLayoutCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineLayoutCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineMultisampleStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineMultisampleStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineRasterizationConservativeStateCreateFlagsEXT :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineRasterizationConservativeStateCreateFlagsEXT :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineRasterizationStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineRasterizationStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineShaderStageCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineShaderStageCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineStageFlags :: VkPipelineStageFlagBits;
}

#if OS == .LINUX {
    VkPipelineStageFlags :: VkPipelineStageFlagBits;
}

#if OS == .WINDOWS {
    VkPipelineTessellationDomainOriginStateCreateInfoKHR :: VkPipelineTessellationDomainOriginStateCreateInfo;
}

#if OS == .LINUX {
    VkPipelineTessellationDomainOriginStateCreateInfoKHR :: VkPipelineTessellationDomainOriginStateCreateInfo;
}

#if OS == .WINDOWS {
    VkPipelineTessellationStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineTessellationStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineVertexInputStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineVertexInputStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineViewportStateCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineViewportStateCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkPipelineViewportSwizzleStateCreateFlagsNV :: VkFlags;
}

#if OS == .LINUX {
    VkPipelineViewportSwizzleStateCreateFlagsNV :: VkFlags;
}

#if OS == .WINDOWS {
    VkPointClippingBehaviorKHR :: VkPointClippingBehavior;
}

#if OS == .LINUX {
    VkPointClippingBehaviorKHR :: VkPointClippingBehavior;
}

#if OS == .WINDOWS {
    VkQueryControlFlags :: VkQueryControlFlagBits;
}

#if OS == .LINUX {
    VkQueryControlFlags :: VkQueryControlFlagBits;
}

#if OS == .WINDOWS {
    VkQueryPipelineStatisticFlags :: VkQueryPipelineStatisticFlagBits;
}

#if OS == .LINUX {
    VkQueryPipelineStatisticFlags :: VkQueryPipelineStatisticFlagBits;
}

#if OS == .WINDOWS {
    VkQueryPool :: *VkQueryPool_T;
}

#if OS == .LINUX {
    VkQueryPool :: *VkQueryPool_T;
}

#if OS == .WINDOWS {
    VkQueryPoolCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkQueryPoolCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkQueryResultFlags :: VkQueryResultFlagBits;
}

#if OS == .LINUX {
    VkQueryResultFlags :: VkQueryResultFlagBits;
}

#if OS == .WINDOWS {
    VkQueue :: *VkQueue_T;
}

#if OS == .LINUX {
    VkQueue :: *VkQueue_T;
}

#if OS == .WINDOWS {
    VkQueueFamilyProperties2KHR :: VkQueueFamilyProperties2;
}

#if OS == .LINUX {
    VkQueueFamilyProperties2KHR :: VkQueueFamilyProperties2;
}

#if OS == .WINDOWS {
    VkQueueFlags :: VkQueueFlagBits;
}

#if OS == .LINUX {
    VkQueueFlags :: VkQueueFlagBits;
}

#if OS == .WINDOWS {
    VkRenderPass :: *VkRenderPass_T;
}

#if OS == .LINUX {
    VkRenderPass :: *VkRenderPass_T;
}

#if OS == .WINDOWS {
    VkRenderPassCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkRenderPassCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkRenderPassInputAttachmentAspectCreateInfoKHR :: VkRenderPassInputAttachmentAspectCreateInfo;
}

#if OS == .LINUX {
    VkRenderPassInputAttachmentAspectCreateInfoKHR :: VkRenderPassInputAttachmentAspectCreateInfo;
}

#if OS == .WINDOWS {
    VkRenderPassMultiviewCreateInfoKHR :: VkRenderPassMultiviewCreateInfo;
}

#if OS == .LINUX {
    VkRenderPassMultiviewCreateInfoKHR :: VkRenderPassMultiviewCreateInfo;
}

#if OS == .WINDOWS {
    VkSampleCountFlags :: VkSampleCountFlagBits;
}

#if OS == .LINUX {
    VkSampleCountFlags :: VkSampleCountFlagBits;
}

#if OS == .WINDOWS {
    VkSampleMask :: u32;
}

#if OS == .LINUX {
    VkSampleMask :: u32;
}

#if OS == .WINDOWS {
    VkSampler :: *VkSampler_T;
}

#if OS == .LINUX {
    VkSampler :: *VkSampler_T;
}

#if OS == .WINDOWS {
    VkSamplerCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkSamplerCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversion :: *VkSamplerYcbcrConversion_T;
}

#if OS == .LINUX {
    VkSamplerYcbcrConversion :: *VkSamplerYcbcrConversion_T;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionCreateInfoKHR :: VkSamplerYcbcrConversionCreateInfo;
}

#if OS == .LINUX {
    VkSamplerYcbcrConversionCreateInfoKHR :: VkSamplerYcbcrConversionCreateInfo;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionImageFormatPropertiesKHR :: VkSamplerYcbcrConversionImageFormatProperties;
}

#if OS == .LINUX {
    VkSamplerYcbcrConversionImageFormatPropertiesKHR :: VkSamplerYcbcrConversionImageFormatProperties;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionInfoKHR :: VkSamplerYcbcrConversionInfo;
}

#if OS == .LINUX {
    VkSamplerYcbcrConversionInfoKHR :: VkSamplerYcbcrConversionInfo;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrConversionKHR :: VkSamplerYcbcrConversion;
}

#if OS == .LINUX {
    VkSamplerYcbcrConversionKHR :: VkSamplerYcbcrConversion;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrModelConversionKHR :: VkSamplerYcbcrModelConversion;
}

#if OS == .LINUX {
    VkSamplerYcbcrModelConversionKHR :: VkSamplerYcbcrModelConversion;
}

#if OS == .WINDOWS {
    VkSamplerYcbcrRangeKHR :: VkSamplerYcbcrRange;
}

#if OS == .LINUX {
    VkSamplerYcbcrRangeKHR :: VkSamplerYcbcrRange;
}

#if OS == .WINDOWS {
    VkSemaphore :: *VkSemaphore_T;
}

#if OS == .LINUX {
    VkSemaphore :: *VkSemaphore_T;
}

#if OS == .WINDOWS {
    VkSemaphoreCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkSemaphoreCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkSemaphoreImportFlagBitsKHR :: VkSemaphoreImportFlagBits;
}

#if OS == .LINUX {
    VkSemaphoreImportFlagBitsKHR :: VkSemaphoreImportFlagBits;
}

#if OS == .WINDOWS {
    VkSemaphoreImportFlags :: VkSemaphoreImportFlagBits;
}

#if OS == .LINUX {
    VkSemaphoreImportFlags :: VkSemaphoreImportFlagBits;
}

#if OS == .WINDOWS {
    VkSemaphoreImportFlagsKHR :: VkSemaphoreImportFlags;
}

#if OS == .LINUX {
    VkSemaphoreImportFlagsKHR :: VkSemaphoreImportFlags;
}

#if OS == .WINDOWS {
    VkShaderModule :: *VkShaderModule_T;
}

#if OS == .LINUX {
    VkShaderModule :: *VkShaderModule_T;
}

#if OS == .WINDOWS {
    VkShaderModuleCreateFlags :: VkFlags;
}

#if OS == .LINUX {
    VkShaderModuleCreateFlags :: VkFlags;
}

#if OS == .WINDOWS {
    VkShaderStageFlags :: VkShaderStageFlagBits;
}

#if OS == .LINUX {
    VkShaderStageFlags :: VkShaderStageFlagBits;
}

#if OS == .WINDOWS {
    VkSparseImageFormatFlags :: VkSparseImageFormatFlagBits;
}

#if OS == .LINUX {
    VkSparseImageFormatFlags :: VkSparseImageFormatFlagBits;
}

#if OS == .WINDOWS {
    VkSparseImageFormatProperties2KHR :: VkSparseImageFormatProperties2;
}

#if OS == .LINUX {
    VkSparseImageFormatProperties2KHR :: VkSparseImageFormatProperties2;
}

#if OS == .WINDOWS {
    VkSparseImageMemoryRequirements2KHR :: VkSparseImageMemoryRequirements2;
}

#if OS == .LINUX {
    VkSparseImageMemoryRequirements2KHR :: VkSparseImageMemoryRequirements2;
}

#if OS == .WINDOWS {
    VkSparseMemoryBindFlags :: VkSparseMemoryBindFlagBits;
}

#if OS == .LINUX {
    VkSparseMemoryBindFlags :: VkSparseMemoryBindFlagBits;
}

#if OS == .WINDOWS {
    VkStencilFaceFlags :: VkStencilFaceFlagBits;
}

#if OS == .LINUX {
    VkStencilFaceFlags :: VkStencilFaceFlagBits;
}

#if OS == .WINDOWS {
    VkSubgroupFeatureFlags :: VkSubgroupFeatureFlagBits;
}

#if OS == .LINUX {
    VkSubgroupFeatureFlags :: VkSubgroupFeatureFlagBits;
}

#if OS == .WINDOWS {
    VkSubpassDescriptionFlags :: VkSubpassDescriptionFlagBits;
}

#if OS == .LINUX {
    VkSubpassDescriptionFlags :: VkSubpassDescriptionFlagBits;
}

#if OS == .WINDOWS {
    VkSurfaceCounterFlagsEXT :: VkSurfaceCounterFlagBitsEXT;
}

#if OS == .LINUX {
    VkSurfaceCounterFlagsEXT :: VkSurfaceCounterFlagBitsEXT;
}

#if OS == .WINDOWS {
    VkSurfaceKHR :: *VkSurfaceKHR_T;
}

#if OS == .LINUX {
    VkSurfaceKHR :: *VkSurfaceKHR_T;
}

#if OS == .WINDOWS {
    VkSurfaceTransformFlagsKHR :: VkSurfaceTransformFlagBitsKHR;
}

#if OS == .LINUX {
    VkSurfaceTransformFlagsKHR :: VkSurfaceTransformFlagBitsKHR;
}

#if OS == .WINDOWS {
    VkSwapchainCreateFlagsKHR :: VkSwapchainCreateFlagBitsKHR;
}

#if OS == .LINUX {
    VkSwapchainCreateFlagsKHR :: VkSwapchainCreateFlagBitsKHR;
}

#if OS == .WINDOWS {
    VkSwapchainKHR :: *VkSwapchainKHR_T;
}

#if OS == .LINUX {
    VkSwapchainKHR :: *VkSwapchainKHR_T;
}

#if OS == .WINDOWS {
    VkTessellationDomainOriginKHR :: VkTessellationDomainOrigin;
}

#if OS == .LINUX {
    VkTessellationDomainOriginKHR :: VkTessellationDomainOrigin;
}

#if OS == .WINDOWS {
    VkValidationCacheCreateFlagsEXT :: VkFlags;
}

#if OS == .LINUX {
    VkValidationCacheCreateFlagsEXT :: VkFlags;
}

#if OS == .WINDOWS {
    VkValidationCacheEXT :: *VkValidationCacheEXT_T;
}

#if OS == .LINUX {
    VkValidationCacheEXT :: *VkValidationCacheEXT_T;
}

VkVersion :: u32;

#if OS == .WINDOWS {
    VkWin32SurfaceCreateFlagsKHR :: VkFlags;
}

// Native storage

VK_TRUE: VkBool32 : 1;

VK_FALSE: VkBool32 : 0;

VK_WHOLE_SIZE: u64 : xx ~0;

VK_ATTACHMENT_UNUSED: u32 : 0xffff_ffff;

VK_QUEUE_FAMILY_IGNORED: u32 : 0xffff_ffff;

VK_REMAINING_ARRAY_LAYERS: u32 : 0xffff_ffff;

VK_REMAINING_MIP_LEVELS: u32 : 0xffff_ffff;

VK_SUBPASS_EXTERNAL: u32 : 0xffff_ffff;

VK_MAX_PHYSICAL_DEVICE_NAME_SIZE: u32 : 256;

VK_UUID_SIZE: u32 : 16;

VK_MAX_MEMORY_TYPES: u32 : 32;

VK_MAX_MEMORY_HEAPS: u32 : 16;

VK_MAX_EXTENSION_NAME_SIZE: u32 : 256;

VK_MAX_DESCRIPTION_SIZE: u32 : 256;

// Native entry points

#if OS == .WINDOWS {
    vkAcquireNextImage2KHR :: (device: VkDevice, pAcquireInfo: *VkAcquireNextImageInfoKHR, pImageIndex: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkAcquireNextImage2KHR :: (device: VkDevice, pAcquireInfo: *VkAcquireNextImageInfoKHR, pImageIndex: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkAcquireNextImageKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, timeout: u64, semaphore: VkSemaphore, fence: VkFence, pImageIndex: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkAcquireNextImageKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, timeout: u64, semaphore: VkSemaphore, fence: VkFence, pImageIndex: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkAllocateCommandBuffers :: (device: VkDevice, pAllocateInfo: *VkCommandBufferAllocateInfo, pCommandBuffers: *VkCommandBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkAllocateCommandBuffers :: (device: VkDevice, pAllocateInfo: *VkCommandBufferAllocateInfo, pCommandBuffers: *VkCommandBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkAllocateDescriptorSets :: (device: VkDevice, pAllocateInfo: *VkDescriptorSetAllocateInfo, pDescriptorSets: *VkDescriptorSet) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkAllocateDescriptorSets :: (device: VkDevice, pAllocateInfo: *VkDescriptorSetAllocateInfo, pDescriptorSets: *VkDescriptorSet) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkAllocateMemory :: (device: VkDevice, pAllocateInfo: *VkMemoryAllocateInfo, pAllocator: *VkAllocationCallbacks, pMemory: *VkDeviceMemory) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkAllocateMemory :: (device: VkDevice, pAllocateInfo: *VkMemoryAllocateInfo, pAllocator: *VkAllocationCallbacks, pMemory: *VkDeviceMemory) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkBeginCommandBuffer :: (commandBuffer: VkCommandBuffer, pBeginInfo: *VkCommandBufferBeginInfo) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkBeginCommandBuffer :: (commandBuffer: VkCommandBuffer, pBeginInfo: *VkCommandBufferBeginInfo) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkBindBufferMemory :: (device: VkDevice, buffer: VkBuffer, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkBindBufferMemory :: (device: VkDevice, buffer: VkBuffer, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkBindBufferMemory2 :: (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkBindBufferMemory2 :: (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindBufferMemoryInfo) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkBindImageMemory :: (device: VkDevice, image: VkImage, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkBindImageMemory :: (device: VkDevice, image: VkImage, memory: VkDeviceMemory, memoryOffset: VkDeviceSize) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkBindImageMemory2 :: (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkBindImageMemory2 :: (device: VkDevice, bindInfoCount: u32, pBindInfos: *VkBindImageMemoryInfo) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBeginQuery :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32, flags: VkQueryControlFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBeginQuery :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32, flags: VkQueryControlFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBeginRenderPass :: (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, contents: VkSubpassContents) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBeginRenderPass :: (commandBuffer: VkCommandBuffer, pRenderPassBegin: *VkRenderPassBeginInfo, contents: VkSubpassContents) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBindDescriptorSets :: (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, firstSet: u32, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet, dynamicOffsetCount: u32, pDynamicOffsets: *u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBindDescriptorSets :: (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, layout: VkPipelineLayout, firstSet: u32, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet, dynamicOffsetCount: u32, pDynamicOffsets: *u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBindIndexBuffer :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, indexType: VkIndexType) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBindIndexBuffer :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, indexType: VkIndexType) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBindPipeline :: (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, pipeline: VkPipeline) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBindPipeline :: (commandBuffer: VkCommandBuffer, pipelineBindPoint: VkPipelineBindPoint, pipeline: VkPipeline) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBindVertexBuffers :: (commandBuffer: VkCommandBuffer, firstBinding: u32, bindingCount: u32, pBuffers: *VkBuffer, pOffsets: *VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBindVertexBuffers :: (commandBuffer: VkCommandBuffer, firstBinding: u32, bindingCount: u32, pBuffers: *VkBuffer, pOffsets: *VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdBlitImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageBlit, filter: VkFilter) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdBlitImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageBlit, filter: VkFilter) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdClearAttachments :: (commandBuffer: VkCommandBuffer, attachmentCount: u32, pAttachments: *VkClearAttachment, rectCount: u32, pRects: *VkClearRect) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdClearAttachments :: (commandBuffer: VkCommandBuffer, attachmentCount: u32, pAttachments: *VkClearAttachment, rectCount: u32, pRects: *VkClearRect) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdClearColorImage :: (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pColor: *VkClearColorValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdClearColorImage :: (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pColor: *VkClearColorValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdClearDepthStencilImage :: (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pDepthStencil: *VkClearDepthStencilValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdClearDepthStencilImage :: (commandBuffer: VkCommandBuffer, image: VkImage, imageLayout: VkImageLayout, pDepthStencil: *VkClearDepthStencilValue, rangeCount: u32, pRanges: *VkImageSubresourceRange) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdCopyBuffer :: (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferCopy) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdCopyBuffer :: (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferCopy) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdCopyBufferToImage :: (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdCopyBufferToImage :: (commandBuffer: VkCommandBuffer, srcBuffer: VkBuffer, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdCopyImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageCopy) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdCopyImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageCopy) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdCopyImageToBuffer :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdCopyImageToBuffer :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstBuffer: VkBuffer, regionCount: u32, pRegions: *VkBufferImageCopy) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdCopyQueryPoolResults :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, stride: VkDeviceSize, flags: VkQueryResultFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdCopyQueryPoolResults :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, stride: VkDeviceSize, flags: VkQueryResultFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDispatch :: (commandBuffer: VkCommandBuffer, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDispatch :: (commandBuffer: VkCommandBuffer, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDispatchBase :: (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDispatchBase :: (commandBuffer: VkCommandBuffer, baseGroupX: u32, baseGroupY: u32, baseGroupZ: u32, groupCountX: u32, groupCountY: u32, groupCountZ: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDispatchIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDispatchIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDraw :: (commandBuffer: VkCommandBuffer, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDraw :: (commandBuffer: VkCommandBuffer, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDrawIndexed :: (commandBuffer: VkCommandBuffer, indexCount: u32, instanceCount: u32, firstIndex: u32, vertexOffset: s32, firstInstance: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDrawIndexed :: (commandBuffer: VkCommandBuffer, indexCount: u32, instanceCount: u32, firstIndex: u32, vertexOffset: s32, firstInstance: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDrawIndexedIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDrawIndexedIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdDrawIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdDrawIndirect :: (commandBuffer: VkCommandBuffer, buffer: VkBuffer, offset: VkDeviceSize, drawCount: u32, stride: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdEndQuery :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdEndQuery :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, query: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdEndRenderPass :: (commandBuffer: VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdEndRenderPass :: (commandBuffer: VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdExecuteCommands :: (commandBuffer: VkCommandBuffer, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdExecuteCommands :: (commandBuffer: VkCommandBuffer, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdFillBuffer :: (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, size: VkDeviceSize, data: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdFillBuffer :: (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, size: VkDeviceSize, data: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdNextSubpass :: (commandBuffer: VkCommandBuffer, contents: VkSubpassContents) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdNextSubpass :: (commandBuffer: VkCommandBuffer, contents: VkSubpassContents) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdPipelineBarrier :: (commandBuffer: VkCommandBuffer, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, dependencyFlags: VkDependencyFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdPipelineBarrier :: (commandBuffer: VkCommandBuffer, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, dependencyFlags: VkDependencyFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdPushConstants :: (commandBuffer: VkCommandBuffer, layout: VkPipelineLayout, stageFlags: VkShaderStageFlags, offset: u32, size: u32, pValues: *void) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdPushConstants :: (commandBuffer: VkCommandBuffer, layout: VkPipelineLayout, stageFlags: VkShaderStageFlags, offset: u32, size: u32, pValues: *void) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdResetEvent :: (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdResetEvent :: (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdResetQueryPool :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdResetQueryPool :: (commandBuffer: VkCommandBuffer, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdResolveImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageResolve) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdResolveImage :: (commandBuffer: VkCommandBuffer, srcImage: VkImage, srcImageLayout: VkImageLayout, dstImage: VkImage, dstImageLayout: VkImageLayout, regionCount: u32, pRegions: *VkImageResolve) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetBlendConstants :: (commandBuffer: VkCommandBuffer, blendConstants: *[4] float) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetBlendConstants :: (commandBuffer: VkCommandBuffer, blendConstants: *[4] float) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetDepthBias :: (commandBuffer: VkCommandBuffer, depthBiasConstantFactor: float, depthBiasClamp: float, depthBiasSlopeFactor: float) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetDepthBias :: (commandBuffer: VkCommandBuffer, depthBiasConstantFactor: float, depthBiasClamp: float, depthBiasSlopeFactor: float) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetDepthBounds :: (commandBuffer: VkCommandBuffer, minDepthBounds: float, maxDepthBounds: float) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetDepthBounds :: (commandBuffer: VkCommandBuffer, minDepthBounds: float, maxDepthBounds: float) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetDeviceMask :: (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetDeviceMask :: (commandBuffer: VkCommandBuffer, deviceMask: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetEvent :: (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetEvent :: (commandBuffer: VkCommandBuffer, event: VkEvent, stageMask: VkPipelineStageFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetLineWidth :: (commandBuffer: VkCommandBuffer, lineWidth: float) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetLineWidth :: (commandBuffer: VkCommandBuffer, lineWidth: float) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetScissor :: (commandBuffer: VkCommandBuffer, firstScissor: u32, scissorCount: u32, pScissors: *VkRect2D) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetScissor :: (commandBuffer: VkCommandBuffer, firstScissor: u32, scissorCount: u32, pScissors: *VkRect2D) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetStencilCompareMask :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, compareMask: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetStencilCompareMask :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, compareMask: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetStencilReference :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, reference: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetStencilReference :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, reference: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetStencilWriteMask :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, writeMask: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetStencilWriteMask :: (commandBuffer: VkCommandBuffer, faceMask: VkStencilFaceFlags, writeMask: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdSetViewport :: (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewports: *VkViewport) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdSetViewport :: (commandBuffer: VkCommandBuffer, firstViewport: u32, viewportCount: u32, pViewports: *VkViewport) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdUpdateBuffer :: (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, dataSize: VkDeviceSize, pData: *void) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdUpdateBuffer :: (commandBuffer: VkCommandBuffer, dstBuffer: VkBuffer, dstOffset: VkDeviceSize, dataSize: VkDeviceSize, pData: *void) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdWaitEvents :: (commandBuffer: VkCommandBuffer, eventCount: u32, pEvents: *VkEvent, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdWaitEvents :: (commandBuffer: VkCommandBuffer, eventCount: u32, pEvents: *VkEvent, srcStageMask: VkPipelineStageFlags, dstStageMask: VkPipelineStageFlags, memoryBarrierCount: u32, pMemoryBarriers: *VkMemoryBarrier, bufferMemoryBarrierCount: u32, pBufferMemoryBarriers: *VkBufferMemoryBarrier, imageMemoryBarrierCount: u32, pImageMemoryBarriers: *VkImageMemoryBarrier) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCmdWriteTimestamp :: (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, queryPool: VkQueryPool, query: u32) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkCmdWriteTimestamp :: (commandBuffer: VkCommandBuffer, pipelineStage: VkPipelineStageFlagBits, queryPool: VkQueryPool, query: u32) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateBuffer :: (device: VkDevice, pCreateInfo: *VkBufferCreateInfo, pAllocator: *VkAllocationCallbacks, pBuffer: *VkBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateBuffer :: (device: VkDevice, pCreateInfo: *VkBufferCreateInfo, pAllocator: *VkAllocationCallbacks, pBuffer: *VkBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateBufferView :: (device: VkDevice, pCreateInfo: *VkBufferViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkBufferView) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateBufferView :: (device: VkDevice, pCreateInfo: *VkBufferViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkBufferView) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateCommandPool :: (device: VkDevice, pCreateInfo: *VkCommandPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pCommandPool: *VkCommandPool) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateCommandPool :: (device: VkDevice, pCreateInfo: *VkCommandPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pCommandPool: *VkCommandPool) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateComputePipelines :: (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkComputePipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateComputePipelines :: (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkComputePipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDescriptorPool :: (device: VkDevice, pCreateInfo: *VkDescriptorPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorPool: *VkDescriptorPool) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDescriptorPool :: (device: VkDevice, pCreateInfo: *VkDescriptorPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorPool: *VkDescriptorPool) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDescriptorSetLayout :: (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pSetLayout: *VkDescriptorSetLayout) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDescriptorSetLayout :: (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pSetLayout: *VkDescriptorSetLayout) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDescriptorUpdateTemplate :: (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDescriptorUpdateTemplate :: (device: VkDevice, pCreateInfo: *VkDescriptorUpdateTemplateCreateInfo, pAllocator: *VkAllocationCallbacks, pDescriptorUpdateTemplate: *VkDescriptorUpdateTemplate) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDevice :: (physicalDevice: VkPhysicalDevice, pCreateInfo: *VkDeviceCreateInfo, pAllocator: *VkAllocationCallbacks, pDevice: *VkDevice) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDevice :: (physicalDevice: VkPhysicalDevice, pCreateInfo: *VkDeviceCreateInfo, pAllocator: *VkAllocationCallbacks, pDevice: *VkDevice) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDisplayModeKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pCreateInfo: *VkDisplayModeCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pMode: *VkDisplayModeKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDisplayModeKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pCreateInfo: *VkDisplayModeCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pMode: *VkDisplayModeKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateDisplayPlaneSurfaceKHR :: (instance: VkInstance, pCreateInfo: *VkDisplaySurfaceCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSurface: *VkSurfaceKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateDisplayPlaneSurfaceKHR :: (instance: VkInstance, pCreateInfo: *VkDisplaySurfaceCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSurface: *VkSurfaceKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateEvent :: (device: VkDevice, pCreateInfo: *VkEventCreateInfo, pAllocator: *VkAllocationCallbacks, pEvent: *VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateEvent :: (device: VkDevice, pCreateInfo: *VkEventCreateInfo, pAllocator: *VkAllocationCallbacks, pEvent: *VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateFence :: (device: VkDevice, pCreateInfo: *VkFenceCreateInfo, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateFence :: (device: VkDevice, pCreateInfo: *VkFenceCreateInfo, pAllocator: *VkAllocationCallbacks, pFence: *VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateFramebuffer :: (device: VkDevice, pCreateInfo: *VkFramebufferCreateInfo, pAllocator: *VkAllocationCallbacks, pFramebuffer: *VkFramebuffer) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateFramebuffer :: (device: VkDevice, pCreateInfo: *VkFramebufferCreateInfo, pAllocator: *VkAllocationCallbacks, pFramebuffer: *VkFramebuffer) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateGraphicsPipelines :: (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkGraphicsPipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateGraphicsPipelines :: (device: VkDevice, pipelineCache: VkPipelineCache, createInfoCount: u32, pCreateInfos: *VkGraphicsPipelineCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelines: *VkPipeline) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateImage :: (device: VkDevice, pCreateInfo: *VkImageCreateInfo, pAllocator: *VkAllocationCallbacks, pImage: *VkImage) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateImage :: (device: VkDevice, pCreateInfo: *VkImageCreateInfo, pAllocator: *VkAllocationCallbacks, pImage: *VkImage) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateImageView :: (device: VkDevice, pCreateInfo: *VkImageViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkImageView) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateImageView :: (device: VkDevice, pCreateInfo: *VkImageViewCreateInfo, pAllocator: *VkAllocationCallbacks, pView: *VkImageView) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateInstance :: (pCreateInfo: *VkInstanceCreateInfo, pAllocator: *VkAllocationCallbacks, pInstance: *VkInstance) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateInstance :: (pCreateInfo: *VkInstanceCreateInfo, pAllocator: *VkAllocationCallbacks, pInstance: *VkInstance) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreatePipelineCache :: (device: VkDevice, pCreateInfo: *VkPipelineCacheCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineCache: *VkPipelineCache) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreatePipelineCache :: (device: VkDevice, pCreateInfo: *VkPipelineCacheCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineCache: *VkPipelineCache) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreatePipelineLayout :: (device: VkDevice, pCreateInfo: *VkPipelineLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineLayout: *VkPipelineLayout) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreatePipelineLayout :: (device: VkDevice, pCreateInfo: *VkPipelineLayoutCreateInfo, pAllocator: *VkAllocationCallbacks, pPipelineLayout: *VkPipelineLayout) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateQueryPool :: (device: VkDevice, pCreateInfo: *VkQueryPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pQueryPool: *VkQueryPool) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateQueryPool :: (device: VkDevice, pCreateInfo: *VkQueryPoolCreateInfo, pAllocator: *VkAllocationCallbacks, pQueryPool: *VkQueryPool) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateRenderPass :: (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateRenderPass :: (device: VkDevice, pCreateInfo: *VkRenderPassCreateInfo, pAllocator: *VkAllocationCallbacks, pRenderPass: *VkRenderPass) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateSampler :: (device: VkDevice, pCreateInfo: *VkSamplerCreateInfo, pAllocator: *VkAllocationCallbacks, pSampler: *VkSampler) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateSampler :: (device: VkDevice, pCreateInfo: *VkSamplerCreateInfo, pAllocator: *VkAllocationCallbacks, pSampler: *VkSampler) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateSamplerYcbcrConversion :: (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateSamplerYcbcrConversion :: (device: VkDevice, pCreateInfo: *VkSamplerYcbcrConversionCreateInfo, pAllocator: *VkAllocationCallbacks, pYcbcrConversion: *VkSamplerYcbcrConversion) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateSemaphore :: (device: VkDevice, pCreateInfo: *VkSemaphoreCreateInfo, pAllocator: *VkAllocationCallbacks, pSemaphore: *VkSemaphore) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateSemaphore :: (device: VkDevice, pCreateInfo: *VkSemaphoreCreateInfo, pAllocator: *VkAllocationCallbacks, pSemaphore: *VkSemaphore) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateShaderModule :: (device: VkDevice, pCreateInfo: *VkShaderModuleCreateInfo, pAllocator: *VkAllocationCallbacks, pShaderModule: *VkShaderModule) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateShaderModule :: (device: VkDevice, pCreateInfo: *VkShaderModuleCreateInfo, pAllocator: *VkAllocationCallbacks, pShaderModule: *VkShaderModule) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateSharedSwapchainsKHR :: (device: VkDevice, swapchainCount: u32, pCreateInfos: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchains: *VkSwapchainKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateSharedSwapchainsKHR :: (device: VkDevice, swapchainCount: u32, pCreateInfos: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchains: *VkSwapchainKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateSwapchainKHR :: (device: VkDevice, pCreateInfo: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchain: *VkSwapchainKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkCreateSwapchainKHR :: (device: VkDevice, pCreateInfo: *VkSwapchainCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSwapchain: *VkSwapchainKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkCreateWin32SurfaceKHR :: (instance: VkInstance, pCreateInfo: *VkWin32SurfaceCreateInfoKHR, pAllocator: *VkAllocationCallbacks, pSurface: *VkSurfaceKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyBuffer :: (device: VkDevice, buffer: VkBuffer, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyBuffer :: (device: VkDevice, buffer: VkBuffer, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyBufferView :: (device: VkDevice, bufferView: VkBufferView, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyBufferView :: (device: VkDevice, bufferView: VkBufferView, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyCommandPool :: (device: VkDevice, commandPool: VkCommandPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyCommandPool :: (device: VkDevice, commandPool: VkCommandPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyDescriptorPool :: (device: VkDevice, descriptorPool: VkDescriptorPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyDescriptorPool :: (device: VkDevice, descriptorPool: VkDescriptorPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyDescriptorSetLayout :: (device: VkDevice, descriptorSetLayout: VkDescriptorSetLayout, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyDescriptorSetLayout :: (device: VkDevice, descriptorSetLayout: VkDescriptorSetLayout, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyDescriptorUpdateTemplate :: (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyDescriptorUpdateTemplate :: (device: VkDevice, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyDevice :: (device: VkDevice, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyDevice :: (device: VkDevice, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyEvent :: (device: VkDevice, event: VkEvent, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyEvent :: (device: VkDevice, event: VkEvent, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyFence :: (device: VkDevice, fence: VkFence, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyFence :: (device: VkDevice, fence: VkFence, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyFramebuffer :: (device: VkDevice, framebuffer: VkFramebuffer, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyFramebuffer :: (device: VkDevice, framebuffer: VkFramebuffer, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyImage :: (device: VkDevice, image: VkImage, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyImage :: (device: VkDevice, image: VkImage, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyImageView :: (device: VkDevice, imageView: VkImageView, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyImageView :: (device: VkDevice, imageView: VkImageView, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyInstance :: (instance: VkInstance, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyInstance :: (instance: VkInstance, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyPipeline :: (device: VkDevice, pipeline: VkPipeline, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyPipeline :: (device: VkDevice, pipeline: VkPipeline, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyPipelineCache :: (device: VkDevice, pipelineCache: VkPipelineCache, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyPipelineCache :: (device: VkDevice, pipelineCache: VkPipelineCache, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyPipelineLayout :: (device: VkDevice, pipelineLayout: VkPipelineLayout, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyPipelineLayout :: (device: VkDevice, pipelineLayout: VkPipelineLayout, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyQueryPool :: (device: VkDevice, queryPool: VkQueryPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyQueryPool :: (device: VkDevice, queryPool: VkQueryPool, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyRenderPass :: (device: VkDevice, renderPass: VkRenderPass, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyRenderPass :: (device: VkDevice, renderPass: VkRenderPass, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroySampler :: (device: VkDevice, sampler: VkSampler, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroySampler :: (device: VkDevice, sampler: VkSampler, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroySamplerYcbcrConversion :: (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroySamplerYcbcrConversion :: (device: VkDevice, ycbcrConversion: VkSamplerYcbcrConversion, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroySemaphore :: (device: VkDevice, semaphore: VkSemaphore, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroySemaphore :: (device: VkDevice, semaphore: VkSemaphore, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroyShaderModule :: (device: VkDevice, shaderModule: VkShaderModule, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroyShaderModule :: (device: VkDevice, shaderModule: VkShaderModule, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroySurfaceKHR :: (instance: VkInstance, surface: VkSurfaceKHR, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroySurfaceKHR :: (instance: VkInstance, surface: VkSurfaceKHR, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDestroySwapchainKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkDestroySwapchainKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkDeviceWaitIdle :: (device: VkDevice) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkDeviceWaitIdle :: (device: VkDevice) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEndCommandBuffer :: (commandBuffer: VkCommandBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEndCommandBuffer :: (commandBuffer: VkCommandBuffer) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumerateDeviceExtensionProperties :: (physicalDevice: VkPhysicalDevice, pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumerateDeviceExtensionProperties :: (physicalDevice: VkPhysicalDevice, pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumerateDeviceLayerProperties :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumerateDeviceLayerProperties :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumerateInstanceExtensionProperties :: (pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumerateInstanceExtensionProperties :: (pLayerName: *u8, pPropertyCount: *u32, pProperties: *VkExtensionProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumerateInstanceLayerProperties :: (pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumerateInstanceLayerProperties :: (pPropertyCount: *u32, pProperties: *VkLayerProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumerateInstanceVersion :: (pApiVersion: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumerateInstanceVersion :: (pApiVersion: *u32) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumeratePhysicalDeviceGroups :: (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumeratePhysicalDeviceGroups :: (instance: VkInstance, pPhysicalDeviceGroupCount: *u32, pPhysicalDeviceGroupProperties: *VkPhysicalDeviceGroupProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkEnumeratePhysicalDevices :: (instance: VkInstance, pPhysicalDeviceCount: *u32, pPhysicalDevices: *VkPhysicalDevice) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkEnumeratePhysicalDevices :: (instance: VkInstance, pPhysicalDeviceCount: *u32, pPhysicalDevices: *VkPhysicalDevice) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkFlushMappedMemoryRanges :: (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkFlushMappedMemoryRanges :: (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkFreeCommandBuffers :: (device: VkDevice, commandPool: VkCommandPool, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkFreeCommandBuffers :: (device: VkDevice, commandPool: VkCommandPool, commandBufferCount: u32, pCommandBuffers: *VkCommandBuffer) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkFreeDescriptorSets :: (device: VkDevice, descriptorPool: VkDescriptorPool, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkFreeDescriptorSets :: (device: VkDevice, descriptorPool: VkDescriptorPool, descriptorSetCount: u32, pDescriptorSets: *VkDescriptorSet) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkFreeMemory :: (device: VkDevice, memory: VkDeviceMemory, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkFreeMemory :: (device: VkDevice, memory: VkDeviceMemory, pAllocator: *VkAllocationCallbacks) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetBufferMemoryRequirements :: (device: VkDevice, buffer: VkBuffer, pMemoryRequirements: *VkMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetBufferMemoryRequirements :: (device: VkDevice, buffer: VkBuffer, pMemoryRequirements: *VkMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetBufferMemoryRequirements2 :: (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetBufferMemoryRequirements2 :: (device: VkDevice, pInfo: *VkBufferMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDescriptorSetLayoutSupport :: (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDescriptorSetLayoutSupport :: (device: VkDevice, pCreateInfo: *VkDescriptorSetLayoutCreateInfo, pSupport: *VkDescriptorSetLayoutSupport) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceGroupPeerMemoryFeatures :: (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceGroupPeerMemoryFeatures :: (device: VkDevice, heapIndex: u32, localDeviceIndex: u32, remoteDeviceIndex: u32, pPeerMemoryFeatures: *VkPeerMemoryFeatureFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceGroupPresentCapabilitiesKHR :: (device: VkDevice, pDeviceGroupPresentCapabilities: *VkDeviceGroupPresentCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceGroupPresentCapabilitiesKHR :: (device: VkDevice, pDeviceGroupPresentCapabilities: *VkDeviceGroupPresentCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceGroupSurfacePresentModesKHR :: (device: VkDevice, surface: VkSurfaceKHR, pModes: *VkDeviceGroupPresentModeFlagsKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceGroupSurfacePresentModesKHR :: (device: VkDevice, surface: VkSurfaceKHR, pModes: *VkDeviceGroupPresentModeFlagsKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceMemoryCommitment :: (device: VkDevice, memory: VkDeviceMemory, pCommittedMemoryInBytes: *VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceMemoryCommitment :: (device: VkDevice, memory: VkDeviceMemory, pCommittedMemoryInBytes: *VkDeviceSize) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceProcAddr :: (device: VkDevice, pName: *u8) -> PFN_vkVoidFunction #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceProcAddr :: (device: VkDevice, pName: *u8) -> PFN_vkVoidFunction #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceQueue :: (device: VkDevice, queueFamilyIndex: u32, queueIndex: u32, pQueue: *VkQueue) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceQueue :: (device: VkDevice, queueFamilyIndex: u32, queueIndex: u32, pQueue: *VkQueue) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDeviceQueue2 :: (device: VkDevice, pQueueInfo: *VkDeviceQueueInfo2, pQueue: *VkQueue) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDeviceQueue2 :: (device: VkDevice, pQueueInfo: *VkDeviceQueueInfo2, pQueue: *VkQueue) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDisplayModeProperties2KHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModeProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDisplayModeProperties2KHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModeProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDisplayModePropertiesKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModePropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDisplayModePropertiesKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR, pPropertyCount: *u32, pProperties: *VkDisplayModePropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDisplayPlaneCapabilities2KHR :: (physicalDevice: VkPhysicalDevice, pDisplayPlaneInfo: *VkDisplayPlaneInfo2KHR, pCapabilities: *VkDisplayPlaneCapabilities2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDisplayPlaneCapabilities2KHR :: (physicalDevice: VkPhysicalDevice, pDisplayPlaneInfo: *VkDisplayPlaneInfo2KHR, pCapabilities: *VkDisplayPlaneCapabilities2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDisplayPlaneCapabilitiesKHR :: (physicalDevice: VkPhysicalDevice, mode: VkDisplayModeKHR, planeIndex: u32, pCapabilities: *VkDisplayPlaneCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDisplayPlaneCapabilitiesKHR :: (physicalDevice: VkPhysicalDevice, mode: VkDisplayModeKHR, planeIndex: u32, pCapabilities: *VkDisplayPlaneCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetDisplayPlaneSupportedDisplaysKHR :: (physicalDevice: VkPhysicalDevice, planeIndex: u32, pDisplayCount: *u32, pDisplays: *VkDisplayKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetDisplayPlaneSupportedDisplaysKHR :: (physicalDevice: VkPhysicalDevice, planeIndex: u32, pDisplayCount: *u32, pDisplays: *VkDisplayKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetEventStatus :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetEventStatus :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetFenceStatus :: (device: VkDevice, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetFenceStatus :: (device: VkDevice, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetImageMemoryRequirements :: (device: VkDevice, image: VkImage, pMemoryRequirements: *VkMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetImageMemoryRequirements :: (device: VkDevice, image: VkImage, pMemoryRequirements: *VkMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetImageMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetImageMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageMemoryRequirementsInfo2, pMemoryRequirements: *VkMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetImageSparseMemoryRequirements :: (device: VkDevice, image: VkImage, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetImageSparseMemoryRequirements :: (device: VkDevice, image: VkImage, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetImageSparseMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetImageSparseMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2, pSparseMemoryRequirementCount: *u32, pSparseMemoryRequirements: *VkSparseImageMemoryRequirements2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetImageSubresourceLayout :: (device: VkDevice, image: VkImage, pSubresource: *VkImageSubresource, pLayout: *VkSubresourceLayout) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetImageSubresourceLayout :: (device: VkDevice, image: VkImage, pSubresource: *VkImageSubresource, pLayout: *VkSubresourceLayout) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetInstanceProcAddr :: (instance: VkInstance, pName: *u8) -> PFN_vkVoidFunction #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetInstanceProcAddr :: (instance: VkInstance, pName: *u8) -> PFN_vkVoidFunction #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlaneProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlaneProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlanePropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPlanePropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayProperties2KHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayProperties2KHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayProperties2KHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPropertiesKHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPropertiesKHR :: (physicalDevice: VkPhysicalDevice, pPropertyCount: *u32, pProperties: *VkDisplayPropertiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceExternalBufferProperties :: (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceExternalBufferProperties :: (physicalDevice: VkPhysicalDevice, pExternalBufferInfo: *VkPhysicalDeviceExternalBufferInfo, pExternalBufferProperties: *VkExternalBufferProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceExternalFenceProperties :: (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceExternalFenceProperties :: (physicalDevice: VkPhysicalDevice, pExternalFenceInfo: *VkPhysicalDeviceExternalFenceInfo, pExternalFenceProperties: *VkExternalFenceProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceExternalSemaphoreProperties :: (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceExternalSemaphoreProperties :: (physicalDevice: VkPhysicalDevice, pExternalSemaphoreInfo: *VkPhysicalDeviceExternalSemaphoreInfo, pExternalSemaphoreProperties: *VkExternalSemaphoreProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceFeatures :: (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceFeatures :: (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceFeatures2 :: (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceFeatures2 :: (physicalDevice: VkPhysicalDevice, pFeatures: *VkPhysicalDeviceFeatures2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceFormatProperties2 :: (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceFormatProperties2 :: (physicalDevice: VkPhysicalDevice, format: VkFormat, pFormatProperties: *VkFormatProperties2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, pImageFormatProperties: *VkImageFormatProperties) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, tiling: VkImageTiling, usage: VkImageUsageFlags, flags: VkImageCreateFlags, pImageFormatProperties: *VkImageFormatProperties) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pImageFormatInfo: *VkPhysicalDeviceImageFormatInfo2, pImageFormatProperties: *VkImageFormatProperties2) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceMemoryProperties :: (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceMemoryProperties :: (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceMemoryProperties2 :: (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceMemoryProperties2 :: (physicalDevice: VkPhysicalDevice, pMemoryProperties: *VkPhysicalDeviceMemoryProperties2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDevicePresentRectanglesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pRectCount: *u32, pRects: *VkRect2D) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDevicePresentRectanglesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pRectCount: *u32, pRects: *VkRect2D) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceProperties :: (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceProperties :: (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceProperties2 :: (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceProperties2 :: (physicalDevice: VkPhysicalDevice, pProperties: *VkPhysicalDeviceProperties2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceQueueFamilyProperties :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceQueueFamilyProperties :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceQueueFamilyProperties2 :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceQueueFamilyProperties2 :: (physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *u32, pQueueFamilyProperties: *VkQueueFamilyProperties2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSparseImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSparseImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSparseImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSparseImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2, pPropertyCount: *u32, pProperties: *VkSparseImageFormatProperties2) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *VkSurfaceCapabilitiesKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfaceFormatsKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormatKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfaceFormatsKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceFormatCount: *u32, pSurfaceFormats: *VkSurfaceFormatKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfacePresentModesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *u32, pPresentModes: *VkPresentModeKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfacePresentModesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *u32, pPresentModes: *VkPresentModeKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfaceSupportKHR :: (physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *VkBool32) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfaceSupportKHR :: (physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *VkBool32) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceWin32PresentationSupportKHR :: (physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32) -> VkBool32 #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetPipelineCacheData :: (device: VkDevice, pipelineCache: VkPipelineCache, pDataSize: *u64, pData: *void) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetPipelineCacheData :: (device: VkDevice, pipelineCache: VkPipelineCache, pDataSize: *u64, pData: *void) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetQueryPoolResults :: (device: VkDevice, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dataSize: u64, pData: *void, stride: VkDeviceSize, flags: VkQueryResultFlags) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetQueryPoolResults :: (device: VkDevice, queryPool: VkQueryPool, firstQuery: u32, queryCount: u32, dataSize: u64, pData: *void, stride: VkDeviceSize, flags: VkQueryResultFlags) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetRenderAreaGranularity :: (device: VkDevice, renderPass: VkRenderPass, pGranularity: *VkExtent2D) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetRenderAreaGranularity :: (device: VkDevice, renderPass: VkRenderPass, pGranularity: *VkExtent2D) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkGetSwapchainImagesKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, pSwapchainImageCount: *u32, pSwapchainImages: *VkImage) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkGetSwapchainImagesKHR :: (device: VkDevice, swapchain: VkSwapchainKHR, pSwapchainImageCount: *u32, pSwapchainImages: *VkImage) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkInvalidateMappedMemoryRanges :: (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkInvalidateMappedMemoryRanges :: (device: VkDevice, memoryRangeCount: u32, pMemoryRanges: *VkMappedMemoryRange) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkMapMemory :: (device: VkDevice, memory: VkDeviceMemory, offset: VkDeviceSize, size: VkDeviceSize, flags: VkMemoryMapFlags, ppData: **void) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkMapMemory :: (device: VkDevice, memory: VkDeviceMemory, offset: VkDeviceSize, size: VkDeviceSize, flags: VkMemoryMapFlags, ppData: **void) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkMergePipelineCaches :: (device: VkDevice, dstCache: VkPipelineCache, srcCacheCount: u32, pSrcCaches: *VkPipelineCache) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkMergePipelineCaches :: (device: VkDevice, dstCache: VkPipelineCache, srcCacheCount: u32, pSrcCaches: *VkPipelineCache) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkQueueBindSparse :: (queue: VkQueue, bindInfoCount: u32, pBindInfo: *VkBindSparseInfo, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkQueueBindSparse :: (queue: VkQueue, bindInfoCount: u32, pBindInfo: *VkBindSparseInfo, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkQueuePresentKHR :: (queue: VkQueue, pPresentInfo: *VkPresentInfoKHR) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkQueuePresentKHR :: (queue: VkQueue, pPresentInfo: *VkPresentInfoKHR) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkQueueSubmit :: (queue: VkQueue, submitCount: u32, pSubmits: *VkSubmitInfo, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkQueueSubmit :: (queue: VkQueue, submitCount: u32, pSubmits: *VkSubmitInfo, fence: VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkQueueWaitIdle :: (queue: VkQueue) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkQueueWaitIdle :: (queue: VkQueue) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkResetCommandBuffer :: (commandBuffer: VkCommandBuffer, flags: VkCommandBufferResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkResetCommandBuffer :: (commandBuffer: VkCommandBuffer, flags: VkCommandBufferResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkResetCommandPool :: (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkResetCommandPool :: (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkResetDescriptorPool :: (device: VkDevice, descriptorPool: VkDescriptorPool, flags: VkDescriptorPoolResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkResetDescriptorPool :: (device: VkDevice, descriptorPool: VkDescriptorPool, flags: VkDescriptorPoolResetFlags) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkResetEvent :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkResetEvent :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkResetFences :: (device: VkDevice, fenceCount: u32, pFences: *VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkResetFences :: (device: VkDevice, fenceCount: u32, pFences: *VkFence) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkSetEvent :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkSetEvent :: (device: VkDevice, event: VkEvent) -> VkResult #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkTrimCommandPool :: (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkTrimCommandPool :: (device: VkDevice, commandPool: VkCommandPool, flags: VkCommandPoolTrimFlags) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkUnmapMemory :: (device: VkDevice, memory: VkDeviceMemory) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkUnmapMemory :: (device: VkDevice, memory: VkDeviceMemory) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkUpdateDescriptorSets :: (device: VkDevice, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet, descriptorCopyCount: u32, pDescriptorCopies: *VkCopyDescriptorSet) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkUpdateDescriptorSets :: (device: VkDevice, descriptorWriteCount: u32, pDescriptorWrites: *VkWriteDescriptorSet, descriptorCopyCount: u32, pDescriptorCopies: *VkCopyDescriptorSet) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkUpdateDescriptorSetWithTemplate :: (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #foreign libvulkan;
}

#if OS == .LINUX {
    vkUpdateDescriptorSetWithTemplate :: (device: VkDevice, descriptorSet: VkDescriptorSet, descriptorUpdateTemplate: VkDescriptorUpdateTemplate, pData: *void) -> void #foreign libvulkan;
}

#if OS == .WINDOWS {
    vkWaitForFences :: (device: VkDevice, fenceCount: u32, pFences: *VkFence, waitAll: VkBool32, timeout: u64) -> VkResult #foreign libvulkan;
}

#if OS == .LINUX {
    vkWaitForFences :: (device: VkDevice, fenceCount: u32, pFences: *VkFence, waitAll: VkBool32, timeout: u64) -> VkResult #foreign libvulkan;
}

// Convenience operations

VK_API_VERSION_MAJOR :: (version: u32) -> u32 {
    return (version >> 22) & 0x7f;
}

VK_API_VERSION_MINOR :: (version: u32) -> u32 {
    return (version >> 12) & 0x3ff;
}

VK_API_VERSION_PATCH :: (version: u32) -> u32 {
    return version & 0xfff;
}

VK_API_VERSION_VARIANT :: (version: u32) -> u32 {
    return version >> 29;
}

VK_MAKE_API_VERSION :: (variant: u32, major: u32, minor: u32, patch: u32) -> u32 {
    return (variant << 29) | (major << 22) | (minor << 12) | patch;
}

VK_MAKE_VERSION :: (major: u32, minor: u32, patch: u32) -> u32 {
    return (major << 22) | (minor << 12) | patch;
}

vk_unmake_version :: (vk_version: VkVersion) -> VK_Version {
    return VK_Version.{ VK_VERSION_MAJOR(vk_version), VK_VERSION_MINOR(vk_version), VK_VERSION_PATCH(vk_version) };
}

VK_VERSION_MAJOR :: (version: u32) -> u32 {
    return version >> 22;
}

VK_VERSION_MINOR :: (version: u32) -> u32 {
    return (version >> 12) & 0x3ff;
}

VK_VERSION_PATCH :: (version: u32) -> u32 {
    return version & 0xfff;
}

#if OS == .WINDOWS {
    vkEnumerateDeviceExtensionProperties :: (physicalDevice: VkPhysicalDevice, pLayerName: *u8) -> [] VkExtensionProperties, VkResult {
        count: u32;
        status := vkEnumerateDeviceExtensionProperties(physicalDevice, pLayerName, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkExtensionProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateDeviceExtensionProperties(physicalDevice, pLayerName, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumerateDeviceExtensionProperties :: (physicalDevice: VkPhysicalDevice, pLayerName: *u8) -> [] VkExtensionProperties, VkResult {
        count: u32;
        status := vkEnumerateDeviceExtensionProperties(physicalDevice, pLayerName, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkExtensionProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateDeviceExtensionProperties(physicalDevice, pLayerName, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkEnumerateDeviceLayerProperties :: (physicalDevice: VkPhysicalDevice) -> [] VkLayerProperties, VkResult {
        count: u32;
        status := vkEnumerateDeviceLayerProperties(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkLayerProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateDeviceLayerProperties(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumerateDeviceLayerProperties :: (physicalDevice: VkPhysicalDevice) -> [] VkLayerProperties, VkResult {
        count: u32;
        status := vkEnumerateDeviceLayerProperties(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkLayerProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateDeviceLayerProperties(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkEnumerateInstanceExtensionProperties :: (pLayerName: *u8) -> [] VkExtensionProperties, VkResult {
        count: u32;
        status := vkEnumerateInstanceExtensionProperties(pLayerName, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkExtensionProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateInstanceExtensionProperties(pLayerName, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumerateInstanceExtensionProperties :: (pLayerName: *u8) -> [] VkExtensionProperties, VkResult {
        count: u32;
        status := vkEnumerateInstanceExtensionProperties(pLayerName, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkExtensionProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateInstanceExtensionProperties(pLayerName, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkEnumerateInstanceLayerProperties :: () -> [] VkLayerProperties, VkResult {
        count: u32;
        status := vkEnumerateInstanceLayerProperties(*count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkLayerProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateInstanceLayerProperties(*count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumerateInstanceLayerProperties :: () -> [] VkLayerProperties, VkResult {
        count: u32;
        status := vkEnumerateInstanceLayerProperties(*count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkLayerProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumerateInstanceLayerProperties(*count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkEnumeratePhysicalDeviceGroups :: (instance: VkInstance) -> [] VkPhysicalDeviceGroupProperties, VkResult {
        count: u32;
        status := vkEnumeratePhysicalDeviceGroups(instance, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPhysicalDeviceGroupProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumeratePhysicalDeviceGroups(instance, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumeratePhysicalDeviceGroups :: (instance: VkInstance) -> [] VkPhysicalDeviceGroupProperties, VkResult {
        count: u32;
        status := vkEnumeratePhysicalDeviceGroups(instance, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPhysicalDeviceGroupProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumeratePhysicalDeviceGroups(instance, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumeratePhysicalDeviceGroupsKHR :: (instance: VkInstance) -> [] VkPhysicalDeviceGroupProperties, VkResult {
        count: u32;
        status := binding_vkEnumeratePhysicalDeviceGroupsKHR(instance, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPhysicalDeviceGroupProperties);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = binding_vkEnumeratePhysicalDeviceGroupsKHR(instance, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkEnumeratePhysicalDevices :: (instance: VkInstance) -> [] VkPhysicalDevice, VkResult {
        count: u32;
        status := vkEnumeratePhysicalDevices(instance, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPhysicalDevice);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumeratePhysicalDevices(instance, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkEnumeratePhysicalDevices :: (instance: VkInstance) -> [] VkPhysicalDevice, VkResult {
        count: u32;
        status := vkEnumeratePhysicalDevices(instance, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPhysicalDevice);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkEnumeratePhysicalDevices(instance, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetDisplayModeProperties2KHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> [] VkDisplayModeProperties2KHR, VkResult {
        count: u32;
        status := vkGetDisplayModeProperties2KHR(physicalDevice, display, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayModeProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayModeProperties2KHR(physicalDevice, display, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetDisplayModeProperties2KHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> [] VkDisplayModeProperties2KHR, VkResult {
        count: u32;
        status := vkGetDisplayModeProperties2KHR(physicalDevice, display, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayModeProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayModeProperties2KHR(physicalDevice, display, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetDisplayModePropertiesKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> [] VkDisplayModePropertiesKHR, VkResult {
        count: u32;
        status := vkGetDisplayModePropertiesKHR(physicalDevice, display, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayModePropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayModePropertiesKHR(physicalDevice, display, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetDisplayModePropertiesKHR :: (physicalDevice: VkPhysicalDevice, display: VkDisplayKHR) -> [] VkDisplayModePropertiesKHR, VkResult {
        count: u32;
        status := vkGetDisplayModePropertiesKHR(physicalDevice, display, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayModePropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayModePropertiesKHR(physicalDevice, display, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetDisplayPlaneSupportedDisplaysKHR :: (physicalDevice: VkPhysicalDevice, planeIndex: u32) -> [] VkDisplayKHR, VkResult {
        count: u32;
        status := vkGetDisplayPlaneSupportedDisplaysKHR(physicalDevice, planeIndex, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayPlaneSupportedDisplaysKHR(physicalDevice, planeIndex, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetDisplayPlaneSupportedDisplaysKHR :: (physicalDevice: VkPhysicalDevice, planeIndex: u32) -> [] VkDisplayKHR, VkResult {
        count: u32;
        status := vkGetDisplayPlaneSupportedDisplaysKHR(physicalDevice, planeIndex, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetDisplayPlaneSupportedDisplaysKHR(physicalDevice, planeIndex, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetImageSparseMemoryRequirements :: (device: VkDevice, image: VkImage) -> [] VkSparseImageMemoryRequirements {
        count: u32;

        vkGetImageSparseMemoryRequirements(device, image, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageMemoryRequirements);
        if !base binding_abort();

        vkGetImageSparseMemoryRequirements(device, image, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetImageSparseMemoryRequirements :: (device: VkDevice, image: VkImage) -> [] VkSparseImageMemoryRequirements {
        count: u32;

        vkGetImageSparseMemoryRequirements(device, image, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageMemoryRequirements);
        if !base binding_abort();

        vkGetImageSparseMemoryRequirements(device, image, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetImageSparseMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2) -> [] VkSparseImageMemoryRequirements2 {
        count: u32;

        vkGetImageSparseMemoryRequirements2(device, pInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageMemoryRequirements2);
        if !base binding_abort();

        vkGetImageSparseMemoryRequirements2(device, pInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetImageSparseMemoryRequirements2 :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2) -> [] VkSparseImageMemoryRequirements2 {
        count: u32;

        vkGetImageSparseMemoryRequirements2(device, pInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageMemoryRequirements2);
        if !base binding_abort();

        vkGetImageSparseMemoryRequirements2(device, pInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetImageSparseMemoryRequirements2KHR :: (device: VkDevice, pInfo: *VkImageSparseMemoryRequirementsInfo2) -> [] VkSparseImageMemoryRequirements2 {
        count: u32;

        binding_vkGetImageSparseMemoryRequirements2KHR(device, pInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageMemoryRequirements2);
        if !base binding_abort();

        binding_vkGetImageSparseMemoryRequirements2KHR(device, pInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPastPresentationTimingGOOGLE :: (device: VkDevice, swapchain: VkSwapchainKHR) -> [] VkPastPresentationTimingGOOGLE, VkResult {
        count: u32;
        status := binding_vkGetPastPresentationTimingGOOGLE(device, swapchain, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPastPresentationTimingGOOGLE);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = binding_vkGetPastPresentationTimingGOOGLE(device, swapchain, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPlaneProperties2KHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPlaneProperties2KHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPlaneProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPlaneProperties2KHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPlaneProperties2KHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPlaneProperties2KHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPlaneProperties2KHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPlaneProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPlaneProperties2KHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPlanePropertiesKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPlanePropertiesKHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPlanePropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPlanePropertiesKHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPlanePropertiesKHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPlanePropertiesKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPlanePropertiesKHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPlanePropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPlanePropertiesKHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayProperties2KHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayProperties2KHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayProperties2KHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayProperties2KHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayProperties2KHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayProperties2KHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayProperties2KHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayProperties2KHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayProperties2KHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceDisplayPropertiesKHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPropertiesKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPropertiesKHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPropertiesKHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceDisplayPropertiesKHR :: (physicalDevice: VkPhysicalDevice) -> [] VkDisplayPropertiesKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceDisplayPropertiesKHR(physicalDevice, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkDisplayPropertiesKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceDisplayPropertiesKHR(physicalDevice, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDevicePresentRectanglesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkRect2D, VkResult {
        count: u32;
        status := vkGetPhysicalDevicePresentRectanglesKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkRect2D);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDevicePresentRectanglesKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDevicePresentRectanglesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkRect2D, VkResult {
        count: u32;
        status := vkGetPhysicalDevicePresentRectanglesKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkRect2D);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDevicePresentRectanglesKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceQueueFamilyProperties :: (physicalDevice: VkPhysicalDevice) -> [] VkQueueFamilyProperties {
        count: u32;

        vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkQueueFamilyProperties);
        if !base binding_abort();

        vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceQueueFamilyProperties :: (physicalDevice: VkPhysicalDevice) -> [] VkQueueFamilyProperties {
        count: u32;

        vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkQueueFamilyProperties);
        if !base binding_abort();

        vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceQueueFamilyProperties2 :: (physicalDevice: VkPhysicalDevice) -> [] VkQueueFamilyProperties2 {
        count: u32;

        vkGetPhysicalDeviceQueueFamilyProperties2(physicalDevice, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkQueueFamilyProperties2);
        if !base binding_abort();

        vkGetPhysicalDeviceQueueFamilyProperties2(physicalDevice, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceQueueFamilyProperties2 :: (physicalDevice: VkPhysicalDevice) -> [] VkQueueFamilyProperties2 {
        count: u32;

        vkGetPhysicalDeviceQueueFamilyProperties2(physicalDevice, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkQueueFamilyProperties2);
        if !base binding_abort();

        vkGetPhysicalDeviceQueueFamilyProperties2(physicalDevice, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceQueueFamilyProperties2KHR :: (physicalDevice: VkPhysicalDevice) -> [] VkQueueFamilyProperties2 {
        count: u32;

        binding_vkGetPhysicalDeviceQueueFamilyProperties2KHR(physicalDevice, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkQueueFamilyProperties2);
        if !base binding_abort();

        binding_vkGetPhysicalDeviceQueueFamilyProperties2KHR(physicalDevice, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSparseImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling) -> [] VkSparseImageFormatProperties {
        count: u32;

        vkGetPhysicalDeviceSparseImageFormatProperties(physicalDevice, format, type, samples, usage, tiling, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageFormatProperties);
        if !base binding_abort();

        vkGetPhysicalDeviceSparseImageFormatProperties(physicalDevice, format, type, samples, usage, tiling, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSparseImageFormatProperties :: (physicalDevice: VkPhysicalDevice, format: VkFormat, type: VkImageType, samples: VkSampleCountFlagBits, usage: VkImageUsageFlags, tiling: VkImageTiling) -> [] VkSparseImageFormatProperties {
        count: u32;

        vkGetPhysicalDeviceSparseImageFormatProperties(physicalDevice, format, type, samples, usage, tiling, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageFormatProperties);
        if !base binding_abort();

        vkGetPhysicalDeviceSparseImageFormatProperties(physicalDevice, format, type, samples, usage, tiling, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSparseImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2) -> [] VkSparseImageFormatProperties2 {
        count: u32;

        vkGetPhysicalDeviceSparseImageFormatProperties2(physicalDevice, pFormatInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageFormatProperties2);
        if !base binding_abort();

        vkGetPhysicalDeviceSparseImageFormatProperties2(physicalDevice, pFormatInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSparseImageFormatProperties2 :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2) -> [] VkSparseImageFormatProperties2 {
        count: u32;

        vkGetPhysicalDeviceSparseImageFormatProperties2(physicalDevice, pFormatInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageFormatProperties2);
        if !base binding_abort();

        vkGetPhysicalDeviceSparseImageFormatProperties2(physicalDevice, pFormatInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSparseImageFormatProperties2KHR :: (physicalDevice: VkPhysicalDevice, pFormatInfo: *VkPhysicalDeviceSparseImageFormatInfo2) -> [] VkSparseImageFormatProperties2 {
        count: u32;

        binding_vkGetPhysicalDeviceSparseImageFormatProperties2KHR(physicalDevice, pFormatInfo, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSparseImageFormatProperties2);
        if !base binding_abort();

        binding_vkGetPhysicalDeviceSparseImageFormatProperties2KHR(physicalDevice, pFormatInfo, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfaceFormatsKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkSurfaceFormatKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSurfaceFormatKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfaceFormatsKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkSurfaceFormatKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkSurfaceFormatKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .WINDOWS {
    vkGetPhysicalDeviceSurfacePresentModesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkPresentModeKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPresentModeKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetPhysicalDeviceSurfacePresentModesKHR :: (physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR) -> [] VkPresentModeKHR, VkResult {
        count: u32;
        status := vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice, surface, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkPresentModeKHR);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice, surface, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetQueueCheckpointDataNV :: (queue: VkQueue) -> [] VkCheckpointDataNV {
        count: u32;

        binding_vkGetQueueCheckpointDataNV(queue, *count, null);
        if count == 0 return .[];

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkCheckpointDataNV);
        if !base binding_abort();

        binding_vkGetQueueCheckpointDataNV(queue, *count, values.data);
        values.count = count;

        return values;
    }
}

#if OS == .WINDOWS {
    vkGetSwapchainImagesKHR :: (device: VkDevice, swapchain: VkSwapchainKHR) -> [] VkImage, VkResult {
        count: u32;
        status := vkGetSwapchainImagesKHR(device, swapchain, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkImage);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetSwapchainImagesKHR(device, swapchain, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}

#if OS == .LINUX {
    vkGetSwapchainImagesKHR :: (device: VkDevice, swapchain: VkSwapchainKHR) -> [] VkImage, VkResult {
        count: u32;
        status := vkGetSwapchainImagesKHR(device, swapchain, *count, null);

        if status != .SUCCESS || count == 0 return .[], status;

        values, base := Binding_Arrays.allocate_values(cast(s64) count, VkImage);
        if !base return .[], .ERROR_OUT_OF_HOST_MEMORY;

        status = vkGetSwapchainImagesKHR(device, swapchain, *count, values.data);
        if cast,trunc(s32) status < 0 {
            Binding_Arrays.release_values(base);
            return .[], status;
        }

        values.count = count;

        return values, status;
    }
}
