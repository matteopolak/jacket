# Machine_X64

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `x86_Vendor :: enum u8`
- `x86_Feature_Leaf :: enum u16`
- `x86_Feature_Flag :: enum u16`
- `Cpu_X86 :: struct`
- `get_cpu_info :: () -> Cpu_X86`
- `check_feature :: (leaves: [Leaf.NUMBER] u32, flag: x86_Feature_Flag) -> bool`
- `enable_feature :: (leaves: *[Leaf.NUMBER] u32, flag: x86_Feature_Flag)`
- `disable_feature :: (leaves: *[Leaf.NUMBER] u32, flag: x86_Feature_Flag)`
- `Prefetch_Hint :: enum`
- `prefetch :: (pointer: *void, $hint: Prefetch_Hint) #expand`
- `rdtsc :: () -> u64 #expand`
- `rdtscp :: () -> (timestamp: u64, msr: u32) #expand`
- `rdseed :: () -> (success: bool, value: u64) #expand`
- `rdrand :: () -> (success: bool, value: u64) #expand`
- `mfence :: () #expand`
- `lfence :: () #expand`
- `sfence :: () #expand`
- `pause :: () #expand`
- `Machine_Options_X86 :: struct`
- `get_machine_options_x86 :: (build_options: *$T) -> *Machine_Options_X86 #modify`

## Public type members

#### `x86_Vendor` (enum)

- `UNKNOWN             :: 0`
- `INTEL               :: 1`
- `AMD                 :: 2`

#### `x86_Feature_Leaf` (enum)

- `_01H_EDX            :: 0`
- `_01H_ECX            :: 1`
- `_19H_EBX            :: 2`
- `_01H_EXT_EDX        :: 3`
- `_01H_EXT_ECX        :: 4`
- `_08H_EXT_EBX        :: 5`
- `_07H_00H_EBX        :: 6`
- `_07H_00H_ECX        :: 7`
- `_07H_00H_EDX        :: 8`
- `_07H_01H_EAX        :: 9`
- `_0DH_01H_EAX        :: 10`
- `NUMBER              :: 11`

#### `x86_Feature_Flag` (enum)

