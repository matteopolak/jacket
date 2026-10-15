// Original Codex v2 consumer authored solely from the reviewed wire contract.
// Fixed offsets follow the reviewed canonical little-endian record field widths.
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned char *bytes;
static size_t length;
static uint16_t u16(size_t at) {
    assert(at <= length && length - at >= 2);
    return (uint16_t)((uint16_t)bytes[at] | (uint16_t)((uint16_t)bytes[at + 1] << 8));
}
static uint32_t u32(size_t at) {
    assert(at <= length && length - at >= 4);
    return (uint32_t)bytes[at] | ((uint32_t)bytes[at+1] << 8) |
           ((uint32_t)bytes[at+2] << 16) | ((uint32_t)bytes[at+3] << 24);
}
static int32_t signed16(uint16_t v) {
    return v & 0x8000u ? (int32_t)v - 65536 : (int32_t)v;
}
static size_t string_at(size_t field, size_t *cursor) {
    int32_t count = signed16(u16(field));
    uint16_t encoded = u16(field + 2);
    assert(count >= 0 && *cursor < length && (size_t)count < length - *cursor);
    if (encoded != 0) {
        int32_t offset = signed16((uint16_t)(encoded ^ 0x8000u));
        assert((int64_t)field + 2 + offset == (int64_t)*cursor);
    } else assert(count == 0);
    size_t start = *cursor;
    assert(bytes[start + (size_t)count] == 0);
    *cursor += (size_t)count + 1;
    return start;
}
int main(int argc, char **argv) {
    assert(argc == 2);
    FILE *f = fopen(argv[1], "rb");
    assert(f);
    assert(fseek(f, 0, SEEK_END) == 0);
    long size = ftell(f);
    assert(size >= 332);
    length = (size_t)size;
    assert(fseek(f, 0, SEEK_SET) == 0);
    bytes = malloc(length);
    assert(bytes && fread(bytes, 1, length, f) == length);
    assert(fclose(f) == 0);
    assert(u32(0) == 0x05040003u && u32(4) == 2);
    uint32_t modules = u32(40), files = u32(44), procedures = u32(48);
    assert(modules > 0 && modules < length / 9 && files > 0 && files < length / 9);
    assert(procedures > 0 && procedures < length / 80);
    size_t cursor = 332;
    (void)string_at(314, &cursor); // project
    (void)string_at(318, &cursor); // executable
    (void)string_at(326, &cursor); // comment is physically before URL
    (void)string_at(322, &cursor); // URL
    for (uint32_t i = 0; i < modules; ++i) {
        size_t record = cursor;
        assert(cursor <= length && length - cursor >= 9);
        cursor += 9;
        (void)string_at(record + 1, &cursor);
        (void)string_at(record + 5, &cursor);
    }
    for (uint32_t i = 0; i < files; ++i) {
        size_t record = cursor;
        assert(cursor <= length && length - cursor >= 9);
        assert(u32(record + 4) < modules);
        cursor += 9;
        (void)string_at(record, &cursor);
    }
    int conditional_found = 0, allocation_found = 0;
    for (uint32_t i = 0; i < procedures; ++i) {
        size_t record = cursor;
        assert(cursor <= length && length - cursor >= 80);
        assert(u32(record) < files && u32(record + 24) > 0);
        cursor += 80;
        size_t name = string_at(record + 76, &cursor);
        if (strcmp((const char *)bytes + name, "conditional") == 0) {
            assert(u32(record + 28) == 10 && u32(record + 32) == 2);
            conditional_found = 1;
        }
        if (strcmp((const char *)bytes + name, "allocation") == 0) {
            assert(u32(record + 56) == 3 && u32(record + 60) == 3);
            allocation_found = 1;
        }
    }
    assert(cursor == length && conditional_found && allocation_found);
    free(bytes);
    puts("ok");
}
