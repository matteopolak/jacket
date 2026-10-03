// Generated from docs/specs/stdlib/modules/macos.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "POSIX";
#import "Objective_C";

Boolean :: u8;
UTF32Char :: u32;
UTF16Char :: u16;
StringPtr :: *u8;
ConstStringPtr :: *u8;
ConstStr255Param :: *u8;
LangCode :: s16;
RegionCode :: s16;
ScriptCode :: s16;
Block :: *Objective_C_Block_Basic;
CoreServices :: #library,system "CoreServices";
corefoundation :: #library,system "corefoundation";
libc :: #library,system "c";
#scope_export

// Types and layouts

#if OS == .MACOS {
    __CFAllocator :: struct {}

}

#if OS == .MACOS {
    __CFArray :: struct {}

}

#if OS == .MACOS {
    __CFAttributedString :: struct {}

}

#if OS == .MACOS {
    __CFBag :: struct {}

}

#if OS == .MACOS {
    __CFBinaryHeap :: struct {}

}

#if OS == .MACOS {
    __CFBitVector :: struct {}

}

#if OS == .MACOS {
    __CFBoolean :: struct {}

}

#if OS == .MACOS {
    __CFBundle :: struct {}

}

#if OS == .MACOS {
    __CFByteOrder :: enum u32 {
        Unknown :: 0;
        LittleEndian :: 1;
        BigEndian :: 2;
        CFByteOrderUnknown :: Unknown;
        CFByteOrderLittleEndian :: LittleEndian;
        CFByteOrderBigEndian :: BigEndian;
    }

}

#if OS == .MACOS {
    __CFCalendar :: struct {}

}

#if OS == .MACOS {
    __CFCharacterSet :: struct {}

}

#if OS == .MACOS {
    __CFData :: struct {}

}

#if OS == .MACOS {
    __CFDate :: struct {}

}

#if OS == .MACOS {
    __CFDateFormatter :: struct {}

}

#if OS == .MACOS {
    __CFDictionary :: struct {}

}

#if OS == .MACOS {
    __CFError :: struct {}

}

#if OS == .MACOS {
    __CFFileDescriptor :: struct {}

}

#if OS == .MACOS {
    __CFLocale :: struct {}

}

#if OS == .MACOS {
    __CFMachPort :: struct {}

}

#if OS == .MACOS {
    __CFMessagePort :: struct {}

}

#if OS == .MACOS {
    __CFNotificationCenter :: struct {}

}

#if OS == .MACOS {
    __CFNull :: struct {}

}

#if OS == .MACOS {
    __CFNumber :: struct {}

}

#if OS == .MACOS {
    __CFNumberFormatter :: struct {}

}

#if OS == .MACOS {
    __CFPlugInInstance :: struct {}

}

#if OS == .MACOS {
    __CFReadStream :: struct {}

}

#if OS == .MACOS {
    __CFRunLoop :: struct {}

}

#if OS == .MACOS {
    __CFRunLoopObserver :: struct {}

}

#if OS == .MACOS {
    __CFRunLoopSource :: struct {}

}

#if OS == .MACOS {
    __CFRunLoopTimer :: struct {}

}

#if OS == .MACOS {
    __CFSet :: struct {}

}

#if OS == .MACOS {
    __CFSocket :: struct {}

}

#if OS == .MACOS {
    __CFString :: struct {}

}

#if OS == .MACOS {
    __CFStringTokenizer :: struct {}

}

#if OS == .MACOS {
    __CFTimeZone :: struct {}

}

#if OS == .MACOS {
    __CFTree :: struct {}

}

#if OS == .MACOS {
    __CFURL :: struct {}

}

#if OS == .MACOS {
    __CFURLEnumerator :: struct {}

}

#if OS == .MACOS {
    __CFUserNotification :: struct {}

}

#if OS == .MACOS {
    __CFUUID :: struct {}

}

#if OS == .MACOS {
    __CFWriteStream :: struct {}

}

#if OS == .MACOS {
    __mbstate_t :: struct {
        __mbstate8: [128] u8;
        #overlay (__mbstate8) _mbstateL: s64;
    }

}

#if OS == .MACOS {
    __Reply__lock_set_create_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        new_lock_set: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__lock_set_destroy_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__mach_ports_lookup_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        init_port_set: mach_msg_ool_ports_descriptor_t;
        NDR: NDR_record_t;
        init_port_setCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Reply__mach_ports_register_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__semaphore_create_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        semaphore: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__semaphore_destroy_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_assign_default_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_assign_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_create_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        child_task: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_generate_corpse_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        corpse_task_port: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_assignment_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        assigned_set: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        dyld_images: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        dyld_imagesCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_emulation_vector_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        emulation_vector: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        vector_start: s32;
        emulation_vectorCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        old_handlers: [32] mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        masksCnt: mach_msg_type_number_t;
        masks: [32] exception_mask_t;
        old_behaviors: [32] exception_behavior_t;
        old_flavors: [32] thread_state_flavor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        voucher: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_special_port_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        special_port: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_get_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        old_stateCnt: mach_msg_type_number_t;
        old_state: [614] u32;
    }

}

#if OS == .MACOS {
    __Reply__task_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        task_info_outCnt: mach_msg_type_number_t;
        task_info_out: [52] s32;
    }

}

#if OS == .MACOS {
    __Reply__task_inspect_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        info_outCnt: mach_msg_type_number_t;
        info_out: [4] s32;
    }

}

#if OS == .MACOS {
    __Reply__task_map_corpse_info_64_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        kcd_addr_begin: mach_vm_address_t #align 4;
        kcd_size: mach_vm_size_t #align 4;
    }

}

#if OS == .MACOS {
    __Reply__task_map_corpse_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        kcd_addr_begin: vm_address_t #align 4;
        kcd_size: u32;
    }

}

#if OS == .MACOS {
    __Reply__task_policy_get_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        policy_infoCnt: mach_msg_type_number_t;
        policy_info: [16] s32;
        get_default: u32;
    }

}

#if OS == .MACOS {
    __Reply__task_policy_set_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_policy_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_purgable_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        stats: task_purgable_info_t #align 4;
    }

}

#if OS == .MACOS {
    __Reply__task_register_dyld_get_process_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        dyld_process_state: dyld_kernel_process_info_t #align 4;
    }

}

#if OS == .MACOS {
    __Reply__task_register_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_register_dyld_set_dyld_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_register_dyld_shared_cache_image_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_resume2_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_resume_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_sample_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_emulation_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_emulation_vector_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_phys_footprint_limit_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
        old_limit: s32;
    }

}

#if OS == .MACOS {
    __Reply__task_set_policy_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_ras_pc_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_special_port_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_set_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_suspend2_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        suspend_token: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_suspend_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_swap_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        old_handlerss: [32] mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        masksCnt: mach_msg_type_number_t;
        masks: [32] exception_mask_t;
        old_behaviors: [32] exception_behavior_t;
        old_flavors: [32] thread_state_flavor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_swap_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        old_voucher: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__task_terminate_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_threads_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        act_list: mach_msg_ool_ports_descriptor_t;
        NDR: NDR_record_t;
        act_listCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Reply__task_unregister_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        RetCode: kern_return_t;
    }

}

#if OS == .MACOS {
    __Reply__task_zone_info_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        names: mach_msg_ool_descriptor_t;
        info: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        namesCnt: mach_msg_type_number_t;
        infoCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Reply__thread_create_running_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        child_act: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Reply__thread_create_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        child_act: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __ReplyUnion__task_subsystem :: struct {
        Reply_task_get_state: __Reply__task_get_state_t; // largest member, moved to the start of the struct/union to support #overlay.;
        #overlay (Reply_task_get_state) Reply_task_create: __Reply__task_create_t;
        #overlay (Reply_task_get_state) Reply_task_terminate: __Reply__task_terminate_t;
        #overlay (Reply_task_get_state) Reply_task_threads: __Reply__task_threads_t;
        #overlay (Reply_task_get_state) Reply_mach_ports_register: __Reply__mach_ports_register_t;
        #overlay (Reply_task_get_state) Reply_mach_ports_lookup: __Reply__mach_ports_lookup_t;
        #overlay (Reply_task_get_state) Reply_task_info: __Reply__task_info_t;
        #overlay (Reply_task_get_state) Reply_task_set_info: __Reply__task_set_info_t;
        #overlay (Reply_task_get_state) Reply_task_suspend: __Reply__task_suspend_t;
        #overlay (Reply_task_get_state) Reply_task_resume: __Reply__task_resume_t;
        #overlay (Reply_task_get_state) Reply_task_get_special_port: __Reply__task_get_special_port_t;
        #overlay (Reply_task_get_state) Reply_task_set_special_port: __Reply__task_set_special_port_t;
        #overlay (Reply_task_get_state) Reply_thread_create: __Reply__thread_create_t;
        #overlay (Reply_task_get_state) Reply_thread_create_running: __Reply__thread_create_running_t;
        #overlay (Reply_task_get_state) Reply_task_set_exception_ports: __Reply__task_set_exception_ports_t;
        #overlay (Reply_task_get_state) Reply_task_get_exception_ports: __Reply__task_get_exception_ports_t;
        #overlay (Reply_task_get_state) Reply_task_swap_exception_ports: __Reply__task_swap_exception_ports_t;
        #overlay (Reply_task_get_state) Reply_lock_set_create: __Reply__lock_set_create_t;
        #overlay (Reply_task_get_state) Reply_lock_set_destroy: __Reply__lock_set_destroy_t;
        #overlay (Reply_task_get_state) Reply_semaphore_create: __Reply__semaphore_create_t;
        #overlay (Reply_task_get_state) Reply_semaphore_destroy: __Reply__semaphore_destroy_t;
        #overlay (Reply_task_get_state) Reply_task_policy_set: __Reply__task_policy_set_t;
        #overlay (Reply_task_get_state) Reply_task_policy_get: __Reply__task_policy_get_t;
        #overlay (Reply_task_get_state) Reply_task_sample: __Reply__task_sample_t;
        #overlay (Reply_task_get_state) Reply_task_policy: __Reply__task_policy_t;
        #overlay (Reply_task_get_state) Reply_task_set_emulation: __Reply__task_set_emulation_t;
        #overlay (Reply_task_get_state) Reply_task_get_emulation_vector: __Reply__task_get_emulation_vector_t;
        #overlay (Reply_task_get_state) Reply_task_set_emulation_vector: __Reply__task_set_emulation_vector_t;
        #overlay (Reply_task_get_state) Reply_task_set_ras_pc: __Reply__task_set_ras_pc_t;
        #overlay (Reply_task_get_state) Reply_task_zone_info: __Reply__task_zone_info_t;
        #overlay (Reply_task_get_state) Reply_task_assign: __Reply__task_assign_t;
        #overlay (Reply_task_get_state) Reply_task_assign_default: __Reply__task_assign_default_t;
        #overlay (Reply_task_get_state) Reply_task_get_assignment: __Reply__task_get_assignment_t;
        #overlay (Reply_task_get_state) Reply_task_set_policy: __Reply__task_set_policy_t;
        #overlay (Reply_task_get_state) Reply_task_set_state: __Reply__task_set_state_t;
        #overlay (Reply_task_get_state) Reply_task_set_phys_footprint_limit: __Reply__task_set_phys_footprint_limit_t;
        #overlay (Reply_task_get_state) Reply_task_suspend2: __Reply__task_suspend2_t;
        #overlay (Reply_task_get_state) Reply_task_resume2: __Reply__task_resume2_t;
        #overlay (Reply_task_get_state) Reply_task_purgable_info: __Reply__task_purgable_info_t;
        #overlay (Reply_task_get_state) Reply_task_get_mach_voucher: __Reply__task_get_mach_voucher_t;
        #overlay (Reply_task_get_state) Reply_task_set_mach_voucher: __Reply__task_set_mach_voucher_t;
        #overlay (Reply_task_get_state) Reply_task_swap_mach_voucher: __Reply__task_swap_mach_voucher_t;
        #overlay (Reply_task_get_state) Reply_task_generate_corpse: __Reply__task_generate_corpse_t;
        #overlay (Reply_task_get_state) Reply_task_map_corpse_info: __Reply__task_map_corpse_info_t;
        #overlay (Reply_task_get_state) Reply_task_register_dyld_image_infos: __Reply__task_register_dyld_image_infos_t;
        #overlay (Reply_task_get_state) Reply_task_unregister_dyld_image_infos: __Reply__task_unregister_dyld_image_infos_t;
        #overlay (Reply_task_get_state) Reply_task_get_dyld_image_infos: __Reply__task_get_dyld_image_infos_t;
        #overlay (Reply_task_get_state) Reply_task_register_dyld_shared_cache_image_info: __Reply__task_register_dyld_shared_cache_image_info_t;
        #overlay (Reply_task_get_state) Reply_task_register_dyld_set_dyld_state: __Reply__task_register_dyld_set_dyld_state_t;
        #overlay (Reply_task_get_state) Reply_task_register_dyld_get_process_state: __Reply__task_register_dyld_get_process_state_t;
        #overlay (Reply_task_get_state) Reply_task_map_corpse_info_64: __Reply__task_map_corpse_info_64_t;
        #overlay (Reply_task_get_state) Reply_task_inspect: __Reply__task_inspect_t;
    }

}

#if OS == .MACOS {
    __Request__lock_set_create_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        n_ulocks: s32;
        policy: s32;
    }

}

#if OS == .MACOS {
    __Request__lock_set_destroy_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        lock_set: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__mach_ports_lookup_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__mach_ports_register_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        init_port_set: mach_msg_ool_ports_descriptor_t;
        NDR: NDR_record_t;
        init_port_setCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__semaphore_create_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        policy: s32;
        value: s32;
    }

}

#if OS == .MACOS {
    __Request__semaphore_destroy_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        semaphore: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_assign_default_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        assign_threads: u32;
    }

}

#if OS == .MACOS {
    __Request__task_assign_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        new_set: mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        assign_threads: u32;
    }

}

#if OS == .MACOS {
    __Request__task_create_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        ledgers: mach_msg_ool_ports_descriptor_t;
        NDR: NDR_record_t;
        ledgersCnt: mach_msg_type_number_t;
        inherit_memory: u32;
    }

}

#if OS == .MACOS {
    __Request__task_generate_corpse_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_assignment_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_emulation_vector_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        exception_mask: exception_mask_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        which: mach_voucher_selector_t;
    }

}

#if OS == .MACOS {
    __Request__task_get_special_port_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        which_port: s32;
    }

}

#if OS == .MACOS {
    __Request__task_get_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: thread_state_flavor_t;
        old_stateCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: task_flavor_t;
        task_info_outCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_inspect_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: task_inspect_flavor_t;
        info_outCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_map_corpse_info_64_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        corspe_task: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_map_corpse_info_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        corspe_task: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_policy_get_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: task_policy_flavor_t;
        policy_infoCnt: mach_msg_type_number_t;
        get_default: u32;
    }

}

#if OS == .MACOS {
    __Request__task_policy_set_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: task_policy_flavor_t;
        policy_infoCnt: mach_msg_type_number_t;
        policy_info: [16] s32;
    }

}

#if OS == .MACOS {
    __Request__task_policy_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        policy: policy_t;
        baseCnt: mach_msg_type_number_t;
        base: [5] s32;
        set_limit: u32;
        change: u32;
    }

}

#if OS == .MACOS {
    __Request__task_purgable_info_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_register_dyld_get_process_state_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_register_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        dyld_images: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        dyld_imagesCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_register_dyld_set_dyld_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        dyld_state: u8;
        dyld_statePad: [3] u8;
    }

}

#if OS == .MACOS {
    __Request__task_register_dyld_shared_cache_image_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        dyld_cache_image: dyld_kernel_image_info_t #align 4;
        no_cache: u32;
        private_cache: u32;
    }

}

#if OS == .MACOS {
    __Request__task_resume2_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_resume_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_sample_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        reply: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_set_emulation_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        routine_entry_pt: vm_address_t #align 4;
        routine_number: s32;
    }

}

#if OS == .MACOS {
    __Request__task_set_emulation_vector_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        emulation_vector: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        vector_start: s32;
        emulation_vectorCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_set_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        new_port: mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        exception_mask: exception_mask_t;
        behavior: exception_behavior_t;
        new_flavor: thread_state_flavor_t;
    }

}

#if OS == .MACOS {
    __Request__task_set_info_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: task_flavor_t;
        task_info_inCnt: mach_msg_type_number_t;
        task_info_in: [52] s32;
    }

}

#if OS == .MACOS {
    __Request__task_set_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        voucher: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_set_phys_footprint_limit_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        new_limit: s32;
    }

}

#if OS == .MACOS {
    __Request__task_set_policy_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        pset: mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        policy: policy_t;
        baseCnt: mach_msg_type_number_t;
        base: [5] s32;
        limitCnt: mach_msg_type_number_t;
        limit: [1] s32;
        change: u32;
    }

}

#if OS == .MACOS {
    __Request__task_set_ras_pc_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        basepc: vm_address_t #align 4;
        boundspc: vm_address_t #align 4;
    }

}

#if OS == .MACOS {
    __Request__task_set_special_port_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        special_port: mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        which_port: s32;
    }

}

#if OS == .MACOS {
    __Request__task_set_state_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: thread_state_flavor_t;
        new_stateCnt: mach_msg_type_number_t;
        new_state: [614] u32;
    }

}

#if OS == .MACOS {
    __Request__task_suspend2_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_suspend_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_swap_exception_ports_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        new_port: mach_msg_port_descriptor_t;
        NDR: NDR_record_t;
        exception_mask: exception_mask_t;
        behavior: exception_behavior_t;
        new_flavor: thread_state_flavor_t;
    }

}

#if OS == .MACOS {
    __Request__task_swap_mach_voucher_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        new_voucher: mach_msg_port_descriptor_t;
        old_voucher: mach_msg_port_descriptor_t;
    }

}

#if OS == .MACOS {
    __Request__task_terminate_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_threads_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__task_unregister_dyld_image_infos_t :: struct {
        Head: mach_msg_header_t;
        msgh_body: mach_msg_body_t;
        dyld_images: mach_msg_ool_descriptor_t;
        NDR: NDR_record_t;
        dyld_imagesCnt: mach_msg_type_number_t;
    }

}

#if OS == .MACOS {
    __Request__task_zone_info_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __Request__thread_create_running_t :: struct {
        Head: mach_msg_header_t;
        NDR: NDR_record_t;
        flavor: thread_state_flavor_t;
        new_stateCnt: mach_msg_type_number_t;
        new_state: [614] u32;
    }

}

#if OS == .MACOS {
    __Request__thread_create_t :: struct {
        Head: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    __RequestUnion__task_subsystem :: struct {
        Request_thread_create_running: __Request__thread_create_running_t; // largest member, moved to the start of the struct/union to support #overlay.;
        #overlay (Request_thread_create_running) Request_task_create: __Request__task_create_t;
        #overlay (Request_thread_create_running) Request_task_terminate: __Request__task_terminate_t;
        #overlay (Request_thread_create_running) Request_task_threads: __Request__task_threads_t;
        #overlay (Request_thread_create_running) Request_mach_ports_register: __Request__mach_ports_register_t;
        #overlay (Request_thread_create_running) Request_mach_ports_lookup: __Request__mach_ports_lookup_t;
        #overlay (Request_thread_create_running) Request_task_info: __Request__task_info_t;
        #overlay (Request_thread_create_running) Request_task_set_info: __Request__task_set_info_t;
        #overlay (Request_thread_create_running) Request_task_suspend: __Request__task_suspend_t;
        #overlay (Request_thread_create_running) Request_task_resume: __Request__task_resume_t;
        #overlay (Request_thread_create_running) Request_task_get_special_port: __Request__task_get_special_port_t;
        #overlay (Request_thread_create_running) Request_task_set_special_port: __Request__task_set_special_port_t;
        #overlay (Request_thread_create_running) Request_thread_create: __Request__thread_create_t;
        #overlay (Request_thread_create_running) Request_task_set_exception_ports: __Request__task_set_exception_ports_t;
        #overlay (Request_thread_create_running) Request_task_get_exception_ports: __Request__task_get_exception_ports_t;
        #overlay (Request_thread_create_running) Request_task_swap_exception_ports: __Request__task_swap_exception_ports_t;
        #overlay (Request_thread_create_running) Request_lock_set_create: __Request__lock_set_create_t;
        #overlay (Request_thread_create_running) Request_lock_set_destroy: __Request__lock_set_destroy_t;
        #overlay (Request_thread_create_running) Request_semaphore_create: __Request__semaphore_create_t;
        #overlay (Request_thread_create_running) Request_semaphore_destroy: __Request__semaphore_destroy_t;
        #overlay (Request_thread_create_running) Request_task_policy_set: __Request__task_policy_set_t;
        #overlay (Request_thread_create_running) Request_task_policy_get: __Request__task_policy_get_t;
        #overlay (Request_thread_create_running) Request_task_sample: __Request__task_sample_t;
        #overlay (Request_thread_create_running) Request_task_policy: __Request__task_policy_t;
        #overlay (Request_thread_create_running) Request_task_set_emulation: __Request__task_set_emulation_t;
        #overlay (Request_thread_create_running) Request_task_get_emulation_vector: __Request__task_get_emulation_vector_t;
        #overlay (Request_thread_create_running) Request_task_set_emulation_vector: __Request__task_set_emulation_vector_t;
        #overlay (Request_thread_create_running) Request_task_set_ras_pc: __Request__task_set_ras_pc_t;
        #overlay (Request_thread_create_running) Request_task_zone_info: __Request__task_zone_info_t;
        #overlay (Request_thread_create_running) Request_task_assign: __Request__task_assign_t;
        #overlay (Request_thread_create_running) Request_task_assign_default: __Request__task_assign_default_t;
        #overlay (Request_thread_create_running) Request_task_get_assignment: __Request__task_get_assignment_t;
        #overlay (Request_thread_create_running) Request_task_set_policy: __Request__task_set_policy_t;
        #overlay (Request_thread_create_running) Request_task_get_state: __Request__task_get_state_t;
        #overlay (Request_thread_create_running) Request_task_set_state: __Request__task_set_state_t;
        #overlay (Request_thread_create_running) Request_task_set_phys_footprint_limit: __Request__task_set_phys_footprint_limit_t;
        #overlay (Request_thread_create_running) Request_task_suspend2: __Request__task_suspend2_t;
        #overlay (Request_thread_create_running) Request_task_resume2: __Request__task_resume2_t;
        #overlay (Request_thread_create_running) Request_task_purgable_info: __Request__task_purgable_info_t;
        #overlay (Request_thread_create_running) Request_task_get_mach_voucher: __Request__task_get_mach_voucher_t;
        #overlay (Request_thread_create_running) Request_task_set_mach_voucher: __Request__task_set_mach_voucher_t;
        #overlay (Request_thread_create_running) Request_task_swap_mach_voucher: __Request__task_swap_mach_voucher_t;
        #overlay (Request_thread_create_running) Request_task_generate_corpse: __Request__task_generate_corpse_t;
        #overlay (Request_thread_create_running) Request_task_map_corpse_info: __Request__task_map_corpse_info_t;
        #overlay (Request_thread_create_running) Request_task_register_dyld_image_infos: __Request__task_register_dyld_image_infos_t;
        #overlay (Request_thread_create_running) Request_task_unregister_dyld_image_infos: __Request__task_unregister_dyld_image_infos_t;
        #overlay (Request_thread_create_running) Request_task_get_dyld_image_infos: __Request__task_get_dyld_image_infos_t;
        #overlay (Request_thread_create_running) Request_task_register_dyld_shared_cache_image_info: __Request__task_register_dyld_shared_cache_image_info_t;
        #overlay (Request_thread_create_running) Request_task_register_dyld_set_dyld_state: __Request__task_register_dyld_set_dyld_state_t;
        #overlay (Request_thread_create_running) Request_task_register_dyld_get_process_state: __Request__task_register_dyld_get_process_state_t;
        #overlay (Request_thread_create_running) Request_task_map_corpse_info_64: __Request__task_map_corpse_info_64_t;
        #overlay (Request_thread_create_running) Request_task_inspect: __Request__task_inspect_t;
    }

}

#if OS == .MACOS {
    anon_enum_6 :: enum u32 {
        kCFCalendarComponentsWrap :: 1;
    }

}

#if OS == .MACOS {
    anon_enum_7 :: enum u32 {
        kCFNumberFormatterParseIntegersOnly :: 1;
    }

}

#if OS == .MACOS {
    anon_enum_8 :: enum u32 {
        kCFXMLNodeCurrentVersion :: 1;
    }

}

#if OS == .MACOS {
    audit_token_t :: struct {
        val: [8] u32;
    }

}

#if OS == .MACOS {
    CFAllocatorContext :: struct {
        version: CFIndex;
        info: *void;
        retain: CFAllocatorRetainCallBack;
        release: CFAllocatorReleaseCallBack;
        copyDescription: CFAllocatorCopyDescriptionCallBack;
        allocate: CFAllocatorAllocateCallBack;
        reallocate: CFAllocatorReallocateCallBack;
        deallocate: CFAllocatorDeallocateCallBack;
        preferredSize: CFAllocatorPreferredSizeCallBack;
    }

}

#if OS == .MACOS {
    CFArrayCallBacks :: struct {
        version: CFIndex;
        retain: CFArrayRetainCallBack;
        release: CFArrayReleaseCallBack;
        copyDescription: CFArrayCopyDescriptionCallBack;
        equal: CFArrayEqualCallBack;
    }

}

#if OS == .MACOS {
    CFCalendarUnit :: enum CFOptionFlags {
        Era :: 2;
        Year :: 4;
        Month :: 8;
        Day :: 16;
        Hour :: 32;
        Minute :: 64;
        Second :: 128;
        Week :: 256;
        Weekday :: 512;
        WeekdayOrdinal :: 1024;
        Quarter :: 2048;
        WeekOfMonth :: 4096;
        WeekOfYear :: 8192;
        YearForWeekOfYear :: 16384;
        kCFCalendarUnitEra :: Era;
        kCFCalendarUnitYear :: Year;
        kCFCalendarUnitMonth :: Month;
        kCFCalendarUnitDay :: Day;
        kCFCalendarUnitHour :: Hour;
        kCFCalendarUnitMinute :: Minute;
        kCFCalendarUnitSecond :: Second;
        kCFCalendarUnitWeek :: Week;
        kCFCalendarUnitWeekday :: Weekday;
        kCFCalendarUnitWeekdayOrdinal :: WeekdayOrdinal;
        kCFCalendarUnitQuarter :: Quarter;
        kCFCalendarUnitWeekOfMonth :: WeekOfMonth;
        kCFCalendarUnitWeekOfYear :: WeekOfYear;
        kCFCalendarUnitYearForWeekOfYear :: YearForWeekOfYear;
    }

}

#if OS == .MACOS {
    CFCharacterSetPredefinedSet :: enum CFIndex {
        Control :: 1;
        Whitespace :: 2;
        WhitespaceAndNewline :: 3;
        DecimalDigit :: 4;
        Letter :: 5;
        LowercaseLetter :: 6;
        UppercaseLetter :: 7;
        NonBase :: 8;
        Decomposable :: 9;
        AlphaNumeric :: 10;
        Punctuation :: 11;
        CapitalizedLetter :: 13;
        Symbol :: 14;
        Newline :: 15;
        Illegal :: 12;
        kCFCharacterSetControl :: Control;
        kCFCharacterSetWhitespace :: Whitespace;
        kCFCharacterSetWhitespaceAndNewline :: WhitespaceAndNewline;
        kCFCharacterSetDecimalDigit :: DecimalDigit;
        kCFCharacterSetLetter :: Letter;
        kCFCharacterSetLowercaseLetter :: LowercaseLetter;
        kCFCharacterSetUppercaseLetter :: UppercaseLetter;
        kCFCharacterSetNonBase :: NonBase;
        kCFCharacterSetDecomposable :: Decomposable;
        kCFCharacterSetAlphaNumeric :: AlphaNumeric;
        kCFCharacterSetPunctuation :: Punctuation;
        kCFCharacterSetCapitalizedLetter :: CapitalizedLetter;
        kCFCharacterSetSymbol :: Symbol;
        kCFCharacterSetNewline :: Newline;
        kCFCharacterSetIllegal :: Illegal;
    }

}

#if OS == .MACOS {
    CFComparisonResult :: enum CFIndex {
        LessThan :: -1;
        EqualTo :: 0;
        GreaterThan :: 1;
        kCFCompareLessThan :: LessThan;
        kCFCompareEqualTo :: EqualTo;
        kCFCompareGreaterThan :: GreaterThan;
    }

}

#if OS == .MACOS {
    CFDataSearchFlags :: enum_flags CFOptionFlags {
        Backwards :: 0x1;
        Anchored :: 0x2;
        kCFDataSearchBackwards :: Backwards;
        kCFDataSearchAnchored :: Anchored;
    }

}

#if OS == .MACOS {
    CFDictionaryKeyCallBacks :: struct {
        version: CFIndex;
        retain: CFDictionaryRetainCallBack;
        release: CFDictionaryReleaseCallBack;
        copyDescription: CFDictionaryCopyDescriptionCallBack;
        equal: CFDictionaryEqualCallBack;
        hash: CFDictionaryHashCallBack;
    }

}

#if OS == .MACOS {
    CFDictionaryValueCallBacks :: struct {
        version: CFIndex;
        retain: CFDictionaryRetainCallBack;
        release: CFDictionaryReleaseCallBack;
        copyDescription: CFDictionaryCopyDescriptionCallBack;
        equal: CFDictionaryEqualCallBack;
    }

}

#if OS == .MACOS {
    CFFileDescriptorContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFGregorianDate :: struct {
        year: s32;
        month: s8;
        day: s8;
        hour: s8;
        minute: s8;
        second: float64;
    }

}

#if OS == .MACOS {
    CFGregorianUnits :: struct {
        years: s32;
        months: s32;
        days: s32;
        hours: s32;
        minutes: s32;
        seconds: float64;
    }

}

#if OS == .MACOS {
    CFLocaleLanguageDirection :: enum CFIndex {
        Unknown :: 0;
        LeftToRight :: 1;
        RightToLeft :: 2;
        TopToBottom :: 3;
        BottomToTop :: 4;
        kCFLocaleLanguageDirectionUnknown :: Unknown;
        kCFLocaleLanguageDirectionLeftToRight :: LeftToRight;
        kCFLocaleLanguageDirectionRightToLeft :: RightToLeft;
        kCFLocaleLanguageDirectionTopToBottom :: TopToBottom;
        kCFLocaleLanguageDirectionBottomToTop :: BottomToTop;
    }

}

#if OS == .MACOS {
    CFMachPortContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFMessagePortContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFNotificationSuspensionBehavior :: enum CFIndex {
        Drop :: 1;
        Coalesce :: 2;
        Hold :: 3;
        DeliverImmediately :: 4;
        CFNotificationSuspensionBehaviorDrop :: Drop;
        CFNotificationSuspensionBehaviorCoalesce :: Coalesce;
        CFNotificationSuspensionBehaviorHold :: Hold;
        CFNotificationSuspensionBehaviorDeliverImmediately :: DeliverImmediately;
    }

}

#if OS == .MACOS {
    CFRange :: struct {
        location: CFIndex;
        length: CFIndex;
    }

}

#if OS == .MACOS {
    CFRunLoopActivity :: enum_flags CFOptionFlags {
        Entry :: 0x1;
        BeforeTimers :: 0x2;
        BeforeSources :: 0x4;
        BeforeWaiting :: 0x20;
        AfterWaiting :: 0x40;
        Exit :: 0x80;
        AllActivities :: 0xfffffff;
        kCFRunLoopEntry :: Entry;
        kCFRunLoopBeforeTimers :: BeforeTimers;
        kCFRunLoopBeforeSources :: BeforeSources;
        kCFRunLoopBeforeWaiting :: BeforeWaiting;
        kCFRunLoopAfterWaiting :: AfterWaiting;
        kCFRunLoopExit :: Exit;
        kCFRunLoopAllActivities :: AllActivities;
    }

}

#if OS == .MACOS {
    CFRunLoopObserverContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFRunLoopRunResult :: enum s32 {
        Finished :: 1;
        Stopped :: 2;
        TimedOut :: 3;
        HandledSource :: 4;
        kCFRunLoopRunFinished :: Finished;
        kCFRunLoopRunStopped :: Stopped;
        kCFRunLoopRunTimedOut :: TimedOut;
        kCFRunLoopRunHandledSource :: HandledSource;
    }

}

#if OS == .MACOS {
    CFRunLoopSourceContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
        equal: #type (info1: *void, info2: *void) -> Boolean #c_call;
        hash: #type (info: *void) -> CFHashCode #c_call;
        schedule: #type (info: *void, rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #c_call;
        cancel: #type (info: *void, rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #c_call;
        perform: #type (info: *void) -> void #c_call;
    }

}

#if OS == .MACOS {
    CFRunLoopSourceContext1 :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
        equal: #type (info1: *void, info2: *void) -> Boolean #c_call;
        hash: #type (info: *void) -> CFHashCode #c_call;
        getPort: #type (info: *void) -> mach_port_t #c_call;
        perform: #type (msg: *void, size: CFIndex, allocator: CFAllocatorRef, info: *void) -> *void #c_call;
    }

}

#if OS == .MACOS {
    CFRunLoopTimerContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFSocketContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFSocketSignature :: struct {
        protocolFamily: s32;
        socketType: s32;
        protocol: s32;
        address: CFDataRef;
    }

}

#if OS == .MACOS {
    CFStreamClientContext :: struct {
        version: CFIndex;
        info: *void;
        retain: #type (info: *void) -> *void #c_call;
        release: #type (info: *void) -> void #c_call;
        copyDescription: #type (info: *void) -> CFStringRef #c_call;
    }

}

#if OS == .MACOS {
    CFStreamError :: struct {
        domain: CFIndex;
        error: s32;
    }

}

#if OS == .MACOS {
    CFStreamErrorDomain :: enum CFIndex {
        Custom :: -1;
        POSIX :: 1;
        MacOSStatus :: 2;
        kCFStreamErrorDomainCustom :: Custom;
        kCFStreamErrorDomainPOSIX :: POSIX;
        kCFStreamErrorDomainMacOSStatus :: MacOSStatus;
    }

}

#if OS == .MACOS {
    CFStreamStatus :: enum CFIndex {
        NotOpen :: 0;
        Opening :: 1;
        Open :: 2;
        Reading :: 3;
        Writing :: 4;
        AtEnd :: 5;
        Closed :: 6;
        Error :: 7;
        kCFStreamStatusNotOpen :: NotOpen;
        kCFStreamStatusOpening :: Opening;
        kCFStreamStatusOpen :: Open;
        kCFStreamStatusReading :: Reading;
        kCFStreamStatusWriting :: Writing;
        kCFStreamStatusAtEnd :: AtEnd;
        kCFStreamStatusClosed :: Closed;
        kCFStreamStatusError :: Error;
    }

}

#if OS == .MACOS {
    CFStringCompareFlags :: enum_flags CFOptionFlags {
        CaseInsensitive :: 0x1;
        Backwards :: 0x4;
        Anchored :: 0x8;
        Nonliteral :: 0x10;
        Localized :: 0x20;
        Numerically :: 0x40;
        DiacriticInsensitive :: 0x80;
        WidthInsensitive :: 0x100;
        ForcedOrdering :: 0x200;
        kCFCompareCaseInsensitive :: CaseInsensitive;
        kCFCompareBackwards :: Backwards;
        kCFCompareAnchored :: Anchored;
        kCFCompareNonliteral :: Nonliteral;
        kCFCompareLocalized :: Localized;
        kCFCompareNumerically :: Numerically;
        kCFCompareDiacriticInsensitive :: DiacriticInsensitive;
        kCFCompareWidthInsensitive :: WidthInsensitive;
        kCFCompareForcedOrdering :: ForcedOrdering;
    }

}

