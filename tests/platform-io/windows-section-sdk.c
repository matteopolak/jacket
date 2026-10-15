// Independent fixture oracle: SDK image queries, not the production PE walker.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dbghelp.h>
#include <stdint.h>

__declspec(dllexport) void *platform_io_sdk_image_probe(int writable) {
    unsigned char *base = (unsigned char *)GetModuleHandleW(NULL);
    if (base == NULL) return NULL;
    PIMAGE_NT_HEADERS headers = ImageNtHeader(base);
    if (headers == NULL) return NULL;
    const IMAGE_SECTION_HEADER *sections = IMAGE_FIRST_SECTION(headers);
    for (unsigned i = 0; i < headers->FileHeader.NumberOfSections; ++i) {
        DWORD flags = sections[i].Characteristics;
        // ImageRvaToSection's documented file-mapped RVA contract is used only
        // for file-backed probes, never to infer a BSS/global storage boundary.
        if (sections[i].Misc.VirtualSize == 0 || sections[i].SizeOfRawData == 0) continue;
        if (writable ? ((flags & IMAGE_SCN_MEM_WRITE) && (flags & IMAGE_SCN_CNT_INITIALIZED_DATA))
                     : ((flags & IMAGE_SCN_CNT_CODE) && !(flags & IMAGE_SCN_MEM_WRITE)))
            return base + sections[i].VirtualAddress;
    }
    return NULL;
}

__declspec(dllexport) int platform_io_sdk_main_image_mapping(const void *pointer, int *readonly) {
    *readonly = 0;
    HMODULE image = GetModuleHandleW(NULL);
    if (image == NULL) return -1;
    MEMORY_BASIC_INFORMATION memory;
    SIZE_T queried = VirtualQuery(pointer, &memory, sizeof(memory));
    if (queried == 0)
        return GetLastError() == ERROR_INVALID_PARAMETER ? 0 : -1;
    if (queried != sizeof(memory)) return -1;
    if (memory.Type != MEM_IMAGE || memory.AllocationBase != image) return 0;
    DWORD protection = memory.Protect & 0xff;
    if (protection == PAGE_READONLY || protection == PAGE_EXECUTE_READ || protection == PAGE_EXECUTE)
        *readonly = 1;
    else if (protection != PAGE_READWRITE && protection != PAGE_WRITECOPY &&
             protection != PAGE_EXECUTE_READWRITE && protection != PAGE_EXECUTE_WRITECOPY)
        return -1;
    return 1;
}

__declspec(dllexport) int platform_io_sdk_section(const void *pointer, int *readonly,
                                                 const char **name, uint64_t *length) {
    *readonly = 0;
    *name = NULL;
    *length = 0;
    HMODULE image = GetModuleHandleW(NULL);
    if (image == NULL) return -1; // Fixture failure, distinct from not-a-section.
    MEMORY_BASIC_INFORMATION memory;
    SIZE_T queried = VirtualQuery(pointer, &memory, sizeof(memory));
    if (queried == 0) {
        return GetLastError() == ERROR_INVALID_PARAMETER ? 0 : -1;
    }
    if (queried != sizeof(memory)) return -1;
    if (memory.Type != MEM_IMAGE || memory.AllocationBase != image) return 0;
    PIMAGE_NT_HEADERS headers = ImageNtHeader(image);
    if (headers == NULL) return -1;
    uintptr_t rva = (uintptr_t)pointer - (uintptr_t)image;
    if (rva > UINT32_MAX) return -1;
    PIMAGE_SECTION_HEADER section = ImageRvaToSection(headers, image, (ULONG)rva);
    if (section == NULL) return 0;
    *readonly = (section->Characteristics & IMAGE_SCN_MEM_WRITE) == 0;
    *name = (const char *)section->Name;
    while (*length < IMAGE_SIZEOF_SHORT_NAME && section->Name[*length] != 0) ++*length;
    return 1;
}
