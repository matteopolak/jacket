// Selected complete public arithmetic definitions; no SDK functions are replaced.
// Copyright Microsoft Corporation. Public SDK header SHA-256: f5c5a1325f94bf70fd1e1a607b7ff36f5559fae032101d0afef1275598e4738f
#include <stdint.h>
#include <stdlib.h>

typedef
enum D3D12_FILTER
    {
        D3D12_FILTER_MIN_MAG_MIP_POINT	= 0,
        D3D12_FILTER_MIN_MAG_POINT_MIP_LINEAR	= 0x1,
        D3D12_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT	= 0x4,
        D3D12_FILTER_MIN_POINT_MAG_MIP_LINEAR	= 0x5,
        D3D12_FILTER_MIN_LINEAR_MAG_MIP_POINT	= 0x10,
        D3D12_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR	= 0x11,
        D3D12_FILTER_MIN_MAG_LINEAR_MIP_POINT	= 0x14,
        D3D12_FILTER_MIN_MAG_MIP_LINEAR	= 0x15,
        D3D12_FILTER_MIN_MAG_ANISOTROPIC_MIP_POINT	= 0x54,
        D3D12_FILTER_ANISOTROPIC	= 0x55,
        D3D12_FILTER_COMPARISON_MIN_MAG_MIP_POINT	= 0x80,
        D3D12_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR	= 0x81,
        D3D12_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT	= 0x84,
        D3D12_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR	= 0x85,
        D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT	= 0x90,
        D3D12_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR	= 0x91,
        D3D12_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT	= 0x94,
        D3D12_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR	= 0x95,
        D3D12_FILTER_COMPARISON_MIN_MAG_ANISOTROPIC_MIP_POINT	= 0xd4,
        D3D12_FILTER_COMPARISON_ANISOTROPIC	= 0xd5,
        D3D12_FILTER_MINIMUM_MIN_MAG_MIP_POINT	= 0x100,
        D3D12_FILTER_MINIMUM_MIN_MAG_POINT_MIP_LINEAR	= 0x101,
        D3D12_FILTER_MINIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT	= 0x104,
        D3D12_FILTER_MINIMUM_MIN_POINT_MAG_MIP_LINEAR	= 0x105,
        D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_MIP_POINT	= 0x110,
        D3D12_FILTER_MINIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR	= 0x111,
        D3D12_FILTER_MINIMUM_MIN_MAG_LINEAR_MIP_POINT	= 0x114,
        D3D12_FILTER_MINIMUM_MIN_MAG_MIP_LINEAR	= 0x115,
        D3D12_FILTER_MINIMUM_MIN_MAG_ANISOTROPIC_MIP_POINT	= 0x154,
        D3D12_FILTER_MINIMUM_ANISOTROPIC	= 0x155,
        D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_POINT	= 0x180,
        D3D12_FILTER_MAXIMUM_MIN_MAG_POINT_MIP_LINEAR	= 0x181,
        D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT	= 0x184,
        D3D12_FILTER_MAXIMUM_MIN_POINT_MAG_MIP_LINEAR	= 0x185,
        D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_MIP_POINT	= 0x190,
        D3D12_FILTER_MAXIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR	= 0x191,
        D3D12_FILTER_MAXIMUM_MIN_MAG_LINEAR_MIP_POINT	= 0x194,
        D3D12_FILTER_MAXIMUM_MIN_MAG_MIP_LINEAR	= 0x195,
        D3D12_FILTER_MAXIMUM_MIN_MAG_ANISOTROPIC_MIP_POINT	= 0x1d4,
        D3D12_FILTER_MAXIMUM_ANISOTROPIC	= 0x1d5
    } 	D3D12_FILTER;

typedef
enum D3D12_FILTER_REDUCTION_TYPE
    {
        D3D12_FILTER_REDUCTION_TYPE_STANDARD	= 0,
        D3D12_FILTER_REDUCTION_TYPE_COMPARISON	= 1,
        D3D12_FILTER_REDUCTION_TYPE_MINIMUM	= 2,
        D3D12_FILTER_REDUCTION_TYPE_MAXIMUM	= 3
    } 	D3D12_FILTER_REDUCTION_TYPE;