#if OS == .MACOS {
    CFStringEncoding :: enum u32 {
        MacRoman :: 0;
        WindowsLatin1 :: 1280;
        ISOLatin1 :: 513;
        NextStepLatin :: 2817;
        ASCII :: 1536;
        Unicode :: 256;
        UTF8 :: 134217984;
        NonLossyASCII :: 3071;
        UTF16 :: 256;
        UTF16BE :: 268435712;
        UTF16LE :: 335544576;
        UTF32 :: 201326848;
        UTF32BE :: 402653440;
        UTF32LE :: 469762304;
        kCFStringEncodingMacRoman :: MacRoman;
        kCFStringEncodingWindowsLatin1 :: WindowsLatin1;
        kCFStringEncodingISOLatin1 :: ISOLatin1;
        kCFStringEncodingNextStepLatin :: NextStepLatin;
        kCFStringEncodingASCII :: ASCII;
        kCFStringEncodingUnicode :: Unicode;
        kCFStringEncodingUTF8 :: UTF8;
        kCFStringEncodingNonLossyASCII :: NonLossyASCII;
        kCFStringEncodingUTF16 :: UTF16;
        kCFStringEncodingUTF16BE :: UTF16BE;
        kCFStringEncodingUTF16LE :: UTF16LE;
        kCFStringEncodingUTF32 :: UTF32;
        kCFStringEncodingUTF32BE :: UTF32BE;
        kCFStringEncodingUTF32LE :: UTF32LE;
    }

}

#if OS == .MACOS {
    CFStringInlineBuffer :: struct {
        buffer: [64] u16;
        theString: CFStringRef;
        directUniCharBuffer: *u16;
        directCStringBuffer: *u8;
        rangeToBuffer: CFRange;
        bufferedRangeStart: CFIndex;
        bufferedRangeEnd: CFIndex;
    }

}

#if OS == .MACOS {
    CFStringNormalizationForm :: enum CFIndex {
        D :: 0;
        KD :: 1;
        C :: 2;
        KC :: 3;
        kCFStringNormalizationFormD :: D;
        kCFStringNormalizationFormKD :: KD;
        kCFStringNormalizationFormC :: C;
        kCFStringNormalizationFormKC :: KC;
    }

}

#if OS == .MACOS {
    CFSwappedFloat32 :: struct {
        v: u32;
    }

}

#if OS == .MACOS {
    CFSwappedFloat64 :: struct {
        v: u64;
    }

}

#if OS == .MACOS {
    CFTimeZoneNameStyle :: enum CFIndex {
        Standard :: 0;
        ShortStandard :: 1;
        DaylightSaving :: 2;
        ShortDaylightSaving :: 3;
        Generic :: 4;
        ShortGeneric :: 5;
        kCFTimeZoneNameStyleStandard :: Standard;
        kCFTimeZoneNameStyleShortStandard :: ShortStandard;
        kCFTimeZoneNameStyleDaylightSaving :: DaylightSaving;
        kCFTimeZoneNameStyleShortDaylightSaving :: ShortDaylightSaving;
        kCFTimeZoneNameStyleGeneric :: Generic;
        kCFTimeZoneNameStyleShortGeneric :: ShortGeneric;
    }

}

#if OS == .MACOS {
    CFUUIDBytes :: struct {
        byte0: u8;
        byte1: u8;
        byte2: u8;
        byte3: u8;
        byte4: u8;
        byte5: u8;
        byte6: u8;
        byte7: u8;
        byte8: u8;
        byte9: u8;
        byte10: u8;
        byte11: u8;
        byte12: u8;
        byte13: u8;
        byte14: u8;
        byte15: u8;
    }

}

#if OS == .MACOS {
    dyld_kernel_image_info :: struct {
        uuid: uuid_t;
        fsobjid: fsobj_id_t;
        fsid: fsid_t;
        load_addr: u64;
    }

}

#if OS == .MACOS {
    dyld_kernel_process_info :: struct {
        cache_image_info: dyld_kernel_image_info;
        timestamp: u64;
        imageCount: u32;
        initialImageCount: u32;
        dyldState: u8;
        no_cache: u32;
        private_cache: u32;
    }

}

#if OS == .MACOS {
    FSEventStreamContext :: struct {
        copyDescription: CFAllocatorCopyDescriptionCallBack;
        info: *void;
        retain: CFAllocatorRetainCallBack;
        release: CFAllocatorReleaseCallBack;
        version: CFIndex;
    }

}

#if OS == .MACOS {
    fsid :: struct {
        val: [2] s32;
    }

}

#if OS == .MACOS {
    fsobj_id :: struct {
        fid_objno: u32;
        fid_generation: u32;
    }

}

#if OS == .MACOS {
    FSRef :: struct {}

}

#if OS == .MACOS {
    gpu_energy_data :: struct {
        task_gpu_utilisation: u64 #align 4;
        task_gpu_stat_reserved0: u64 #align 4;
        task_gpu_stat_reserved1: u64 #align 4;
        task_gpu_stat_reserved2: u64 #align 4;
    }

}

#if OS == .MACOS {
    ipc_info_name :: struct {
        iin_name: mach_port_name_t;
        iin_collision: s32;
        iin_type: mach_port_type_t;
        iin_urefs: mach_port_urefs_t;
        iin_object: u32;
        iin_next: u32;
        iin_hash: u32;
    }

}

#if OS == .MACOS {
    ipc_info_space :: struct {
        iis_genno_mask: u32;
        iis_table_size: u32;
        iis_table_next: u32;
        iis_tree_size: u32;
        iis_tree_small: u32;
        iis_tree_hash: u32;
    }

}

#if OS == .MACOS {
    ipc_info_space_basic :: struct {
        iisb_genno_mask: u32;
        iisb_table_size: u32;
        iisb_table_next: u32;
        iisb_table_inuse: u32;
        iisb_reserved: [2] u32;
    }

}

#if OS == .MACOS {
    ipc_info_tree_name :: struct {
        iitn_name: ipc_info_name_t;
        iitn_lchild: mach_port_name_t;
        iitn_rchild: mach_port_name_t;
    }

}

#if OS == .MACOS {
    kCF :: enum u32 {
        URLBookmarkResolutionWithoutUIMask :: 256;
        URLBookmarkResolutionWithoutMountingMask :: 512;
        URLBookmarkResolutionWithSecurityScope :: 1024;
        BookmarkResolutionWithoutUIMask :: 256;
        BookmarkResolutionWithoutMountingMask :: 512;
        kCFURLBookmarkResolutionWithoutUIMask :: URLBookmarkResolutionWithoutUIMask;
        kCFURLBookmarkResolutionWithoutMountingMask :: URLBookmarkResolutionWithoutMountingMask;
        kCFURLBookmarkResolutionWithSecurityScope :: URLBookmarkResolutionWithSecurityScope;
        kCFBookmarkResolutionWithoutUIMask :: BookmarkResolutionWithoutUIMask;
        kCFBookmarkResolutionWithoutMountingMask :: BookmarkResolutionWithoutMountingMask;
    }

}

#if OS == .MACOS {
    kCFBundleExecutableArchitecture :: enum u32 {
        I386 :: 7;
        PPC :: 18;
        X86_64 :: 16777223;
        PPC64 :: 16777234;
        kCFBundleExecutableArchitectureI386 :: I386;
        kCFBundleExecutableArchitecturePPC :: PPC;
        kCFBundleExecutableArchitectureX86_64 :: X86_64;
        kCFBundleExecutableArchitecturePPC64 :: PPC64;
    }

}

#if OS == .MACOS {
    kCFDateFormatter :: enum u32 {
        NoStyle :: 0;
        ShortStyle :: 1;
        MediumStyle :: 2;
        LongStyle :: 3;
        FullStyle :: 4;
        kCFDateFormatterNoStyle :: NoStyle;
        kCFDateFormatterShortStyle :: ShortStyle;
        kCFDateFormatterMediumStyle :: MediumStyle;
        kCFDateFormatterLongStyle :: LongStyle;
        kCFDateFormatterFullStyle :: FullStyle;
    }

}

#if OS == .MACOS {
    kCFFileDescriptor :: enum u32 {
        ReadCallBack :: 1;
        WriteCallBack :: 2;
        kCFFileDescriptorReadCallBack :: ReadCallBack;
        kCFFileDescriptorWriteCallBack :: WriteCallBack;
    }

}

#if OS == .MACOS {
    kCFGregorian :: enum u32 {
        UnitsYears :: 1;
        UnitsMonths :: 2;
        UnitsDays :: 4;
        UnitsHours :: 8;
        UnitsMinutes :: 16;
        UnitsSeconds :: 32;
        AllUnits :: 16777215;
        kCFGregorianUnitsYears :: UnitsYears;
        kCFGregorianUnitsMonths :: UnitsMonths;
        kCFGregorianUnitsDays :: UnitsDays;
        kCFGregorianUnitsHours :: UnitsHours;
        kCFGregorianUnitsMinutes :: UnitsMinutes;
        kCFGregorianUnitsSeconds :: UnitsSeconds;
        kCFGregorianAllUnits :: AllUnits;
    }

}

#if OS == .MACOS {
    kCFISO8601DateFormatWith :: enum u32 {
        Year :: 1;
        Month :: 2;
        WeekOfYear :: 4;
        Day :: 16;
        Time :: 32;
        TimeZone :: 64;
        SpaceBetweenDateAndTime :: 128;
        DashSeparatorInDate :: 256;
        ColonSeparatorInTime :: 512;
        ColonSeparatorInTimeZone :: 1024;
        FractionalSeconds :: 2048;
        FullDate :: 275;
        FullTime :: 1632;
        InternetDateTime :: 1907;
        kCFISO8601DateFormatWithYear :: Year;
        kCFISO8601DateFormatWithMonth :: Month;
        kCFISO8601DateFormatWithWeekOfYear :: WeekOfYear;
        kCFISO8601DateFormatWithDay :: Day;
        kCFISO8601DateFormatWithTime :: Time;
        kCFISO8601DateFormatWithTimeZone :: TimeZone;
        kCFISO8601DateFormatWithSpaceBetweenDateAndTime :: SpaceBetweenDateAndTime;
        kCFISO8601DateFormatWithDashSeparatorInDate :: DashSeparatorInDate;
        kCFISO8601DateFormatWithColonSeparatorInTime :: ColonSeparatorInTime;
        kCFISO8601DateFormatWithColonSeparatorInTimeZone :: ColonSeparatorInTimeZone;
        kCFISO8601DateFormatWithFractionalSeconds :: FractionalSeconds;
        kCFISO8601DateFormatWithFullDate :: FullDate;
        kCFISO8601DateFormatWithFullTime :: FullTime;
        kCFISO8601DateFormatWithInternetDateTime :: InternetDateTime;
    }

}

#if OS == .MACOS {
    kCFMessagePort :: enum s32 {
        Success :: 0;
        SendTimeout :: -1;
        ReceiveTimeout :: -2;
        IsInvalid :: -3;
        TransportError :: -4;
        BecameInvalidError :: -5;
        kCFMessagePortSuccess :: Success;
        kCFMessagePortSendTimeout :: SendTimeout;
        kCFMessagePortReceiveTimeout :: ReceiveTimeout;
        kCFMessagePortIsInvalid :: IsInvalid;
        kCFMessagePortTransportError :: TransportError;
        kCFMessagePortBecameInvalidError :: BecameInvalidError;
    }

}

#if OS == .MACOS {
    kCFNotification :: enum u32 {
        DeliverImmediately :: 1;
        PostToAllSessions :: 2;
        kCFNotificationDeliverImmediately :: DeliverImmediately;
        kCFNotificationPostToAllSessions :: PostToAllSessions;
    }

}

#if OS == .MACOS {
    kCFNumber :: enum u32 {
        SInt8Type :: 1;
        SInt16Type :: 2;
        SInt32Type :: 3;
        SInt64Type :: 4;
        Float32Type :: 5;
        Float64Type :: 6;
        CharType :: 7;
        ShortType :: 8;
        IntType :: 9;
        LongType :: 10;
        LongLongType :: 11;
        FloatType :: 12;
        DoubleType :: 13;
        CFIndexType :: 14;
        NSIntegerType :: 15;
        CGFloatType :: 16;
        MaxType :: 16;
        kCFNumberSInt8Type :: SInt8Type;
        kCFNumberSInt16Type :: SInt16Type;
        kCFNumberSInt32Type :: SInt32Type;
        kCFNumberSInt64Type :: SInt64Type;
        kCFNumberFloat32Type :: Float32Type;
        kCFNumberFloat64Type :: Float64Type;
        kCFNumberCharType :: CharType;
        kCFNumberShortType :: ShortType;
        kCFNumberIntType :: IntType;
        kCFNumberLongType :: LongType;
        kCFNumberLongLongType :: LongLongType;
        kCFNumberFloatType :: FloatType;
        kCFNumberDoubleType :: DoubleType;
        kCFNumberCFIndexType :: CFIndexType;
        kCFNumberNSIntegerType :: NSIntegerType;
        kCFNumberCGFloatType :: CGFloatType;
        kCFNumberMaxType :: MaxType;
    }

}

#if OS == .MACOS {
    kCFNumberFormatter :: enum u32 {
        NoStyle :: 0;
        DecimalStyle :: 1;
        CurrencyStyle :: 2;
        PercentStyle :: 3;
        ScientificStyle :: 4;
        SpellOutStyle :: 5;
        OrdinalStyle :: 6;
        CurrencyISOCodeStyle :: 8;
        CurrencyPluralStyle :: 9;
        CurrencyAccountingStyle :: 10;
        kCFNumberFormatterNoStyle :: NoStyle;
        kCFNumberFormatterDecimalStyle :: DecimalStyle;
        kCFNumberFormatterCurrencyStyle :: CurrencyStyle;
        kCFNumberFormatterPercentStyle :: PercentStyle;
        kCFNumberFormatterScientificStyle :: ScientificStyle;
        kCFNumberFormatterSpellOutStyle :: SpellOutStyle;
        kCFNumberFormatterOrdinalStyle :: OrdinalStyle;
        kCFNumberFormatterCurrencyISOCodeStyle :: CurrencyISOCodeStyle;
        kCFNumberFormatterCurrencyPluralStyle :: CurrencyPluralStyle;
        kCFNumberFormatterCurrencyAccountingStyle :: CurrencyAccountingStyle;
    }

}

#if OS == .MACOS {
    kCFNumberFormatterPad :: enum u32 {
        BeforePrefix :: 0;
        AfterPrefix :: 1;
        BeforeSuffix :: 2;
        AfterSuffix :: 3;
        kCFNumberFormatterPadBeforePrefix :: BeforePrefix;
        kCFNumberFormatterPadAfterPrefix :: AfterPrefix;
        kCFNumberFormatterPadBeforeSuffix :: BeforeSuffix;
        kCFNumberFormatterPadAfterSuffix :: AfterSuffix;
    }

}

#if OS == .MACOS {
    kCFNumberFormatterRound :: enum u32 {
        Ceiling :: 0;
        Floor :: 1;
        Down :: 2;
        Up :: 3;
        HalfEven :: 4;
        HalfDown :: 5;
        HalfUp :: 6;
        kCFNumberFormatterRoundCeiling :: Ceiling;
        kCFNumberFormatterRoundFloor :: Floor;
        kCFNumberFormatterRoundDown :: Down;
        kCFNumberFormatterRoundUp :: Up;
        kCFNumberFormatterRoundHalfEven :: HalfEven;
        kCFNumberFormatterRoundHalfDown :: HalfDown;
        kCFNumberFormatterRoundHalfUp :: HalfUp;
    }

}

#if OS == .MACOS {
    kCFPropertyList :: enum u32 {
        Immutable :: 0;
        MutableContainers :: 1;
        MutableContainersAndLeaves :: 2;
        kCFPropertyListImmutable :: Immutable;
        kCFPropertyListMutableContainers :: MutableContainers;
        kCFPropertyListMutableContainersAndLeaves :: MutableContainersAndLeaves;
    }

}

#if OS == .MACOS {
    kCFPropertyList_1 :: enum u32 {
        OpenStepFormat :: 1;
        XMLFormat_v1_0 :: 100;
        BinaryFormat_v1_0 :: 200;
        kCFPropertyListOpenStepFormat :: OpenStepFormat;
        kCFPropertyListXMLFormat_v1_0 :: XMLFormat_v1_0;
        kCFPropertyListBinaryFormat_v1_0 :: BinaryFormat_v1_0;
    }

}

#if OS == .MACOS {
    kCFPropertyList_2 :: enum u32 {
        ReadCorruptError :: 3840;
        ReadUnknownVersionError :: 3841;
        ReadStreamError :: 3842;
        WriteStreamError :: 3851;
        kCFPropertyListReadCorruptError :: ReadCorruptError;
        kCFPropertyListReadUnknownVersionError :: ReadUnknownVersionError;
        kCFPropertyListReadStreamError :: ReadStreamError;
        kCFPropertyListWriteStreamError :: WriteStreamError;
    }

}

#if OS == .MACOS {
    kCFSocket :: enum s32 {
        Success :: 0;
        Error :: -1;
        Timeout :: -2;
        kCFSocketSuccess :: Success;
        kCFSocketError :: Error;
        kCFSocketTimeout :: Timeout;
    }

}

#if OS == .MACOS {
    kCFSocket_1 :: enum u32 {
        NoCallBack :: 0;
        ReadCallBack :: 1;
        AcceptCallBack :: 2;
        DataCallBack :: 3;
        ConnectCallBack :: 4;
        WriteCallBack :: 8;
        kCFSocketNoCallBack :: NoCallBack;
        kCFSocketReadCallBack :: ReadCallBack;
        kCFSocketAcceptCallBack :: AcceptCallBack;
        kCFSocketDataCallBack :: DataCallBack;
        kCFSocketConnectCallBack :: ConnectCallBack;
        kCFSocketWriteCallBack :: WriteCallBack;
    }

}

#if OS == .MACOS {
    kCFSocket_2 :: enum u32 {
        AutomaticallyReenableReadCallBack :: 1;
        AutomaticallyReenableAcceptCallBack :: 2;
        AutomaticallyReenableDataCallBack :: 3;
        AutomaticallyReenableWriteCallBack :: 8;
        LeaveErrors :: 64;
        CloseOnInvalidate :: 128;
        kCFSocketAutomaticallyReenableReadCallBack :: AutomaticallyReenableReadCallBack;
        kCFSocketAutomaticallyReenableAcceptCallBack :: AutomaticallyReenableAcceptCallBack;
        kCFSocketAutomaticallyReenableDataCallBack :: AutomaticallyReenableDataCallBack;
        kCFSocketAutomaticallyReenableWriteCallBack :: AutomaticallyReenableWriteCallBack;
        kCFSocketLeaveErrors :: LeaveErrors;
        kCFSocketCloseOnInvalidate :: CloseOnInvalidate;
    }

}

#if OS == .MACOS {
    kCFStreamEvent :: enum u32 {
        None :: 0;
        OpenCompleted :: 1;
        HasBytesAvailable :: 2;
        CanAcceptBytes :: 4;
        ErrorOccurred :: 8;
        EndEncountered :: 16;
        kCFStreamEventNone :: None;
        kCFStreamEventOpenCompleted :: OpenCompleted;
        kCFStreamEventHasBytesAvailable :: HasBytesAvailable;
        kCFStreamEventCanAcceptBytes :: CanAcceptBytes;
        kCFStreamEventErrorOccurred :: ErrorOccurred;
        kCFStreamEventEndEncountered :: EndEncountered;
    }

}

#if OS == .MACOS {
    kCFStringEncoding_1 :: enum u32 {
        MacJapanese :: 1;
        MacChineseTrad :: 2;
        MacKorean :: 3;
        MacArabic :: 4;
        MacHebrew :: 5;
        MacGreek :: 6;
        MacCyrillic :: 7;
        MacDevanagari :: 9;
        MacGurmukhi :: 10;
        MacGujarati :: 11;
        MacOriya :: 12;
        MacBengali :: 13;
        MacTamil :: 14;
        MacTelugu :: 15;
        MacKannada :: 16;
        MacMalayalam :: 17;
        MacSinhalese :: 18;
        MacBurmese :: 19;
        MacKhmer :: 20;
        MacThai :: 21;
        MacLaotian :: 22;
        MacGeorgian :: 23;
        MacArmenian :: 24;
        MacChineseSimp :: 25;
        MacTibetan :: 26;
        MacMongolian :: 27;
        MacEthiopic :: 28;
        MacCentralEurRoman :: 29;
        MacVietnamese :: 30;
        MacExtArabic :: 31;
        MacSymbol :: 33;
        MacDingbats :: 34;
        MacTurkish :: 35;
        MacCroatian :: 36;
        MacIcelandic :: 37;
        MacRomanian :: 38;
        MacCeltic :: 39;
        MacGaelic :: 40;
        MacFarsi :: 140;
        MacUkrainian :: 152;
        MacInuit :: 236;
        MacVT100 :: 252;
        MacHFS :: 255;
        ISOLatin2 :: 514;
        ISOLatin3 :: 515;
        ISOLatin4 :: 516;
        ISOLatinCyrillic :: 517;
        ISOLatinArabic :: 518;
        ISOLatinGreek :: 519;
        ISOLatinHebrew :: 520;
        ISOLatin5 :: 521;
        ISOLatin6 :: 522;
        ISOLatinThai :: 523;
        ISOLatin7 :: 525;
        ISOLatin8 :: 526;
        ISOLatin9 :: 527;
        ISOLatin10 :: 528;
        DOSLatinUS :: 1024;
        DOSGreek :: 1029;
        DOSBalticRim :: 1030;
        DOSLatin1 :: 1040;
        DOSGreek1 :: 1041;
        DOSLatin2 :: 1042;
        DOSCyrillic :: 1043;
        DOSTurkish :: 1044;
        DOSPortuguese :: 1045;
        DOSIcelandic :: 1046;
        DOSHebrew :: 1047;
        DOSCanadianFrench :: 1048;
        DOSArabic :: 1049;
        DOSNordic :: 1050;
        DOSRussian :: 1051;
        DOSGreek2 :: 1052;
        DOSThai :: 1053;
        DOSJapanese :: 1056;
        DOSChineseSimplif :: 1057;
        DOSKorean :: 1058;
        DOSChineseTrad :: 1059;
        WindowsLatin2 :: 1281;
        WindowsCyrillic :: 1282;
        WindowsGreek :: 1283;
        WindowsLatin5 :: 1284;
        WindowsHebrew :: 1285;
        WindowsArabic :: 1286;
        WindowsBalticRim :: 1287;
        WindowsVietnamese :: 1288;
        WindowsKoreanJohab :: 1296;
        ANSEL :: 1537;
        JIS_X0201_76 :: 1568;
        JIS_X0208_83 :: 1569;
        JIS_X0208_90 :: 1570;
        JIS_X0212_90 :: 1571;
        JIS_C6226_78 :: 1572;
        ShiftJIS_X0213 :: 1576;
        ShiftJIS_X0213_MenKuTen :: 1577;
        GB_2312_80 :: 1584;
        GBK_95 :: 1585;
        GB_18030_2000 :: 1586;
        KSC_5601_87 :: 1600;
        KSC_5601_92_Johab :: 1601;
        CNS_11643_92_P1 :: 1617;
        CNS_11643_92_P2 :: 1618;
        CNS_11643_92_P3 :: 1619;
        ISO_2022_JP :: 2080;
        ISO_2022_JP_2 :: 2081;
        ISO_2022_JP_1 :: 2082;
        ISO_2022_JP_3 :: 2083;
        ISO_2022_CN :: 2096;
        ISO_2022_CN_EXT :: 2097;
        ISO_2022_KR :: 2112;
        EUC_JP :: 2336;
        EUC_CN :: 2352;
        EUC_TW :: 2353;
        EUC_KR :: 2368;
        ShiftJIS :: 2561;
        KOI8_R :: 2562;
        Big5 :: 2563;
        MacRomanLatin1 :: 2564;
        HZ_GB_2312 :: 2565;
        Big5_HKSCS_1999 :: 2566;
        VISCII :: 2567;
        KOI8_U :: 2568;
        Big5_E :: 2569;
        NextStepJapanese :: 2818;
        EBCDIC_US :: 3073;
        EBCDIC_CP037 :: 3074;
        UTF7 :: 67109120;
        UTF7_IMAP :: 2576;
        ShiftJIS_X0213_00 :: 1576;
        kCFStringEncodingMacJapanese :: MacJapanese;
        kCFStringEncodingMacChineseTrad :: MacChineseTrad;
        kCFStringEncodingMacKorean :: MacKorean;
        kCFStringEncodingMacArabic :: MacArabic;
        kCFStringEncodingMacHebrew :: MacHebrew;
        kCFStringEncodingMacGreek :: MacGreek;
        kCFStringEncodingMacCyrillic :: MacCyrillic;
        kCFStringEncodingMacDevanagari :: MacDevanagari;
        kCFStringEncodingMacGurmukhi :: MacGurmukhi;
        kCFStringEncodingMacGujarati :: MacGujarati;
        kCFStringEncodingMacOriya :: MacOriya;
        kCFStringEncodingMacBengali :: MacBengali;
        kCFStringEncodingMacTamil :: MacTamil;
        kCFStringEncodingMacTelugu :: MacTelugu;
        kCFStringEncodingMacKannada :: MacKannada;
        kCFStringEncodingMacMalayalam :: MacMalayalam;
        kCFStringEncodingMacSinhalese :: MacSinhalese;
        kCFStringEncodingMacBurmese :: MacBurmese;
        kCFStringEncodingMacKhmer :: MacKhmer;
        kCFStringEncodingMacThai :: MacThai;
        kCFStringEncodingMacLaotian :: MacLaotian;
        kCFStringEncodingMacGeorgian :: MacGeorgian;
        kCFStringEncodingMacArmenian :: MacArmenian;
        kCFStringEncodingMacChineseSimp :: MacChineseSimp;
        kCFStringEncodingMacTibetan :: MacTibetan;
        kCFStringEncodingMacMongolian :: MacMongolian;
        kCFStringEncodingMacEthiopic :: MacEthiopic;
        kCFStringEncodingMacCentralEurRoman :: MacCentralEurRoman;
        kCFStringEncodingMacVietnamese :: MacVietnamese;
        kCFStringEncodingMacExtArabic :: MacExtArabic;
        kCFStringEncodingMacSymbol :: MacSymbol;
        kCFStringEncodingMacDingbats :: MacDingbats;
        kCFStringEncodingMacTurkish :: MacTurkish;
        kCFStringEncodingMacCroatian :: MacCroatian;
        kCFStringEncodingMacIcelandic :: MacIcelandic;
        kCFStringEncodingMacRomanian :: MacRomanian;
        kCFStringEncodingMacCeltic :: MacCeltic;
        kCFStringEncodingMacGaelic :: MacGaelic;
        kCFStringEncodingMacFarsi :: MacFarsi;
        kCFStringEncodingMacUkrainian :: MacUkrainian;
        kCFStringEncodingMacInuit :: MacInuit;
        kCFStringEncodingMacVT100 :: MacVT100;
        kCFStringEncodingMacHFS :: MacHFS;
        kCFStringEncodingISOLatin2 :: ISOLatin2;
        kCFStringEncodingISOLatin3 :: ISOLatin3;
        kCFStringEncodingISOLatin4 :: ISOLatin4;
        kCFStringEncodingISOLatinCyrillic :: ISOLatinCyrillic;
        kCFStringEncodingISOLatinArabic :: ISOLatinArabic;
        kCFStringEncodingISOLatinGreek :: ISOLatinGreek;
        kCFStringEncodingISOLatinHebrew :: ISOLatinHebrew;
        kCFStringEncodingISOLatin5 :: ISOLatin5;
        kCFStringEncodingISOLatin6 :: ISOLatin6;
        kCFStringEncodingISOLatinThai :: ISOLatinThai;
        kCFStringEncodingISOLatin7 :: ISOLatin7;
        kCFStringEncodingISOLatin8 :: ISOLatin8;
        kCFStringEncodingISOLatin9 :: ISOLatin9;
        kCFStringEncodingISOLatin10 :: ISOLatin10;
        kCFStringEncodingDOSLatinUS :: DOSLatinUS;
        kCFStringEncodingDOSGreek :: DOSGreek;
        kCFStringEncodingDOSBalticRim :: DOSBalticRim;
        kCFStringEncodingDOSLatin1 :: DOSLatin1;
        kCFStringEncodingDOSGreek1 :: DOSGreek1;
        kCFStringEncodingDOSLatin2 :: DOSLatin2;
        kCFStringEncodingDOSCyrillic :: DOSCyrillic;
        kCFStringEncodingDOSTurkish :: DOSTurkish;
        kCFStringEncodingDOSPortuguese :: DOSPortuguese;
        kCFStringEncodingDOSIcelandic :: DOSIcelandic;
        kCFStringEncodingDOSHebrew :: DOSHebrew;
        kCFStringEncodingDOSCanadianFrench :: DOSCanadianFrench;
        kCFStringEncodingDOSArabic :: DOSArabic;
        kCFStringEncodingDOSNordic :: DOSNordic;
        kCFStringEncodingDOSRussian :: DOSRussian;
        kCFStringEncodingDOSGreek2 :: DOSGreek2;
        kCFStringEncodingDOSThai :: DOSThai;
        kCFStringEncodingDOSJapanese :: DOSJapanese;
        kCFStringEncodingDOSChineseSimplif :: DOSChineseSimplif;
        kCFStringEncodingDOSKorean :: DOSKorean;
        kCFStringEncodingDOSChineseTrad :: DOSChineseTrad;
        kCFStringEncodingWindowsLatin2 :: WindowsLatin2;
        kCFStringEncodingWindowsCyrillic :: WindowsCyrillic;
        kCFStringEncodingWindowsGreek :: WindowsGreek;
        kCFStringEncodingWindowsLatin5 :: WindowsLatin5;
        kCFStringEncodingWindowsHebrew :: WindowsHebrew;
        kCFStringEncodingWindowsArabic :: WindowsArabic;
        kCFStringEncodingWindowsBalticRim :: WindowsBalticRim;
        kCFStringEncodingWindowsVietnamese :: WindowsVietnamese;
        kCFStringEncodingWindowsKoreanJohab :: WindowsKoreanJohab;
        kCFStringEncodingANSEL :: ANSEL;
        kCFStringEncodingJIS_X0201_76 :: JIS_X0201_76;
        kCFStringEncodingJIS_X0208_83 :: JIS_X0208_83;
        kCFStringEncodingJIS_X0208_90 :: JIS_X0208_90;
        kCFStringEncodingJIS_X0212_90 :: JIS_X0212_90;
        kCFStringEncodingJIS_C6226_78 :: JIS_C6226_78;
        kCFStringEncodingShiftJIS_X0213 :: ShiftJIS_X0213;
        kCFStringEncodingShiftJIS_X0213_MenKuTen :: ShiftJIS_X0213_MenKuTen;
        kCFStringEncodingGB_2312_80 :: GB_2312_80;
        kCFStringEncodingGBK_95 :: GBK_95;
        kCFStringEncodingGB_18030_2000 :: GB_18030_2000;
        kCFStringEncodingKSC_5601_87 :: KSC_5601_87;
        kCFStringEncodingKSC_5601_92_Johab :: KSC_5601_92_Johab;
        kCFStringEncodingCNS_11643_92_P1 :: CNS_11643_92_P1;
        kCFStringEncodingCNS_11643_92_P2 :: CNS_11643_92_P2;
        kCFStringEncodingCNS_11643_92_P3 :: CNS_11643_92_P3;
        kCFStringEncodingISO_2022_JP :: ISO_2022_JP;
        kCFStringEncodingISO_2022_JP_2 :: ISO_2022_JP_2;
        kCFStringEncodingISO_2022_JP_1 :: ISO_2022_JP_1;
        kCFStringEncodingISO_2022_JP_3 :: ISO_2022_JP_3;
        kCFStringEncodingISO_2022_CN :: ISO_2022_CN;
        kCFStringEncodingISO_2022_CN_EXT :: ISO_2022_CN_EXT;
        kCFStringEncodingISO_2022_KR :: ISO_2022_KR;
        kCFStringEncodingEUC_JP :: EUC_JP;
        kCFStringEncodingEUC_CN :: EUC_CN;
        kCFStringEncodingEUC_TW :: EUC_TW;
        kCFStringEncodingEUC_KR :: EUC_KR;
        kCFStringEncodingShiftJIS :: ShiftJIS;
        kCFStringEncodingKOI8_R :: KOI8_R;
        kCFStringEncodingBig5 :: Big5;
        kCFStringEncodingMacRomanLatin1 :: MacRomanLatin1;
        kCFStringEncodingHZ_GB_2312 :: HZ_GB_2312;
        kCFStringEncodingBig5_HKSCS_1999 :: Big5_HKSCS_1999;
        kCFStringEncodingVISCII :: VISCII;
        kCFStringEncodingKOI8_U :: KOI8_U;
        kCFStringEncodingBig5_E :: Big5_E;
        kCFStringEncodingNextStepJapanese :: NextStepJapanese;
        kCFStringEncodingEBCDIC_US :: EBCDIC_US;
        kCFStringEncodingEBCDIC_CP037 :: EBCDIC_CP037;
        kCFStringEncodingUTF7 :: UTF7;
        kCFStringEncodingUTF7_IMAP :: UTF7_IMAP;
        kCFStringEncodingShiftJIS_X0213_00 :: ShiftJIS_X0213_00;
    }

}

#if OS == .MACOS {
    kCFStringTokenizer :: enum u32 {
        UnitWord :: 0;
        UnitSentence :: 1;
        UnitParagraph :: 2;
        UnitLineBreak :: 3;
        UnitWordBoundary :: 4;
        AttributeLatinTranscription :: 65536;
        AttributeLanguage :: 131072;
        kCFStringTokenizerUnitWord :: UnitWord;
        kCFStringTokenizerUnitSentence :: UnitSentence;
        kCFStringTokenizerUnitParagraph :: UnitParagraph;
        kCFStringTokenizerUnitLineBreak :: UnitLineBreak;
        kCFStringTokenizerUnitWordBoundary :: UnitWordBoundary;
        kCFStringTokenizerAttributeLatinTranscription :: AttributeLatinTranscription;
        kCFStringTokenizerAttributeLanguage :: AttributeLanguage;
    }

}

#if OS == .MACOS {
    kCFStringTokenizerToken :: enum u32 {
        None :: 0;
        Normal :: 1;
        HasSubTokensMask :: 2;
        HasDerivedSubTokensMask :: 4;
        HasHasNumbersMask :: 8;
        HasNonLettersMask :: 16;
        IsCJWordMask :: 32;
        kCFStringTokenizerTokenNone :: None;
        kCFStringTokenizerTokenNormal :: Normal;
        kCFStringTokenizerTokenHasSubTokensMask :: HasSubTokensMask;
        kCFStringTokenizerTokenHasDerivedSubTokensMask :: HasDerivedSubTokensMask;
        kCFStringTokenizerTokenHasHasNumbersMask :: HasHasNumbersMask;
        kCFStringTokenizerTokenHasNonLettersMask :: HasNonLettersMask;
        kCFStringTokenizerTokenIsCJWordMask :: IsCJWordMask;
    }

}

