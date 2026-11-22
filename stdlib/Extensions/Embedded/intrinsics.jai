// Volatile accesses and the CPU hints and barriers firmware needs.

// Reads `pointer.*` exactly once, in program order with the other volatile accesses: for device
// registers, where the read itself has an effect. At compile time the address must be ordinary
// memory; there is no device to answer.
volatile_load :: (pointer: *$T) -> T #intrinsic;

// Writes `value` to `pointer.*` exactly once, in program order with the other volatile accesses.
volatile_store :: (pointer: *$T, value: T) #intrinsic;

#if CPU == .ARM64 {
    hint_instruction :: (number: s32) #intrinsic "llvm.aarch64.hint";
    data_sync_instruction :: (domain: s32) #intrinsic "llvm.aarch64.dsb";
    data_memory_instruction :: (domain: s32) #intrinsic "llvm.aarch64.dmb";
    instruction_sync_instruction :: (domain: s32) #intrinsic "llvm.aarch64.isb";

    // Sleeps the core until an interrupt (`wfi`).
    wait_for_interrupt :: () #no_context {
        hint_instruction(5);
    }

    // Sleeps the core until an event, which `send_event` on any core also raises (`wfe`).
    wait_for_event :: () #no_context {
        hint_instruction(2);
    }

    // Raises the event `wait_for_event` waits for (`sev`).
    send_event :: () #no_context {
        hint_instruction(4);
    }

    // Waits until every earlier memory access has completed (`dsb sy`).
    data_sync_barrier :: () #no_context {
        data_sync_instruction(15);
    }

    // Orders memory accesses made before and after it (`dmb sy`).
    data_memory_barrier :: () #no_context {
        data_memory_instruction(15);
    }

    // Discards instructions fetched before it, so a change to system state applies (`isb`).
    instruction_barrier :: () #no_context {
        instruction_sync_instruction(15);
    }
}

#if ARCHITECTURE == .ARM {
    hint_instruction :: (number: s32) #intrinsic "llvm.arm.hint";
    data_sync_instruction :: (domain: s32) #intrinsic "llvm.arm.dsb";
    data_memory_instruction :: (domain: s32) #intrinsic "llvm.arm.dmb";
    instruction_sync_instruction :: (domain: s32) #intrinsic "llvm.arm.isb";

    // Sleeps the core until an interrupt (`wfi`).
    wait_for_interrupt :: () #no_context {
        hint_instruction(3);
    }

    // Sleeps the core until an event, which `send_event` on any core also raises (`wfe`).
    wait_for_event :: () #no_context {
        hint_instruction(2);
    }

    // Raises the event `wait_for_event` waits for (`sev`).
    send_event :: () #no_context {
        hint_instruction(4);
    }

    // Waits until every earlier memory access has completed (`dsb sy`).
    data_sync_barrier :: () #no_context {
        data_sync_instruction(15);
    }

    // Orders memory accesses made before and after it (`dmb sy`).
    data_memory_barrier :: () #no_context {
        data_memory_instruction(15);
    }

    // Discards instructions fetched before it, so a change to system state applies (`isb`).
    instruction_barrier :: () #no_context {
        instruction_sync_instruction(15);
    }
}
