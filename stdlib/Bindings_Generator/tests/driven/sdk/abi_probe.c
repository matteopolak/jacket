#define LZ4_STATIC_LINKING_ONLY
#define LZ4_HC_STATIC_LINKING_ONLY
#define XXH_STATIC_LINKING_ONLY
#include "lz4.h"
#include "lz4hc.h"
#include "xxhash.h"
#include "meshoptimizer.h"
#include "mojoshader.h"
#include <ft2build.h>
#include FT_FREETYPE_H
#include <stddef.h>
#include <stdint.h>

uint64_t binding_size(int which) {
    static const size_t sizes[] = {
        sizeof(LZ4_stream_t), sizeof(LZ4_streamDecode_t), sizeof(LZ4_streamHC_t),
        sizeof(LZ4_stream_t_internal), sizeof(LZ4HC_CCtx_internal),
        sizeof(XXH32_state_t), sizeof(XXH64_state_t),
        sizeof(struct meshopt_Stream), sizeof(struct meshopt_Meshlet), sizeof(struct meshopt_Bounds),
        sizeof(FT_FaceRec), sizeof(FT_GlyphSlotRec), sizeof(FT_Bitmap), sizeof(FT_Vector),
        sizeof(MOJOSHADER_uniform), sizeof(MOJOSHADER_constant),
        sizeof(MOJOSHADER_parseData), sizeof(MOJOSHADER_preprocessData)
    };
    return which >= 0 && (size_t)which < sizeof(sizes)/sizeof(*sizes) ? sizes[which] : UINT64_MAX;
}
uint64_t binding_offset(int which) {
    static const size_t offsets[] = {
        offsetof(LZ4_stream_t_internal, dictionary),
        offsetof(LZ4HC_CCtx_internal, compressionLevel),
        offsetof(LZ4HC_CCtx_internal, dictCtx),
        offsetof(XXH32_state_t, mem32), offsetof(XXH64_state_t, mem64),
        offsetof(FT_FaceRec, glyph), offsetof(FT_GlyphSlotRec, bitmap)
    };
    return which >= 0 && (size_t)which < sizeof(offsets)/sizeof(*offsets) ? offsets[which] : UINT64_MAX;
}
int binding_mojoshader_version_constant(void) { return MOJOSHADER_VERSION; }