#if OS == .MACOS {
    kCFURL :: enum u32 {
        POSIXPathStyle :: 0;
        HFSPathStyle :: 1;
        WindowsPathStyle :: 2;
        kCFURLPOSIXPathStyle :: POSIXPathStyle;
        kCFURLHFSPathStyle :: HFSPathStyle;
        kCFURLWindowsPathStyle :: WindowsPathStyle;
    }

}

#if OS == .MACOS {
    kCFURL_1 :: enum s32 {
        UnknownError :: -10;
        UnknownSchemeError :: -11;
        ResourceNotFoundError :: -12;
        ResourceAccessViolationError :: -13;
        RemoteHostUnavailableError :: -14;
        ImproperArgumentsError :: -15;
        UnknownPropertyKeyError :: -16;
        PropertyKeyUnavailableError :: -17;
        TimeoutError :: -18;
        kCFURLUnknownError :: UnknownError;
        kCFURLUnknownSchemeError :: UnknownSchemeError;
        kCFURLResourceNotFoundError :: ResourceNotFoundError;
        kCFURLResourceAccessViolationError :: ResourceAccessViolationError;
        kCFURLRemoteHostUnavailableError :: RemoteHostUnavailableError;
        kCFURLImproperArgumentsError :: ImproperArgumentsError;
        kCFURLUnknownPropertyKeyError :: UnknownPropertyKeyError;
        kCFURLPropertyKeyUnavailableError :: PropertyKeyUnavailableError;
        kCFURLTimeoutError :: TimeoutError;
    }

}

#if OS == .MACOS {
    kCFURLBookmarkCreation :: enum u32 {
        MinimalBookmarkMask :: 512;
        SuitableForBookmarkFile :: 1024;
        WithSecurityScope :: 2048;
        SecurityScopeAllowOnlyReadAccess :: 4096;
        PreferFileIDResolutionMask :: 256;
        kCFURLBookmarkCreationMinimalBookmarkMask :: MinimalBookmarkMask;
        kCFURLBookmarkCreationSuitableForBookmarkFile :: SuitableForBookmarkFile;
        kCFURLBookmarkCreationWithSecurityScope :: WithSecurityScope;
        kCFURLBookmarkCreationSecurityScopeAllowOnlyReadAccess :: SecurityScopeAllowOnlyReadAccess;
        kCFURLBookmarkCreationPreferFileIDResolutionMask :: PreferFileIDResolutionMask;
    }

}

#if OS == .MACOS {
    kCFURLComponent :: enum u32 {
        Scheme :: 1;
        NetLocation :: 2;
        Path :: 3;
        ResourceSpecifier :: 4;
        User :: 5;
        Password :: 6;
        UserInfo :: 7;
        Host :: 8;
        Port :: 9;
        ParameterString :: 10;
        Query :: 11;
        Fragment :: 12;
        kCFURLComponentScheme :: Scheme;
        kCFURLComponentNetLocation :: NetLocation;
        kCFURLComponentPath :: Path;
        kCFURLComponentResourceSpecifier :: ResourceSpecifier;
        kCFURLComponentUser :: User;
        kCFURLComponentPassword :: Password;
        kCFURLComponentUserInfo :: UserInfo;
        kCFURLComponentHost :: Host;
        kCFURLComponentPort :: Port;
        kCFURLComponentParameterString :: ParameterString;
        kCFURLComponentQuery :: Query;
        kCFURLComponentFragment :: Fragment;
    }

}

#if OS == .MACOS {
    kCFURLEnumerator :: enum u32 {
        DefaultBehavior :: 0;
        DescendRecursively :: 1;
        SkipInvisibles :: 2;
        GenerateFileReferenceURLs :: 4;
        SkipPackageContents :: 8;
        IncludeDirectoriesPreOrder :: 16;
        IncludeDirectoriesPostOrder :: 32;
        kCFURLEnumeratorDefaultBehavior :: DefaultBehavior;
        kCFURLEnumeratorDescendRecursively :: DescendRecursively;
        kCFURLEnumeratorSkipInvisibles :: SkipInvisibles;
        kCFURLEnumeratorGenerateFileReferenceURLs :: GenerateFileReferenceURLs;
        kCFURLEnumeratorSkipPackageContents :: SkipPackageContents;
        kCFURLEnumeratorIncludeDirectoriesPreOrder :: IncludeDirectoriesPreOrder;
        kCFURLEnumeratorIncludeDirectoriesPostOrder :: IncludeDirectoriesPostOrder;
    }

}

#if OS == .MACOS {
    kCFURLEnumerator_1 :: enum u32 {
        Success :: 1;
        End :: 2;
        Error :: 3;
        DirectoryPostOrderSuccess :: 4;
        kCFURLEnumeratorSuccess :: Success;
        kCFURLEnumeratorEnd :: End;
        kCFURLEnumeratorError :: Error;
        kCFURLEnumeratorDirectoryPostOrderSuccess :: DirectoryPostOrderSuccess;
    }

}

#if OS == .MACOS {
    kCFUserNotification :: enum u32 {
        StopAlertLevel :: 0;
        NoteAlertLevel :: 1;
        CautionAlertLevel :: 2;
        PlainAlertLevel :: 3;
        kCFUserNotificationStopAlertLevel :: StopAlertLevel;
        kCFUserNotificationNoteAlertLevel :: NoteAlertLevel;
        kCFUserNotificationCautionAlertLevel :: CautionAlertLevel;
        kCFUserNotificationPlainAlertLevel :: PlainAlertLevel;
    }

}

#if OS == .MACOS {
    kCFUserNotification_1 :: enum u32 {
        DefaultResponse :: 0;
        AlternateResponse :: 1;
        OtherResponse :: 2;
        CancelResponse :: 3;
        kCFUserNotificationDefaultResponse :: DefaultResponse;
        kCFUserNotificationAlternateResponse :: AlternateResponse;
        kCFUserNotificationOtherResponse :: OtherResponse;
        kCFUserNotificationCancelResponse :: CancelResponse;
    }

}

#if OS == .MACOS {
    kCFUserNotification_2 :: enum u32 {
        NoDefaultButtonFlag :: 32;
        UseRadioButtonsFlag :: 64;
        kCFUserNotificationNoDefaultButtonFlag :: NoDefaultButtonFlag;
        kCFUserNotificationUseRadioButtonsFlag :: UseRadioButtonsFlag;
    }

}

#if OS == .MACOS {
    KERN :: enum u32 {
        SUCCESS :: 0;
        INVALID_ADDRESS :: 1;
        PROTECTION_FAILURE :: 2;
        NO_SPACE :: 3;
        INVALID_ARGUMENT :: 4;
        FAILURE :: 5;
        RESOURCE_SHORTAGE :: 6;
        NOT_RECEIVER :: 7;
        NO_ACCESS :: 8;
        MEMORY_FAILURE :: 9;
        MEMORY_ERROR :: 10;
        ALREADY_IN_SET :: 11;
        NOT_IN_SET :: 12;
        NAME_EXISTS :: 13;
        ABORTED :: 14;
        INVALID_NAME :: 15;
        INVALID_TASK :: 16;
        INVALID_RIGHT :: 17;
        INVALID_VALUE :: 18;
        UREFS_OVERFLOW :: 19;
        INVALID_CAPABILITY :: 20;
        RIGHT_EXISTS :: 21;
        INVALID_HOST :: 22;
        MEMORY_PRESENT :: 23;
        MEMORY_DATA_MOVED :: 24;
        MEMORY_RESTART_COPY :: 25;
        INVALID_PROCESSOR_SET :: 26;
        POLICY_LIMIT :: 27;
        INVALID_POLICY :: 28;
        INVALID_OBJECT :: 29;
        ALREADY_WAITING :: 30;
        DEFAULT_SET :: 31;
        EXCEPTION_PROTECTED :: 32;
        INVALID_LEDGER :: 33;
        INVALID_MEMORY_CONTROL :: 34;
        INVALID_SECURITY :: 35;
        NOT_DEPRESSED :: 36;
        TERMINATED :: 37;
        LOCK_SET_DESTROYED :: 38;
        LOCK_UNSTABLE :: 39;
        LOCK_OWNED :: 40;
        LOCK_OWNED_SELF :: 41;
        SEMAPHORE_DESTROYED :: 42;
        RPC_SERVER_TERMINATED :: 43;
        RPC_TERMINATE_ORPHAN :: 44;
        RPC_CONTINUE_ORPHAN :: 45;
        NOT_SUPPORTED :: 46;
        NODE_DOWN :: 47;
        NOT_WAITING :: 48;
        OPERATION_TIMED_OUT :: 49;
        CODESIGN_ERROR :: 50;
        POLICY_STATIC :: 51;
        INSUFFICIENT_BUFFER_SIZE :: 52;
        RETURN_MAX :: 0x100;
        CODE_SELECTOR :: 0x0008;
        DATA_SELECTOR :: 0x0010;
        KERN_SUCCESS :: SUCCESS;
        KERN_INVALID_ADDRESS :: INVALID_ADDRESS;
        KERN_PROTECTION_FAILURE :: PROTECTION_FAILURE;
        KERN_NO_SPACE :: NO_SPACE;
        KERN_INVALID_ARGUMENT :: INVALID_ARGUMENT;
        KERN_FAILURE :: FAILURE;
        KERN_RESOURCE_SHORTAGE :: RESOURCE_SHORTAGE;
        KERN_NOT_RECEIVER :: NOT_RECEIVER;
        KERN_NO_ACCESS :: NO_ACCESS;
        KERN_MEMORY_FAILURE :: MEMORY_FAILURE;
        KERN_MEMORY_ERROR :: MEMORY_ERROR;
        KERN_ALREADY_IN_SET :: ALREADY_IN_SET;
        KERN_NOT_IN_SET :: NOT_IN_SET;
        KERN_NAME_EXISTS :: NAME_EXISTS;
        KERN_ABORTED :: ABORTED;
        KERN_INVALID_NAME :: INVALID_NAME;
        KERN_INVALID_TASK :: INVALID_TASK;
        KERN_INVALID_RIGHT :: INVALID_RIGHT;
        KERN_INVALID_VALUE :: INVALID_VALUE;
        KERN_UREFS_OVERFLOW :: UREFS_OVERFLOW;
        KERN_INVALID_CAPABILITY :: INVALID_CAPABILITY;
        KERN_RIGHT_EXISTS :: RIGHT_EXISTS;
        KERN_INVALID_HOST :: INVALID_HOST;
        KERN_MEMORY_PRESENT :: MEMORY_PRESENT;
        KERN_MEMORY_DATA_MOVED :: MEMORY_DATA_MOVED;
        KERN_MEMORY_RESTART_COPY :: MEMORY_RESTART_COPY;
        KERN_INVALID_PROCESSOR_SET :: INVALID_PROCESSOR_SET;
        KERN_POLICY_LIMIT :: POLICY_LIMIT;
        KERN_INVALID_POLICY :: INVALID_POLICY;
        KERN_INVALID_OBJECT :: INVALID_OBJECT;
        KERN_ALREADY_WAITING :: ALREADY_WAITING;
        KERN_DEFAULT_SET :: DEFAULT_SET;
        KERN_EXCEPTION_PROTECTED :: EXCEPTION_PROTECTED;
        KERN_INVALID_LEDGER :: INVALID_LEDGER;
        KERN_INVALID_MEMORY_CONTROL :: INVALID_MEMORY_CONTROL;
        KERN_INVALID_SECURITY :: INVALID_SECURITY;
        KERN_NOT_DEPRESSED :: NOT_DEPRESSED;
        KERN_TERMINATED :: TERMINATED;
        KERN_LOCK_SET_DESTROYED :: LOCK_SET_DESTROYED;
        KERN_LOCK_UNSTABLE :: LOCK_UNSTABLE;
        KERN_LOCK_OWNED :: LOCK_OWNED;
        KERN_LOCK_OWNED_SELF :: LOCK_OWNED_SELF;
        KERN_SEMAPHORE_DESTROYED :: SEMAPHORE_DESTROYED;
        KERN_RPC_SERVER_TERMINATED :: RPC_SERVER_TERMINATED;
        KERN_RPC_TERMINATE_ORPHAN :: RPC_TERMINATE_ORPHAN;
        KERN_RPC_CONTINUE_ORPHAN :: RPC_CONTINUE_ORPHAN;
        KERN_NOT_SUPPORTED :: NOT_SUPPORTED;
        KERN_NODE_DOWN :: NODE_DOWN;
        KERN_NOT_WAITING :: NOT_WAITING;
        KERN_OPERATION_TIMED_OUT :: OPERATION_TIMED_OUT;
        KERN_CODESIGN_ERROR :: CODESIGN_ERROR;
        KERN_POLICY_STATIC :: POLICY_STATIC;
        KERN_INSUFFICIENT_BUFFER_SIZE :: INSUFFICIENT_BUFFER_SIZE;
        KERN_RETURN_MAX :: RETURN_MAX;
        KERN_CODE_SELECTOR :: CODE_SELECTOR;
        KERN_DATA_SELECTOR :: DATA_SELECTOR;
    }

}

#if OS == .MACOS {
    Kevent64 :: struct {
        ident: u64;
        filter: Kevent_Filter;
        flags: Kevent_Flags;
        fflags: Kevent_Fflags;
        data: s64;
        udata: *void;
        ext: [2] u64;
    }

}

#if OS == .MACOS {
    Kevent_Call_Flags :: enum_flags u32 {
        KEVENT_FLAG_NONE :: 0x000000;
        KEVENT_FLAG_IMMEDIATE :: 0x000001;
        KEVENT_FLAG_ERROR_EVENTS :: 0x000002;
    }

}

#if OS == .MACOS {
    Kevent_Fflags :: enum_flags u32 {
        LOWAT :: 0x00000001;
        EXIT :: 0x80000000;
        FORK :: 0x40000000;
        EXEC :: 0x20000000;
        SIGNAL :: 0x08000000;
        EXITSTATUS :: 0x04000000;
        EXIT_DETAIL :: 0x02000000;
        PDATAMASK :: 0x000fffff;
        PCTRLMASK :: (~PDATAMASK);
        EXIT_DETAIL_MASK :: 0x00070000;
        EXIT_DECRYPTFAIL :: 0x00010000;
        EXIT_MEMORY :: 0x00020000;
        EXIT_CSERROR :: 0x00040000;
        VM_PRESSURE :: 0x80000000;
        VM_PRESSURE_TERMINATE :: 0x40000000;
        VM_PRESSURE_SUDDEN_TERMINATE :: 0x20000000;
        VM_ERROR :: 0x10000000;
        SECONDS :: 0x00000001;
        USECONDS :: 0x00000002;
        NSECONDS :: 0x00000004;
        ABSOLUTE :: 0x00000008;
        LEEWAY :: 0x00000010;
        CRITICAL :: 0x00000020;
        BACKGROUND :: 0x00000040;
        MACH_CONTINUOUS_TIME :: 0x00000080;
        MACHTIME :: 0x00000100;
        TRIGGER :: 0x01000000;
        FFNOP :: 0x00000000;
        FFAND :: 0x40000000;
        FFOR :: 0x80000000;
        FFCOPY :: 0xc0000000;
        FFCTRLMASK :: 0xc0000000;
        FFLAGSMASK :: 0x00ffffff;
    }

}

#if OS == .MACOS {
    Kevent_Filter :: enum s16 {
        READ :: -1;
        WRITE :: -2;
        AIO :: -3;
        VNODE :: -4;
        PROC :: -5;
        SIGNAL :: -6;
        TIMER :: -7;
        MACHPORT :: -8;
        FS :: -9;
        USER :: -10;
        VM :: -12;
        EXCEPT :: -15;
    }

}

#if OS == .MACOS {
    Kevent_Flags :: enum_flags u16 {
        ADD :: 0x0001;
        DELETE :: 0x0002;
        ENABLE :: 0x0004;
        DISABLE :: 0x0008;
        ONESHOT :: 0x0010;
        CLEAR :: 0x0020;
        RECEIPT :: 0x0040;
        DISPATCH :: 0x0080;
        UDATA_SPECIFIC :: 0x0100;
        DISPATCH2 :: (DISPATCH | UDATA_SPECIFIC);
        VANISHED :: 0x0200;
        SYSFLAGS :: 0xF000;
        FLAG0 :: 0x1000;
        FLAG1 :: 0x2000;
        ERROR :: 0x4000;
        EOF :: 0x8000;
        POLL :: FLAG0;
        OOBAND :: FLAG1;
    }

}

#if OS == .MACOS {
    mach_memory_info :: struct {
        flags: u64;
        site: u64;
        size: u64;
        free: u64;
        largest: u64;
        collectable_bytes: u64;
        mapped: u64;
        peak: u64;
        tag: u16;
        zone: u16;
        _resvA: [2] u16;
        _resv: [3] u64;
        name: [80] u8;
    }

}

#if OS == .MACOS {
    mach_msg_audit_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
        msgh_seqno: mach_port_seqno_t;
        msgh_sender: security_token_t;
        msgh_audit: audit_token_t;
    }

}

#if OS == .MACOS {
    mach_msg_base_t :: struct {
        header: mach_msg_header_t;
        body: mach_msg_body_t;
    }

}

#if OS == .MACOS {
    mach_msg_body_t :: struct {
        msgh_descriptor_count: mach_msg_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_context_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
        msgh_seqno: mach_port_seqno_t;
        msgh_sender: security_token_t;
        msgh_audit: audit_token_t;
        msgh_context: mach_port_context_t #align 4;
    }

}

#if OS == .MACOS {
    mach_msg_descriptor_t :: struct {
        out_of_line: mach_msg_ool_descriptor_t;
        #overlay (out_of_line) port: mach_msg_port_descriptor_t;
        #overlay (out_of_line) ool_ports: mach_msg_ool_ports_descriptor_t;
        #overlay (out_of_line) type: mach_msg_type_descriptor_t;
    }

}

#if OS == .MACOS {
    mach_msg_empty_rcv_t :: struct {
        header: mach_msg_header_t;
        trailer: mach_msg_trailer_t;
    }

}

#if OS == .MACOS {
    mach_msg_empty_send_t :: struct {
        header: mach_msg_header_t;
    }

}

#if OS == .MACOS {
    mach_msg_empty_t :: struct {
        rcv: mach_msg_empty_rcv_t;
        #overlay (rcv) send: mach_msg_empty_send_t;
    }

}

#if OS == .MACOS {
    mach_msg_header_t :: struct {
        msgh_bits: mach_msg_bits_t;
        msgh_size: mach_msg_size_t;
        msgh_remote_port: mach_port_t;
        msgh_local_port: mach_port_t;
        msgh_voucher_port: mach_port_name_t;
        msgh_id: mach_msg_id_t;
    }

}

#if OS == .MACOS {
    mach_msg_mac_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
        msgh_seqno: mach_port_seqno_t;
        msgh_sender: security_token_t;
        msgh_audit: audit_token_t;
        msgh_context: mach_port_context_t #align 4;
        msgh_ad: s32;
        msgh_labels: msg_labels_t;
    }

}

#if OS == .MACOS {
    mach_msg_ool_descriptor32_t :: struct {
        address: u32;
        size: mach_msg_size_t;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        pad1: u32;
        type: mach_msg_descriptor_type_t;;
    }

}

#if OS == .MACOS {
    mach_msg_ool_descriptor64_t :: struct {
        address: u64 #align 4;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        pad1: u32;
        type: mach_msg_descriptor_type_t;;
        size: mach_msg_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_ool_descriptor_t :: struct {
        address: *void #align 4;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        pad1: u32;
        type: mach_msg_descriptor_type_t;;
        size: mach_msg_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_ool_ports_descriptor32_t :: struct {
        address: u32;
        count: mach_msg_size_t;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        disposition: mach_msg_type_name_t;
        type: mach_msg_descriptor_type_t;;
    }

}

#if OS == .MACOS {
    mach_msg_ool_ports_descriptor64_t :: struct {
        address: u64 #align 4;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        disposition: mach_msg_type_name_t;
        type: mach_msg_descriptor_type_t;;
        count: mach_msg_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_ool_ports_descriptor_t :: struct {
        address: *void #align 4;
        __bitfield: u32;
        copy: mach_msg_copy_options_t;
        disposition: mach_msg_type_name_t;
        type: mach_msg_descriptor_type_t;;
        count: mach_msg_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_port_descriptor_t :: struct {
        name: mach_port_t;
        pad1: mach_msg_size_t;
        __bitfield: u32;
    }

}

#if OS == .MACOS {
    mach_msg_security_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
        msgh_seqno: mach_port_seqno_t;
        msgh_sender: security_token_t;
    }

}

#if OS == .MACOS {
    mach_msg_seqno_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
        msgh_seqno: mach_port_seqno_t;
    }

}

#if OS == .MACOS {
    mach_msg_trailer_t :: struct {
        msgh_trailer_type: mach_msg_trailer_type_t;
        msgh_trailer_size: mach_msg_trailer_size_t;
    }

}

#if OS == .MACOS {
    mach_msg_type_descriptor_t :: struct {
        pad1: u32;
        pad2: mach_msg_size_t;
        __bitfield: u32;
    }

}

#if OS == .MACOS {
    mach_port_guard_exception_codes :: enum_flags u32 {
        DESTROY :: 0x1;
        MOD_REFS :: 0x2;
        SET_CONTEXT :: 0x4;
        UNGUARDED :: 0x8;
        INCORRECT_GUARD :: 0x10;
        kGUARD_EXC_DESTROY :: DESTROY;
        kGUARD_EXC_MOD_REFS :: MOD_REFS;
        kGUARD_EXC_SET_CONTEXT :: SET_CONTEXT;
        kGUARD_EXC_UNGUARDED :: UNGUARDED;
        kGUARD_EXC_INCORRECT_GUARD :: INCORRECT_GUARD;
    }

}

#if OS == .MACOS {
    mach_port_info_ext :: struct {
        mpie_status: mach_port_status_t;
        mpie_boost_cnt: mach_port_msgcount_t;
        reserved: [6] u32;
    }

}

#if OS == .MACOS {
    mach_port_limits :: struct {
        mpl_qlimit: mach_port_msgcount_t;
    }

}

#if OS == .MACOS {
    mach_port_options :: struct {
        flags: u32;
        mpl: mach_port_limits_t;
        reserved: [2] u64;
    }

}

#if OS == .MACOS {
    mach_port_qos :: struct {
        __bitfield: u32;
        prealloc: u32;
        pad1: u32;;
        len: u32;
    }

}

#if OS == .MACOS {
    mach_port_status :: struct {
        mps_pset: mach_port_rights_t;
        mps_seqno: mach_port_seqno_t;
        mps_mscount: mach_port_mscount_t;
        mps_qlimit: mach_port_msgcount_t;
        mps_msgcount: mach_port_msgcount_t;
        mps_sorights: mach_port_rights_t;
        mps_srights: u32;
        mps_pdrequest: u32;
        mps_nsrequest: u32;
        mps_flags: u32;
    }

}

#if OS == .MACOS {
    mach_task_basic_info :: struct {
        virtual_size: mach_vm_size_t #align 4;
        resident_size: mach_vm_size_t #align 4;
        resident_size_max: mach_vm_size_t #align 4;
        user_time: time_value_t;
        system_time: time_value_t;
        policy: policy_t;
        suspend_count: s32;
    }

}

#if OS == .MACOS {
    mach_timespec :: struct {
        tv_sec: u32;
        tv_nsec: clock_res_t;
    }

}

#if OS == .MACOS {
    mach_voucher_attr_recipe_data :: struct {
        key: mach_voucher_attr_key_t #align 1;
        command: mach_voucher_attr_recipe_command_t #align 1;
        previous_voucher: mach_voucher_name_t #align 1;
        content_size: mach_voucher_attr_content_size_t #align 1;
    }

}

#if OS == .MACOS {
    mach_zone_info_data :: struct {
        mzi_count: u64;
        mzi_cur_size: u64;
        mzi_max_size: u64;
        mzi_elem_size: u64;
        mzi_alloc_size: u64;
        mzi_sum_size: u64;
        mzi_exhaustible: u64;
        mzi_collectable: u64;
    }

}

#if OS == .MACOS {
    mach_zone_name :: struct {
        mzn_name: [80] u8;
    }

}

#if OS == .MACOS {
    msg_labels_t :: struct {
        sender: mach_port_name_t;
    }

}

#if OS == .MACOS {
    NDR_record_t :: struct {
        mig_vers: u8;
        if_vers: u8;
        reserved1: u8;
        mig_encoding: u8;
        int_rep: u8;
        char_rep: u8;
        float_rep: u8;
        reserved2: u8;
    }

}

#if OS == .MACOS {
    policy_bases :: struct {
        ts: policy_timeshare_base_data_t;
        rr: policy_rr_base_data_t;
        fifo: policy_fifo_base_data_t;
    }

}

#if OS == .MACOS {
    policy_fifo_base :: struct {
        base_priority: s32;
    }

}

#if OS == .MACOS {
    policy_fifo_info :: struct {
        max_priority: s32;
        base_priority: s32;
        depressed: u32;
        depress_priority: s32;
    }

}

#if OS == .MACOS {
    policy_fifo_limit :: struct {
        max_priority: s32;
    }

}

#if OS == .MACOS {
    policy_infos :: struct {
        ts: policy_timeshare_info_data_t;
        rr: policy_rr_info_data_t;
        fifo: policy_fifo_info_data_t;
    }

}

#if OS == .MACOS {
    policy_limits :: struct {
        ts: policy_timeshare_limit_data_t;
        rr: policy_rr_limit_data_t;
        fifo: policy_fifo_limit_data_t;
    }

}

#if OS == .MACOS {
    policy_rr_base :: struct {
        base_priority: s32;
        quantum: s32;
    }

}

#if OS == .MACOS {
    policy_rr_info :: struct {
        max_priority: s32;
        base_priority: s32;
        quantum: s32;
        depressed: u32;
        depress_priority: s32;
    }

}

#if OS == .MACOS {
    policy_rr_limit :: struct {
        max_priority: s32;
    }

}

#if OS == .MACOS {
    policy_timeshare_base :: struct {
        base_priority: s32;
    }

}

#if OS == .MACOS {
    policy_timeshare_info :: struct {
        max_priority: s32;
        base_priority: s32;
        cur_priority: s32;
        depressed: u32;
        depress_priority: s32;
    }

}

#if OS == .MACOS {
    policy_timeshare_limit :: struct {
        max_priority: s32;
    }

}

#if OS == .MACOS {
    security_token_t :: struct {
        val: [2] u32;
    }

}

#if OS == .MACOS {
    task_absolutetime_info :: struct {
        total_user: u64 #align 4;
        total_system: u64 #align 4;
        threads_user: u64 #align 4;
        threads_system: u64 #align 4;
    }

}

#if OS == .MACOS {
    task_affinity_tag_info :: struct {
        set_count: s32;
        min: s32;
        max: s32;
        task_count: s32;
    }

}

#if OS == .MACOS {
    task_basic_info :: struct {
        suspend_count: s32;
        virtual_size: vm_size_t #align 4;
        resident_size: vm_size_t #align 4;
        user_time: time_value_t;
        system_time: time_value_t;
        policy: policy_t;
    }

}

#if OS == .MACOS {
    task_basic_info_32 :: struct {
        suspend_count: s32;
        virtual_size: u32;
        resident_size: u32;
        user_time: time_value_t;
        system_time: time_value_t;
        policy: policy_t;
    }

}

#if OS == .MACOS {
    task_basic_info_64 :: struct {
        suspend_count: s32;
        virtual_size: mach_vm_size_t #align 4;
        resident_size: mach_vm_size_t #align 4;
        user_time: time_value_t;
        system_time: time_value_t;
        policy: policy_t;
    }

}

#if OS == .MACOS {
    task_category_policy :: struct {
        role: task_role_t;
    }

}

#if OS == .MACOS {
    task_dyld_info :: struct {
        all_image_info_addr: mach_vm_address_t #align 4;
        all_image_info_size: mach_vm_size_t #align 4;
        all_image_info_format: s32;
    }

}

#if OS == .MACOS {
    task_events_info :: struct {
        faults: s32;
        pageins: s32;
        cow_faults: s32;
        messages_sent: s32;
        messages_received: s32;
        syscalls_mach: s32;
        syscalls_unix: s32;
        csw: s32;
    }

}

#if OS == .MACOS {
    task_extmod_info :: struct {
        task_uuid: [16] u8;
        extmod_statistics: vm_extmod_statistics_data_t #align 4;
    }

}

#if OS == .MACOS {
    task_flags_info :: struct {
        flags: u32;
    }

}

#if OS == .MACOS {
    task_inspect_basic_counts :: struct {
        instructions: u64;
        cycles: u64;
    }

}

#if OS == .MACOS {
    task_inspect_flavor :: enum u32 {
        TASK_INSPECT_BASIC_COUNTS :: 1;
    }

}

#if OS == .MACOS {
    task_kernelmemory_info :: struct {
        total_palloc: u64 #align 4;
        total_pfree: u64 #align 4;
        total_salloc: u64 #align 4;
        total_sfree: u64 #align 4;
    }

}

#if OS == .MACOS {
    task_latency_qos :: enum u32 {
        UNSPECIFIED :: 0;
        _0 :: 16711681;
        _1 :: 16711682;
        _2 :: 16711683;
        _3 :: 16711684;
        _4 :: 16711685;
        _5 :: 16711686;
        LATENCY_QOS_TIER_UNSPECIFIED :: UNSPECIFIED;
        LATENCY_QOS_TIER_0 :: _0;
        LATENCY_QOS_TIER_1 :: _1;
        LATENCY_QOS_TIER_2 :: _2;
        LATENCY_QOS_TIER_3 :: _3;
        LATENCY_QOS_TIER_4 :: _4;
        LATENCY_QOS_TIER_5 :: _5;
    }

}

#if OS == .MACOS {
    task_power_info :: struct {
        total_user: u64 #align 4;
        total_system: u64 #align 4;
        task_interrupt_wakeups: u64 #align 4;
        task_platform_idle_wakeups: u64 #align 4;
        task_timer_wakeups_bin_1: u64 #align 4;
        task_timer_wakeups_bin_2: u64 #align 4;
    }

}

#if OS == .MACOS {
    task_power_info_v2 :: struct {
        cpu_energy: task_power_info_data_t;
        gpu_energy: gpu_energy_data;
        task_ptime: u64 #align 4;
        task_pset_switches: u64 #align 4;
    }

}

#if OS == .MACOS {
    task_qos_policy :: struct {
        task_latency_qos_tier: task_latency_qos_t;
        task_throughput_qos_tier: task_throughput_qos_t;
    }

}

#if OS == .MACOS {
    task_role :: enum s32 {
        RENICED :: -1;
        UNSPECIFIED :: 0;
        FOREGROUND_APPLICATION :: 1;
        BACKGROUND_APPLICATION :: 2;
        CONTROL_APPLICATION :: 3;
        GRAPHICS_SERVER :: 4;
        THROTTLE_APPLICATION :: 5;
        NONUI_APPLICATION :: 6;
        DEFAULT_APPLICATION :: 7;
        TASK_RENICED :: RENICED;
        TASK_UNSPECIFIED :: UNSPECIFIED;
        TASK_FOREGROUND_APPLICATION :: FOREGROUND_APPLICATION;
        TASK_BACKGROUND_APPLICATION :: BACKGROUND_APPLICATION;
        TASK_CONTROL_APPLICATION :: CONTROL_APPLICATION;
        TASK_GRAPHICS_SERVER :: GRAPHICS_SERVER;
        TASK_THROTTLE_APPLICATION :: THROTTLE_APPLICATION;
        TASK_NONUI_APPLICATION :: NONUI_APPLICATION;
        TASK_DEFAULT_APPLICATION :: DEFAULT_APPLICATION;
    }

}

#if OS == .MACOS {
    task_thread_times_info :: struct {
        user_time: time_value_t;
        system_time: time_value_t;
    }

}

#if OS == .MACOS {
    task_throughput_qos :: enum u32 {
        UNSPECIFIED :: 0;
        _0 :: 16646145;
        _1 :: 16646146;
        _2 :: 16646147;
        _3 :: 16646148;
        _4 :: 16646149;
        _5 :: 16646150;
        THROUGHPUT_QOS_TIER_UNSPECIFIED :: UNSPECIFIED;
        THROUGHPUT_QOS_TIER_0 :: _0;
        THROUGHPUT_QOS_TIER_1 :: _1;
        THROUGHPUT_QOS_TIER_2 :: _2;
        THROUGHPUT_QOS_TIER_3 :: _3;
        THROUGHPUT_QOS_TIER_4 :: _4;
        THROUGHPUT_QOS_TIER_5 :: _5;
    }

}

#if OS == .MACOS {
    task_trace_memory_info :: struct {
        user_memory_address: u64 #align 4;
        buffer_size: u64 #align 4;
        mailbox_array_size: u64 #align 4;
    }

}

#if OS == .MACOS {
    task_vm_info :: struct {
        virtual_size: mach_vm_size_t #align 4;
        region_count: s32;
        page_size: s32;
        resident_size: mach_vm_size_t #align 4;
        resident_size_peak: mach_vm_size_t #align 4;
        device: mach_vm_size_t #align 4;
        device_peak: mach_vm_size_t #align 4;
        internal: mach_vm_size_t #align 4;
        internal_peak: mach_vm_size_t #align 4;
        external: mach_vm_size_t #align 4;
        external_peak: mach_vm_size_t #align 4;
        reusable: mach_vm_size_t #align 4;
        reusable_peak: mach_vm_size_t #align 4;
        purgeable_volatile_pmap: mach_vm_size_t #align 4;
        purgeable_volatile_resident: mach_vm_size_t #align 4;
        purgeable_volatile_virtual: mach_vm_size_t #align 4;
        compressed: mach_vm_size_t #align 4;
        compressed_peak: mach_vm_size_t #align 4;
        compressed_lifetime: mach_vm_size_t #align 4;
        phys_footprint: mach_vm_size_t #align 4;
        min_address: mach_vm_address_t #align 4;
        max_address: mach_vm_address_t #align 4;
    }

}

#if OS == .MACOS {
    task_wait_state_info :: struct {
        total_wait_state_time: u64 #align 4;
        total_wait_sfi_state_time: u64 #align 4;
        _reserved: [4] u32;
    }

}

#if OS == .MACOS {
    task_zone_info_data :: struct {
        tzi_count: u64;
        tzi_cur_size: u64;
        tzi_max_size: u64;
        tzi_elem_size: u64;
        tzi_alloc_size: u64;
        tzi_sum_size: u64;
        tzi_exhaustible: u64;
        tzi_collectable: u64;
        tzi_caller_acct: u64;
        tzi_task_alloc: u64;
        tzi_task_free: u64;
    }

}

#if OS == .MACOS {
    thread_affinity_policy :: struct {
        affinity_tag: s32;
    }

}

#if OS == .MACOS {
    thread_background_policy :: struct {
        priority: s32;
    }

}

#if OS == .MACOS {
    thread_extended_policy :: struct {
        timeshare: u32;
    }

}

#if OS == .MACOS {
    thread_latency_qos_policy :: struct {
        thread_latency_qos_tier: thread_latency_qos_t;
    }

}

#if OS == .MACOS {
    thread_precedence_policy :: struct {
        importance: s32;
    }

}

#if OS == .MACOS {
    thread_standard_policy :: struct {
        no_data: u32;
    }

}

#if OS == .MACOS {
    thread_throughput_qos_policy :: struct {
        thread_throughput_qos_tier: thread_throughput_qos_t;
    }

}

#if OS == .MACOS {
    thread_time_constraint_policy :: struct {
        period: u32;
        computation: u32;
        constraint: u32;
        preemptible: u32;
    }

}