- `FPU                 :: 32 * Leaf._01H_EDX     + 0`
- `VME                 :: 32 * Leaf._01H_EDX     + 1`
- `DE                  :: 32 * Leaf._01H_EDX     + 2`
- `PSE                 :: 32 * Leaf._01H_EDX     + 3`
- `TSC                 :: 32 * Leaf._01H_EDX     + 4`
- `MSR                 :: 32 * Leaf._01H_EDX     + 5`
- `PAE                 :: 32 * Leaf._01H_EDX     + 6`
- `MCE                 :: 32 * Leaf._01H_EDX     + 7`
- `CX8                 :: 32 * Leaf._01H_EDX     + 8`
- `APIC                :: 32 * Leaf._01H_EDX     + 9`
- `SEP                 :: 32 * Leaf._01H_EDX     + 11`
- `MTRR                :: 32 * Leaf._01H_EDX     + 12`
- `PGE                 :: 32 * Leaf._01H_EDX     + 13`
- `MCA                 :: 32 * Leaf._01H_EDX     + 14`
- `CMOV                :: 32 * Leaf._01H_EDX     + 15`
- `PAT                 :: 32 * Leaf._01H_EDX     + 16`
- `PSE_36              :: 32 * Leaf._01H_EDX     + 17`
- `PSN                 :: 32 * Leaf._01H_EDX     + 18`
- `CLFLUSH             :: 32 * Leaf._01H_EDX     + 19`
- `DS                  :: 32 * Leaf._01H_EDX     + 21`
- `ACPI                :: 32 * Leaf._01H_EDX     + 22`
- `MMX                 :: 32 * Leaf._01H_EDX     + 23`
- `FXSR                :: 32 * Leaf._01H_EDX     + 24`
- `SSE                 :: 32 * Leaf._01H_EDX     + 25`
- `SSE2                :: 32 * Leaf._01H_EDX     + 26`
- `SS                  :: 32 * Leaf._01H_EDX     + 27`
- `HTT                 :: 32 * Leaf._01H_EDX     + 28`
- `TM                  :: 32 * Leaf._01H_EDX     + 29`
- `IA64                :: 32 * Leaf._01H_EDX     + 30`
- `PBE                 :: 32 * Leaf._01H_EDX     + 31`
- `SSE3                :: 32 * Leaf._01H_ECX     + 0`
- `PCLMULQDQ           :: 32 * Leaf._01H_ECX     + 1`
- `DTEST64             :: 32 * Leaf._01H_ECX     + 2`
- `MONITOR             :: 32 * Leaf._01H_ECX     + 3`
- `DS_CPL              :: 32 * Leaf._01H_ECX     + 4`
- `VMX                 :: 32 * Leaf._01H_ECX     + 5`
- `SMX                 :: 32 * Leaf._01H_ECX     + 6`
- `EIST                :: 32 * Leaf._01H_ECX     + 7`
- `TM2                 :: 32 * Leaf._01H_ECX     + 8`
- `SSSE3               :: 32 * Leaf._01H_ECX     + 9`
- `CNXT_ID             :: 32 * Leaf._01H_ECX     + 10`
- `SDBG                :: 32 * Leaf._01H_ECX     + 11`
- `FMA                 :: 32 * Leaf._01H_ECX     + 12`
- `CX16                :: 32 * Leaf._01H_ECX     + 13`
- `XTPR                :: 32 * Leaf._01H_ECX     + 14`
- `PDCM                :: 32 * Leaf._01H_ECX     + 15`
- `PCID                :: 32 * Leaf._01H_ECX     + 17`
- `DCA                 :: 32 * Leaf._01H_ECX     + 18`
- `SSE4_1              :: 32 * Leaf._01H_ECX     + 19`
- `SSE4_2              :: 32 * Leaf._01H_ECX     + 20`
- `X2APIC              :: 32 * Leaf._01H_ECX     + 21`
- `MOVBE               :: 32 * Leaf._01H_ECX     + 22`
- `POPCNT              :: 32 * Leaf._01H_ECX     + 23`
- `TSC_DEADLINE        :: 32 * Leaf._01H_ECX     + 24`
- `AES                 :: 32 * Leaf._01H_ECX     + 25`
- `XSAVE               :: 32 * Leaf._01H_ECX     + 26`
- `OSXSAVE             :: 32 * Leaf._01H_ECX     + 27`
- `AVX                 :: 32 * Leaf._01H_ECX     + 28`
- `F16C                :: 32 * Leaf._01H_ECX     + 29`
- `RDRAND              :: 32 * Leaf._01H_ECX     + 30`
- `HYPERVISOR          :: 32 * Leaf._01H_ECX     + 31`
- `AESKLE              :: 32 * Leaf._19H_EBX     + 0`
- `AESKLE_WIDE         :: 32 * Leaf._19H_EBX     + 2`
- `KEY_LOCKER_MSR      :: 32 * Leaf._19H_EBX     + 4`
- `SYSCALL_SYSRET      :: 32 * Leaf._01H_EXT_EDX + 11`
- `MP                  :: 32 * Leaf._01H_EXT_EDX + 19`
- `NX                  :: 32 * Leaf._01H_EXT_EDX + 20`
- `MMX_EXT             :: 32 * Leaf._01H_EXT_EDX + 22`
- `FXSR_OPT            :: 32 * Leaf._01H_EXT_EDX + 25`
- `PDPE1GB             :: 32 * Leaf._01H_EXT_EDX + 26`
- `RDTSCP              :: 32 * Leaf._01H_EXT_EDX + 27`
- `_64BIT_MODE         :: 32 * Leaf._01H_EXT_EDX + 29`
- `_3DNOW_EXT          :: 32 * Leaf._01H_EXT_EDX + 30`
- `_3DNOW              :: 32 * Leaf._01H_EXT_EDX + 31`
- `LAHF_SAHF           :: 32 * Leaf._01H_EXT_ECX + 0`
- `CMP_LEGACY          :: 32 * Leaf._01H_EXT_ECX + 1`
- `SVM                 :: 32 * Leaf._01H_EXT_ECX + 2`
- `EXTAPIC             :: 32 * Leaf._01H_EXT_ECX + 3`
- `CR8_LEGACY          :: 32 * Leaf._01H_EXT_ECX + 4`
- `ABM                 :: 32 * Leaf._01H_EXT_ECX + 5`
- `SSE4A               :: 32 * Leaf._01H_EXT_ECX + 6`
- `MISALIGNED_SSE      :: 32 * Leaf._01H_EXT_ECX + 7`
- `_3DNOW_PREFETCH     :: 32 * Leaf._01H_EXT_ECX + 8`
- `OSVW                :: 32 * Leaf._01H_EXT_ECX + 9`
- `IBS                 :: 32 * Leaf._01H_EXT_ECX + 10`
- `XOP                 :: 32 * Leaf._01H_EXT_ECX + 11`
- `SKINIT              :: 32 * Leaf._01H_EXT_ECX + 12`
- `WDT                 :: 32 * Leaf._01H_EXT_ECX + 13`
- `LWP                 :: 32 * Leaf._01H_EXT_ECX + 15`
- `FMA4                :: 32 * Leaf._01H_EXT_ECX + 16`
- `TCE                 :: 32 * Leaf._01H_EXT_ECX + 17`
- `NODE_ID             :: 32 * Leaf._01H_EXT_ECX + 19`
- `TBM                 :: 32 * Leaf._01H_EXT_ECX + 21`
- `TOPOLOGY_EXTENSIONS :: 32 * Leaf._01H_EXT_ECX + 22`
- `PERFCTR_CORE        :: 32 * Leaf._01H_EXT_ECX + 23`
- `PERFCTR_NB          :: 32 * Leaf._01H_EXT_ECX + 24`
- `DBX                 :: 32 * Leaf._01H_EXT_ECX + 26`
- `PERFTSC             :: 32 * Leaf._01H_EXT_ECX + 27`
- `PCX_L2I             :: 32 * Leaf._01H_EXT_ECX + 28`
- `CLZERO              :: 32 * Leaf._08H_EXT_EBX + 0`
- `INVLPGB             :: 32 * Leaf._08H_EXT_EBX + 3`
- `RDPRU               :: 32 * Leaf._08H_EXT_EBX + 4`
- `MCOMMIT             :: 32 * Leaf._08H_EXT_EBX + 8`
- `WBNOINVD            :: 32 * Leaf._08H_EXT_EBX + 9`
- `FSGSBASE            :: 32 * Leaf._07H_00H_EBX + 0`
- `TSC_ADJUST          :: 32 * Leaf._07H_00H_EBX + 1`
- `SGX                 :: 32 * Leaf._07H_00H_EBX + 2`
- `BMI1                :: 32 * Leaf._07H_00H_EBX + 3`
- `HLE                 :: 32 * Leaf._07H_00H_EBX + 4`
- `AVX2                :: 32 * Leaf._07H_00H_EBX + 5`
- `FDP_EXCPTN_ONLY     :: 32 * Leaf._07H_00H_EBX + 6`
- `SMEP                :: 32 * Leaf._07H_00H_EBX + 7`
- `BMI2                :: 32 * Leaf._07H_00H_EBX + 8`
- `ENHANCED_REP        :: 32 * Leaf._07H_00H_EBX + 9`
- `INVPCID             :: 32 * Leaf._07H_00H_EBX + 10`
- `RTM                 :: 32 * Leaf._07H_00H_EBX + 11`
- `RDT_M               :: 32 * Leaf._07H_00H_EBX + 12`
- `DEP_FPU_CS_DS       :: 32 * Leaf._07H_00H_EBX + 13`
- `MPX                 :: 32 * Leaf._07H_00H_EBX + 14`
- `RDT_A               :: 32 * Leaf._07H_00H_EBX + 15`
- `AVX512F             :: 32 * Leaf._07H_00H_EBX + 16`
- `AVX512DQ            :: 32 * Leaf._07H_00H_EBX + 17`
- `RDSEED              :: 32 * Leaf._07H_00H_EBX + 18`
- `ADX                 :: 32 * Leaf._07H_00H_EBX + 19`
- `SMAP                :: 32 * Leaf._07H_00H_EBX + 20`
- `AVX512_IFMA         :: 32 * Leaf._07H_00H_EBX + 21`
- `PCOMMIT             :: 32 * Leaf._07H_00H_EBX + 22`
- `CLFLUSHOPT          :: 32 * Leaf._07H_00H_EBX + 23`
- `CLWB                :: 32 * Leaf._07H_00H_EBX + 24`
- `INTEL_PT            :: 32 * Leaf._07H_00H_EBX + 25`
- `AVX512PF            :: 32 * Leaf._07H_00H_EBX + 26`
- `AVX512ER            :: 32 * Leaf._07H_00H_EBX + 27`
- `AVX512CD            :: 32 * Leaf._07H_00H_EBX + 28`
- `SHA                 :: 32 * Leaf._07H_00H_EBX + 29`
- `AVX512BW            :: 32 * Leaf._07H_00H_EBX + 30`
- `AVX512VL            :: 32 * Leaf._07H_00H_EBX + 31`
- `PREFETCHWT1         :: 32 * Leaf._07H_00H_ECX + 0`
- `AVX512_VBMI         :: 32 * Leaf._07H_00H_ECX + 1`
- `UMIP                :: 32 * Leaf._07H_00H_ECX + 2`
- `PKU                 :: 32 * Leaf._07H_00H_ECX + 3`
- `OSPKE               :: 32 * Leaf._07H_00H_ECX + 4`
- `WAITPKG             :: 32 * Leaf._07H_00H_ECX + 5`
- `AVX512_VBMI2        :: 32 * Leaf._07H_00H_ECX + 6`
- `CET_SS              :: 32 * Leaf._07H_00H_ECX + 7`
- `GFNI                :: 32 * Leaf._07H_00H_ECX + 8`
- `VAES                :: 32 * Leaf._07H_00H_ECX + 9`
- `VPCLMULQDQ          :: 32 * Leaf._07H_00H_ECX + 10`
- `AVX512_VNNI         :: 32 * Leaf._07H_00H_ECX + 11`
- `AVX512_BITALG       :: 32 * Leaf._07H_00H_ECX + 12`
- `TME_EN              :: 32 * Leaf._07H_00H_ECX + 13`
- `AVX512_VPOPCNTDQ    :: 32 * Leaf._07H_00H_ECX + 14`
- `FIVE_LEVEL_PAGING   :: 32 * Leaf._07H_00H_ECX + 16`
- `MAWAU_0             :: 32 * Leaf._07H_00H_ECX + 17`
- `MAWAU_1             :: 32 * Leaf._07H_00H_ECX + 18`
- `MAWAU_2             :: 32 * Leaf._07H_00H_ECX + 19`
- `MAWAU_3             :: 32 * Leaf._07H_00H_ECX + 20`
- `MAWAU_4             :: 32 * Leaf._07H_00H_ECX + 21`
- `RDPID               :: 32 * Leaf._07H_00H_ECX + 22`
- `KL                  :: 32 * Leaf._07H_00H_ECX + 23`
- `CLDEMOTE            :: 32 * Leaf._07H_00H_ECX + 25`
- `MOVDIRI             :: 32 * Leaf._07H_00H_ECX + 27`
- `MOVDIR64B           :: 32 * Leaf._07H_00H_ECX + 28`
- `ENQCMD              :: 32 * Leaf._07H_00H_ECX + 29`
- `SGX_LC              :: 32 * Leaf._07H_00H_ECX + 30`
- `PKS                 :: 32 * Leaf._07H_00H_ECX + 31`
- `AVX512_4VNNIW       :: 32 * Leaf._07H_00H_EDX + 2`
- `AVX512_4FMAPS       :: 32 * Leaf._07H_00H_EDX + 3`
- `FSRM                :: 32 * Leaf._07H_00H_EDX + 4`
- `AVX512_VP2INTERSECT :: 32 * Leaf._07H_00H_EDX + 8`
- `SRBDS_CTRL          :: 32 * Leaf._07H_00H_EDX + 9`
- `MD_CLEAR            :: 32 * Leaf._07H_00H_EDX + 10`
- `TSX_FORCE_ABORT     :: 32 * Leaf._07H_00H_EDX + 13`
- `SERIALIZE           :: 32 * Leaf._07H_00H_EDX + 14`
- `HYBRID              :: 32 * Leaf._07H_00H_EDX + 15`
- `TSXLDTRK            :: 32 * Leaf._07H_00H_EDX + 16`
- `PCONFIG             :: 32 * Leaf._07H_00H_EDX + 18`
- `LBR                 :: 32 * Leaf._07H_00H_EDX + 19`
- `CET_IBT             :: 32 * Leaf._07H_00H_EDX + 20`
- `AMX_BF16            :: 32 * Leaf._07H_00H_EDX + 22`
- `AVX512_FP16         :: 32 * Leaf._07H_00H_EDX + 23`
- `AMX_TILE            :: 32 * Leaf._07H_00H_EDX + 24`
- `AMX_INT8            :: 32 * Leaf._07H_00H_EDX + 25`
- `SPEC_CTRL           :: 32 * Leaf._07H_00H_EDX + 26`
- `STIBP               :: 32 * Leaf._07H_00H_EDX + 27`
- `L1D_FLUSH           :: 32 * Leaf._07H_00H_EDX + 28`
- `ARCH_CAPABILITIES   :: 32 * Leaf._07H_00H_EDX + 29`
- `CORE_CAPABILITIES   :: 32 * Leaf._07H_00H_EDX + 30`
- `SSBD                :: 32 * Leaf._07H_00H_EDX + 31`
- `AVX_VNNI            :: 32 * Leaf._07H_01H_EAX + 4`
- `AVX512_BF16         :: 32 * Leaf._07H_01H_EAX + 5`
- `FZRM                :: 32 * Leaf._07H_01H_EAX + 10`
- `FSRS                :: 32 * Leaf._07H_01H_EAX + 11`
- `FSRCS               :: 32 * Leaf._07H_01H_EAX + 12`
- `HRESET              :: 32 * Leaf._07H_01H_EAX + 22`
- `LAM                 :: 32 * Leaf._07H_01H_EAX + 26`
- `XSAVEOPT            :: 32 * Leaf._0DH_01H_EAX + 0`
- `XSAVEC              :: 32 * Leaf._0DH_01H_EAX + 1`
- `XGETBV_ECX1         :: 32 * Leaf._0DH_01H_EAX + 2`
- `XSAVES_XRSTORS      :: 32 * Leaf._0DH_01H_EAX + 3`
- `XFD                 :: 32 * Leaf._0DH_01H_EAX + 4`

#### `Cpu_X86` (struct)

- `vendor: x86_Vendor`
- `feature_leaves: [Leaf.NUMBER] u32`

#### `Prefetch_Hint` (enum)

- `T0  :: 0`
- `T1  :: 1`
- `T2  :: 2`
- `NTA :: 3`

#### `Machine_Options_X86` (struct)

- `features: struct { leaves: [Leaf.NUMBER]u32; }`

## Behavioral contract and constraints

The module contains routines specific to x86-64 CPUs. It is target-restricted to x64; generic equivalents belong in portable modules. Callers must satisfy instruction/feature prerequisites described by each routine. No runtime dispatch or non-x64 fallback is implied.