typedef
enum D3D12_FILTER_TYPE
    {
        D3D12_FILTER_TYPE_POINT	= 0,
        D3D12_FILTER_TYPE_LINEAR	= 1
    } 	D3D12_FILTER_TYPE;

typedef
enum D3D12_SHADER_COMPONENT_MAPPING
    {
        D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_0	= 0,
        D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_1	= 1,
        D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_2	= 2,
        D3D12_SHADER_COMPONENT_MAPPING_FROM_MEMORY_COMPONENT_3	= 3,
        D3D12_SHADER_COMPONENT_MAPPING_FORCE_VALUE_0	= 4,
        D3D12_SHADER_COMPONENT_MAPPING_FORCE_VALUE_1	= 5
    } 	D3D12_SHADER_COMPONENT_MAPPING;

#define D3D12_SHADER_COMPONENT_MAPPING_MASK 0x7

#define D3D12_SHADER_COMPONENT_MAPPING_SHIFT 3

#define D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES (1<<(D3D12_SHADER_COMPONENT_MAPPING_SHIFT*4))

#define D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(Src0,Src1,Src2,Src3) ((((Src0)&D3D12_SHADER_COMPONENT_MAPPING_MASK)| \
                                                                (((Src1)&D3D12_SHADER_COMPONENT_MAPPING_MASK)<<D3D12_SHADER_COMPONENT_MAPPING_SHIFT)| \
                                                                (((Src2)&D3D12_SHADER_COMPONENT_MAPPING_MASK)<<(D3D12_SHADER_COMPONENT_MAPPING_SHIFT*2))| \
                                                                (((Src3)&D3D12_SHADER_COMPONENT_MAPPING_MASK)<<(D3D12_SHADER_COMPONENT_MAPPING_SHIFT*3))| \
                                                                D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES))

#define D3D12_DECODE_SHADER_4_COMPONENT_MAPPING(ComponentToExtract,Mapping) ((D3D12_SHADER_COMPONENT_MAPPING)(Mapping >> (D3D12_SHADER_COMPONENT_MAPPING_SHIFT*ComponentToExtract) & D3D12_SHADER_COMPONENT_MAPPING_MASK))

#define D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(0,1,2,3)

#define	D3D12_FILTER_REDUCTION_TYPE_MASK	( 0x3 )

#define	D3D12_FILTER_REDUCTION_TYPE_SHIFT	( 7 )

#define	D3D12_FILTER_TYPE_MASK	( 0x3 )

#define	D3D12_MIN_FILTER_SHIFT	( 4 )

#define	D3D12_MAG_FILTER_SHIFT	( 2 )

#define	D3D12_MIP_FILTER_SHIFT	( 0 )

#define	D3D12_ANISOTROPIC_FILTERING_BIT	( 0x40 )

#define D3D12_ENCODE_BASIC_FILTER( min, mag, mip, reduction )                                                     \
                                   ( ( D3D12_FILTER ) (                                                           \
                                   ( ( ( min ) & D3D12_FILTER_TYPE_MASK ) << D3D12_MIN_FILTER_SHIFT ) |           \
                                   ( ( ( mag ) & D3D12_FILTER_TYPE_MASK ) << D3D12_MAG_FILTER_SHIFT ) |           \
                                   ( ( ( mip ) & D3D12_FILTER_TYPE_MASK ) << D3D12_MIP_FILTER_SHIFT ) |           \
                                   ( ( ( reduction ) & D3D12_FILTER_REDUCTION_TYPE_MASK ) << D3D12_FILTER_REDUCTION_TYPE_SHIFT ) ) )

#define D3D12_ENCODE_ANISOTROPIC_FILTER( reduction )                                                  \
                                         ( ( D3D12_FILTER ) (                                         \
                                         D3D12_ANISOTROPIC_FILTERING_BIT |                            \
                                         D3D12_ENCODE_BASIC_FILTER( D3D12_FILTER_TYPE_LINEAR,         \
                                                                    D3D12_FILTER_TYPE_LINEAR,         \
                                                                    D3D12_FILTER_TYPE_LINEAR,         \
                                                                    reduction ) ) )