#if OS == .MACOS {
    time_value :: struct {
        seconds: s32;
        microseconds: s32;
    }

}

#if OS == .MACOS {
    vm_extmod_statistics :: struct {
        task_for_pid_count: s64;
        task_for_pid_caller_count: s64;
        thread_creation_count: s64;
        thread_creation_caller_count: s64;
        thread_set_state_count: s64;
        thread_set_state_caller_count: s64;
    }

}

#if OS == .MACOS {
    vm_purgeable_info :: struct {
        fifo_data: [8] vm_purgeable_stat_t;
        obsolete_data: vm_purgeable_stat_t;
        lifo_data: [8] vm_purgeable_stat_t;
    }

}

#if OS == .MACOS {
    vm_purgeable_stat :: struct {
        count: u64;
        size: u64;
    }

}

#if OS == .MACOS {
    vm_statistics :: struct {
        free_count: u32;
        active_count: u32;
        inactive_count: u32;
        wire_count: u32;
        zero_fill_count: u32;
        reactivations: u32;
        pageins: u32;
        pageouts: u32;
        faults: u32;
        cow_faults: u32;
        lookups: u32;
        hits: u32;
        purgeable_count: u32;
        purges: u32;
        speculative_count: u32;
    }

}

#if OS == .MACOS {
    vm_statistics64 :: struct {
        free_count: u32;
        active_count: u32;
        inactive_count: u32;
        wire_count: u32;
        zero_fill_count: u64;
        reactivations: u64;
        pageins: u64;
        pageouts: u64;
        faults: u64;
        cow_faults: u64;
        lookups: u64;
        hits: u64;
        purges: u64;
        purgeable_count: u32;
        speculative_count: u32;
        decompressions: u64;
        compressions: u64;
        swapins: u64;
        swapouts: u64;
        compressor_page_count: u32;
        throttled_count: u32;
        external_page_count: u32;
        internal_page_count: u32;
        total_uncompressed_pages_in_compressor: u64;
    }

}

#if OS == .MACOS {
    zone_info :: struct {
        zi_count: s32;
        zi_cur_size: vm_size_t;
        zi_max_size: vm_size_t;
        zi_elem_size: vm_size_t;
        zi_alloc_size: vm_size_t;
        zi_pageable: s32;
        zi_sleepable: s32;
        zi_exhaustible: s32;
        zi_collectable: s32;
    }

}

#if OS == .MACOS {
    zone_name :: struct {
        zn_name: [80] u8;
    }

}

// Callback and opaque types

#if OS == .MACOS {
    CFAllocatorAllocateCallBack :: #type (allocSize: CFIndex, hint: CFOptionFlags, info: *void) -> *void #c_call;
}

#if OS == .MACOS {
    CFAllocatorCopyDescriptionCallBack :: #type (info: *void) -> CFStringRef #c_call;
}

#if OS == .MACOS {
    CFAllocatorDeallocateCallBack :: #type (ptr: *void, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFAllocatorPreferredSizeCallBack :: #type (size: CFIndex, hint: CFOptionFlags, info: *void) -> CFIndex #c_call;
}

#if OS == .MACOS {
    CFAllocatorReallocateCallBack :: #type (ptr: *void, newsize: CFIndex, hint: CFOptionFlags, info: *void) -> *void #c_call;
}

#if OS == .MACOS {
    CFAllocatorReleaseCallBack :: #type (info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFAllocatorRetainCallBack :: #type (info: *void) -> *void #c_call;
}

#if OS == .MACOS {
    CFArrayApplierFunction :: #type (value: *void, _context: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFArrayCopyDescriptionCallBack :: #type (value: *void) -> CFStringRef #c_call;
}

#if OS == .MACOS {
    CFArrayEqualCallBack :: #type (value1: *void, value2: *void) -> Boolean #c_call;
}

#if OS == .MACOS {
    CFArrayReleaseCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFArrayRetainCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> *void #c_call;
}

#if OS == .MACOS {
    CFComparatorFunction :: #type (val1: *void, val2: *void, _context: *void) -> CFComparisonResult #c_call;
}

#if OS == .MACOS {
    CFDictionaryApplierFunction :: #type (key: *void, value: *void, _context: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFDictionaryCopyDescriptionCallBack :: #type (value: *void) -> CFStringRef #c_call;
}

#if OS == .MACOS {
    CFDictionaryEqualCallBack :: #type (value1: *void, value2: *void) -> Boolean #c_call;
}

#if OS == .MACOS {
    CFDictionaryHashCallBack :: #type (value: *void) -> CFHashCode #c_call;
}

#if OS == .MACOS {
    CFDictionaryReleaseCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFDictionaryRetainCallBack :: #type (allocator: CFAllocatorRef, value: *void) -> *void #c_call;
}

#if OS == .MACOS {
    CFFileDescriptorCallBack :: #type (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFMachPortCallBack :: #type (port: CFMachPortRef, msg: *void, size: CFIndex, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFMachPortInvalidationCallBack :: #type (port: CFMachPortRef, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFMessagePortCallBack :: #type (local: CFMessagePortRef, msgid: s32, data: CFDataRef, info: *void) -> CFDataRef #c_call;
}

#if OS == .MACOS {
    CFMessagePortInvalidationCallBack :: #type (ms: CFMessagePortRef, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFNotificationCallback :: #type (center: CFNotificationCenterRef, observer: *void, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef) -> void #c_call;
}

#if OS == .MACOS {
    CFPlugInDynamicRegisterFunction :: #type (plugIn: CFPlugInRef) -> void #c_call;
}

#if OS == .MACOS {
    CFPlugInFactoryFunction :: #type (allocator: CFAllocatorRef, typeUUID: CFUUIDRef) -> *void #c_call;
}

#if OS == .MACOS {
    CFPlugInInstanceDeallocateInstanceDataFunction :: #type (instanceData: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFPlugInInstanceGetInterfaceFunction :: #type (instance: CFPlugInInstanceRef, interfaceName: CFStringRef, ftbl: **void) -> Boolean #c_call;
}

#if OS == .MACOS {
    CFPlugInUnloadFunction :: #type (plugIn: CFPlugInRef) -> void #c_call;
}

#if OS == .MACOS {
    CFReadStreamClientCallBack :: #type (stream: CFReadStreamRef, type: CFStreamEventType, clientCallBackInfo: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFRunLoopObserverCallBack :: #type (observer: CFRunLoopObserverRef, activity: CFRunLoopActivity, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFRunLoopTimerCallBack :: #type (timer: CFRunLoopTimerRef, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFSocketCallBack :: #type (s: CFSocketRef, type: CFSocketCallBackType, address: CFDataRef, data: *void, info: *void) -> void #c_call;
}

#if OS == .MACOS {
    CFUserNotificationCallBack :: #type (userNotification: CFUserNotificationRef, responseFlags: CFOptionFlags) -> void #c_call;
}

#if OS == .MACOS {
    CFWriteStreamClientCallBack :: #type (stream: CFWriteStreamRef, type: CFStreamEventType, clientCallBackInfo: *void) -> void #c_call;
}

#if OS == .MACOS {
    FSEventStreamCallback :: #type (streamRef: ConstFSEventStreamRef, clientCallBackInfo: *void, numEvents: u64, eventPaths: *void, eventFlags: *FSEventStreamEventFlags, eventIds: *FSEventStreamEventId) #c_call;
}

#if OS == .MACOS {
    FSEventStreamRef :: *struct {};
}

// Aliases and constants

#if OS == .MACOS {
    __kCFStringInlineBufferLength :: 64;
}

#if OS == .MACOS {
    __NDR_convert__ :: 0;
}

#if OS == .MACOS {
    __NDR_convert__char_rep__ :: 0;
}

#if OS == .MACOS {
    __NDR_convert__float_rep__ :: 0;
}

#if OS == .MACOS {
    __NDR_convert__int_rep__ :: __NDR_convert__;
}

#if OS == .MACOS {
    addr64_t :: u64;
}

#if OS == .MACOS {
    ALARM_NULL :: cast(alarm_t) 0;
}

#if OS == .MACOS {
    alarm_port_t :: alarm_t;
}

#if OS == .MACOS {
    alarm_t :: mach_port_t;
}

#if OS == .MACOS {
    alarm_type_t :: s32;
}

#if OS == .MACOS {
    ALRMTYPE :: 0xff;
}

#if OS == .MACOS {
    bootstrap_t :: mach_port_t;
}

#if OS == .MACOS {
    CALENDAR_CLOCK :: 1;
}

#if OS == .MACOS {
    CFAbsoluteTime :: CFTimeInterval;
}

#if OS == .MACOS {
    CFAllocatorRef :: *__CFAllocator;
}

#if OS == .MACOS {
    CFArrayRef :: *__CFArray;
}

#if OS == .MACOS {
    CFAttributedStringRef :: *__CFAttributedString;
}

#if OS == .MACOS {
    CFBit :: u32;
}

#if OS == .MACOS {
    CFBooleanRef :: *__CFBoolean;
}

#if OS == .MACOS {
    CFBundleRef :: *__CFBundle;
}

#if OS == .MACOS {
    CFBundleRefNum :: s32;
}

#if OS == .MACOS {
    CFByteOrder :: CFIndex;
}

#if OS == .MACOS {
    CFCalendarIdentifier :: CFStringRef;
}

#if OS == .MACOS {
    CFCalendarRef :: *__CFCalendar;
}

#if OS == .MACOS {
    CFCharacterSetRef :: *__CFCharacterSet;
}

#if OS == .MACOS {
    CFDataRef :: *__CFData;
}

#if OS == .MACOS {
    CFDateFormatterKey :: CFStringRef;
}

#if OS == .MACOS {
    CFDateFormatterRef :: *__CFDateFormatter;
}

#if OS == .MACOS {
    CFDateFormatterStyle :: CFIndex;
}

#if OS == .MACOS {
    CFDateRef :: *__CFDate;
}

#if OS == .MACOS {
    CFDictionaryRef :: *__CFDictionary;
}

#if OS == .MACOS {
    CFErrorDomain :: CFStringRef;
}

#if OS == .MACOS {
    CFErrorRef :: *__CFError;
}

#if OS == .MACOS {
    CFFileDescriptorNativeDescriptor :: s32;
}

#if OS == .MACOS {
    CFFileDescriptorRef :: *__CFFileDescriptor;
}

#if OS == .MACOS {
    CFGregorianUnitFlags :: CFOptionFlags;
}

#if OS == .MACOS {
    CFHashCode :: u64;
}

#if OS == .MACOS {
    CFIndex :: s64;
}

#if OS == .MACOS {
    CFISO8601DateFormatOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFLocaleIdentifier :: CFStringRef;
}

#if OS == .MACOS {
    CFLocaleKey :: CFStringRef;
}

#if OS == .MACOS {
    CFLocaleRef :: *__CFLocale;
}

#if OS == .MACOS {
    CFMachPortRef :: *__CFMachPort;
}

#if OS == .MACOS {
    CFMessagePortRef :: *__CFMessagePort;
}

#if OS == .MACOS {
    CFMutableArrayRef :: *__CFArray;
}

#if OS == .MACOS {
    CFMutableAttributedStringRef :: *__CFAttributedString;
}

#if OS == .MACOS {
    CFMutableBagRef :: *__CFBag;
}

#if OS == .MACOS {
    CFMutableBitVectorRef :: *__CFBitVector;
}

#if OS == .MACOS {
    CFMutableCharacterSetRef :: *__CFCharacterSet;
}

#if OS == .MACOS {
    CFMutableDataRef :: *__CFData;
}

#if OS == .MACOS {
    CFMutableDictionaryRef :: *__CFDictionary;
}

#if OS == .MACOS {
    CFMutableSetRef :: *__CFSet;
}

#if OS == .MACOS {
    CFMutableStringRef :: *__CFString;
}

#if OS == .MACOS {
    CFNotificationCenterRef :: *__CFNotificationCenter;
}

#if OS == .MACOS {
    CFNotificationName :: CFStringRef;
}

#if OS == .MACOS {
    CFNullRef :: *__CFNull;
}

#if OS == .MACOS {
    CFNumberFormatterKey :: CFStringRef;
}

#if OS == .MACOS {
    CFNumberFormatterOptionFlags :: CFOptionFlags;
}

#if OS == .MACOS {
    CFNumberFormatterPadPosition :: CFIndex;
}

#if OS == .MACOS {
    CFNumberFormatterRef :: *__CFNumberFormatter;
}

#if OS == .MACOS {
    CFNumberFormatterRoundingMode :: CFIndex;
}

#if OS == .MACOS {
    CFNumberFormatterStyle :: CFIndex;
}

#if OS == .MACOS {
    CFNumberRef :: *__CFNumber;
}

#if OS == .MACOS {
    CFNumberType :: CFIndex;
}

#if OS == .MACOS {
    CFOptionFlags :: u64;
}

#if OS == .MACOS {
    CFPlugInInstanceRef :: *__CFPlugInInstance;
}

#if OS == .MACOS {
    CFPlugInRef :: *__CFBundle;
}

#if OS == .MACOS {
    CFPropertyListFormat :: CFIndex;
}

#if OS == .MACOS {
    CFPropertyListMutabilityOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFPropertyListRef :: CFTypeRef;
}

#if OS == .MACOS {
    CFReadStreamRef :: *__CFReadStream;
}

#if OS == .MACOS {
    CFRunLoopMode :: CFStringRef;
}

#if OS == .MACOS {
    CFRunLoopObserverRef :: *__CFRunLoopObserver;
}

#if OS == .MACOS {
    CFRunLoopRef :: *__CFRunLoop;
}

#if OS == .MACOS {
    CFRunLoopSourceRef :: *__CFRunLoopSource;
}

#if OS == .MACOS {
    CFRunLoopTimerRef :: *__CFRunLoopTimer;
}

#if OS == .MACOS {
    CFSocketCallBackType :: CFOptionFlags;
}

#if OS == .MACOS {
    CFSocketError :: CFIndex;
}

#if OS == .MACOS {
    CFSocketNativeHandle :: s32;
}

#if OS == .MACOS {
    CFSocketRef :: *__CFSocket;
}

#if OS == .MACOS {
    CFStreamEventType :: CFOptionFlags;
}

#if OS == .MACOS {
    CFStreamPropertyKey :: CFStringRef;
}

#if OS == .MACOS {
    CFStringBuiltInEncodings :: CFStringEncoding;
}

#if OS == .MACOS {
    CFStringEncodings :: CFIndex;
}

#if OS == .MACOS {
    CFStringRef :: *__CFString;
}

#if OS == .MACOS {
    CFStringTokenizerRef :: *__CFStringTokenizer;
}

#if OS == .MACOS {
    CFStringTokenizerTokenType :: CFOptionFlags;
}

#if OS == .MACOS {
    CFTimeInterval :: float64;
}

#if OS == .MACOS {
    CFTimeZoneRef :: *__CFTimeZone;
}

#if OS == .MACOS {
    CFTypeID :: u64;
}

#if OS == .MACOS {
    CFTypeRef :: *void;
}

#if OS == .MACOS {
    CFURLBookmarkCreationOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFURLBookmarkFileCreationOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFURLBookmarkResolutionOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFURLComponentType :: CFIndex;
}

#if OS == .MACOS {
    CFURLEnumeratorOptions :: CFOptionFlags;
}

#if OS == .MACOS {
    CFURLEnumeratorRef :: *__CFURLEnumerator;
}

#if OS == .MACOS {
    CFURLEnumeratorResult :: CFIndex;
}

#if OS == .MACOS {
    CFURLError :: CFIndex;
}

#if OS == .MACOS {
    CFURLPathStyle :: CFIndex;
}

#if OS == .MACOS {
    CFURLRef :: *__CFURL;
}

#if OS == .MACOS {
    CFUserNotificationRef :: *__CFUserNotification;
}

#if OS == .MACOS {
    CFUUIDRef :: *__CFUUID;
}

#if OS == .MACOS {
    CFWriteStreamRef :: *__CFWriteStream;
}

#if OS == .MACOS {
    CLOCK_ALARM_CURRES :: 3;
}

#if OS == .MACOS {
    CLOCK_ALARM_MAXRES :: 5;
}

#if OS == .MACOS {
    CLOCK_ALARM_MINRES :: 4;
}

#if OS == .MACOS {
    clock_attr_t :: *s32;
}

#if OS == .MACOS {
    clock_ctrl_port_t :: clock_ctrl_t;
}

#if OS == .MACOS {
    clock_ctrl_t :: mach_port_t;
}

#if OS == .MACOS {
    clock_flavor_t :: s32;
}

#if OS == .MACOS {
    CLOCK_GET_TIME_RES :: 1;
}

#if OS == .MACOS {
    clock_id_t :: s32;
}

#if OS == .MACOS {
    clock_reply_t :: mach_port_t;
}

#if OS == .MACOS {
    clock_res_t :: s32;
}

#if OS == .MACOS {
    clock_serv_port_t :: clock_serv_t;
}

#if OS == .MACOS {
    clock_serv_t :: mach_port_t;
}

#if OS == .MACOS {
    COALITION_NULL :: cast(coalition_t) 0;
}

#if OS == .MACOS {
    coalition_t :: mach_port_t;
}

#if OS == .MACOS {
    ConstFSEventStreamRef :: FSEventStreamRef;
}

#if OS == .MACOS {
    CPU_ARCH_ABI64 :: 0x01000000;
}

#if OS == .MACOS {
    CPU_ARCH_MASK :: 0xff000000;
}

#if OS == .MACOS {
    CPU_STATE_IDLE :: 2;
}

#if OS == .MACOS {
    CPU_STATE_MAX :: 4;
}

#if OS == .MACOS {
    CPU_STATE_NICE :: 3;
}

#if OS == .MACOS {
    CPU_STATE_SYSTEM :: 1;
}

#if OS == .MACOS {
    CPU_STATE_USER :: 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM64_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM64_V8 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V4T :: cast(cpu_subtype_t) 5;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V5TEJ :: cast(cpu_subtype_t) 7;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V6 :: cast(cpu_subtype_t) 6;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V6M :: cast(cpu_subtype_t) 14;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7 :: cast(cpu_subtype_t) 9;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7EM :: cast(cpu_subtype_t) 16;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7F :: cast(cpu_subtype_t) 10;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7K :: cast(cpu_subtype_t) 12;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7M :: cast(cpu_subtype_t) 15;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V7S :: cast(cpu_subtype_t) 11;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_V8 :: cast(cpu_subtype_t) 13;
}

#if OS == .MACOS {
    CPU_SUBTYPE_ARM_XSCALE :: cast(cpu_subtype_t) 8;
}

#if OS == .MACOS {
    CPU_SUBTYPE_BIG_ENDIAN :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_HPPA_7100 :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_HPPA_7100LC :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_HPPA_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_I860_860 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_I860_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_INTEL_FAMILY_MAX :: 15;
}

#if OS == .MACOS {
    CPU_SUBTYPE_INTEL_MODEL_ALL :: 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_LIB64 :: 0x80000000;
}

#if OS == .MACOS {
    CPU_SUBTYPE_LITTLE_ENDIAN :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MASK :: 0xff000000;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC68030 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC68030_ONLY :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC68040 :: cast(cpu_subtype_t) 2;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC680x0_ALL :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC88000_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC88100 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC88110 :: cast(cpu_subtype_t) 2;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC98000_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MC98601 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R2000 :: cast(cpu_subtype_t) 5;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R2000a :: cast(cpu_subtype_t) 4;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R2300 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R2600 :: cast(cpu_subtype_t) 2;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R2800 :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R3000 :: cast(cpu_subtype_t) 7;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MIPS_R3000a :: cast(cpu_subtype_t) 6;
}

#if OS == .MACOS {
    CPU_SUBTYPE_MULTIPLE :: cast,trunc(cpu_subtype_t) -1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_601 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_602 :: cast(cpu_subtype_t) 2;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_603 :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_603e :: cast(cpu_subtype_t) 4;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_603ev :: cast(cpu_subtype_t) 5;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_604 :: cast(cpu_subtype_t) 6;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_604e :: cast(cpu_subtype_t) 7;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_620 :: cast(cpu_subtype_t) 8;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_7400 :: cast(cpu_subtype_t) 10;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_7450 :: cast(cpu_subtype_t) 11;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_750 :: cast(cpu_subtype_t) 9;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_970 :: cast(cpu_subtype_t) 100;
}

#if OS == .MACOS {
    CPU_SUBTYPE_POWERPC_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_SPARC_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    cpu_subtype_t :: s32;
}

#if OS == .MACOS {
    CPU_SUBTYPE_UVAXI :: cast(cpu_subtype_t) 5;
}

#if OS == .MACOS {
    CPU_SUBTYPE_UVAXII :: cast(cpu_subtype_t) 6;
}

#if OS == .MACOS {
    CPU_SUBTYPE_UVAXIII :: cast(cpu_subtype_t) 12;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX730 :: cast(cpu_subtype_t) 4;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX750 :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX780 :: cast(cpu_subtype_t) 1;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX785 :: cast(cpu_subtype_t) 2;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX8200 :: cast(cpu_subtype_t) 7;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX8500 :: cast(cpu_subtype_t) 8;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX8600 :: cast(cpu_subtype_t) 9;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX8650 :: cast(cpu_subtype_t) 10;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX8800 :: cast(cpu_subtype_t) 11;
}

#if OS == .MACOS {
    CPU_SUBTYPE_VAX_ALL :: cast(cpu_subtype_t) 0;
}

#if OS == .MACOS {
    CPU_SUBTYPE_X86_64_ALL :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_X86_64_H :: cast(cpu_subtype_t) 8;
}

#if OS == .MACOS {
    CPU_SUBTYPE_X86_ALL :: cast(cpu_subtype_t) 3;
}

#if OS == .MACOS {
    CPU_SUBTYPE_X86_ARCH1 :: cast(cpu_subtype_t) 4;
}

#if OS == .MACOS {
    CPU_THREADTYPE_INTEL_HTT :: cast(cpu_threadtype_t) 1;
}

#if OS == .MACOS {
    CPU_THREADTYPE_NONE :: cast(cpu_threadtype_t) 0;
}

#if OS == .MACOS {
    cpu_threadtype_t :: s32;
}

#if OS == .MACOS {
    CPU_TYPE_ANY :: cast,trunc(cpu_type_t) -1;
}

#if OS == .MACOS {
    CPU_TYPE_ARM :: cast(cpu_type_t) 12;
}

#if OS == .MACOS {
    CPU_TYPE_ARM64 :: CPU_TYPE_ARM | CPU_ARCH_ABI64;
}

#if OS == .MACOS {
    CPU_TYPE_HPPA :: cast(cpu_type_t) 11;
}

#if OS == .MACOS {
    CPU_TYPE_I386 :: CPU_TYPE_X86;
}

#if OS == .MACOS {
    CPU_TYPE_I860 :: cast(cpu_type_t) 15;
}

#if OS == .MACOS {
    CPU_TYPE_MC680x0 :: cast(cpu_type_t) 6;
}

#if OS == .MACOS {
    CPU_TYPE_MC88000 :: cast(cpu_type_t) 13;
}

#if OS == .MACOS {
    CPU_TYPE_MC98000 :: cast(cpu_type_t) 10;
}

#if OS == .MACOS {
    CPU_TYPE_POWERPC :: cast(cpu_type_t) 18;
}

#if OS == .MACOS {
    CPU_TYPE_POWERPC64 :: CPU_TYPE_POWERPC | CPU_ARCH_ABI64;
}

#if OS == .MACOS {
    CPU_TYPE_SPARC :: cast(cpu_type_t) 14;
}

#if OS == .MACOS {
    cpu_type_t :: s32;
}

#if OS == .MACOS {
    CPU_TYPE_VAX :: cast(cpu_type_t) 1;
}

#if OS == .MACOS {
    CPU_TYPE_X86 :: cast(cpu_type_t) 7;
}

#if OS == .MACOS {
    CPU_TYPE_X86_64 :: CPU_TYPE_X86 | CPU_ARCH_ABI64;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_11 :: 0x8ff620d8;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_12 :: 0xbd1b0ae9;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_13 :: 0x0cc90e64;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_14 :: 0x96077ef1;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_15 :: 0xa8511bca;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_9 :: 0xe73283ae;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_CYCLONE :: 0x37a09642;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_HURRICANE :: 0x67ceee93;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_MONSOON_MISTRAL :: 0xe81e7ef6;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_SWIFT :: 0x1e2d6381;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_TWISTER :: 0x92fb37c8;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_TYPHOON :: 0x2c91a47e;
}

#if OS == .MACOS {
    CPUFAMILY_ARM_XSCALE :: 0x53b005f5;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_6_13 :: 0xaa33392b;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_6_23 :: CPUFAMILY_INTEL_PENRYN;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_6_26 :: CPUFAMILY_INTEL_NEHALEM;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_BROADWELL :: 0x582ed09c;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_HASWELL :: 0x10b282dc;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_IVYBRIDGE :: 0x1f65e835;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_KABYLAKE :: 0x0f817246;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_NEHALEM :: 0x6b5a4cd2;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_PENRYN :: 0x78ea4fbc;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_SANDYBRIDGE :: 0x5490b78c;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_SKYLAKE :: 0x37fc219f;
}

#if OS == .MACOS {
    CPUFAMILY_INTEL_WESTMERE :: 0x573b5eec;
}

#if OS == .MACOS {
    CPUFAMILY_POWERPC_G3 :: 0xcee41549;
}

#if OS == .MACOS {
    CPUFAMILY_POWERPC_G4 :: 0x77c184ae;
}

#if OS == .MACOS {
    CPUFAMILY_POWERPC_G5 :: 0xed76d8aa;
}

#if OS == .MACOS {
    CPUFAMILY_UNKNOWN :: 0;
}

#if OS == .MACOS {
    dyld_kernel_image_info_array_t :: *dyld_kernel_image_info_t;
}

#if OS == .MACOS {
    dyld_kernel_image_info_t :: dyld_kernel_image_info;
}

#if OS == .MACOS {
    dyld_kernel_process_info_t :: dyld_kernel_process_info;
}

#if OS == .MACOS {
    emulation_vector_t :: *mach_vm_offset_t;
}

#if OS == .MACOS {
    EXC_ARITHMETIC :: 3;
}

#if OS == .MACOS {
    EXC_BAD_ACCESS :: 1;
}

#if OS == .MACOS {
    EXC_BAD_INSTRUCTION :: 2;
}

#if OS == .MACOS {
    EXC_BREAKPOINT :: 6;
}

#if OS == .MACOS {
    EXC_CORPSE_NOTIFY :: 13;
}

#if OS == .MACOS {
    EXC_CORPSE_VARIANT_BIT :: 0x100;
}

#if OS == .MACOS {
    EXC_CRASH :: 10;
}

#if OS == .MACOS {
    EXC_EMULATION :: 4;
}

#if OS == .MACOS {
    EXC_GUARD :: 12;
}

#if OS == .MACOS {
    EXC_MACF_MAX :: 0x2FFFF;
}

#if OS == .MACOS {
    EXC_MACF_MIN :: 0x20000;
}

#if OS == .MACOS {
    EXC_MACH_SYSCALL :: 8;
}

#if OS == .MACOS {
    EXC_MASK_ARITHMETIC :: 1 << EXC_ARITHMETIC;
}

#if OS == .MACOS {
    EXC_MASK_BAD_ACCESS :: 1 << EXC_BAD_ACCESS;
}

#if OS == .MACOS {
    EXC_MASK_BAD_INSTRUCTION :: 1 << EXC_BAD_INSTRUCTION;
}

#if OS == .MACOS {
    EXC_MASK_BREAKPOINT :: 1 << EXC_BREAKPOINT;
}

#if OS == .MACOS {
    EXC_MASK_CORPSE_NOTIFY :: 1 << EXC_CORPSE_NOTIFY;
}

#if OS == .MACOS {
    EXC_MASK_CRASH :: 1 << EXC_CRASH;
}

#if OS == .MACOS {
    EXC_MASK_EMULATION :: 1 << EXC_EMULATION;
}

#if OS == .MACOS {
    EXC_MASK_GUARD :: 1 << EXC_GUARD;
}

#if OS == .MACOS {
    EXC_MASK_MACH_SYSCALL :: 1 << EXC_MACH_SYSCALL;
}

#if OS == .MACOS {
    EXC_MASK_RESOURCE :: 1 << EXC_RESOURCE;
}

#if OS == .MACOS {
    EXC_MASK_RPC_ALERT :: 1 << EXC_RPC_ALERT;
}

#if OS == .MACOS {
    EXC_MASK_SOFTWARE :: 1 << EXC_SOFTWARE;
}

#if OS == .MACOS {
    EXC_MASK_SYSCALL :: 1 << EXC_SYSCALL;
}

#if OS == .MACOS {
    EXC_RESOURCE :: 11;
}

#if OS == .MACOS {
    EXC_RPC_ALERT :: 9;
}

#if OS == .MACOS {
    EXC_SOFT_SIGNAL :: 0x10003;
}

#if OS == .MACOS {
    EXC_SOFTWARE :: 5;
}

#if OS == .MACOS {
    EXC_SYSCALL :: 7;
}

#if OS == .MACOS {
    exception_behavior_array_t :: *exception_behavior_t;
}

#if OS == .MACOS {
    exception_behavior_t :: s32;
}

#if OS == .MACOS {
    exception_data_t :: *exception_data_type_t;
}

#if OS == .MACOS {
    exception_data_type_t :: s32;
}

#if OS == .MACOS {
    EXCEPTION_DEFAULT :: 1;
}

#if OS == .MACOS {
    exception_flavor_array_t :: *thread_state_flavor_t;
}

#if OS == .MACOS {
    exception_handler_array_t :: *exception_handler_t;
}

#if OS == .MACOS {
    exception_handler_t :: mach_port_t;
}

#if OS == .MACOS {
    exception_mask_array_t :: *exception_mask_t;
}

#if OS == .MACOS {
    exception_mask_t :: u32;
}

#if OS == .MACOS {
    exception_port_arrary_t :: exception_handler_array_t;
}

#if OS == .MACOS {
    exception_port_array_t :: *mach_port_t;
}

#if OS == .MACOS {
    exception_port_t :: exception_handler_t;
}

#if OS == .MACOS {
    EXCEPTION_STATE :: 2;
}

#if OS == .MACOS {
    EXCEPTION_STATE_IDENTITY :: 3;
}

#if OS == .MACOS {
    exception_type_t :: s32;
}

#if OS == .MACOS {
    FIRST_EXCEPTION :: 1;
}

#if OS == .MACOS {
    FSEventStreamCreateFlags :: u32;
}

#if OS == .MACOS {
    FSEventStreamEventFlags :: u32;
}

#if OS == .MACOS {
    FSEventStreamEventId :: u64;
}

#if OS == .MACOS {
    fsid_t :: fsid;
}

#if OS == .MACOS {
    fsobj_id_t :: fsobj_id;
}

#if OS == .MACOS {
    gpu_energy_data_t :: *gpu_energy_data;
}

#if OS == .MACOS {
    GUARD_TYPE_MACH_PORT :: 0x1;
}

#if OS == .MACOS {
    host_name_port_t :: host_t;
}

#if OS == .MACOS {
    host_name_t :: host_t;
}

#if OS == .MACOS {
    HOST_NULL :: cast(host_t) 0;
}

#if OS == .MACOS {
    HOST_PRIV_NULL :: cast(host_priv_t) 0;
}

#if OS == .MACOS {
    host_priv_t :: mach_port_t;
}

#if OS == .MACOS {
    HOST_SECURITY_NULL :: cast(host_security_t) 0;
}

#if OS == .MACOS {
    host_security_t :: mach_port_t;
}

#if OS == .MACOS {
    host_t :: mach_port_t;
}

#if OS == .MACOS {
    io_master_t :: mach_port_t;
}

#if OS == .MACOS {
    ipc_info_name_array_t :: *ipc_info_name_t;
}

#if OS == .MACOS {
    ipc_info_name_t :: ipc_info_name;
}

#if OS == .MACOS {
    ipc_info_space_basic_t :: ipc_info_space_basic;
}

#if OS == .MACOS {
    ipc_info_space_t :: ipc_info_space;
}

#if OS == .MACOS {
    ipc_info_tree_name_array_t :: *ipc_info_tree_name_t;
}

#if OS == .MACOS {
    ipc_info_tree_name_t :: ipc_info_tree_name;
}

#if OS == .MACOS {
    IPC_SPACE_INSPECT_NULL :: cast(ipc_space_inspect_t) 0;
}

#if OS == .MACOS {
    ipc_space_inspect_t :: mach_port_t;
}

#if OS == .MACOS {
    IPC_SPACE_NULL :: cast(ipc_space_t) 0;
}

#if OS == .MACOS {
    ipc_space_port_t :: ipc_space_t;
}

#if OS == .MACOS {
    ipc_space_t :: mach_port_t;
}

#if OS == .MACOS {
    IPC_VOUCHER_ATTR_CONTROL_NULL :: cast(ipc_voucher_attr_control_t) 0;
}

#if OS == .MACOS {
    ipc_voucher_attr_control_t :: mach_port_t;
}

#if OS == .MACOS {
    IPC_VOUCHER_ATTR_MANAGER_NULL :: cast(ipc_voucher_attr_manager_t) 0;
}

#if OS == .MACOS {
    ipc_voucher_attr_manager_t :: mach_port_t;
}

#if OS == .MACOS {
    IPC_VOUCHER_NULL :: cast(ipc_voucher_t) 0;
}

#if OS == .MACOS {
    ipc_voucher_t :: mach_voucher_t;
}

#if OS == .MACOS {
    kCFStringEncodingInvalidId :: 0xffffffff;
}

#if OS == .MACOS {
    kern_return_t :: KERN;
}

#if OS == .MACOS {
    kFSEventStreamCreateFlagFileEvents :: 0x00000010;
}

#if OS == .MACOS {
    kFSEventStreamCreateFlagUseCFTypes :: 0x00000001;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagChangeOwner :: 0x00004000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemCreated :: 0x00000100;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemFinderInfoMod :: 0x00002000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemInodeMetaMod :: 0x00000400;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemIsDir :: 0x00020000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemIsFile :: 0x00010000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemIsSymlink :: 0x00040000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemModified :: 0x00001000;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemRemoved :: 0x00000200;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagItemRenamed :: 0x00000800;
}

#if OS == .MACOS {
    kFSEventStreamEventFlagMustScanSubDirs :: 0x00000001;
}

#if OS == .MACOS {
    labelstr_t :: *u8;
}

#if OS == .MACOS {
    ledger_amount_t :: s64;
}

#if OS == .MACOS {
    ledger_array_t :: *ledger_t;
}

#if OS == .MACOS {
    LEDGER_ITEM_INFINITY :: cast,trunc(ledger_item_t) (~0);
}

#if OS == .MACOS {
    ledger_item_t :: u32;
}

#if OS == .MACOS {
    LEDGER_LIMIT_INFINITY :: cast(ledger_amount_t) ((1 << 63) - 1);
}

#if OS == .MACOS {
    LEDGER_NULL :: cast(ledger_t) 0;
}

#if OS == .MACOS {
    ledger_port_array_t :: ledger_array_t;
}

#if OS == .MACOS {
    ledger_port_t :: ledger_t;
}

#if OS == .MACOS {
    ledger_t :: mach_port_t;
}

#if OS == .MACOS {
    LOCK_SET_NULL :: cast(lock_set_t) 0;
}

#if OS == .MACOS {
    lock_set_port_t :: lock_set_t;
}

#if OS == .MACOS {
    lock_set_t :: mach_port_t;
}

