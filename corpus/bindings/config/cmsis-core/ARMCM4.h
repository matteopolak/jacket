/* Corpus device header for CMSIS-Core's Cortex-M4 core: the macros and IRQ numbers core_cm4.h reads. */
#ifndef ARMCM4_H
#define ARMCM4_H
#define __CM4_REV 0x0001U
#define __FPU_PRESENT 1U
#define __MPU_PRESENT 0U
#define __NVIC_PRIO_BITS 4U
#define __Vendor_SysTickConfig 0U
#define __VTOR_PRESENT 1U
typedef enum IRQn {
    Reset_IRQn = -15,
    NonMaskableInt_IRQn = -14,
    HardFault_IRQn = -13,
    SysTick_IRQn = -1,
    WWDG_IRQn = 0,
    UART0_IRQn = 1
} IRQn_Type;
#endif
