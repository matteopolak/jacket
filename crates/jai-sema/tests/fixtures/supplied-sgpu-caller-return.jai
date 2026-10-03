return_if_error :: (vk_result: VkResult) #expand {
    if vk_result != .SUCCESS {
        #if DEBUG_ASSERTS {
            debug_break();
        }
        // #todo: convert VkResult to GpuResult
        `return Gpu_Result.FATAL_ERROR_UNKNOWN;
    }
}