#if OS == .MACOS {
    MACH_ACTIVITY_ID_COUNT_MAX :: 16;
}

#if OS == .MACOS {
    mach_exception_code_t :: mach_exception_data_type_t;
}

#if OS == .MACOS {
    MACH_EXCEPTION_CODES :: 0x80000000;
}

#if OS == .MACOS {
    mach_exception_data_t :: *mach_exception_data_type_t;
}

#if OS == .MACOS {
    mach_exception_data_type_t :: s64;
}

#if OS == .MACOS {
    mach_exception_subcode_t :: mach_exception_data_type_t;
}

#if OS == .MACOS {
    mach_memory_info_array_t :: *mach_memory_info_t;
}

#if OS == .MACOS {
    MACH_MEMORY_INFO_NAME_MAX_LEN :: 80;
}

#if OS == .MACOS {
    mach_memory_info_t :: mach_memory_info;
}

#if OS == .MACOS {
    MACH_MSG_ALLOCATE :: 2;
}

#if OS == .MACOS {
    mach_msg_bits_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_BODY_NULL :: cast(*mach_msg_body_t) 0;
}

#if OS == .MACOS {
    mach_msg_copy_options_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_DESCRIPTOR_NULL :: cast(*mach_msg_descriptor_t) 0;
}

#if OS == .MACOS {
    mach_msg_descriptor_type_t :: u32;
}

#if OS == .MACOS {
    mach_msg_format_0_trailer_t :: mach_msg_security_trailer_t;
}

#if OS == .MACOS {
    mach_msg_id_t :: s32;
}

#if OS == .MACOS {
    MACH_MSG_IPC_KERNEL :: 0x00000800;
}

#if OS == .MACOS {
    MACH_MSG_IPC_SPACE :: 0x00002000;
}

#if OS == .MACOS {
    mach_msg_kind_t :: mach_port_seqno_t;
}

#if OS == .MACOS {
    MACH_MSG_MASK :: 0x00003e00;
}

#if OS == .MACOS {
    mach_msg_max_trailer_t :: mach_msg_mac_trailer_t;
}

#if OS == .MACOS {
    MACH_MSG_NULL :: cast(*mach_msg_header_t) 0;
}

#if OS == .MACOS {
    MACH_MSG_OOL_DESCRIPTOR :: 1;
}

#if OS == .MACOS {
    MACH_MSG_OOL_PORTS_DESCRIPTOR :: 2;
}

#if OS == .MACOS {
    MACH_MSG_OOL_VOLATILE_DESCRIPTOR :: 3;
}

#if OS == .MACOS {
    MACH_MSG_OPTION_NONE :: 0x00000000;
}

#if OS == .MACOS {
    mach_msg_option_t :: s32;
}

#if OS == .MACOS {
    mach_msg_options_t :: s32;
}

#if OS == .MACOS {
    MACH_MSG_OVERWRITE :: 3;
}

#if OS == .MACOS {
    MACH_MSG_PHYSICAL_COPY :: 0;
}

#if OS == .MACOS {
    MACH_MSG_PORT_DESCRIPTOR :: 0;
}

#if OS == .MACOS {
    mach_msg_priority_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_PRIORITY_UNSPECIFIED :: cast(mach_msg_priority_t) 0;
}

#if OS == .MACOS {
    mach_msg_return_t :: kern_return_t;
}

#if OS == .MACOS {
    MACH_MSG_SIZE_MAX :: cast,trunc(mach_msg_size_t) ~0;
}

#if OS == .MACOS {
    MACH_MSG_SIZE_NULL :: cast(*mach_msg_size_t) 0;
}

#if OS == .MACOS {
    MACH_MSG_SIZE_RELIABLE :: cast(mach_msg_size_t) 256 * 1024;
}

#if OS == .MACOS {
    mach_msg_size_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_SUCCESS :: 0x00000000;
}

#if OS == .MACOS {
    MACH_MSG_TIMEOUT_NONE :: cast(mach_msg_timeout_t) 0;
}

#if OS == .MACOS {
    mach_msg_timeout_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_TRAILER_FORMAT_0 :: 0;
}

#if OS == .MACOS {
    mach_msg_trailer_info_t :: *u8;
}

#if OS == .MACOS {
    mach_msg_trailer_size_t :: u32;
}

#if OS == .MACOS {
    mach_msg_trailer_type_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_COPY_RECEIVE :: 22;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_COPY_SEND :: 19;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_DISPOSE_RECEIVE :: 24;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_DISPOSE_SEND :: 25;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_DISPOSE_SEND_ONCE :: 26;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_LAST :: 22;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_MAKE_SEND :: 20;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_MAKE_SEND_ONCE :: 21;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_MOVE_RECEIVE :: 16;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_MOVE_SEND :: 17;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_MOVE_SEND_ONCE :: 18;
}

#if OS == .MACOS {
    mach_msg_type_name_t :: u32;
}

#if OS == .MACOS {
    mach_msg_type_number_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_POLYMORPHIC :: cast,trunc(mach_msg_type_name_t) -1;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_PORT_NAME :: 15;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_PORT_NONE :: 0;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_PORT_RECEIVE :: MACH_MSG_TYPE_MOVE_RECEIVE;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_PORT_SEND :: MACH_MSG_TYPE_MOVE_SEND;
}

#if OS == .MACOS {
    MACH_MSG_TYPE_PORT_SEND_ONCE :: MACH_MSG_TYPE_MOVE_SEND_ONCE;
}

#if OS == .MACOS {
    mach_msg_type_size_t :: u32;
}

#if OS == .MACOS {
    MACH_MSG_VIRTUAL_COPY :: 1;
}

#if OS == .MACOS {
    MACH_MSG_VM_KERNEL :: 0x00000400;
}

#if OS == .MACOS {
    MACH_MSG_VM_SPACE :: 0x00001000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_CIRCULAR :: 0x10000000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_COMPLEX :: 0x80000000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_DENAP :: MACH_MSGH_BITS_RAISEIMP;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_DENAPHOLDASRT :: MACH_MSGH_BITS_IMPHOLDASRT;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_IMPHOLDASRT :: 0x10000000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_LOCAL_MASK :: 0x00001f00;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_PORTS_MASK :: MACH_MSGH_BITS_REMOTE_MASK | MACH_MSGH_BITS_LOCAL_MASK | MACH_MSGH_BITS_VOUCHER_MASK;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_RAISEIMP :: 0x20000000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_REMOTE_MASK :: 0x0000001f;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_USED :: 0xb01f1f1f;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_USER :: 0x801f1f1f;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_VOUCHER_MASK :: 0x001f0000;
}

#if OS == .MACOS {
    MACH_MSGH_BITS_ZERO :: 0x00000000;
}

#if OS == .MACOS {
    MACH_MSGH_KIND_NORMAL :: 0x00000000;
}

#if OS == .MACOS {
    MACH_MSGH_KIND_NOTIFICATION :: 0x00000001;
}

#if OS == .MACOS {
    mach_port_array_t :: *mach_port_t;
}

#if OS == .MACOS {
    mach_port_context_t :: mach_vm_address_t;
}

#if OS == .MACOS {
    MACH_PORT_DEAD :: cast,trunc(mach_port_name_t) ~0;
}

#if OS == .MACOS {
    mach_port_delta_t :: s32;
}

#if OS == .MACOS {
    MACH_PORT_DENAP_RECEIVER :: 6;
}

#if OS == .MACOS {
    MACH_PORT_DNREQUESTS_SIZE :: 3;
}

#if OS == .MACOS {
    MACH_PORT_DNREQUESTS_SIZE_COUNT :: 1;
}

#if OS == .MACOS {
    mach_port_flavor_t :: s32;
}

#if OS == .MACOS {
    MACH_PORT_IMPORTANCE_RECEIVER :: 5;
}

#if OS == .MACOS {
    MACH_PORT_INFO_EXT :: 7;
}

#if OS == .MACOS {
    mach_port_info_ext_t :: mach_port_info_ext;
}

#if OS == .MACOS {
    mach_port_info_t :: *s32;
}

#if OS == .MACOS {
    MACH_PORT_LIMITS_INFO :: 1;
}

#if OS == .MACOS {
    mach_port_limits_t :: mach_port_limits;
}

#if OS == .MACOS {
    mach_port_mscount_t :: u32;
}

#if OS == .MACOS {
    mach_port_msgcount_t :: u32;
}

#if OS == .MACOS {
    mach_port_name_array_t :: *mach_port_name_t;
}

#if OS == .MACOS {
    mach_port_name_t :: u32;
}

#if OS == .MACOS {
    MACH_PORT_NULL :: 0;
}

#if OS == .MACOS {
    mach_port_options_ptr_t :: *mach_port_options_t;
}

#if OS == .MACOS {
    mach_port_options_t :: mach_port_options;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_BASIC :: 5;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_DEFAULT :: MACH_PORT_QLIMIT_BASIC;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_KERNEL :: 65534;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_LARGE :: 1024;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_MAX :: MACH_PORT_QLIMIT_LARGE;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_MIN :: MACH_PORT_QLIMIT_ZERO;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_SMALL :: 16;
}

#if OS == .MACOS {
    MACH_PORT_QLIMIT_ZERO :: 0;
}

#if OS == .MACOS {
    mach_port_qos_t :: mach_port_qos;
}

#if OS == .MACOS {
    MACH_PORT_RECEIVE_STATUS :: 2;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_DEAD_NAME :: cast(mach_port_right_t) 4;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_LABELH :: cast(mach_port_right_t) 5;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_NUMBER :: cast(mach_port_right_t) 6;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_PORT_SET :: cast(mach_port_right_t) 3;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_RECEIVE :: cast(mach_port_right_t) 1;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_SEND :: cast(mach_port_right_t) 0;
}

#if OS == .MACOS {
    MACH_PORT_RIGHT_SEND_ONCE :: cast(mach_port_right_t) 2;
}

#if OS == .MACOS {
    mach_port_right_t :: u32;
}

#if OS == .MACOS {
    mach_port_rights_t :: u32;
}

#if OS == .MACOS {
    mach_port_seqno_t :: u32;
}

#if OS == .MACOS {
    MACH_PORT_SRIGHTS_NONE :: 0;
}

#if OS == .MACOS {
    MACH_PORT_SRIGHTS_PRESENT :: 1;
}

#if OS == .MACOS {
    mach_port_srights_t :: u32;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_GUARDED :: 0x02;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_IMP_DONATION :: 0x08;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_REVIVE :: 0x10;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_STRICT_GUARD :: 0x04;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_TASKPTR :: 0x20;
}

#if OS == .MACOS {
    MACH_PORT_STATUS_FLAG_TEMPOWNER :: 0x01;
}

#if OS == .MACOS {
    mach_port_status_t :: mach_port_status;
}

#if OS == .MACOS {
    MACH_PORT_TEMPOWNER :: 4;
}

#if OS == .MACOS {
    mach_port_type_array_t :: *mach_port_type_t;
}

#if OS == .MACOS {
    MACH_PORT_TYPE_DNREQUEST :: 0x80000000;
}

#if OS == .MACOS {
    MACH_PORT_TYPE_NONE :: cast(mach_port_type_t) 0;
}

#if OS == .MACOS {
    MACH_PORT_TYPE_SPREQUEST :: 0x40000000;
}

#if OS == .MACOS {
    MACH_PORT_TYPE_SPREQUEST_DELAYED :: 0x20000000;
}

#if OS == .MACOS {
    mach_port_type_t :: u32;
}

#if OS == .MACOS {
    mach_port_urefs_t :: u32;
}

#if OS == .MACOS {
    MACH_RCV_BODY_ERROR :: 0x1000400c;
}

#if OS == .MACOS {
    MACH_RCV_HEADER_ERROR :: 0x1000400b;
}

#if OS == .MACOS {
    MACH_RCV_IN_PROGRESS :: 0x10004001;
}

#if OS == .MACOS {
    MACH_RCV_IN_PROGRESS_TIMED :: 0x10004011;
}

#if OS == .MACOS {
    MACH_RCV_IN_SET :: 0x1000400a;
}

#if OS == .MACOS {
    MACH_RCV_INTERRUPT :: 0x00000400;
}

#if OS == .MACOS {
    MACH_RCV_INTERRUPTED :: 0x10004005;
}

#if OS == .MACOS {
    MACH_RCV_INVALID_DATA :: 0x10004008;
}

#if OS == .MACOS {
    MACH_RCV_INVALID_NAME :: 0x10004002;
}

#if OS == .MACOS {
    MACH_RCV_INVALID_NOTIFY :: 0x10004007;
}

#if OS == .MACOS {
    MACH_RCV_INVALID_TRAILER :: 0x1000400f;
}

#if OS == .MACOS {
    MACH_RCV_INVALID_TYPE :: 0x1000400d;
}

#if OS == .MACOS {
    MACH_RCV_LARGE :: 0x00000004;
}

#if OS == .MACOS {
    MACH_RCV_LARGE_IDENTITY :: 0x00000008;
}

#if OS == .MACOS {
    MACH_RCV_MSG :: 0x00000002;
}

#if OS == .MACOS {
    MACH_RCV_NOTIFY :: 0x00000200;
}

#if OS == .MACOS {
    MACH_RCV_OVERWRITE :: 0x00001000;
}

#if OS == .MACOS {
    MACH_RCV_PORT_CHANGED :: 0x10004006;
}

#if OS == .MACOS {
    MACH_RCV_PORT_DIED :: 0x10004009;
}

#if OS == .MACOS {
    MACH_RCV_SCATTER_SMALL :: 0x1000400e;
}

#if OS == .MACOS {
    MACH_RCV_SYNC_WAIT :: 0x00004000;
}

#if OS == .MACOS {
    MACH_RCV_TIMED_OUT :: 0x10004003;
}

#if OS == .MACOS {
    MACH_RCV_TIMEOUT :: 0x00000100;
}

#if OS == .MACOS {
    MACH_RCV_TOO_LARGE :: 0x10004004;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_AUDIT :: 3;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_AV :: 7;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_CTX :: 4;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_LABELS :: 8;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_MASK :: (0xf << 24);
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_NULL :: 0;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_SENDER :: 2;
}

#if OS == .MACOS {
    MACH_RCV_TRAILER_SEQNO :: 1;
}

#if OS == .MACOS {
    MACH_RCV_VOUCHER :: 0x00000800;
}

#if OS == .MACOS {
    MACH_SEND_ALWAYS :: 0x00010000;
}

#if OS == .MACOS {
    MACH_SEND_IMPORTANCE :: 0x00080000;
}

#if OS == .MACOS {
    MACH_SEND_IN_PROGRESS :: 0x10000001;
}

#if OS == .MACOS {
    MACH_SEND_INTERRUPT :: 0x00000040;
}

#if OS == .MACOS {
    MACH_SEND_INTERRUPTED :: 0x10000007;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_DATA :: 0x10000002;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_DEST :: 0x10000003;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_HEADER :: 0x10000010;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_MEMORY :: 0x1000000c;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_NOTIFY :: 0x1000000b;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_REPLY :: 0x10000009;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_RIGHT :: 0x1000000a;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_RT_OOL_SIZE :: 0x10000015;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_TRAILER :: 0x10000011;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_TYPE :: 0x1000000f;
}

#if OS == .MACOS {
    MACH_SEND_INVALID_VOUCHER :: 0x10000005;
}

#if OS == .MACOS {
    MACH_SEND_MSG :: 0x00000001;
}

#if OS == .MACOS {
    MACH_SEND_MSG_TOO_SMALL :: 0x10000008;
}

#if OS == .MACOS {
    MACH_SEND_NO_BUFFER :: 0x1000000d;
}

#if OS == .MACOS {
    MACH_SEND_NODENAP :: MACH_SEND_NOIMPORTANCE;
}

#if OS == .MACOS {
    MACH_SEND_NOIMPORTANCE :: 0x00040000;
}

#if OS == .MACOS {
    MACH_SEND_NOTIFY :: 0x00000080;
}

#if OS == .MACOS {
    MACH_SEND_OVERRIDE :: 0x00000020;
}

#if OS == .MACOS {
    MACH_SEND_SYNC_OVERRIDE :: 0x00100000;
}

#if OS == .MACOS {
    MACH_SEND_TIMED_OUT :: 0x10000004;
}

#if OS == .MACOS {
    MACH_SEND_TIMEOUT :: 0x00000010;
}

#if OS == .MACOS {
    MACH_SEND_TOO_LARGE :: 0x1000000e;
}

#if OS == .MACOS {
    MACH_SEND_TRAILER :: 0x00020000;
}

#if OS == .MACOS {
    MACH_TASK_BASIC_INFO :: 20;
}

#if OS == .MACOS {
    mach_task_basic_info_data_t :: mach_task_basic_info;
}

#if OS == .MACOS {
    mach_task_basic_info_t :: *mach_task_basic_info;
}

#if OS == .MACOS {
    mach_timespec_t :: mach_timespec;
}

#if OS == .MACOS {
    mach_vm_address_t :: u64;
}

#if OS == .MACOS {
    mach_vm_offset_t :: u64;
}

#if OS == .MACOS {
    mach_vm_size_t :: u64;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_AUTO_REDEEM :: cast(mach_voucher_attr_recipe_command_t) 4;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_BITS_STORE :: MACH_VOUCHER_ATTR_USER_DATA_STORE;
}

#if OS == .MACOS {
    mach_voucher_attr_command_t :: u32;
}

#if OS == .MACOS {
    mach_voucher_attr_content_size_t :: u32;
}

#if OS == .MACOS {
    mach_voucher_attr_content_t :: *u8;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_CONTROL_FLAGS_NONE :: cast(mach_voucher_attr_control_flags_t) 0;
}

#if OS == .MACOS {
    mach_voucher_attr_control_flags_t :: u32;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_CONTROL_NULL :: cast(mach_voucher_attr_control_t) 0;
}

#if OS == .MACOS {
    mach_voucher_attr_control_t :: mach_port_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_COPY :: cast(mach_voucher_attr_recipe_command_t) 1;
}

#if OS == .MACOS {
    mach_voucher_attr_importance_refs :: u32;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_IMPORTANCE_SELF :: cast(mach_voucher_attr_recipe_command_t) 200;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_ALL :: cast,trunc(mach_voucher_attr_key_t) ~0;
}

#if OS == .MACOS {
    mach_voucher_attr_key_array_t :: *mach_voucher_attr_key_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_ATM :: cast(mach_voucher_attr_key_t) 1;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_BANK :: cast(mach_voucher_attr_key_t) 3;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_BITS :: MACH_VOUCHER_ATTR_KEY_USER_DATA;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_IMPORTANCE :: cast(mach_voucher_attr_key_t) 2;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_NONE :: cast(mach_voucher_attr_key_t) 0;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_NUM_WELL_KNOWN :: MACH_VOUCHER_ATTR_KEY_TEST;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_PTHPRIORITY :: cast(mach_voucher_attr_key_t) 4;
}

#if OS == .MACOS {
    mach_voucher_attr_key_t :: u32;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_TEST :: cast(mach_voucher_attr_key_t) 8;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_KEY_USER_DATA :: cast(mach_voucher_attr_key_t) 7;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_MANAGER_NULL :: cast(mach_voucher_attr_manager_t) 0;
}

#if OS == .MACOS {
    mach_voucher_attr_manager_t :: mach_port_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_MAX_RAW_RECIPE_ARRAY_SIZE :: 5120;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_NOOP :: cast(mach_voucher_attr_recipe_command_t) 0;
}

#if OS == .MACOS {
    mach_voucher_attr_raw_recipe_array_size_t :: mach_msg_type_number_t;
}

#if OS == .MACOS {
    mach_voucher_attr_raw_recipe_array_t :: mach_voucher_attr_raw_recipe_t;
}

#if OS == .MACOS {
    mach_voucher_attr_raw_recipe_size_t :: mach_msg_type_number_t;
}

#if OS == .MACOS {
    mach_voucher_attr_raw_recipe_t :: *u8;
}

#if OS == .MACOS {
    mach_voucher_attr_recipe_command_array_t :: *mach_voucher_attr_recipe_command_t;
}

#if OS == .MACOS {
    mach_voucher_attr_recipe_command_t :: u32;
}

#if OS == .MACOS {
    mach_voucher_attr_recipe_data_t :: mach_voucher_attr_recipe_data;
}

#if OS == .MACOS {
    mach_voucher_attr_recipe_size_t :: mach_msg_type_number_t;
}

#if OS == .MACOS {
    mach_voucher_attr_recipe_t :: *mach_voucher_attr_recipe_data_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_REDEEM :: cast(mach_voucher_attr_recipe_command_t) 10;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_REMOVE :: cast(mach_voucher_attr_recipe_command_t) 2;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_SEND_PREPROCESS :: cast(mach_voucher_attr_recipe_command_t) 5;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_SET_VALUE_HANDLE :: cast(mach_voucher_attr_recipe_command_t) 3;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_TEST_STORE :: MACH_VOUCHER_ATTR_USER_DATA_STORE;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_USER_DATA_STORE :: cast(mach_voucher_attr_recipe_command_t) 211;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_VALUE_FLAGS_NONE :: cast(mach_voucher_attr_value_flags_t) 0;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_VALUE_FLAGS_PERSIST :: cast(mach_voucher_attr_value_flags_t) 1;
}

#if OS == .MACOS {
    mach_voucher_attr_value_flags_t :: u32;
}

#if OS == .MACOS {
    mach_voucher_attr_value_handle_array_size_t :: mach_msg_type_number_t;
}

#if OS == .MACOS {
    mach_voucher_attr_value_handle_array_t :: *mach_voucher_attr_value_handle_t;
}

#if OS == .MACOS {
    mach_voucher_attr_value_handle_t :: u64;
}

#if OS == .MACOS {
    MACH_VOUCHER_ATTR_VALUE_MAX_NESTED :: cast(mach_voucher_attr_value_handle_array_size_t) 4;
}

#if OS == .MACOS {
    mach_voucher_attr_value_reference_t :: u32;
}

#if OS == .MACOS {
    MACH_VOUCHER_IMPORTANCE_ATTR_ADD_EXTERNAL :: 1;
}

#if OS == .MACOS {
    MACH_VOUCHER_IMPORTANCE_ATTR_DROP_EXTERNAL :: 2;
}

#if OS == .MACOS {
    MACH_VOUCHER_NAME_ARRAY_NULL :: cast(mach_voucher_name_array_t) 0;
}

#if OS == .MACOS {
    mach_voucher_name_array_t :: *mach_voucher_name_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_NAME_NULL :: cast(mach_voucher_name_t) 0;
}

#if OS == .MACOS {
    mach_voucher_name_t :: mach_port_name_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_NULL :: cast(mach_voucher_t) 0;
}

#if OS == .MACOS {
    MACH_VOUCHER_SELECTOR_CURRENT :: cast(mach_voucher_selector_t) 0;
}

#if OS == .MACOS {
    MACH_VOUCHER_SELECTOR_EFFECTIVE :: cast(mach_voucher_selector_t) 1;
}

#if OS == .MACOS {
    mach_voucher_selector_t :: u32;
}

#if OS == .MACOS {
    mach_voucher_t :: mach_port_t;
}

#if OS == .MACOS {
    MACH_VOUCHER_TRAP_STACK_LIMIT :: 256;
}

#if OS == .MACOS {
    mach_zone_info_array_t :: *mach_zone_info_t;
}

#if OS == .MACOS {
    mach_zone_info_t :: mach_zone_info_data;
}

#if OS == .MACOS {
    mach_zone_name_array_t :: *mach_zone_name_t;
}

#if OS == .MACOS {
    MACH_ZONE_NAME_MAX_LEN :: 80;
}

#if OS == .MACOS {
    mach_zone_name_t :: mach_zone_name;
}

#if OS == .MACOS {
    MACHINE_THREAD_STATE :: x86_THREAD_STATE;
}

#if OS == .MACOS {
    mem_entry_name_port_t :: mach_port_t;
}

#if OS == .MACOS {
    MPO_CONTEXT_AS_GUARD :: 0x01;
}

#if OS == .MACOS {
    MPO_DENAP_RECEIVER :: 0x40;
}

#if OS == .MACOS {
    MPO_IMPORTANCE_RECEIVER :: 0x08;
}

#if OS == .MACOS {
    MPO_INSERT_SEND_RIGHT :: 0x10;
}

#if OS == .MACOS {
    MPO_QLIMIT :: 0x02;
}

#if OS == .MACOS {
    MPO_STRICT :: 0x20;
}

#if OS == .MACOS {
    MPO_TEMPOWNER :: 0x04;
}

#if OS == .MACOS {
    NDR_CHAR_ASCII :: 0;
}

#if OS == .MACOS {
    NDR_CHAR_EBCDIC :: 1;
}

#if OS == .MACOS {
    NDR_FLOAT_CRAY :: 2;
}

#if OS == .MACOS {
    NDR_FLOAT_IBM :: 3;
}

#if OS == .MACOS {
    NDR_FLOAT_IEEE :: 0;
}

#if OS == .MACOS {
    NDR_FLOAT_VAX :: 1;
}

#if OS == .MACOS {
    NDR_INT_BIG_ENDIAN :: 0;
}

#if OS == .MACOS {
    NDR_INT_LITTLE_ENDIAN :: 1;
}

#if OS == .MACOS {
    NDR_PROTOCOL_2_0 :: 0;
}

#if OS == .MACOS {
    NSEC_PER_MSEC :: 1000000;
}

#if OS == .MACOS {
    NSEC_PER_SEC :: 1000000000;
}

#if OS == .MACOS {
    NSEC_PER_USEC :: 1000;
}

#if OS == .MACOS {
    pointer_t :: vm_offset_t;
}

#if OS == .MACOS {
    policy_base_data_t :: policy_bases;
}

#if OS == .MACOS {
    policy_base_t :: *s32;
}

#if OS == .MACOS {
    POLICY_FIFO :: 4;
}

#if OS == .MACOS {
    policy_fifo_base_data_t :: policy_fifo_base;
}

#if OS == .MACOS {
    policy_fifo_base_t :: *policy_fifo_base;
}

#if OS == .MACOS {
    policy_fifo_info_data_t :: policy_fifo_info;
}

#if OS == .MACOS {
    policy_fifo_info_t :: *policy_fifo_info;
}

#if OS == .MACOS {
    policy_fifo_limit_data_t :: policy_fifo_limit;
}

#if OS == .MACOS {
    policy_fifo_limit_t :: *policy_fifo_limit;
}

#if OS == .MACOS {
    policy_info_data_t :: policy_infos;
}

#if OS == .MACOS {
    policy_info_t :: *s32;
}

#if OS == .MACOS {
    policy_limit_data_t :: policy_limits;
}

#if OS == .MACOS {
    policy_limit_t :: *s32;
}

#if OS == .MACOS {
    POLICY_NULL :: 0;
}

#if OS == .MACOS {
    POLICY_RR :: 2;
}

#if OS == .MACOS {
    policy_rr_base_data_t :: policy_rr_base;
}

#if OS == .MACOS {
    policy_rr_base_t :: *policy_rr_base;
}

#if OS == .MACOS {
    policy_rr_info_data_t :: policy_rr_info;
}

#if OS == .MACOS {
    policy_rr_info_t :: *policy_rr_info;
}

#if OS == .MACOS {
    policy_rr_limit_data_t :: policy_rr_limit;
}

#if OS == .MACOS {
    policy_rr_limit_t :: *policy_rr_limit;
}

#if OS == .MACOS {
    policy_t :: s32;
}

#if OS == .MACOS {
    POLICY_TIMESHARE :: 1;
}

#if OS == .MACOS {
    policy_timeshare_base_data_t :: policy_timeshare_base;
}

#if OS == .MACOS {
    policy_timeshare_base_t :: *policy_timeshare_base;
}

#if OS == .MACOS {
    policy_timeshare_info_data_t :: policy_timeshare_info;
}

#if OS == .MACOS {
    policy_timeshare_info_t :: *policy_timeshare_info;
}

#if OS == .MACOS {
    policy_timeshare_limit_data_t :: policy_timeshare_limit;
}

#if OS == .MACOS {
    policy_timeshare_limit_t :: *policy_timeshare_limit;
}

#if OS == .MACOS {
    POLICYCLASS_FIXEDPRI :: POLICY_RR | POLICY_FIFO;
}

#if OS == .MACOS {
    ppnum_t :: u32;
}

#if OS == .MACOS {
    PROC_FLAG_ADAPTIVE :: 0x100000;
}

#if OS == .MACOS {
    PROC_FLAG_ADAPTIVE_IMPORTANT :: 0x200000;
}

#if OS == .MACOS {
    PROC_FLAG_APPLICATION :: 0x1000000;
}

#if OS == .MACOS {
    PROC_FLAG_DARWINBG :: 0x8000;
}

#if OS == .MACOS {
    PROC_FLAG_EXT_DARWINBG :: 0x10000;
}

#if OS == .MACOS {
    PROC_FLAG_IMPORTANCE_DONOR :: 0x400000;
}

#if OS == .MACOS {
    PROC_FLAG_IOS_APPLEDAEMON :: 0x20000;
}

#if OS == .MACOS {
    PROC_FLAG_IOS_APPLICATION :: PROC_FLAG_APPLICATION;
}

#if OS == .MACOS {
    PROC_FLAG_IOS_IMPPROMOTION :: 0x80000;
}

#if OS == .MACOS {
    PROC_FLAG_SUPPRESSED :: 0x800000;
}

#if OS == .MACOS {
    processor_array_t :: *processor_t;
}

#if OS == .MACOS {
    PROCESSOR_NULL :: cast(processor_t) 0;
}

#if OS == .MACOS {
    processor_port_array_t :: processor_array_t;
}

#if OS == .MACOS {
    processor_port_t :: processor_t;
}

#if OS == .MACOS {
    processor_set_array_t :: *processor_set_t;
}

#if OS == .MACOS {
    processor_set_control_port_t :: processor_set_t;
}

#if OS == .MACOS {
    processor_set_control_t :: mach_port_t;
}

#if OS == .MACOS {
    processor_set_name_array_t :: *processor_set_t;
}

#if OS == .MACOS {
    processor_set_name_port_array_t :: processor_set_array_t;
}

#if OS == .MACOS {
    processor_set_name_port_t :: processor_set_t;
}

#if OS == .MACOS {
    processor_set_name_t :: processor_set_t;
}

#if OS == .MACOS {
    PROCESSOR_SET_NULL :: cast(processor_set_t) 0;
}

#if OS == .MACOS {
    processor_set_port_t :: processor_set_t;
}

#if OS == .MACOS {
    processor_set_t :: mach_port_t;
}

#if OS == .MACOS {
    processor_t :: mach_port_t;
}

#if OS == .MACOS {
    REALTIME_CLOCK :: 0;
}

#if OS == .MACOS {
    reg64_t :: u32;
}

#if OS == .MACOS {
    register_t :: s64;
}

#if OS == .MACOS {
    SEMAPHORE_NULL :: cast(semaphore_t) 0;
}

#if OS == .MACOS {
    semaphore_port_t :: semaphore_t;
}

#if OS == .MACOS {
    semaphore_t :: mach_port_t;
}

#if OS == .MACOS {
    sleep_type_t :: s32;
}

#if OS == .MACOS {
    SUPERPAGE_NONE :: 0;
}

#if OS == .MACOS {
    SUPERPAGE_SIZE_2MB :: 2;
}

#if OS == .MACOS {
    SUPERPAGE_SIZE_ANY :: 1;
}

#if OS == .MACOS {
    SYNC_POLICY_FIFO :: 0x0;
}

#if OS == .MACOS {
    SYNC_POLICY_FIXED_PRIORITY :: 0x1;
}

#if OS == .MACOS {
    SYNC_POLICY_LIFO :: SYNC_POLICY_FIFO | SYNC_POLICY_REVERSED;
}

#if OS == .MACOS {
    SYNC_POLICY_MAX :: 0x7;
}

#if OS == .MACOS {
    SYNC_POLICY_ORDER_MASK :: 0x3;
}

#if OS == .MACOS {
    SYNC_POLICY_REVERSED :: 0x2;
}

#if OS == .MACOS {
    sync_policy_t :: s32;
}

#if OS == .MACOS {
    syscall_arg_t :: u64;
}

#if OS == .MACOS {
    SYSTEM_CLOCK :: 0;
}

#if OS == .MACOS {
    TASK_ABSOLUTETIME_INFO :: 1;
}

#if OS == .MACOS {
    task_absolutetime_info_data_t :: task_absolutetime_info;
}

#if OS == .MACOS {
    task_absolutetime_info_t :: *task_absolutetime_info;
}

#if OS == .MACOS {
    TASK_AFFINITY_TAG_INFO :: 16;
}

#if OS == .MACOS {
    task_affinity_tag_info_data_t :: task_affinity_tag_info;
}

#if OS == .MACOS {
    task_affinity_tag_info_t :: *task_affinity_tag_info;
}

#if OS == .MACOS {
    task_array_t :: *task_t;
}

#if OS == .MACOS {
    TASK_AUDIT_TOKEN :: 15;
}

#if OS == .MACOS {
    TASK_BASE_LATENCY_QOS_POLICY :: 10;
}

#if OS == .MACOS {
    TASK_BASE_QOS_POLICY :: 8;
}

#if OS == .MACOS {
    TASK_BASE_THROUGHPUT_QOS_POLICY :: 11;
}

#if OS == .MACOS {
    TASK_BASIC2_INFO_32 :: 6;
}

#if OS == .MACOS {
    TASK_BASIC_INFO :: TASK_BASIC_INFO_64;
}

#if OS == .MACOS {
    TASK_BASIC_INFO_32 :: 4;
}

#if OS == .MACOS {
    task_basic_info_32_data_t :: task_basic_info_32;
}

#if OS == .MACOS {
    task_basic_info_32_t :: *task_basic_info_32;
}

#if OS == .MACOS {
    TASK_BASIC_INFO_64 :: 5;
}

#if OS == .MACOS {
    task_basic_info_64_data_t :: task_basic_info_64;
}

#if OS == .MACOS {
    task_basic_info_64_t :: *task_basic_info_64;
}

#if OS == .MACOS {
    task_basic_info_data_t :: task_basic_info;
}

#if OS == .MACOS {
    task_basic_info_t :: *task_basic_info;
}

#if OS == .MACOS {
    TASK_CATEGORY_POLICY :: 1;
}

#if OS == .MACOS {
    task_category_policy_data_t :: task_category_policy;
}

#if OS == .MACOS {
    task_category_policy_t :: *task_category_policy;
}

#if OS == .MACOS {
    TASK_DEBUG_INFO_INTERNAL :: 29;
}

#if OS == .MACOS {
    TASK_DYLD_ALL_IMAGE_INFO_32 :: 0;
}

#if OS == .MACOS {
    TASK_DYLD_ALL_IMAGE_INFO_64 :: 1;
}

#if OS == .MACOS {
    TASK_DYLD_INFO :: 17;
}

#if OS == .MACOS {
    task_dyld_info_data_t :: task_dyld_info;
}

#if OS == .MACOS {
    task_dyld_info_t :: *task_dyld_info;
}

#if OS == .MACOS {
    TASK_EVENTS_INFO :: 2;
}

#if OS == .MACOS {
    task_events_info_data_t :: task_events_info;
}

#if OS == .MACOS {
    task_events_info_t :: *task_events_info;
}

#if OS == .MACOS {
    TASK_EXTMOD_INFO :: 19;
}

#if OS == .MACOS {
    task_extmod_info_data_t :: task_extmod_info;
}

#if OS == .MACOS {
    task_extmod_info_t :: *task_extmod_info;
}