#define D3D12_ENCODE_MIN_MAG_ANISOTROPIC_MIP_POINT_FILTER( reduction )                                \
                                         ( ( D3D12_FILTER ) (                                         \
                                         D3D12_ANISOTROPIC_FILTERING_BIT |                            \
                                         D3D12_ENCODE_BASIC_FILTER( D3D12_FILTER_TYPE_LINEAR,         \
                                                                    D3D12_FILTER_TYPE_LINEAR,         \
                                                                    D3D12_FILTER_TYPE_POINT,          \
                                                                    reduction ) ) )

#define D3D12_DECODE_MIN_FILTER( D3D12Filter )                                                              \
                                 ( ( D3D12_FILTER_TYPE )                                                    \
                                 ( ( ( D3D12Filter ) >> D3D12_MIN_FILTER_SHIFT ) & D3D12_FILTER_TYPE_MASK ) )

#define D3D12_DECODE_MAG_FILTER( D3D12Filter )                                                              \
                                 ( ( D3D12_FILTER_TYPE )                                                    \
                                 ( ( ( D3D12Filter ) >> D3D12_MAG_FILTER_SHIFT ) & D3D12_FILTER_TYPE_MASK ) )

#define D3D12_DECODE_MIP_FILTER( D3D12Filter )                                                              \
                                 ( ( D3D12_FILTER_TYPE )                                                    \
                                 ( ( ( D3D12Filter ) >> D3D12_MIP_FILTER_SHIFT ) & D3D12_FILTER_TYPE_MASK ) )

#define D3D12_DECODE_FILTER_REDUCTION( D3D12Filter )                                                        \
                                 ( ( D3D12_FILTER_REDUCTION_TYPE )                                                      \
                                 ( ( ( D3D12Filter ) >> D3D12_FILTER_REDUCTION_TYPE_SHIFT ) & D3D12_FILTER_REDUCTION_TYPE_MASK ) )

#define D3D12_DECODE_IS_COMPARISON_FILTER( D3D12Filter )                                                    \
                                 ( D3D12_DECODE_FILTER_REDUCTION( D3D12Filter ) == D3D12_FILTER_REDUCTION_TYPE_COMPARISON )

#define D3D12_DECODE_IS_ANISOTROPIC_FILTER( D3D12Filter )                                               \
                            ( ( ( D3D12Filter ) & D3D12_ANISOTROPIC_FILTERING_BIT ) &&                  \
                            ( D3D12_FILTER_TYPE_LINEAR == D3D12_DECODE_MIN_FILTER( D3D12Filter ) ) &&   \
                            ( D3D12_FILTER_TYPE_LINEAR == D3D12_DECODE_MAG_FILTER( D3D12Filter ) ) )

int32_t binding_components(int32_t a, int32_t b, int32_t c, int32_t d) {
    return D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(a,b,c,d);
}
int32_t binding_component(int32_t index, int32_t mapping) {
    return D3D12_DECODE_SHADER_4_COMPONENT_MAPPING(index,mapping);
}
int32_t binding_filter(int32_t a, int32_t b, int32_t c, int32_t d) {
    return D3D12_ENCODE_BASIC_FILTER(a,b,c,d);
}
int32_t binding_anisotropic(int32_t reduction) {
    return D3D12_ENCODE_ANISOTROPIC_FILTER(reduction);
}
int32_t binding_filter_part(int32_t filter, int32_t part) {
    switch (part) {
        case 0: return D3D12_DECODE_MIN_FILTER(filter);
        case 1: return D3D12_DECODE_MAG_FILTER(filter);
        case 2: return D3D12_DECODE_MIP_FILTER(filter);
        case 3: return D3D12_DECODE_FILTER_REDUCTION(filter);
        default: abort();
    }
}
int32_t binding_is_comparison(int32_t filter) { return D3D12_DECODE_IS_COMPARISON_FILTER(filter); }
int32_t binding_is_anisotropic(int32_t filter) { return D3D12_DECODE_IS_ANISOTROPIC_FILTER(filter); }