#if OS == .MACOS {
    TASK_FLAGS_INFO :: 28;
}

#if OS == .MACOS {
    task_flags_info_data_t :: task_flags_info;
}

#if OS == .MACOS {
    task_flags_info_t :: *task_flags_info;
}

#if OS == .MACOS {
    task_flavor_t :: u32;
}

#if OS == .MACOS {
    task_info_data_t :: [1024] s32;
}

#if OS == .MACOS {
    TASK_INFO_MAX :: 1024;
}

#if OS == .MACOS {
    task_info_t :: *s32;
}

#if OS == .MACOS {
    task_inspect_basic_counts_data_t :: task_inspect_basic_counts;
}

#if OS == .MACOS {
    task_inspect_basic_counts_t :: *task_inspect_basic_counts;
}

#if OS == .MACOS {
    task_inspect_flavor_t :: u32;
}

#if OS == .MACOS {
    task_inspect_info_t :: *s32;
}

#if OS == .MACOS {
    TASK_INSPECT_NULL :: cast(task_inspect_t) 0;
}

#if OS == .MACOS {
    task_inspect_t :: mach_port_t;
}

#if OS == .MACOS {
    TASK_KERNELMEMORY_INFO :: 7;
}

#if OS == .MACOS {
    task_kernelmemory_info_data_t :: task_kernelmemory_info;
}

#if OS == .MACOS {
    task_kernelmemory_info_t :: *task_kernelmemory_info;
}

#if OS == .MACOS {
    task_latency_qos_t :: s32;
}

#if OS == .MACOS {
    task_MSG_COUNT :: 52;
}

#if OS == .MACOS {
    TASK_NAME_NULL :: cast(task_name_t) 0;
}

#if OS == .MACOS {
    task_name_t :: mach_port_t;
}

#if OS == .MACOS {
    TASK_NULL :: cast(task_t) 0;
}

#if OS == .MACOS {
    TASK_OVERRIDE_QOS_POLICY :: 9;
}

#if OS == .MACOS {
    task_policy_flavor_t :: u32;
}

#if OS == .MACOS {
    TASK_POLICY_STATE :: 4;
}

#if OS == .MACOS {
    task_policy_t :: *s32;
}

#if OS == .MACOS {
    task_port_array_t :: task_array_t;
}

#if OS == .MACOS {
    task_port_t :: task_t;
}

#if OS == .MACOS {
    TASK_POWER_INFO :: 21;
}

#if OS == .MACOS {
    task_power_info_data_t :: task_power_info;
}

#if OS == .MACOS {
    task_power_info_t :: *task_power_info;
}

#if OS == .MACOS {
    TASK_POWER_INFO_V2 :: 26;
}

#if OS == .MACOS {
    task_power_info_v2_data_t :: task_power_info_v2;
}

#if OS == .MACOS {
    task_power_info_v2_t :: *task_power_info_v2;
}

#if OS == .MACOS {
    task_purgable_info_t :: vm_purgeable_info;
}

#if OS == .MACOS {
    task_qos_policy_t :: *task_qos_policy;
}

#if OS == .MACOS {
    task_role_t :: s32;
}

#if OS == .MACOS {
    TASK_SCHED_FIFO_INFO :: 12;
}

#if OS == .MACOS {
    TASK_SCHED_INFO :: 14;
}

#if OS == .MACOS {
    TASK_SCHED_RR_INFO :: 11;
}

#if OS == .MACOS {
    TASK_SCHED_TIMESHARE_INFO :: 10;
}

#if OS == .MACOS {
    TASK_SECURITY_TOKEN :: 13;
}

#if OS == .MACOS {
    TASK_SUPPRESSION_POLICY :: 3;
}

#if OS == .MACOS {
    task_suspension_token_t :: mach_port_t;
}

#if OS == .MACOS {
    task_t :: mach_port_t;
}

#if OS == .MACOS {
    TASK_THREAD_TIMES_INFO :: 3;
}

#if OS == .MACOS {
    task_thread_times_info_data_t :: task_thread_times_info;
}

#if OS == .MACOS {
    task_thread_times_info_t :: *task_thread_times_info;
}

#if OS == .MACOS {
    task_throughput_qos_t :: s32;
}

#if OS == .MACOS {
    TASK_TRACE_MEMORY_INFO :: 24;
}

#if OS == .MACOS {
    task_trace_memory_info_data_t :: task_trace_memory_info;
}

#if OS == .MACOS {
    task_trace_memory_info_t :: *task_trace_memory_info;
}

#if OS == .MACOS {
    TASK_VM_INFO :: 22;
}

#if OS == .MACOS {
    task_vm_info_data_t :: task_vm_info;
}

#if OS == .MACOS {
    TASK_VM_INFO_PURGEABLE :: 23;
}

#if OS == .MACOS {
    TASK_VM_INFO_PURGEABLE_ACCOUNT :: 27;
}

#if OS == .MACOS {
    task_vm_info_t :: *task_vm_info;
}

#if OS == .MACOS {
    TASK_WAIT_STATE_INFO :: 25;
}

#if OS == .MACOS {
    task_wait_state_info_data_t :: task_wait_state_info;
}

#if OS == .MACOS {
    task_wait_state_info_t :: *task_wait_state_info;
}

#if OS == .MACOS {
    task_zone_info_array_t :: *task_zone_info_t;
}

#if OS == .MACOS {
    task_zone_info_t :: task_zone_info_data;
}

#if OS == .MACOS {
    TF_LP64 :: 0x00000001;
}

#if OS == .MACOS {
    THR_ACT_NULL :: cast(thread_act_t) 0;
}

#if OS == .MACOS {
    thread_act_array_t :: *thread_act_t;
}

#if OS == .MACOS {
    thread_act_port_array_t :: thread_act_array_t;
}

#if OS == .MACOS {
    thread_act_port_t :: thread_act_t;
}

#if OS == .MACOS {
    thread_act_t :: mach_port_t;
}

#if OS == .MACOS {
    THREAD_AFFINITY_POLICY :: 4;
}

#if OS == .MACOS {
    thread_affinity_policy_data_t :: thread_affinity_policy;
}

#if OS == .MACOS {
    thread_affinity_policy_t :: *thread_affinity_policy;
}

#if OS == .MACOS {
    THREAD_AFFINITY_TAG_NULL :: 0;
}

#if OS == .MACOS {
    thread_array_t :: *thread_t;
}

#if OS == .MACOS {
    THREAD_BACKGROUND_POLICY :: 5;
}

#if OS == .MACOS {
    THREAD_BACKGROUND_POLICY_DARWIN_BG :: 0x1000;
}

#if OS == .MACOS {
    thread_background_policy_data_t :: thread_background_policy;
}

#if OS == .MACOS {
    thread_background_policy_t :: *thread_background_policy;
}

#if OS == .MACOS {
    THREAD_EXTENDED_POLICY :: 1;
}

#if OS == .MACOS {
    thread_extended_policy_data_t :: thread_extended_policy;
}

#if OS == .MACOS {
    thread_extended_policy_t :: *thread_extended_policy;
}

#if OS == .MACOS {
    THREAD_INSPECT_NULL :: cast(thread_inspect_t) 0;
}

#if OS == .MACOS {
    thread_inspect_t :: mach_port_t;
}

#if OS == .MACOS {
    THREAD_LATENCY_QOS_POLICY :: 7;
}

#if OS == .MACOS {
    thread_latency_qos_policy_data_t :: thread_latency_qos_policy;
}

#if OS == .MACOS {
    thread_latency_qos_policy_t :: *thread_latency_qos_policy;
}

#if OS == .MACOS {
    thread_latency_qos_t :: s32;
}

#if OS == .MACOS {
    THREAD_NULL :: cast(thread_t) 0;
}

#if OS == .MACOS {
    thread_policy_flavor_t :: u32;
}

#if OS == .MACOS {
    thread_policy_t :: *s32;
}

#if OS == .MACOS {
    thread_port_array_t :: thread_array_t;
}

#if OS == .MACOS {
    thread_port_t :: thread_t;
}

#if OS == .MACOS {
    THREAD_PRECEDENCE_POLICY :: 3;
}

#if OS == .MACOS {
    thread_precedence_policy_data_t :: thread_precedence_policy;
}

#if OS == .MACOS {
    thread_precedence_policy_t :: *thread_precedence_policy;
}

#if OS == .MACOS {
    THREAD_STANDARD_POLICY :: 1;
}

#if OS == .MACOS {
    THREAD_STANDARD_POLICY_COUNT :: 0;
}

#if OS == .MACOS {
    thread_standard_policy_data_t :: thread_standard_policy;
}

#if OS == .MACOS {
    thread_standard_policy_t :: *thread_standard_policy;
}

#if OS == .MACOS {
    thread_state_data_t :: [614] u32;
}

#if OS == .MACOS {
    thread_state_flavor_array_t :: *thread_state_flavor_t;
}

#if OS == .MACOS {
    THREAD_STATE_FLAVOR_LIST :: 0;
}

#if OS == .MACOS {
    THREAD_STATE_FLAVOR_LIST_10_13 :: 130;
}

#if OS == .MACOS {
    THREAD_STATE_FLAVOR_LIST_10_9 :: 129;
}

#if OS == .MACOS {
    THREAD_STATE_FLAVOR_LIST_NEW :: 128;
}

#if OS == .MACOS {
    thread_state_flavor_t :: s32;
}

#if OS == .MACOS {
    THREAD_STATE_NONE :: 13;
}

#if OS == .MACOS {
    thread_state_t :: *u32;
}

#if OS == .MACOS {
    thread_t :: mach_port_t;
}

#if OS == .MACOS {
    THREAD_THROUGHPUT_QOS_POLICY :: 8;
}

#if OS == .MACOS {
    thread_throughput_qos_policy_data_t :: thread_throughput_qos_policy;
}

#if OS == .MACOS {
    thread_throughput_qos_policy_t :: *thread_throughput_qos_policy;
}

#if OS == .MACOS {
    thread_throughput_qos_t :: s32;
}

#if OS == .MACOS {
    THREAD_TIME_CONSTRAINT_POLICY :: 2;
}

#if OS == .MACOS {
    thread_time_constraint_policy_data_t :: thread_time_constraint_policy;
}

#if OS == .MACOS {
    thread_time_constraint_policy_t :: *thread_time_constraint_policy;
}

#if OS == .MACOS {
    TID_NULL :: cast(u64) 0;
}

#if OS == .MACOS {
    TIME_ABSOLUTE :: 0x00;
}

#if OS == .MACOS {
    TIME_MICROS_MAX :: 1000000;
}

#if OS == .MACOS {
    TIME_RELATIVE :: 0x01;
}

#if OS == .MACOS {
    time_value_t :: time_value;
}

#if OS == .MACOS {
    UND_SERVER_NULL :: cast(UNDServerRef) 0;
}

#if OS == .MACOS {
    UNDServerRef :: mach_port_t;
}

#if OS == .MACOS {
    UPL_NULL :: cast(upl_t) 0;
}

#if OS == .MACOS {
    upl_t :: mach_port_t;
}

#if OS == .MACOS {
    USEC_PER_SEC :: 1000000;
}

#if OS == .MACOS {
    USER_ADDR_NULL :: cast(user_addr_t) 0;
}

#if OS == .MACOS {
    user_addr_t :: u64;
}

#if OS == .MACOS {
    USER_CODE_SELECTOR :: 0x0017;
}

#if OS == .MACOS {
    USER_DATA_SELECTOR :: 0x001f;
}

#if OS == .MACOS {
    user_long_t :: s64;
}

#if OS == .MACOS {
    user_off_t :: s64;
}

#if OS == .MACOS {
    user_size_t :: u64;
}

#if OS == .MACOS {
    user_ssize_t :: s64;
}

#if OS == .MACOS {
    user_subsystem_t :: *u8;
}

#if OS == .MACOS {
    user_time_t :: s64;
}

#if OS == .MACOS {
    user_ulong_t :: u64;
}

#if OS == .MACOS {
    vm_address_t :: vm_offset_t;
}

#if OS == .MACOS {
    vm_extmod_statistics_data_t :: vm_extmod_statistics;
}

#if OS == .MACOS {
    vm_extmod_statistics_t :: *vm_extmod_statistics;
}

#if OS == .MACOS {
    VM_FLAGS_4GB_CHUNK :: 0x0004;
}

#if OS == .MACOS {
    VM_FLAGS_ALIAS_MASK :: 0xFF000000;
}

#if OS == .MACOS {
    VM_FLAGS_ANYWHERE :: 0x0001;
}

#if OS == .MACOS {
    VM_FLAGS_FIXED :: 0x0000;
}

#if OS == .MACOS {
    VM_FLAGS_NO_CACHE :: 0x0010;
}

#if OS == .MACOS {
    VM_FLAGS_OVERWRITE :: 0x4000;
}

#if OS == .MACOS {
    VM_FLAGS_PURGABLE :: 0x0002;
}

#if OS == .MACOS {
    VM_FLAGS_RANDOM_ADDR :: 0x0008;
}

#if OS == .MACOS {
    VM_FLAGS_RESILIENT_CODESIGN :: 0x0020;
}

#if OS == .MACOS {
    VM_FLAGS_RESILIENT_MEDIA :: 0x0040;
}

#if OS == .MACOS {
    VM_FLAGS_RETURN_4K_DATA_ADDR :: 0x800000;
}

#if OS == .MACOS {
    VM_FLAGS_RETURN_DATA_ADDR :: 0x100000;
}

#if OS == .MACOS {
    VM_FLAGS_SUPERPAGE_MASK :: 0x70000;
}

#if OS == .MACOS {
    VM_FLAGS_SUPERPAGE_NONE :: SUPERPAGE_NONE << VM_FLAGS_SUPERPAGE_SHIFT;
}

#if OS == .MACOS {
    VM_FLAGS_SUPERPAGE_SHIFT :: 16;
}

#if OS == .MACOS {
    VM_FLAGS_SUPERPAGE_SIZE_2MB :: SUPERPAGE_SIZE_2MB << VM_FLAGS_SUPERPAGE_SHIFT;
}

#if OS == .MACOS {
    VM_FLAGS_SUPERPAGE_SIZE_ANY :: SUPERPAGE_SIZE_ANY << VM_FLAGS_SUPERPAGE_SHIFT;
}

#if OS == .MACOS {
    VM_FLAGS_USER_ALLOCATE :: VM_FLAGS_FIXED | VM_FLAGS_ANYWHERE | VM_FLAGS_PURGABLE | VM_FLAGS_4GB_CHUNK | VM_FLAGS_RANDOM_ADDR | VM_FLAGS_NO_CACHE | VM_FLAGS_OVERWRITE | VM_FLAGS_SUPERPAGE_MASK | VM_FLAGS_ALIAS_MASK;
}

#if OS == .MACOS {
    VM_FLAGS_USER_MAP :: VM_FLAGS_USER_ALLOCATE | VM_FLAGS_RETURN_4K_DATA_ADDR | VM_FLAGS_RETURN_DATA_ADDR;
}

#if OS == .MACOS {
    VM_FLAGS_USER_REMAP :: VM_FLAGS_FIXED | VM_FLAGS_ANYWHERE | VM_FLAGS_RANDOM_ADDR | VM_FLAGS_OVERWRITE | VM_FLAGS_RETURN_DATA_ADDR | VM_FLAGS_RESILIENT_CODESIGN;
}

#if OS == .MACOS {
    vm_map_address_t :: u64;
}

#if OS == .MACOS {
    VM_MAP_NULL :: cast(vm_map_t) 0;
}

#if OS == .MACOS {
    vm_map_offset_t :: u64;
}

#if OS == .MACOS {
    vm_map_size_t :: u64;
}

#if OS == .MACOS {
    vm_map_t :: mach_port_t;
}

#if OS == .MACOS {
    VM_MEMORY_ACCELERATE :: 75;
}

#if OS == .MACOS {
    VM_MEMORY_ANALYSIS_TOOL :: 10;
}

#if OS == .MACOS {
    VM_MEMORY_APPKIT :: 40;
}

#if OS == .MACOS {
    VM_MEMORY_APPLICATION_SPECIFIC_1 :: 240;
}

#if OS == .MACOS {
    VM_MEMORY_APPLICATION_SPECIFIC_16 :: 255;
}

#if OS == .MACOS {
    VM_MEMORY_ASL :: 81;
}

#if OS == .MACOS {
    VM_MEMORY_ASSETSD :: 72;
}

#if OS == .MACOS {
    VM_MEMORY_ATS :: 50;
}

#if OS == .MACOS {
    VM_MEMORY_AUDIO :: 90;
}

#if OS == .MACOS {
    VM_MEMORY_CARBON :: VM_MEMORY_CORESERVICES;
}

#if OS == .MACOS {
    VM_MEMORY_CGIMAGE :: 52;
}

#if OS == .MACOS {
    VM_MEMORY_COREDATA :: 45;
}

#if OS == .MACOS {
    VM_MEMORY_COREDATA_OBJECTIDS :: 46;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS :: 42;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_BACKINGSTORES :: 57;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_DATA :: 54;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_FRAMEBUFFERS :: 56;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_MISC :: VM_MEMORY_COREGRAPHICS;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_SHARED :: 55;
}

#if OS == .MACOS {
    VM_MEMORY_COREGRAPHICS_XALLOC :: 58;
}

#if OS == .MACOS {
    VM_MEMORY_COREIMAGE :: 68;
}

#if OS == .MACOS {
    VM_MEMORY_COREPROFILE :: 71;
}

#if OS == .MACOS {
    VM_MEMORY_CORESERVICES :: 43;
}

#if OS == .MACOS {
    VM_MEMORY_COREUI :: 76;
}

#if OS == .MACOS {
    VM_MEMORY_COREUIFILE :: 77;
}

#if OS == .MACOS {
    VM_MEMORY_CORPSEINFO :: 80;
}

#if OS == .MACOS {
    VM_MEMORY_DHMM :: 84;
}

#if OS == .MACOS {
    VM_MEMORY_DYLD :: 60;
}

#if OS == .MACOS {
    VM_MEMORY_DYLD_MALLOC :: 61;
}

#if OS == .MACOS {
    VM_MEMORY_DYLIB :: 33;
}

#if OS == .MACOS {
    VM_MEMORY_FOUNDATION :: 41;
}

#if OS == .MACOS {
    VM_MEMORY_GENEALOGY :: 78;
}

#if OS == .MACOS {
    VM_MEMORY_GLSL :: 66;
}

#if OS == .MACOS {
    VM_MEMORY_GUARD :: 31;
}

#if OS == .MACOS {
    VM_MEMORY_IMAGEIO :: 70;
}

#if OS == .MACOS {
    VM_MEMORY_IOKIT :: 21;
}

#if OS == .MACOS {
    VM_MEMORY_IOSURFACE :: 88;
}

#if OS == .MACOS {
    VM_MEMORY_JAVA :: 44;
}

#if OS == .MACOS {
    VM_MEMORY_JAVASCRIPT_CORE :: 63;
}

#if OS == .MACOS {
    VM_MEMORY_JAVASCRIPT_JIT_EXECUTABLE_ALLOCATOR :: 64;
}

#if OS == .MACOS {
    VM_MEMORY_JAVASCRIPT_JIT_REGISTER_FILE :: 65;
}

#if OS == .MACOS {
    VM_MEMORY_LAYERKIT :: 51;
}

#if OS == .MACOS {
    VM_MEMORY_LIBDISPATCH :: 74;
}

#if OS == .MACOS {
    VM_MEMORY_LIBNETWORK :: 89;
}

#if OS == .MACOS {
    VM_MEMORY_MACH_MSG :: 20;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC :: 1;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_HUGE :: 4;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_LARGE :: 3;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_LARGE_REUSABLE :: 8;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_LARGE_REUSED :: 9;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_NANO :: 11;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_SMALL :: 2;
}

#if OS == .MACOS {
    VM_MEMORY_MALLOC_TINY :: 7;
}

#if OS == .MACOS {
    VM_MEMORY_OBJC_DISPATCHERS :: 34;
}

#if OS == .MACOS {
    VM_MEMORY_OPENCL :: 67;
}

#if OS == .MACOS {
    VM_MEMORY_OS_ALLOC_ONCE :: 73;
}

#if OS == .MACOS {
    VM_MEMORY_RAWCAMERA :: 79;
}

#if OS == .MACOS {
    VM_MEMORY_REALLOC :: 6;
}

#if OS == .MACOS {
    VM_MEMORY_SBRK :: 5;
}

#if OS == .MACOS {
    VM_MEMORY_SCENEKIT :: 86;
}

#if OS == .MACOS {
    VM_MEMORY_SHARED_PMAP :: 32;
}

#if OS == .MACOS {
    VM_MEMORY_SKYWALK :: 87;
}

#if OS == .MACOS {
    VM_MEMORY_SQLITE :: 62;
}

#if OS == .MACOS {
    VM_MEMORY_STACK :: 30;
}

#if OS == .MACOS {
    VM_MEMORY_SWIFT_METADATA :: 83;
}

#if OS == .MACOS {
    VM_MEMORY_SWIFT_RUNTIME :: 82;
}

#if OS == .MACOS {
    VM_MEMORY_TCMALLOC :: 53;
}

#if OS == .MACOS {
    VM_MEMORY_UNSHARED_PMAP :: 35;
}

#if OS == .MACOS {
    VM_MEMORY_VIDEOBITSTREAM :: 91;
}

#if OS == .MACOS {
    VM_MEMORY_WEBASSEMBLY :: VM_MEMORY_JAVASCRIPT_CORE;
}

#if OS == .MACOS {
    VM_MEMORY_WEBCORE_PURGEABLE_BUFFERS :: 69;
}

#if OS == .MACOS {
    VM_NAMED_ENTRY_NULL :: cast(vm_named_entry_t) 0;
}

#if OS == .MACOS {
    vm_named_entry_t :: mach_port_t;
}

#if OS == .MACOS {
    vm_object_offset_t :: u64;
}

#if OS == .MACOS {
    vm_object_size_t :: u64;
}

#if OS == .MACOS {
    vm_offset_t :: u64;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_COPIED :: 0x20;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_CS_NX :: 0x400;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_CS_TAINTED :: 0x200;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_CS_VALIDATED :: 0x100;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_DIRTY :: 0x8;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_EXTERNAL :: 0x80;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_FICTITIOUS :: 0x2;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_PAGED_OUT :: 0x10;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_PRESENT :: 0x1;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_REF :: 0x4;
}

#if OS == .MACOS {
    VM_PAGE_QUERY_PAGE_SPECULATIVE :: 0x40;
}

#if OS == .MACOS {
    vm_purgeable_info_t :: *vm_purgeable_info;
}

#if OS == .MACOS {
    vm_purgeable_stat_t :: vm_purgeable_stat;
}

#if OS == .MACOS {
    vm_size_t :: u64;
}

#if OS == .MACOS {
    vm_statistics64_data_t :: vm_statistics64;
}

#if OS == .MACOS {
    vm_statistics64_t :: *vm_statistics64;
}

#if OS == .MACOS {
    vm_statistics_data_t :: vm_statistics;
}

#if OS == .MACOS {
    vm_statistics_t :: *vm_statistics;
}

#if OS == .MACOS {
    vm_task_entry_t :: mach_port_t;
}

#if OS == .MACOS {
    x86_THREAD_STATE :: 7;
}

#if OS == .MACOS {
    zone_info_array_t :: *zone_info_t;
}

#if OS == .MACOS {
    zone_info_t :: zone_info;
}

#if OS == .MACOS {
    zone_name_array_t :: *zone_name_t;
}

#if OS == .MACOS {
    ZONE_NAME_MAX_LEN :: 80;
}

#if OS == .MACOS {
    zone_name_t :: zone_name;
}

// Native storage

#if OS == .MACOS {
    kCFNotFound: CFIndex;
}

#if OS == .MACOS {
    kFSEventStreamEventIdSinceNow: u64;
}

#if OS == .MACOS {
    NDR_record: NDR_record_t #elsewhere libc;
}

#if OS == .MACOS {
    kCFAbsoluteTimeIntervalSince1904: CFTimeInterval #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAbsoluteTimeIntervalSince1970: CFTimeInterval #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorDefault: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorMalloc: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorMallocZone: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorNull: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorSystemDefault: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFAllocatorUseContext: CFAllocatorRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBooleanFalse: CFBooleanRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBooleanTrue: CFBooleanRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBuddhistCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleDevelopmentRegionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleExecutableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleInfoDictionaryVersionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleLocalizationsKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFBundleVersionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFChineseCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFCopyStringDictionaryKeyCallBacks: CFDictionaryKeyCallBacks #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterAMSymbol: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterCalendar: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterCalendarName: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterDefaultDate: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterDefaultFormat: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterDoesRelativeDateFormattingKey: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterEraSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterGregorianStartDate: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterIsLenient: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterLongEraSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterPMSymbol: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortStandaloneQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterShortWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterStandaloneQuarterSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterTimeZone: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterTwoDigitStartDate: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterVeryShortMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterVeryShortStandaloneMonthSymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterVeryShortStandaloneWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterVeryShortWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFDateFormatterWeekdaySymbols: CFDateFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorDescriptionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorDomainCocoa: CFErrorDomain #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorDomainMach: CFErrorDomain #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorDomainOSStatus: CFErrorDomain #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorDomainPOSIX: CFErrorDomain #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorFilePathKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorLocalizedDescriptionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorLocalizedFailureKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorLocalizedFailureReasonKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorLocalizedRecoverySuggestionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFErrorUnderlyingErrorKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFGregorianCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFHebrewCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFISO8601Calendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFIndianCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFIslamicCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFIslamicCivilCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFIslamicTabularCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFIslamicUmmAlQuraCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFJapaneseCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleAlternateQuotationBeginDelimiterKey: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleAlternateQuotationEndDelimiterKey: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCalendar: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCalendarIdentifier: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCollationIdentifier: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCollatorIdentifier: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCountryCode: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCurrencyCode: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCurrencySymbol: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleCurrentLocaleDidChangeNotification: CFNotificationName #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleDecimalSeparator: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleExemplarCharacterSet: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleGroupingSeparator: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleIdentifier: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleLanguageCode: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleMeasurementSystem: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleQuotationBeginDelimiterKey: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleQuotationEndDelimiterKey: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleScriptCode: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleUsesMetricSystem: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFLocaleVariantCode: CFLocaleKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNull: CFNullRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterAlwaysShowDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterCurrencyCode: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterCurrencyDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterCurrencyGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterCurrencySymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterDecimalSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterDefaultFormat: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterExponentSymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterFormatWidth: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterGroupingSize: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterInfinitySymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterInternationalCurrencySymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterIsLenient: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMaxFractionDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMaxIntegerDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMaxSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMinFractionDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMinIntegerDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMinSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMinusSign: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterMultiplier: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterNaNSymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterNegativePrefix: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterNegativeSuffix: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPaddingCharacter: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPaddingPosition: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPerMillSymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPercentSymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPlusSign: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPositivePrefix: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterPositiveSuffix: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterRoundingIncrement: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterRoundingMode: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterSecondaryGroupingSize: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterUseGroupingSeparator: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterUseSignificantDigits: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberFormatterZeroSymbol: CFNumberFormatterKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberNaN: CFNumberRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberNegativeInfinity: CFNumberRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFNumberPositiveInfinity: CFNumberRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPersianCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPlugInDynamicRegisterFunctionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPlugInDynamicRegistrationKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPlugInFactoriesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPlugInTypesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPlugInUnloadFunctionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesAnyApplication: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesAnyHost: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesAnyUser: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesCurrentApplication: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesCurrentHost: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFPreferencesCurrentUser: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFRepublicOfChinaCalendar: CFCalendarIdentifier #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFRunLoopCommonModes: CFRunLoopMode #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFRunLoopDefaultMode: CFRunLoopMode #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketCommandKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketErrorKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketRegisterCommand: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketResultKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketRetrieveCommand: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFSocketValueKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertyAppendToFile: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertyDataWritten: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertyFileCurrentOffset: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertySocketNativeHandle: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertySocketRemoteHostName: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStreamPropertySocketRemotePortNumber: CFStreamPropertyKey #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformFullwidthHalfwidth: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformHiraganaKatakana: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinArabic: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinCyrillic: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinGreek: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinHangul: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinHebrew: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinHiragana: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinKatakana: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformLatinThai: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformMandarinLatin: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformStripCombiningMarks: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformStripDiacritics: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformToLatin: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformToUnicodeName: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFStringTransformToXMLHex: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFTimeZoneSystemTimeZoneDidChangeNotification: CFNotificationName #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFTypeArrayCallBacks: CFArrayCallBacks #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFTypeDictionaryKeyCallBacks: CFDictionaryKeyCallBacks #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFTypeDictionaryValueCallBacks: CFDictionaryValueCallBacks #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLAddedToDirectoryDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLApplicationIsScriptableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLAttributeModificationDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLCanonicalPathKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLContentAccessDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLContentModificationDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLCreationDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLCustomIconKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLDocumentIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLEffectiveIconKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileAllocatedSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileDirectoryContents: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileExists: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileLastModificationTime: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileLength: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileOwnerID: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFilePOSIXMode: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileProtectionComplete: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileProtectionCompleteUnlessOpen: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileProtectionCompleteUntilFirstUserAuthentication: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileProtectionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileProtectionNone: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeBlockSpecial: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeCharacterSpecial: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeDirectory: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeNamedPipe: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeRegular: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeSocket: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeSymbolicLink: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileResourceTypeUnknown: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileSecurityKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLFileSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLGenerationIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLHTTPStatusCode: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLHTTPStatusLine: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLHasHiddenExtensionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsAliasFileKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsApplicationKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsDirectoryKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsExcludedFromBackupKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsExecutableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsHiddenKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsMountTriggerKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsPackageKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsReadableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsRegularFileKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsSymbolicLinkKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsSystemImmutableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsUbiquitousItemKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsUserImmutableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsVolumeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLIsWritableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLKeysOfUnsetValuesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLabelColorKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLabelNumberKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLinkCountKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLocalizedLabelKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLocalizedNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLLocalizedTypeDescriptionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLParentDirectoryURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLPathKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLPreferredIOBlockSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLQuarantinePropertiesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLTagNamesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLTotalFileAllocatedSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLTotalFileSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLTypeIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemDownloadingErrorKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemDownloadingStatusCurrent: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemDownloadingStatusDownloaded: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemDownloadingStatusKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemDownloadingStatusNotDownloaded: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemHasUnresolvedConflictsKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemIsDownloadedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemIsDownloadingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemIsUploadedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemIsUploadingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemPercentDownloadedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemPercentUploadedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLUbiquitousItemUploadingErrorKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeAvailableCapacityForImportantUsageKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeAvailableCapacityForOpportunisticUsageKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeAvailableCapacityKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeCreationDateKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIdentifierKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsAutomountedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsBrowsableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsEjectableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsEncryptedKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsInternalKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsJournalingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsLocalKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsReadOnlyKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsRemovableKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeIsRootFileSystemKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeLocalizedFormatDescriptionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeLocalizedNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeMaximumFileSizeKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeNameKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeResourceCountKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsAccessPermissionsKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsAdvisoryFileLockingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsCasePreservedNamesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsCaseSensitiveNamesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsCompressionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsExclusiveRenamingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsExtendedSecurityKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsFileCloningKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsHardLinksKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsImmutableFilesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsJournalingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsPersistentIDsKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsRenamingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsRootDirectoryDatesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsSparseFilesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsSwapRenamingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsSymbolicLinksKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsVolumeSizesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeSupportsZeroRunsKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeTotalCapacityKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeURLForRemountingKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFURLVolumeUUIDStringKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationAlertHeaderKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationAlertMessageKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationAlternateButtonTitleKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationCheckBoxTitlesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationDefaultButtonTitleKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationIconURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationLocalizationURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationOtherButtonTitleKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationPopUpSelectionKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationPopUpTitlesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationProgressIndicatorValueKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationSoundURLKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationTextFieldTitlesKey: CFStringRef #elsewhere corefoundation;
}

#if OS == .MACOS {
    kCFUserNotificationTextFieldValuesKey: CFStringRef #elsewhere corefoundation;
}

// Native entry points

#if OS == .MACOS {
    __CFRangeMake :: (loc: CFIndex, len: CFIndex) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    __CFStringMakeConstantString :: (cStr: *u8) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeAddGregorianUnits :: (at: CFAbsoluteTime, tz: CFTimeZoneRef, units: CFGregorianUnits) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetCurrent :: () -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetDayOfWeek :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetDayOfYear :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetDifferenceAsGregorianUnits :: (at1: CFAbsoluteTime, at2: CFAbsoluteTime, tz: CFTimeZoneRef, unitFlags: CFOptionFlags) -> CFGregorianUnits #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetGregorianDate :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> CFGregorianDate #foreign corefoundation;
}

#if OS == .MACOS {
    CFAbsoluteTimeGetWeekOfYear :: (at: CFAbsoluteTime, tz: CFTimeZoneRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorAllocate :: (allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorCreate :: (allocator: CFAllocatorRef, _context: *CFAllocatorContext) -> CFAllocatorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorDeallocate :: (allocator: CFAllocatorRef, ptr: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorGetContext :: (allocator: CFAllocatorRef, _context: *CFAllocatorContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorGetDefault :: () -> CFAllocatorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorGetPreferredSizeForSize :: (allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorReallocate :: (allocator: CFAllocatorRef, ptr: *void, newsize: CFIndex, hint: CFOptionFlags) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAllocatorSetDefault :: (allocator: CFAllocatorRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayAppendArray :: (theArray: CFMutableArrayRef, otherArray: CFArrayRef, otherRange: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayAppendValue :: (theArray: CFMutableArrayRef, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayApplyFunction :: (theArray: CFArrayRef, range: CFRange, applier: CFArrayApplierFunction, _context: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayBSearchValues :: (theArray: CFArrayRef, range: CFRange, value: *void, comparator: CFComparatorFunction, _context: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayContainsValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayCreate :: (allocator: CFAllocatorRef, values: **void, numValues: CFIndex, callBacks: *CFArrayCallBacks) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayCreateCopy :: (allocator: CFAllocatorRef, theArray: CFArrayRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex, callBacks: *CFArrayCallBacks) -> CFMutableArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theArray: CFArrayRef) -> CFMutableArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayExchangeValuesAtIndices :: (theArray: CFMutableArrayRef, idx1: CFIndex, idx2: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetCount :: (theArray: CFArrayRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetCountOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetFirstIndexOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetLastIndexOfValue :: (theArray: CFArrayRef, range: CFRange, value: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetValueAtIndex :: (theArray: CFArrayRef, idx: CFIndex) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayGetValues :: (theArray: CFArrayRef, range: CFRange, values: **void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayInsertValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayRemoveAllValues :: (theArray: CFMutableArrayRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayRemoveValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArrayReplaceValues :: (theArray: CFMutableArrayRef, range: CFRange, newValues: **void, newCount: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArraySetValueAtIndex :: (theArray: CFMutableArrayRef, idx: CFIndex, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFArraySortValues :: (theArray: CFMutableArrayRef, range: CFRange, comparator: CFComparatorFunction, _context: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringBeginEditing :: (aStr: CFMutableAttributedStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringCreate :: (alloc: CFAllocatorRef, str: CFStringRef, attributes: CFDictionaryRef) -> CFAttributedStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringCreateCopy :: (alloc: CFAllocatorRef, aStr: CFAttributedStringRef) -> CFAttributedStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringCreateMutable :: (alloc: CFAllocatorRef, maxLength: CFIndex) -> CFMutableAttributedStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringCreateMutableCopy :: (alloc: CFAllocatorRef, maxLength: CFIndex, aStr: CFAttributedStringRef) -> CFMutableAttributedStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringCreateWithSubstring :: (alloc: CFAllocatorRef, aStr: CFAttributedStringRef, range: CFRange) -> CFAttributedStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringEndEditing :: (aStr: CFMutableAttributedStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetAttribute :: (aStr: CFAttributedStringRef, loc: CFIndex, attrName: CFStringRef, effectiveRange: *CFRange) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetAttributeAndLongestEffectiveRange :: (aStr: CFAttributedStringRef, loc: CFIndex, attrName: CFStringRef, inRange: CFRange, longestEffectiveRange: *CFRange) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetAttributes :: (aStr: CFAttributedStringRef, loc: CFIndex, effectiveRange: *CFRange) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetAttributesAndLongestEffectiveRange :: (aStr: CFAttributedStringRef, loc: CFIndex, inRange: CFRange, longestEffectiveRange: *CFRange) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetLength :: (aStr: CFAttributedStringRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetMutableString :: (aStr: CFMutableAttributedStringRef) -> CFMutableStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetString :: (aStr: CFAttributedStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringRemoveAttribute :: (aStr: CFMutableAttributedStringRef, range: CFRange, attrName: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringReplaceAttributedString :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFAttributedStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringReplaceString :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringSetAttribute :: (aStr: CFMutableAttributedStringRef, range: CFRange, attrName: CFStringRef, value: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAttributedStringSetAttributes :: (aStr: CFMutableAttributedStringRef, range: CFRange, replacement: CFDictionaryRef, clearOtherAttributes: Boolean) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFAutorelease :: (arg: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBooleanGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFBooleanGetValue :: (boolean: CFBooleanRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCloseBundleResourceMap :: (bundle: CFBundleRef, refNum: CFBundleRefNum) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyAuxiliaryExecutableURL :: (bundle: CFBundleRef, executableName: CFStringRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyBuiltInPlugInsURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyBundleLocalizations :: (bundle: CFBundleRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyBundleURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyExecutableArchitectures :: (bundle: CFBundleRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyExecutableArchitecturesForURL :: (url: CFURLRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyExecutableURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyInfoDictionaryForURL :: (url: CFURLRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyInfoDictionaryInDirectory :: (bundleURL: CFURLRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyLocalizationsForPreferences :: (locArray: CFArrayRef, prefArray: CFArrayRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyLocalizationsForURL :: (url: CFURLRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyLocalizedString :: (bundle: CFBundleRef, key: CFStringRef, value: CFStringRef, tableName: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyPreferredLocalizationsFromArray :: (locArray: CFArrayRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyPrivateFrameworksURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourcesDirectoryURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURL :: (bundle: CFBundleRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURLForLocalization :: (bundle: CFBundleRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef, localizationName: CFStringRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURLInDirectory :: (bundleURL: CFURLRef, resourceName: CFStringRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURLsOfType :: (bundle: CFBundleRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURLsOfTypeForLocalization :: (bundle: CFBundleRef, resourceType: CFStringRef, subDirName: CFStringRef, localizationName: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopyResourceURLsOfTypeInDirectory :: (bundleURL: CFURLRef, resourceType: CFStringRef, subDirName: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopySharedFrameworksURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopySharedSupportURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCopySupportFilesDirectoryURL :: (bundle: CFBundleRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCreate :: (allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleCreateBundlesFromDirectory :: (allocator: CFAllocatorRef, directoryURL: CFURLRef, bundleType: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetAllBundles :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetBundleWithIdentifier :: (bundleID: CFStringRef) -> CFBundleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetDataPointerForName :: (bundle: CFBundleRef, symbolName: CFStringRef) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetDataPointersForNames :: (bundle: CFBundleRef, symbolNames: CFArrayRef, stbl: **void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetDevelopmentRegion :: (bundle: CFBundleRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetFunctionPointerForName :: (bundle: CFBundleRef, functionName: CFStringRef) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetFunctionPointersForNames :: (bundle: CFBundleRef, functionNames: CFArrayRef, ftbl: **void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetIdentifier :: (bundle: CFBundleRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetInfoDictionary :: (bundle: CFBundleRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetLocalInfoDictionary :: (bundle: CFBundleRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetMainBundle :: () -> CFBundleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetPackageInfo :: (bundle: CFBundleRef, packageType: *u32, packageCreator: *u32) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetPackageInfoInDirectory :: (url: CFURLRef, packageType: *u32, packageCreator: *u32) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetPlugIn :: (bundle: CFBundleRef) -> CFPlugInRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetValueForInfoDictionaryKey :: (bundle: CFBundleRef, key: CFStringRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleGetVersionNumber :: (bundle: CFBundleRef) -> u32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleIsExecutableLoaded :: (bundle: CFBundleRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleLoadExecutable :: (bundle: CFBundleRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleLoadExecutableAndReturnError :: (bundle: CFBundleRef, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleOpenBundleResourceFiles :: (bundle: CFBundleRef, refNum: *CFBundleRefNum, localizedRefNum: *CFBundleRefNum) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleOpenBundleResourceMap :: (bundle: CFBundleRef) -> CFBundleRefNum #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundlePreflightExecutable :: (bundle: CFBundleRef, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFBundleUnloadExecutable :: (bundle: CFBundleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarAddComponents :: (calendar: CFCalendarRef, at: *CFAbsoluteTime, options: CFOptionFlags, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarComposeAbsoluteTime :: (calendar: CFCalendarRef, at: *CFAbsoluteTime, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarCopyCurrent :: () -> CFCalendarRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarCopyLocale :: (calendar: CFCalendarRef) -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarCopyTimeZone :: (calendar: CFCalendarRef) -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarCreateWithIdentifier :: (allocator: CFAllocatorRef, identifier: CFCalendarIdentifier) -> CFCalendarRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarDecomposeAbsoluteTime :: (calendar: CFCalendarRef, at: CFAbsoluteTime, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetComponentDifference :: (calendar: CFCalendarRef, startingAT: CFAbsoluteTime, resultAT: CFAbsoluteTime, options: CFOptionFlags, componentDesc: *u8, __args: ..Any) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetFirstWeekday :: (calendar: CFCalendarRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetIdentifier :: (calendar: CFCalendarRef) -> CFCalendarIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetMaximumRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetMinimumDaysInFirstWeek :: (calendar: CFCalendarRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetMinimumRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetOrdinalityOfUnit :: (calendar: CFCalendarRef, smallerUnit: CFCalendarUnit, biggerUnit: CFCalendarUnit, at: CFAbsoluteTime) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetRangeOfUnit :: (calendar: CFCalendarRef, smallerUnit: CFCalendarUnit, biggerUnit: CFCalendarUnit, at: CFAbsoluteTime) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetTimeRangeOfUnit :: (calendar: CFCalendarRef, unit: CFCalendarUnit, at: CFAbsoluteTime, startp: *CFAbsoluteTime, tip: *CFTimeInterval) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarSetFirstWeekday :: (calendar: CFCalendarRef, wkdy: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarSetLocale :: (calendar: CFCalendarRef, locale: CFLocaleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarSetMinimumDaysInFirstWeek :: (calendar: CFCalendarRef, mwd: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCalendarSetTimeZone :: (calendar: CFCalendarRef, tz: CFTimeZoneRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetAddCharactersInRange :: (theSet: CFMutableCharacterSetRef, theRange: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetAddCharactersInString :: (theSet: CFMutableCharacterSetRef, theString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateBitmapRepresentation :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateCopy :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateInvertedSet :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateMutable :: (alloc: CFAllocatorRef) -> CFMutableCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateMutableCopy :: (alloc: CFAllocatorRef, theSet: CFCharacterSetRef) -> CFMutableCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateWithBitmapRepresentation :: (alloc: CFAllocatorRef, theData: CFDataRef) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateWithCharactersInRange :: (alloc: CFAllocatorRef, theRange: CFRange) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetCreateWithCharactersInString :: (alloc: CFAllocatorRef, theString: CFStringRef) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetGetPredefined :: (theSetIdentifier: CFCharacterSetPredefinedSet) -> CFCharacterSetRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetHasMemberInPlane :: (theSet: CFCharacterSetRef, thePlane: CFIndex) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetIntersect :: (theSet: CFMutableCharacterSetRef, theOtherSet: CFCharacterSetRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetInvert :: (theSet: CFMutableCharacterSetRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetIsCharacterMember :: (theSet: CFCharacterSetRef, theChar: u16) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetIsLongCharacterMember :: (theSet: CFCharacterSetRef, theChar: UTF32Char) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetIsSupersetOfSet :: (theSet: CFCharacterSetRef, theOtherset: CFCharacterSetRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetRemoveCharactersInRange :: (theSet: CFMutableCharacterSetRef, theRange: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetRemoveCharactersInString :: (theSet: CFMutableCharacterSetRef, theString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCharacterSetUnion :: (theSet: CFMutableCharacterSetRef, theOtherSet: CFCharacterSetRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFCopyDescription :: (cf: CFPropertyListRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFCopyTypeIDDescription :: (type_id: CFTypeID) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataAppendBytes :: (theData: CFMutableDataRef, bytes: *u8, length: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataCreate :: (allocator: CFAllocatorRef, bytes: *u8, length: CFIndex) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataCreateCopy :: (allocator: CFAllocatorRef, theData: CFDataRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex) -> CFMutableDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theData: CFDataRef) -> CFMutableDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataCreateWithBytesNoCopy :: (allocator: CFAllocatorRef, bytes: *u8, length: CFIndex, bytesDeallocator: CFAllocatorRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataDeleteBytes :: (theData: CFMutableDataRef, range: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataFind :: (theData: CFDataRef, dataToFind: CFDataRef, searchRange: CFRange, compareOptions: CFDataSearchFlags) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataGetBytePtr :: (theData: CFDataRef) -> *u8 #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataGetBytes :: (theData: CFDataRef, range: CFRange, buffer: *u8) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataGetLength :: (theData: CFDataRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataGetMutableBytePtr :: (theData: CFMutableDataRef) -> *u8 #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataIncreaseLength :: (theData: CFMutableDataRef, extraLength: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataReplaceBytes :: (theData: CFMutableDataRef, range: CFRange, newBytes: *u8, newLength: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDataSetLength :: (theData: CFMutableDataRef, length: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateCompare :: (theDate: CFDateRef, otherDate: CFDateRef, _context: *void) -> CFComparisonResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateCreate :: (allocator: CFAllocatorRef, at: CFAbsoluteTime) -> CFDateRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCopyProperty :: (formatter: CFDateFormatterRef, key: CFDateFormatterKey) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreate :: (allocator: CFAllocatorRef, locale: CFLocaleRef, dateStyle: CFDateFormatterStyle, timeStyle: CFDateFormatterStyle) -> CFDateFormatterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreateDateFormatFromTemplate :: (allocator: CFAllocatorRef, tmplate: CFStringRef, options: CFOptionFlags, locale: CFLocaleRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreateDateFromString :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, _string: CFStringRef, rangep: *CFRange) -> CFDateRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreateISO8601Formatter :: (allocator: CFAllocatorRef, formatOptions: CFISO8601DateFormatOptions) -> CFDateFormatterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreateStringWithAbsoluteTime :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, at: CFAbsoluteTime) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterCreateStringWithDate :: (allocator: CFAllocatorRef, formatter: CFDateFormatterRef, date: CFDateRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetAbsoluteTimeFromString :: (formatter: CFDateFormatterRef, _string: CFStringRef, rangep: *CFRange, atp: *CFAbsoluteTime) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetDateStyle :: (formatter: CFDateFormatterRef) -> CFDateFormatterStyle #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetFormat :: (formatter: CFDateFormatterRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetLocale :: (formatter: CFDateFormatterRef) -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetTimeStyle :: (formatter: CFDateFormatterRef) -> CFDateFormatterStyle #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterSetFormat :: (formatter: CFDateFormatterRef, formatString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateFormatterSetProperty :: (formatter: CFDateFormatterRef, key: CFStringRef, value: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateGetAbsoluteTime :: (theDate: CFDateRef) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateGetTimeIntervalSinceDate :: (theDate: CFDateRef, otherDate: CFDateRef) -> CFTimeInterval #foreign corefoundation;
}

#if OS == .MACOS {
    CFDateGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryAddValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryApplyFunction :: (theDict: CFDictionaryRef, applier: CFDictionaryApplierFunction, _context: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryContainsKey :: (theDict: CFDictionaryRef, key: *void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryContainsValue :: (theDict: CFDictionaryRef, value: *void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryCreate :: (allocator: CFAllocatorRef, keys: **void, values: **void, numValues: CFIndex, keyCallBacks: *CFDictionaryKeyCallBacks, valueCallBacks: *CFDictionaryValueCallBacks) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryCreateCopy :: (allocator: CFAllocatorRef, theDict: CFDictionaryRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryCreateMutable :: (allocator: CFAllocatorRef, capacity: CFIndex, keyCallBacks: *CFDictionaryKeyCallBacks, valueCallBacks: *CFDictionaryValueCallBacks) -> CFMutableDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryCreateMutableCopy :: (allocator: CFAllocatorRef, capacity: CFIndex, theDict: CFDictionaryRef) -> CFMutableDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetCount :: (theDict: CFDictionaryRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetCountOfKey :: (theDict: CFDictionaryRef, key: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetCountOfValue :: (theDict: CFDictionaryRef, value: *void) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetKeysAndValues :: (theDict: CFDictionaryRef, keys: **void, values: **void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetValue :: (theDict: CFDictionaryRef, key: *void) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryGetValueIfPresent :: (theDict: CFDictionaryRef, key: *void, value: **void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryRemoveAllValues :: (theDict: CFMutableDictionaryRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryRemoveValue :: (theDict: CFMutableDictionaryRef, key: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionaryReplaceValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFDictionarySetValue :: (theDict: CFMutableDictionaryRef, key: *void, value: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFEqual :: (cf1: CFPropertyListRef, cf2: CFPropertyListRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCopyDescription :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCopyFailureReason :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCopyRecoverySuggestion :: (err: CFErrorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCopyUserInfo :: (err: CFErrorRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCreate :: (allocator: CFAllocatorRef, domain: CFErrorDomain, code: CFIndex, userInfo: CFDictionaryRef) -> CFErrorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorCreateWithUserInfoKeysAndValues :: (allocator: CFAllocatorRef, domain: CFErrorDomain, code: CFIndex, userInfoKeys: **void, userInfoValues: **void, numUserInfoValues: CFIndex) -> CFErrorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorGetCode :: (err: CFErrorRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorGetDomain :: (err: CFErrorRef) -> CFErrorDomain #foreign corefoundation;
}

#if OS == .MACOS {
    CFErrorGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorCreate :: (allocator: CFAllocatorRef, fd: CFFileDescriptorNativeDescriptor, closeOnInvalidate: Boolean, callout: CFFileDescriptorCallBack, _context: *CFFileDescriptorContext) -> CFFileDescriptorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorCreateRunLoopSource :: (allocator: CFAllocatorRef, f: CFFileDescriptorRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorDisableCallBacks :: (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorEnableCallBacks :: (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorGetContext :: (f: CFFileDescriptorRef, _context: *CFFileDescriptorContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorGetNativeDescriptor :: (f: CFFileDescriptorRef) -> CFFileDescriptorNativeDescriptor #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorInvalidate :: (f: CFFileDescriptorRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFFileDescriptorIsValid :: (f: CFFileDescriptorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFGetAllocator :: (cf: CFPropertyListRef) -> CFAllocatorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFGetRetainCount :: (cf: CFPropertyListRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFGetTypeID :: (cf: CFPropertyListRef) -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFGregorianDateGetAbsoluteTime :: (gdate: CFGregorianDate, tz: CFTimeZoneRef) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFGregorianDateIsValid :: (gdate: CFGregorianDate, unitFlags: CFOptionFlags) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFHash :: (cf: CFPropertyListRef) -> CFHashCode #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyAvailableLocaleIdentifiers :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyCommonISOCurrencyCodes :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyCurrent :: () -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyDisplayNameForPropertyValue :: (displayLocale: CFLocaleRef, key: CFLocaleKey, value: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyISOCountryCodes :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyISOCurrencyCodes :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyISOLanguageCodes :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCopyPreferredLanguages :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreate :: (allocator: CFAllocatorRef, localeIdentifier: CFLocaleIdentifier) -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateCanonicalLanguageIdentifierFromString :: (allocator: CFAllocatorRef, localeIdentifier: CFStringRef) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateCanonicalLocaleIdentifierFromScriptManagerCodes :: (allocator: CFAllocatorRef, lcode: LangCode, rcode: RegionCode) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateCanonicalLocaleIdentifierFromString :: (allocator: CFAllocatorRef, localeIdentifier: CFStringRef) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateComponentsFromLocaleIdentifier :: (allocator: CFAllocatorRef, localeID: CFLocaleIdentifier) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateCopy :: (allocator: CFAllocatorRef, locale: CFLocaleRef) -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateLocaleIdentifierFromComponents :: (allocator: CFAllocatorRef, dictionary: CFDictionaryRef) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleCreateLocaleIdentifierFromWindowsLocaleCode :: (allocator: CFAllocatorRef, lcid: u32) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetIdentifier :: (locale: CFLocaleRef) -> CFLocaleIdentifier #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetLanguageCharacterDirection :: (isoLangCode: CFStringRef) -> CFLocaleLanguageDirection #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetLanguageLineDirection :: (isoLangCode: CFStringRef) -> CFLocaleLanguageDirection #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetSystem :: () -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetValue :: (locale: CFLocaleRef, key: CFLocaleKey) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFLocaleGetWindowsLocaleCodeFromLocaleIdentifier :: (localeIdentifier: CFLocaleIdentifier) -> u32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortCreate :: (allocator: CFAllocatorRef, callout: CFMachPortCallBack, _context: *CFMachPortContext, shouldFreeInfo: *Boolean) -> CFMachPortRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortCreateRunLoopSource :: (allocator: CFAllocatorRef, port: CFMachPortRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortCreateWithPort :: (allocator: CFAllocatorRef, portNum: mach_port_t, callout: CFMachPortCallBack, _context: *CFMachPortContext, shouldFreeInfo: *Boolean) -> CFMachPortRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortGetContext :: (port: CFMachPortRef, _context: *CFMachPortContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortGetInvalidationCallBack :: (port: CFMachPortRef) -> CFMachPortInvalidationCallBack #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortGetPort :: (port: CFMachPortRef) -> mach_port_t #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortInvalidate :: (port: CFMachPortRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortIsValid :: (port: CFMachPortRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFMachPortSetInvalidationCallBack :: (port: CFMachPortRef, callout: CFMachPortInvalidationCallBack) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMakeCollectable :: (cf: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortCreateLocal :: (allocator: CFAllocatorRef, name: CFStringRef, callout: CFMessagePortCallBack, _context: *CFMessagePortContext, shouldFreeInfo: *Boolean) -> CFMessagePortRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortCreateRemote :: (allocator: CFAllocatorRef, name: CFStringRef) -> CFMessagePortRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortCreateRunLoopSource :: (allocator: CFAllocatorRef, local: CFMessagePortRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortGetContext :: (ms: CFMessagePortRef, _context: *CFMessagePortContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortGetInvalidationCallBack :: (ms: CFMessagePortRef) -> CFMessagePortInvalidationCallBack #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortGetName :: (ms: CFMessagePortRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortInvalidate :: (ms: CFMessagePortRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortIsRemote :: (ms: CFMessagePortRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortIsValid :: (ms: CFMessagePortRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortSendRequest :: (remote: CFMessagePortRef, msgid: s32, data: CFDataRef, sendTimeout: CFTimeInterval, rcvTimeout: CFTimeInterval, replyMode: CFStringRef, returnData: *CFDataRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortSetDispatchQueue :: (ms: CFMessagePortRef, queue: dispatch_queue_t) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortSetInvalidationCallBack :: (ms: CFMessagePortRef, callout: CFMessagePortInvalidationCallBack) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFMessagePortSetName :: (ms: CFMessagePortRef, newName: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterAddObserver :: (center: CFNotificationCenterRef, observer: *void, callBack: CFNotificationCallback, name: CFStringRef, object: *void, suspensionBehavior: CFNotificationSuspensionBehavior) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterGetDarwinNotifyCenter :: () -> CFNotificationCenterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterGetDistributedCenter :: () -> CFNotificationCenterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterGetLocalCenter :: () -> CFNotificationCenterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterPostNotification :: (center: CFNotificationCenterRef, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef, deliverImmediately: Boolean) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterPostNotificationWithOptions :: (center: CFNotificationCenterRef, name: CFNotificationName, object: *void, userInfo: CFDictionaryRef, options: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterRemoveEveryObserver :: (center: CFNotificationCenterRef, observer: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNotificationCenterRemoveObserver :: (center: CFNotificationCenterRef, observer: *void, name: CFNotificationName, object: *void) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNullGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberCompare :: (number: CFNumberRef, otherNumber: CFNumberRef, _context: *void) -> CFComparisonResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberCreate :: (allocator: CFAllocatorRef, theType: CFNumberType, valuePtr: *void) -> CFNumberRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterCopyProperty :: (formatter: CFNumberFormatterRef, key: CFNumberFormatterKey) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterCreate :: (allocator: CFAllocatorRef, locale: CFLocaleRef, style: CFNumberFormatterStyle) -> CFNumberFormatterRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterCreateNumberFromString :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, _string: CFStringRef, rangep: *CFRange, options: CFOptionFlags) -> CFNumberRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterCreateStringWithNumber :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, number: CFNumberRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterCreateStringWithValue :: (allocator: CFAllocatorRef, formatter: CFNumberFormatterRef, numberType: CFNumberType, valuePtr: *void) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetDecimalInfoForCurrencyCode :: (currencyCode: CFStringRef, defaultFractionDigits: *s32, roundingIncrement: *float64) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetFormat :: (formatter: CFNumberFormatterRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetLocale :: (formatter: CFNumberFormatterRef) -> CFLocaleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetStyle :: (formatter: CFNumberFormatterRef) -> CFNumberFormatterStyle #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterGetValueFromString :: (formatter: CFNumberFormatterRef, _string: CFStringRef, rangep: *CFRange, numberType: CFNumberType, valuePtr: *void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterSetFormat :: (formatter: CFNumberFormatterRef, formatString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberFormatterSetProperty :: (formatter: CFNumberFormatterRef, key: CFNumberFormatterKey, value: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberGetByteSize :: (number: CFNumberRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberGetType :: (number: CFNumberRef) -> CFNumberType #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberGetValue :: (number: CFNumberRef, theType: CFNumberType, valuePtr: *void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFNumberIsFloatType :: (number: CFNumberRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInAddInstanceForFactory :: (factoryID: CFUUIDRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInCreate :: (allocator: CFAllocatorRef, plugInURL: CFURLRef) -> CFPlugInRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInFindFactoriesForPlugInType :: (typeUUID: CFUUIDRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInFindFactoriesForPlugInTypeInPlugIn :: (typeUUID: CFUUIDRef, plugIn: CFPlugInRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInGetBundle :: (plugIn: CFPlugInRef) -> CFBundleRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceCreate :: (allocator: CFAllocatorRef, factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceCreateWithInstanceDataSize :: (allocator: CFAllocatorRef, instanceDataSize: CFIndex, deallocateInstanceFunction: CFPlugInInstanceDeallocateInstanceDataFunction, factoryName: CFStringRef, getInterfaceFunction: CFPlugInInstanceGetInterfaceFunction) -> CFPlugInInstanceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceGetFactoryName :: (instance: CFPlugInInstanceRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceGetInstanceData :: (instance: CFPlugInInstanceRef) -> *void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceGetInterfaceFunctionTable :: (instance: CFPlugInInstanceRef, interfaceName: CFStringRef, ftbl: **void) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInInstanceGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInIsLoadOnDemand :: (plugIn: CFPlugInRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInRegisterFactoryFunction :: (factoryUUID: CFUUIDRef, func: CFPlugInFactoryFunction) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInRegisterFactoryFunctionByName :: (factoryUUID: CFUUIDRef, plugIn: CFPlugInRef, functionName: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInRegisterPlugInType :: (factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInRemoveInstanceForFactory :: (factoryID: CFUUIDRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInSetLoadOnDemand :: (plugIn: CFPlugInRef, flag: Boolean) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInUnregisterFactory :: (factoryUUID: CFUUIDRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPlugInUnregisterPlugInType :: (factoryUUID: CFUUIDRef, typeUUID: CFUUIDRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesAddSuitePreferencesToApp :: (applicationID: CFStringRef, suiteID: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesAppSynchronize :: (applicationID: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesAppValueIsForced :: (key: CFStringRef, applicationID: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesCopyApplicationList :: (userName: CFStringRef, hostName: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesCopyAppValue :: (key: CFStringRef, applicationID: CFStringRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesCopyKeyList :: (applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesCopyMultiple :: (keysToFetch: CFArrayRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesCopyValue :: (key: CFStringRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesGetAppBooleanValue :: (key: CFStringRef, applicationID: CFStringRef, keyExistsAndHasValidFormat: *Boolean) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesGetAppIntegerValue :: (key: CFStringRef, applicationID: CFStringRef, keyExistsAndHasValidFormat: *Boolean) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesRemoveSuitePreferencesFromApp :: (applicationID: CFStringRef, suiteID: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesSetAppValue :: (key: CFStringRef, value: CFPropertyListRef, applicationID: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesSetMultiple :: (keysToSet: CFDictionaryRef, keysToRemove: CFArrayRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesSetValue :: (key: CFStringRef, value: CFPropertyListRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFPreferencesSynchronize :: (applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateData :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef, format: CFPropertyListFormat, options: CFOptionFlags, error: *CFErrorRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateDeepCopy :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef, mutabilityOption: CFOptionFlags) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateFromStream :: (allocator: CFAllocatorRef, stream: CFReadStreamRef, streamLength: CFIndex, mutabilityOption: CFOptionFlags, format: *CFPropertyListFormat, errorString: *CFStringRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateFromXMLData :: (allocator: CFAllocatorRef, xmlData: CFDataRef, mutabilityOption: CFOptionFlags, errorString: *CFStringRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateWithData :: (allocator: CFAllocatorRef, data: CFDataRef, options: CFOptionFlags, format: *CFPropertyListFormat, error: *CFErrorRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateWithStream :: (allocator: CFAllocatorRef, stream: CFReadStreamRef, streamLength: CFIndex, options: CFOptionFlags, format: *CFPropertyListFormat, error: *CFErrorRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListCreateXMLData :: (allocator: CFAllocatorRef, propertyList: CFPropertyListRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListIsValid :: (plist: CFPropertyListRef, format: CFPropertyListFormat) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListWrite :: (propertyList: CFPropertyListRef, stream: CFWriteStreamRef, format: CFPropertyListFormat, options: CFOptionFlags, error: *CFErrorRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFPropertyListWriteToStream :: (propertyList: CFPropertyListRef, stream: CFWriteStreamRef, format: CFPropertyListFormat, errorString: *CFStringRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamClose :: (stream: CFReadStreamRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamCopyDispatchQueue :: (stream: CFReadStreamRef) -> dispatch_queue_t #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamCopyError :: (stream: CFReadStreamRef) -> CFErrorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamCopyProperty :: (stream: CFReadStreamRef, propertyName: CFStreamPropertyKey) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamCreateWithBytesNoCopy :: (alloc: CFAllocatorRef, bytes: *u8, length: CFIndex, bytesDeallocator: CFAllocatorRef) -> CFReadStreamRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamCreateWithFile :: (alloc: CFAllocatorRef, fileURL: CFURLRef) -> CFReadStreamRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamGetBuffer :: (stream: CFReadStreamRef, maxBytesToRead: CFIndex, numBytesRead: *CFIndex) -> *u8 #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamGetError :: (stream: CFReadStreamRef) -> CFStreamError #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamGetStatus :: (stream: CFReadStreamRef) -> CFStreamStatus #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamHasBytesAvailable :: (stream: CFReadStreamRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamOpen :: (stream: CFReadStreamRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamRead :: (stream: CFReadStreamRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamScheduleWithRunLoop :: (stream: CFReadStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamSetClient :: (stream: CFReadStreamRef, streamEvents: CFOptionFlags, clientCB: CFReadStreamClientCallBack, clientContext: *CFStreamClientContext) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamSetDispatchQueue :: (stream: CFReadStreamRef, q: dispatch_queue_t) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamSetProperty :: (stream: CFReadStreamRef, propertyName: CFStreamPropertyKey, propertyValue: CFPropertyListRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFReadStreamUnscheduleFromRunLoop :: (stream: CFReadStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRelease :: (cf: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRetain :: (cf: CFPropertyListRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopAddCommonMode :: (rl: CFRunLoopRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopAddObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopAddSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopAddTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopContainsObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopContainsSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopContainsTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopCopyAllModes :: (rl: CFRunLoopRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopCopyCurrentMode :: (rl: CFRunLoopRef) -> CFRunLoopMode #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopGetCurrent :: () -> CFRunLoopRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopGetMain :: () -> CFRunLoopRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopGetNextTimerFireDate :: (rl: CFRunLoopRef, mode: CFRunLoopMode) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopIsWaiting :: (rl: CFRunLoopRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverCreate :: (allocator: CFAllocatorRef, activities: CFOptionFlags, repeats: Boolean, order: CFIndex, callout: CFRunLoopObserverCallBack, _context: *CFRunLoopObserverContext) -> CFRunLoopObserverRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverCreateWithHandler :: (allocator: CFAllocatorRef, activities: CFOptionFlags, repeats: Boolean, order: CFIndex, block: Block) -> CFRunLoopObserverRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverDoesRepeat :: (observer: CFRunLoopObserverRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverGetActivities :: (observer: CFRunLoopObserverRef) -> CFOptionFlags #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverGetContext :: (observer: CFRunLoopObserverRef, _context: *CFRunLoopObserverContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverGetOrder :: (observer: CFRunLoopObserverRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverInvalidate :: (observer: CFRunLoopObserverRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopObserverIsValid :: (observer: CFRunLoopObserverRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopPerformBlock :: (rl: CFRunLoopRef, mode: CFPropertyListRef, block: Block) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopRemoveObserver :: (rl: CFRunLoopRef, observer: CFRunLoopObserverRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopRemoveSource :: (rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopRemoveTimer :: (rl: CFRunLoopRef, timer: CFRunLoopTimerRef, mode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopRun :: () -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopRunInMode :: (mode: CFRunLoopMode, seconds: CFTimeInterval, returnAfterSourceHandled: Boolean) -> CFRunLoopRunResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceCreate :: (allocator: CFAllocatorRef, order: CFIndex, _context: *CFRunLoopSourceContext) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceGetContext :: (source: CFRunLoopSourceRef, _context: *CFRunLoopSourceContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceGetOrder :: (source: CFRunLoopSourceRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceInvalidate :: (source: CFRunLoopSourceRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceIsValid :: (source: CFRunLoopSourceRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopSourceSignal :: (source: CFRunLoopSourceRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopStop :: (rl: CFRunLoopRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerCreate :: (allocator: CFAllocatorRef, fireDate: CFAbsoluteTime, interval: CFTimeInterval, flags: CFOptionFlags, order: CFIndex, callout: CFRunLoopTimerCallBack, _context: *CFRunLoopTimerContext) -> CFRunLoopTimerRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerCreateWithHandler :: (allocator: CFAllocatorRef, fireDate: CFAbsoluteTime, interval: CFTimeInterval, flags: CFOptionFlags, order: CFIndex, block: Block) -> CFRunLoopTimerRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerDoesRepeat :: (timer: CFRunLoopTimerRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetContext :: (timer: CFRunLoopTimerRef, _context: *CFRunLoopTimerContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetInterval :: (timer: CFRunLoopTimerRef) -> CFTimeInterval #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetNextFireDate :: (timer: CFRunLoopTimerRef) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetOrder :: (timer: CFRunLoopTimerRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetTolerance :: (timer: CFRunLoopTimerRef) -> CFTimeInterval #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerInvalidate :: (timer: CFRunLoopTimerRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerIsValid :: (timer: CFRunLoopTimerRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerSetNextFireDate :: (timer: CFRunLoopTimerRef, fireDate: CFAbsoluteTime) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopTimerSetTolerance :: (timer: CFRunLoopTimerRef, tolerance: CFTimeInterval) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFRunLoopWakeUp :: (rl: CFRunLoopRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFShow :: (obj: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFShowStr :: (str: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketConnectToAddress :: (s: CFSocketRef, address: CFDataRef, timeout: CFTimeInterval) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCopyAddress :: (s: CFSocketRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCopyPeerAddress :: (s: CFSocketRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCopyRegisteredSocketSignature :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, signature: *CFSocketSignature, nameServerAddress: *CFDataRef) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCopyRegisteredValue :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, value: *CFPropertyListRef, nameServerAddress: *CFDataRef) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCreate :: (allocator: CFAllocatorRef, protocolFamily: s32, socketType: s32, protocol: s32, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCreateConnectedToSocketSignature :: (allocator: CFAllocatorRef, signature: *CFSocketSignature, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext, timeout: CFTimeInterval) -> CFSocketRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCreateRunLoopSource :: (allocator: CFAllocatorRef, s: CFSocketRef, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCreateWithNative :: (allocator: CFAllocatorRef, sock: CFSocketNativeHandle, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketCreateWithSocketSignature :: (allocator: CFAllocatorRef, signature: *CFSocketSignature, callBackTypes: CFOptionFlags, callout: CFSocketCallBack, _context: *CFSocketContext) -> CFSocketRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketDisableCallBacks :: (s: CFSocketRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketEnableCallBacks :: (s: CFSocketRef, callBackTypes: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketGetContext :: (s: CFSocketRef, _context: *CFSocketContext) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketGetDefaultNameRegistryPortNumber :: () -> u16 #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketGetNative :: (s: CFSocketRef) -> CFSocketNativeHandle #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketGetSocketFlags :: (s: CFSocketRef) -> CFOptionFlags #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketInvalidate :: (s: CFSocketRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketIsValid :: (s: CFSocketRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketRegisterSocketSignature :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, signature: *CFSocketSignature) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketRegisterValue :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef, value: CFPropertyListRef) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketSendData :: (s: CFSocketRef, address: CFDataRef, data: CFDataRef, timeout: CFTimeInterval) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketSetAddress :: (s: CFSocketRef, address: CFDataRef) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketSetDefaultNameRegistryPortNumber :: (port: u16) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketSetSocketFlags :: (s: CFSocketRef, flags: CFOptionFlags) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFSocketUnregister :: (nameServerSignature: *CFSocketSignature, timeout: CFTimeInterval, name: CFStringRef) -> CFSocketError #foreign corefoundation;
}

#if OS == .MACOS {
    CFStreamCreateBoundPair :: (alloc: CFAllocatorRef, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef, transferBufferSize: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStreamCreatePairWithPeerSocketSignature :: (alloc: CFAllocatorRef, signature: *CFSocketSignature, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStreamCreatePairWithSocket :: (alloc: CFAllocatorRef, sock: CFSocketNativeHandle, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStreamCreatePairWithSocketToHost :: (alloc: CFAllocatorRef, host: CFStringRef, port: u32, readStream: *CFReadStreamRef, writeStream: *CFWriteStreamRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringAppend :: (theString: CFMutableStringRef, appendedString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringAppendCharacters :: (theString: CFMutableStringRef, chars: *u16, numChars: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringAppendCString :: (theString: CFMutableStringRef, cStr: *u8, encoding: CFStringBuiltInEncodings) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringAppendFormat :: (theString: CFMutableStringRef, formatOptions: CFDictionaryRef, format: CFStringRef, __args: ..Any) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringAppendPascalString :: (theString: CFMutableStringRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCapitalize :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCompare :: (theString1: CFStringRef, theString2: CFStringRef, compareOptions: CFStringCompareFlags) -> CFComparisonResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCompareWithOptions :: (theString1: CFStringRef, theString2: CFStringRef, rangeToCompare: CFRange, compareOptions: CFStringCompareFlags) -> CFComparisonResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCompareWithOptionsAndLocale :: (theString1: CFStringRef, theString2: CFStringRef, rangeToCompare: CFRange, compareOptions: CFStringCompareFlags, locale: CFLocaleRef) -> CFComparisonResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertEncodingToIANACharSetName :: (encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertEncodingToNSStringEncoding :: (encoding: CFStringBuiltInEncodings) -> u64 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertEncodingToWindowsCodepage :: (encoding: CFStringBuiltInEncodings) -> u32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertIANACharSetNameToEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertNSStringEncodingToEncoding :: (encoding: u64) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringConvertWindowsCodepageToEncoding :: (codepage: u32) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateArrayBySeparatingStrings :: (alloc: CFAllocatorRef, theString: CFStringRef, separatorString: CFStringRef) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateArrayWithFindResults :: (alloc: CFAllocatorRef, theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, compareOptions: CFStringCompareFlags) -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateByCombiningStrings :: (alloc: CFAllocatorRef, theArray: CFArrayRef, separatorString: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateCopy :: (alloc: CFAllocatorRef, theString: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateExternalRepresentation :: (alloc: CFAllocatorRef, theString: CFStringRef, encoding: CFStringBuiltInEncodings, lossByte: u8) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateFromExternalRepresentation :: (alloc: CFAllocatorRef, data: CFDataRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateMutable :: (alloc: CFAllocatorRef, maxLength: CFIndex) -> CFMutableStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateMutableCopy :: (alloc: CFAllocatorRef, maxLength: CFIndex, theString: CFStringRef) -> CFMutableStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateMutableWithExternalCharactersNoCopy :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex, capacity: CFIndex, externalCharactersAllocator: CFAllocatorRef) -> CFMutableStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithBytes :: (alloc: CFAllocatorRef, bytes: *u8, numBytes: CFIndex, encoding: CFStringBuiltInEncodings, isExternalRepresentation: Boolean) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithBytesNoCopy :: (alloc: CFAllocatorRef, bytes: *u8, numBytes: CFIndex, encoding: CFStringBuiltInEncodings, isExternalRepresentation: Boolean, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithCharacters :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithCharactersNoCopy :: (alloc: CFAllocatorRef, chars: *u16, numChars: CFIndex, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithCString :: (alloc: CFAllocatorRef, cStr: *u8, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithCStringNoCopy :: (alloc: CFAllocatorRef, cStr: *u8, encoding: CFStringBuiltInEncodings, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithFileSystemRepresentation :: (alloc: CFAllocatorRef, buffer: *u8) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithFormat :: (alloc: CFAllocatorRef, formatOptions: CFDictionaryRef, format: CFStringRef, __args: ..Any) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithPascalString :: (alloc: CFAllocatorRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithPascalStringNoCopy :: (alloc: CFAllocatorRef, pStr: ConstStr255Param, encoding: CFStringBuiltInEncodings, contentsDeallocator: CFAllocatorRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringCreateWithSubstring :: (alloc: CFAllocatorRef, str: CFStringRef, range: CFRange) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringDelete :: (theString: CFMutableStringRef, range: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFind :: (theString: CFStringRef, stringToFind: CFStringRef, compareOptions: CFStringCompareFlags) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFindAndReplace :: (theString: CFMutableStringRef, stringToFind: CFStringRef, replacementString: CFStringRef, rangeToSearch: CFRange, compareOptions: CFStringCompareFlags) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFindCharacterFromSet :: (theString: CFStringRef, theSet: CFCharacterSetRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, result: *CFRange) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFindWithOptions :: (theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, result: *CFRange) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFindWithOptionsAndLocale :: (theString: CFStringRef, stringToFind: CFStringRef, rangeToSearch: CFRange, searchOptions: CFStringCompareFlags, locale: CFLocaleRef, result: *CFRange) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringFold :: (theString: CFMutableStringRef, theFlags: CFStringCompareFlags, theLocale: CFLocaleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetBytes :: (theString: CFStringRef, range: CFRange, encoding: CFStringBuiltInEncodings, lossByte: u8, isExternalRepresentation: Boolean, buffer: *u8, maxBufLen: CFIndex, usedBufLen: *CFIndex) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetCharacterAtIndex :: (theString: CFStringRef, idx: CFIndex) -> u16 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetCharacters :: (theString: CFStringRef, range: CFRange, buffer: *u16) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetCharactersPtr :: (theString: CFStringRef) -> *u16 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetCString :: (theString: CFStringRef, buffer: *u8, bufferSize: CFIndex, encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetCStringPtr :: (theString: CFStringRef, encoding: CFStringBuiltInEncodings) -> *u8 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetDoubleValue :: (str: CFStringRef) -> float64 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetFastestEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetFileSystemRepresentation :: (_string: CFStringRef, buffer: *u8, maxBufLen: CFIndex) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetHyphenationLocationBeforeIndex :: (_string: CFStringRef, location: CFIndex, limitRange: CFRange, options: CFOptionFlags, locale: CFLocaleRef, character: *UTF32Char) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetIntValue :: (str: CFStringRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetLength :: (theString: CFStringRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetLineBounds :: (theString: CFStringRef, range: CFRange, lineBeginIndex: *CFIndex, lineEndIndex: *CFIndex, contentsEndIndex: *CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetListOfAvailableEncodings :: () -> *CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetMaximumSizeForEncoding :: (length: CFIndex, encoding: CFStringBuiltInEncodings) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetMaximumSizeOfFileSystemRepresentation :: (_string: CFStringRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetMostCompatibleMacStringEncoding :: (encoding: CFStringBuiltInEncodings) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetNameOfEncoding :: (encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetParagraphBounds :: (_string: CFStringRef, range: CFRange, parBeginIndex: *CFIndex, parEndIndex: *CFIndex, contentsEndIndex: *CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetPascalString :: (theString: CFStringRef, buffer: StringPtr, bufferSize: CFIndex, encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetPascalStringPtr :: (theString: CFStringRef, encoding: CFStringBuiltInEncodings) -> ConstStringPtr #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetRangeOfComposedCharactersAtIndex :: (theString: CFStringRef, theIndex: CFIndex) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetSmallestEncoding :: (theString: CFStringRef) -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetSystemEncoding :: () -> CFStringBuiltInEncodings #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringHasPrefix :: (theString: CFStringRef, prefix: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringHasSuffix :: (theString: CFStringRef, suffix: CFStringRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringInsert :: (str: CFMutableStringRef, idx: CFIndex, insertedStr: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringIsEncodingAvailable :: (encoding: CFStringBuiltInEncodings) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringIsHyphenationAvailableForLocale :: (locale: CFLocaleRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringLowercase :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringNormalize :: (theString: CFMutableStringRef, theForm: CFStringNormalizationForm) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringPad :: (theString: CFMutableStringRef, padString: CFStringRef, length: CFIndex, indexIntoPad: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringReplace :: (theString: CFMutableStringRef, range: CFRange, replacement: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringReplaceAll :: (theString: CFMutableStringRef, replacement: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringSetExternalCharactersNoCopy :: (theString: CFMutableStringRef, chars: *u16, length: CFIndex, capacity: CFIndex) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerAdvanceToNextToken :: (tokenizer: CFStringTokenizerRef) -> CFStringTokenizerTokenType #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerCopyBestStringLanguage :: (_string: CFStringRef, range: CFRange) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerCopyCurrentTokenAttribute :: (tokenizer: CFStringTokenizerRef, attribute: CFOptionFlags) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerCreate :: (alloc: CFAllocatorRef, _string: CFStringRef, range: CFRange, options: CFOptionFlags, locale: CFLocaleRef) -> CFStringTokenizerRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerGetCurrentSubTokens :: (tokenizer: CFStringTokenizerRef, ranges: *CFRange, maxRangeLength: CFIndex, derivedSubTokens: CFMutableArrayRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerGetCurrentTokenRange :: (tokenizer: CFStringTokenizerRef) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerGoToTokenAtIndex :: (tokenizer: CFStringTokenizerRef, index: CFIndex) -> CFStringTokenizerTokenType #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTokenizerSetString :: (tokenizer: CFStringTokenizerRef, _string: CFStringRef, range: CFRange) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTransform :: (_string: CFMutableStringRef, range: *CFRange, transform: CFStringRef, reverse: Boolean) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTrim :: (theString: CFMutableStringRef, trimString: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringTrimWhitespace :: (theString: CFMutableStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFStringUppercase :: (theString: CFMutableStringRef, locale: CFLocaleRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopyAbbreviation :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopyAbbreviationDictionary :: () -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopyDefault :: () -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopyKnownNames :: () -> CFArrayRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopyLocalizedName :: (tz: CFTimeZoneRef, style: CFTimeZoneNameStyle, locale: CFLocaleRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCopySystem :: () -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCreate :: (allocator: CFAllocatorRef, name: CFStringRef, data: CFDataRef) -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCreateWithName :: (allocator: CFAllocatorRef, name: CFStringRef, tryAbbrev: Boolean) -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneCreateWithTimeIntervalFromGMT :: (allocator: CFAllocatorRef, ti: CFTimeInterval) -> CFTimeZoneRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetData :: (tz: CFTimeZoneRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetDaylightSavingTimeOffset :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFTimeInterval #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetName :: (tz: CFTimeZoneRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetNextDaylightSavingTimeTransition :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFAbsoluteTime #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetSecondsFromGMT :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> CFTimeInterval #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneIsDaylightSavingTime :: (tz: CFTimeZoneRef, at: CFAbsoluteTime) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneResetSystem :: () -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneSetAbbreviationDictionary :: (dict: CFDictionaryRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFTimeZoneSetDefault :: (tz: CFTimeZoneRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCanBeDecomposed :: (anURL: CFURLRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLClearResourcePropertyCache :: (url: CFURLRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLClearResourcePropertyCacheForKey :: (url: CFURLRef, key: CFStringRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyAbsoluteURL :: (relativeURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyFileSystemPath :: (anURL: CFURLRef, pathStyle: CFURLPathStyle) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyFragment :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyHostName :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyLastPathComponent :: (url: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyNetLocation :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyParameterString :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyPassword :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyPath :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyPathExtension :: (url: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyQueryString :: (anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyResourcePropertiesForKeys :: (url: CFURLRef, keys: CFArrayRef, error: *CFErrorRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValueTypeRefPtr: *void, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyResourceSpecifier :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyScheme :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyStrictPath :: (anURL: CFURLRef, isAbsolute: *Boolean) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCopyUserName :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateAbsoluteURLWithBytes :: (alloc: CFAllocatorRef, relativeURLBytes: *u8, length: CFIndex, encoding: CFStringBuiltInEncodings, baseURL: CFURLRef, useCompatibilityMode: Boolean) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateBookmarkData :: (allocator: CFAllocatorRef, url: CFURLRef, options: CFURLBookmarkCreationOptions, resourcePropertiesToInclude: CFArrayRef, relativeToURL: CFURLRef, error: *CFErrorRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateBookmarkDataFromAliasRecord :: (allocatorRef: CFAllocatorRef, aliasRecordDataRef: CFDataRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateBookmarkDataFromFile :: (allocator: CFAllocatorRef, fileURL: CFURLRef, errorRef: *CFErrorRef) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateByResolvingBookmarkData :: (allocator: CFAllocatorRef, bookmark: CFDataRef, options: CFURLBookmarkResolutionOptions, relativeToURL: CFURLRef, resourcePropertiesToInclude: CFArrayRef, isStale: *Boolean, error: *CFErrorRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateCopyAppendingPathComponent :: (allocator: CFAllocatorRef, url: CFURLRef, pathComponent: CFStringRef, isDirectory: Boolean) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateCopyAppendingPathExtension :: (allocator: CFAllocatorRef, url: CFURLRef, extension: CFStringRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateCopyDeletingLastPathComponent :: (allocator: CFAllocatorRef, url: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateCopyDeletingPathExtension :: (allocator: CFAllocatorRef, url: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateData :: (allocator: CFAllocatorRef, url: CFURLRef, encoding: CFStringBuiltInEncodings, escapeWhitespace: Boolean) -> CFDataRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateDataAndPropertiesFromResource :: (alloc: CFAllocatorRef, url: CFURLRef, resourceData: *CFDataRef, properties: *CFDictionaryRef, desiredProperties: CFArrayRef, errorCode: *s32) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateFilePathURL :: (allocator: CFAllocatorRef, url: CFURLRef, error: *CFErrorRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateFileReferenceURL :: (allocator: CFAllocatorRef, url: CFURLRef, error: *CFErrorRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateFromFileSystemRepresentation :: (allocator: CFAllocatorRef, buffer: *u8, bufLen: CFIndex, isDirectory: Boolean) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateFromFileSystemRepresentationRelativeToBase :: (allocator: CFAllocatorRef, buffer: *u8, bufLen: CFIndex, isDirectory: Boolean, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateFromFSRef :: (allocator: CFAllocatorRef, fsRef: *FSRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreatePropertyFromResource :: (alloc: CFAllocatorRef, url: CFURLRef, property: CFStringRef, errorCode: *s32) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateResourcePropertiesForKeysFromBookmarkData :: (allocator: CFAllocatorRef, resourcePropertiesToReturn: CFArrayRef, bookmark: CFDataRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateResourcePropertyForKeyFromBookmarkData :: (allocator: CFAllocatorRef, resourcePropertyKey: CFStringRef, bookmark: CFDataRef) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateStringByAddingPercentEscapes :: (allocator: CFAllocatorRef, originalString: CFStringRef, charactersToLeaveUnescaped: CFStringRef, legalURLCharactersToBeEscaped: CFStringRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateStringByReplacingPercentEscapes :: (allocator: CFAllocatorRef, originalString: CFStringRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateStringByReplacingPercentEscapesUsingEncoding :: (allocator: CFAllocatorRef, origString: CFStringRef, charsToLeaveEscaped: CFStringRef, encoding: CFStringBuiltInEncodings) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateWithBytes :: (allocator: CFAllocatorRef, URLBytes: *u8, length: CFIndex, encoding: CFStringBuiltInEncodings, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateWithFileSystemPath :: (allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateWithFileSystemPathRelativeToBase :: (allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLCreateWithString :: (allocator: CFAllocatorRef, URLString: CFStringRef, baseURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLDestroyResource :: (url: CFURLRef, errorCode: *s32) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorCreateForDirectoryURL :: (alloc: CFAllocatorRef, directoryURL: CFURLRef, option: CFURLEnumeratorOptions, propertyKeys: CFArrayRef) -> CFURLEnumeratorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorCreateForMountedVolumes :: (alloc: CFAllocatorRef, option: CFURLEnumeratorOptions, propertyKeys: CFArrayRef) -> CFURLEnumeratorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorGetDescendentLevel :: (enumerator: CFURLEnumeratorRef) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorGetNextURL :: (enumerator: CFURLEnumeratorRef, url: *CFURLRef, error: *CFErrorRef) -> CFURLEnumeratorResult #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorGetSourceDidChange :: (enumerator: CFURLEnumeratorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLEnumeratorSkipDescendents :: (enumerator: CFURLEnumeratorRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetBaseURL :: (anURL: CFURLRef) -> CFURLRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetByteRangeForComponent :: (url: CFURLRef, component: CFURLComponentType, rangeIncludingSeparators: *CFRange) -> CFRange #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetBytes :: (url: CFURLRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetFileSystemRepresentation :: (url: CFURLRef, resolveAgainstBase: Boolean, buffer: *u8, maxBufLen: CFIndex) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetFSRef :: (url: CFURLRef, fsRef: *FSRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetPortNumber :: (anURL: CFURLRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetString :: (anURL: CFURLRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLHasDirectoryPath :: (anURL: CFURLRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLIsFileReferenceURL :: (url: CFURLRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLResourceIsReachable :: (url: CFURLRef, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLSetResourcePropertiesForKeys :: (url: CFURLRef, keyedPropertyValues: CFDictionaryRef, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLSetResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValue: CFPropertyListRef, error: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLSetTemporaryResourcePropertyForKey :: (url: CFURLRef, key: CFStringRef, propertyValue: CFPropertyListRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLStartAccessingSecurityScopedResource :: (url: CFURLRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLStopAccessingSecurityScopedResource :: (url: CFURLRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLWriteBookmarkDataToFile :: (bookmarkRef: CFDataRef, fileURL: CFURLRef, options: CFURLBookmarkFileCreationOptions, errorRef: *CFErrorRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFURLWriteDataAndPropertiesToResource :: (url: CFURLRef, dataToWrite: CFDataRef, propertiesToWrite: CFDictionaryRef, errorCode: *s32) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationCancel :: (userNotification: CFUserNotificationRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationCreate :: (allocator: CFAllocatorRef, timeout: CFTimeInterval, flags: CFOptionFlags, error: *s32, dictionary: CFDictionaryRef) -> CFUserNotificationRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationCreateRunLoopSource :: (allocator: CFAllocatorRef, userNotification: CFUserNotificationRef, callout: CFUserNotificationCallBack, order: CFIndex) -> CFRunLoopSourceRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationDisplayAlert :: (timeout: CFTimeInterval, flags: CFOptionFlags, iconURL: CFURLRef, soundURL: CFURLRef, localizationURL: CFURLRef, alertHeader: CFStringRef, alertMessage: CFStringRef, defaultButtonTitle: CFStringRef, alternateButtonTitle: CFStringRef, otherButtonTitle: CFStringRef, responseFlags: *CFOptionFlags) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationDisplayNotice :: (timeout: CFTimeInterval, flags: CFOptionFlags, iconURL: CFURLRef, soundURL: CFURLRef, localizationURL: CFURLRef, alertHeader: CFStringRef, alertMessage: CFStringRef, defaultButtonTitle: CFStringRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationGetResponseDictionary :: (userNotification: CFUserNotificationRef) -> CFDictionaryRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationGetResponseValue :: (userNotification: CFUserNotificationRef, key: CFStringRef, idx: CFIndex) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationReceiveResponse :: (userNotification: CFUserNotificationRef, timeout: CFTimeInterval, responseFlags: *CFOptionFlags) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFUserNotificationUpdate :: (userNotification: CFUserNotificationRef, timeout: CFTimeInterval, flags: CFOptionFlags, dictionary: CFDictionaryRef) -> s32 #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDCreate :: (alloc: CFAllocatorRef) -> CFUUIDRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDCreateFromString :: (alloc: CFAllocatorRef, uuidStr: CFStringRef) -> CFUUIDRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDCreateFromUUIDBytes :: (alloc: CFAllocatorRef, bytes: CFUUIDBytes) -> CFUUIDRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDCreateString :: (alloc: CFAllocatorRef, uuid: CFUUIDRef) -> CFStringRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDCreateWithBytes :: (alloc: CFAllocatorRef, byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8, byte5: u8, byte6: u8, byte7: u8, byte8: u8, byte9: u8, byte10: u8, byte11: u8, byte12: u8, byte13: u8, byte14: u8, byte15: u8) -> CFUUIDRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDGetConstantUUIDWithBytes :: (alloc: CFAllocatorRef, byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte4: u8, byte5: u8, byte6: u8, byte7: u8, byte8: u8, byte9: u8, byte10: u8, byte11: u8, byte12: u8, byte13: u8, byte14: u8, byte15: u8) -> CFUUIDRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFUUIDGetUUIDBytes :: (uuid: CFUUIDRef) -> CFUUIDBytes #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCanAcceptBytes :: (stream: CFWriteStreamRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamClose :: (stream: CFWriteStreamRef) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCopyDispatchQueue :: (stream: CFWriteStreamRef) -> dispatch_queue_t #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCopyError :: (stream: CFWriteStreamRef) -> CFErrorRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCopyProperty :: (stream: CFWriteStreamRef, propertyName: CFStreamPropertyKey) -> CFPropertyListRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCreateWithAllocatedBuffers :: (alloc: CFAllocatorRef, bufferAllocator: CFAllocatorRef) -> CFWriteStreamRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCreateWithBuffer :: (alloc: CFAllocatorRef, buffer: *u8, bufferCapacity: CFIndex) -> CFWriteStreamRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamCreateWithFile :: (alloc: CFAllocatorRef, fileURL: CFURLRef) -> CFWriteStreamRef #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamGetError :: (stream: CFWriteStreamRef) -> CFStreamError #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamGetStatus :: (stream: CFWriteStreamRef) -> CFStreamStatus #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamGetTypeID :: () -> CFTypeID #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamOpen :: (stream: CFWriteStreamRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamScheduleWithRunLoop :: (stream: CFWriteStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamSetClient :: (stream: CFWriteStreamRef, streamEvents: CFOptionFlags, clientCB: CFWriteStreamClientCallBack, clientContext: *CFStreamClientContext) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamSetDispatchQueue :: (stream: CFWriteStreamRef, q: dispatch_queue_t) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamSetProperty :: (stream: CFWriteStreamRef, propertyName: CFStreamPropertyKey, propertyValue: CFPropertyListRef) -> Boolean #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamUnscheduleFromRunLoop :: (stream: CFWriteStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFRunLoopMode) -> void #foreign corefoundation;
}

#if OS == .MACOS {
    CFWriteStreamWrite :: (stream: CFWriteStreamRef, buffer: *u8, bufferLength: CFIndex) -> CFIndex #foreign corefoundation;
}

#if OS == .MACOS {
    FSEventStreamInvalidate :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices;
}

#if OS == .MACOS {
    FSEventStreamRelease :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices;
}

#if OS == .MACOS {
    FSEventStreamScheduleWithRunLoop :: (streamRef: ConstFSEventStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFStringRef) #foreign CoreServices;
}

#if OS == .MACOS {
    FSEventStreamStart :: (streamRef: ConstFSEventStreamRef) -> bool #foreign CoreServices;
}

#if OS == .MACOS {
    FSEventStreamStop :: (streamRef: ConstFSEventStreamRef) #foreign CoreServices;
}

#if OS == .MACOS {
    FSEventStreamUnscheduleFromRunLoop :: (streamRef: ConstFSEventStreamRef, runLoop: CFRunLoopRef, runLoopMode: CFStringRef) #foreign CoreServices;
}

#if OS == .MACOS {
    kevent64 :: (kq: s32, changelist: *Kevent64, nchanges: s32, eventlist: *Kevent64, nevents: s32, flags: Kevent_Call_Flags, timeout: *timespec) -> s32 #foreign libc;
}

#if OS == .MACOS {
    kqueue :: () -> s32 #foreign libc;
}

#if OS == .MACOS {
    lock_set_create :: (task: task_t, new_lock_set: *lock_set_t, n_ulocks: s32, policy: s32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    lock_set_destroy :: (task: task_t, lock_set: lock_set_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    mach_msg :: (msg: *mach_msg_header_t, option: mach_msg_option_t, send_size: mach_msg_size_t, rcv_size: mach_msg_size_t, rcv_name: mach_port_name_t, timeout: mach_msg_timeout_t, notify: mach_port_name_t) -> mach_msg_return_t #foreign libc;
}

#if OS == .MACOS {
    mach_msg_overwrite :: (msg: *mach_msg_header_t, option: mach_msg_option_t, send_size: mach_msg_size_t, rcv_size: mach_msg_size_t, rcv_name: mach_port_name_t, timeout: mach_msg_timeout_t, notify: mach_port_name_t, rcv_msg: *mach_msg_header_t, rcv_limit: mach_msg_size_t) -> mach_msg_return_t #foreign libc;
}

#if OS == .MACOS {
    mach_ports_lookup :: (target_task: task_t, init_port_set: *mach_port_array_t, init_port_setCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    mach_ports_register :: (target_task: task_t, init_port_set: mach_port_array_t, init_port_setCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    mach_task_self :: () -> mach_port_t #foreign libc;
}

#if OS == .MACOS {
    mach_voucher_deallocate :: (voucher: mach_port_name_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_create :: (task: task_t, semaphore: *semaphore_t, policy: s32, value: s32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_destroy :: (task: task_t, semaphore: semaphore_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_signal :: (semaphore: semaphore_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_signal_all :: (semaphore: semaphore_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_signal_thread :: (semaphore: semaphore_t, thread: thread_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_timedwait :: (semaphore: semaphore_t, wait_time: mach_timespec_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_timedwait_signal :: (wait_semaphore: semaphore_t, signal_semaphore: semaphore_t, wait_time: mach_timespec_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_wait :: (semaphore: semaphore_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    semaphore_wait_signal :: (wait_semaphore: semaphore_t, signal_semaphore: semaphore_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_assign :: (task: task_t, new_set: processor_set_t, assign_threads: u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_assign_default :: (task: task_t, assign_threads: u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_create :: (target_task: task_t, ledgers: ledger_port_array_t, ledgersCnt: mach_msg_type_number_t, inherit_memory: u32, child_task: *task_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_generate_corpse :: (task: task_t, corpse_task_port: *mach_port_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_assignment :: (task: task_t, assigned_set: *processor_set_name_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_dyld_image_infos :: (task: task_inspect_t, dyld_images: *dyld_kernel_image_info_array_t, dyld_imagesCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_emulation_vector :: (task: task_t, vector_start: *s32, emulation_vector: *emulation_vector_t, emulation_vectorCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_exception_ports :: (task: task_inspect_t, exception_mask: exception_mask_t, masks: exception_mask_array_t, masksCnt: *mach_msg_type_number_t, old_handlers: exception_port_arrary_t, old_behaviors: exception_behavior_array_t, old_flavors: exception_flavor_array_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_mach_voucher :: (task: task_t, which: mach_voucher_selector_t, voucher: *ipc_voucher_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_special_port :: (task: task_inspect_t, which_port: s32, special_port: *mach_port_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_get_state :: (task: task_t, flavor: thread_state_flavor_t, old_state: thread_state_t, old_stateCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_info :: (target_task: task_name_t, flavor: task_flavor_t, task_info_out: task_info_t, task_info_outCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_inspect :: (task: task_inspect_t, flavor: task_inspect_flavor_t, info_out: task_inspect_info_t, info_outCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_map_corpse_info :: (task: task_t, corspe_task: task_t, kcd_addr_begin: *vm_address_t, kcd_size: *u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_map_corpse_info_64 :: (task: task_t, corspe_task: task_t, kcd_addr_begin: *mach_vm_address_t, kcd_size: *mach_vm_size_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_policy :: (task: task_t, policy: policy_t, base: policy_base_t, baseCnt: mach_msg_type_number_t, set_limit: u32, change: u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_policy_get :: (task: task_t, flavor: task_policy_flavor_t, policy_info: task_policy_t, policy_infoCnt: *mach_msg_type_number_t, get_default: *u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_policy_set :: (task: task_t, flavor: task_policy_flavor_t, policy_info: task_policy_t, policy_infoCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_purgable_info :: (task: task_t, stats: *task_purgable_info_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_register_dyld_get_process_state :: (task: task_t, dyld_process_state: *dyld_kernel_process_info_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_register_dyld_image_infos :: (task: task_t, dyld_images: dyld_kernel_image_info_array_t, dyld_imagesCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_register_dyld_set_dyld_state :: (task: task_t, dyld_state: u8) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_register_dyld_shared_cache_image_info :: (task: task_t, dyld_cache_image: dyld_kernel_image_info_t, no_cache: u32, private_cache: u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_resume :: (target_task: task_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_resume2 :: (suspend_token: task_suspension_token_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_sample :: (task: task_t, reply: mach_port_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_emulation :: (target_port: task_t, routine_entry_pt: vm_address_t, routine_number: s32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_emulation_vector :: (task: task_t, vector_start: s32, emulation_vector: emulation_vector_t, emulation_vectorCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_exception_ports :: (task: task_t, exception_mask: exception_mask_t, new_port: mach_port_t, behavior: exception_behavior_t, new_flavor: thread_state_flavor_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_info :: (target_task: task_t, flavor: task_flavor_t, task_info_in: task_info_t, task_info_inCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_mach_voucher :: (task: task_t, voucher: ipc_voucher_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_phys_footprint_limit :: (task: task_t, new_limit: s32, old_limit: *s32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_policy :: (task: task_t, pset: processor_set_t, policy: policy_t, base: policy_base_t, baseCnt: mach_msg_type_number_t, limit: policy_limit_t, limitCnt: mach_msg_type_number_t, change: u32) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_ras_pc :: (target_task: task_t, basepc: vm_address_t, boundspc: vm_address_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_special_port :: (task: task_t, which_port: s32, special_port: mach_port_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_set_state :: (task: task_t, flavor: thread_state_flavor_t, new_state: thread_state_t, new_stateCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_suspend :: (target_task: task_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_suspend2 :: (target_task: task_t, suspend_token: *task_suspension_token_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_swap_exception_ports :: (task: task_t, exception_mask: exception_mask_t, new_port: mach_port_t, behavior: exception_behavior_t, new_flavor: thread_state_flavor_t, masks: exception_mask_array_t, masksCnt: *mach_msg_type_number_t, old_handlerss: exception_port_arrary_t, old_behaviors: exception_behavior_array_t, old_flavors: exception_flavor_array_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_swap_mach_voucher :: (task: task_t, new_voucher: ipc_voucher_t, old_voucher: *ipc_voucher_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_terminate :: (target_task: task_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_threads :: (target_task: task_inspect_t, act_list: *thread_act_port_array_t, act_listCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_unregister_dyld_image_infos :: (task: task_t, dyld_images: dyld_kernel_image_info_array_t, dyld_imagesCnt: mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    task_zone_info :: (target_task: task_t, names: *mach_zone_name_array_t, namesCnt: *mach_msg_type_number_t, info: *task_zone_info_array_t, infoCnt: *mach_msg_type_number_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    thread_create :: (parent_task: task_t, child_act: *thread_act_t) -> kern_return_t #foreign libc;
}

#if OS == .MACOS {
    thread_create_running :: (parent_task: task_t, flavor: thread_state_flavor_t, new_state: thread_state_t, new_stateCnt: mach_msg_type_number_t, child_act: *thread_act_t) -> kern_return_t #foreign libc;
}

// Convenience operations

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_descriptor32_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_descriptor64_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_descriptor_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_copy :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_copy_options_t {
        return cast(mach_msg_copy_options_t) ((fields.__bitfield >> 8) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_descriptor32_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_descriptor64_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_descriptor_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_ports_descriptor32_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_ports_descriptor64_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_deallocate :: (fields: mach_msg_ool_ports_descriptor_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 255);
    }
}

#if OS == .MACOS {
    get_disposition :: (fields: mach_msg_port_descriptor_t) -> mach_msg_type_name_t {
        return cast(mach_msg_type_name_t) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_disposition :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_type_name_t {
        return cast(mach_msg_type_name_t) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_disposition :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_type_name_t {
        return cast(mach_msg_type_name_t) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_disposition :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_type_name_t {
        return cast(mach_msg_type_name_t) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_name :: (fields: mach_port_qos_t) -> bool {
        return ((fields.__bitfield >> 0) & 1) != 0;
    }
}

#if OS == .MACOS {
    get_pad1 :: (fields: mach_port_qos_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 2) & 1073741823);
    }
}

#if OS == .MACOS {
    get_pad1 :: (fields: mach_msg_ool_descriptor32_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_pad1 :: (fields: mach_msg_ool_descriptor64_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_pad1 :: (fields: mach_msg_ool_descriptor_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 16) & 255);
    }
}

#if OS == .MACOS {
    get_pad2 :: (fields: mach_msg_port_descriptor_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 65535);
    }
}

#if OS == .MACOS {
    get_pad3 :: (fields: mach_msg_type_descriptor_t) -> u32 {
        return cast(u32) ((fields.__bitfield >> 0) & 16777215);
    }
}

#if OS == .MACOS {
    get_prealloc :: (fields: mach_port_qos_t) -> bool {
        return ((fields.__bitfield >> 1) & 1) != 0;
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_type_descriptor_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_port_descriptor_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_descriptor32_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_descriptor64_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_descriptor_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_ports_descriptor32_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_ports_descriptor64_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    get_type :: (fields: mach_msg_ool_ports_descriptor_t) -> mach_msg_descriptor_type_t {
        return cast(mach_msg_descriptor_type_t) ((fields.__bitfield >> 24) & 255);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_descriptor32_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_descriptor64_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_descriptor_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_copy :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_copy_options_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65280)) | (((cast(u32) value) & 255) << 8);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_descriptor32_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_descriptor64_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_descriptor_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_ports_descriptor32_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_ports_descriptor64_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_deallocate :: (fields: *mach_msg_ool_ports_descriptor_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 255)) | ((value & 255) << 0);
    }
}

#if OS == .MACOS {
    set_disposition :: (fields: *mach_msg_port_descriptor_t, value: mach_msg_type_name_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | (((cast(u32) value) & 255) << 16);
    }
}

#if OS == .MACOS {
    set_disposition :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_type_name_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | (((cast(u32) value) & 255) << 16);
    }
}

#if OS == .MACOS {
    set_disposition :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_type_name_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | (((cast(u32) value) & 255) << 16);
    }
}

#if OS == .MACOS {
    set_disposition :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_type_name_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | (((cast(u32) value) & 255) << 16);
    }
}

#if OS == .MACOS {
    set_name :: (fields: *mach_port_qos_t, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 1)) | (((cast(u32) value) & 1) << 0);
    }
}

#if OS == .MACOS {
    set_pad1 :: (fields: *mach_port_qos_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4294967292)) | ((value & 1073741823) << 2);
    }
}

#if OS == .MACOS {
    set_pad1 :: (fields: *mach_msg_ool_descriptor32_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | ((value & 255) << 16);
    }
}

#if OS == .MACOS {
    set_pad1 :: (fields: *mach_msg_ool_descriptor64_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | ((value & 255) << 16);
    }
}

#if OS == .MACOS {
    set_pad1 :: (fields: *mach_msg_ool_descriptor_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16711680)) | ((value & 255) << 16);
    }
}

#if OS == .MACOS {
    set_pad2 :: (fields: *mach_msg_port_descriptor_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 65535)) | ((value & 65535) << 0);
    }
}

#if OS == .MACOS {
    set_pad3 :: (fields: *mach_msg_type_descriptor_t, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 16777215)) | ((value & 16777215) << 0);
    }
}

#if OS == .MACOS {
    set_prealloc :: (fields: *mach_port_qos_t, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 2)) | (((cast(u32) value) & 1) << 1);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_type_descriptor_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_port_descriptor_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_descriptor32_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_descriptor64_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_descriptor_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_ports_descriptor32_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_ports_descriptor64_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    set_type :: (fields: *mach_msg_ool_ports_descriptor_t, value: mach_msg_descriptor_type_t) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4278190080)) | (((cast(u32) value) & 255) << 24);
    }
}

#if OS == .MACOS {
    VM_MAKE_TAG :: (val: s32) -> s32 {
        return val << 24;
    }
}
