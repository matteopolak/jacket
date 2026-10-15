"""C mirror layouts from reviewed public records; never SDK layout aliases."""
import re
from rpmalloc_context import ORDER, RAW_NAMES, rewrite_records, raw_records


def mirror_records(spec, raw=False):
    records = rewrite_records(spec)
    if not raw:
        return records
    source = raw_records(spec, records)
    inverse = {v: k for k, v in RAW_NAMES.items()}
    return {owner: [(name, re.sub(r'\b(?:'+ '|'.join(inverse)+r')\b',
                                 lambda m: inverse[m[0]], ty), condition)
                    for name, ty, condition in source[RAW_NAMES[owner]]]
            for owner in ORDER}


def mirror_c_header(spec, raw):
    records = mirror_records(spec, raw)
    lines = ['/* Generated reviewed public layouts, independently organized. */',
             '#include <stdint.h>', '#include <stddef.h>']
    lines += ['typedef struct Public_'+name+' Public_'+name+';' for name in ORDER]
    scalars = {'s64':'int64_t','u64':'uint64_t','s32':'int32_t','u32':'uint32_t','void':'void',
               'size_t':'uint64_t', '_int':'int32_t', 'int32_t':'int32_t',
               'atomic32_t':'Public_Atomic32', 'atomic64_t':'Public_Atomic64', 'atomicptr_t':'Public_AtomicPtr'}
    if raw:
        lines += ['#if defined(_MSC_VER)',
                  'typedef volatile int32_t Public_Atomic32;',
                  'typedef volatile int64_t Public_Atomic64;',
                  'typedef volatile void* Public_AtomicPtr;', '#else',
                  'typedef volatile _Atomic(int32_t) Public_Atomic32;',
                  'typedef volatile _Atomic(int64_t) Public_Atomic64;',
                  'typedef volatile _Atomic(void*) Public_AtomicPtr;', '#endif']
    for name in ORDER:
        outer = name == 'Size_Class_Use'
        if outer: lines += ['#if ENABLE_STATISTICS']
        lines += ['struct Public_'+name+' {']
        for field, ty, condition in records[name]:
            if condition: lines += ['#if '+condition.replace('RPMALLOC_FIRST_CLASS_HEAPS','BINDING_RPMALLOC_PUBLIC_HEAPS')]
            array = re.match(r'\[(\d+)\]\s*(.*)',ty)
            count = array[1] if array else ''
            ty = array[2] if array else ty
            pointers = len(ty)-len(ty.lstrip('*'))
            ty = ty.lstrip('*').strip()
            if ty.startswith('enum_flags u32'): native = 'uint32_t'
            elif ty in scalars: native = scalars[ty]
            elif ty in records: native = 'Public_'+ty
            else: raise ValueError('Unreviewed public C mirror field type: '+ty)
            lines += [native+' '+('*'*pointers)+field+('['+count+']' if count else '')+';']
            if condition: lines += ['#endif']
        lines += ['};']
        if outer: lines += ['#endif']
    lines += ['_Static_assert(sizeof(Public_Span)==96,"reviewed public span");']
    return '\n'.join(lines)+'\n'


def public_c_header(spec):
    return '#if BINDING_RPMALLOC_RAW\n'+mirror_c_header(spec, True)+'#else\n'+mirror_c_header(spec, False)+'#endif\n'


def layout_probes(spec, raw=False, public_heaps=True):
    records = mirror_records(spec, raw)
    rows = [(owner,name,condition) for owner in ORDER for name,_,condition in records[owner]]
    code = '/* Generated reviewed public C mirror layout measurements. */\n'
    groups = [('layout_size',[(name,'','ENABLE_STATISTICS' if name=='Size_Class_Use' else '') for name in ORDER]),
              ('layout_offset',rows),('layout_width',rows)]
    for function, entries in groups:
        code += 'uint64_t API('+function+')(unsigned index) { switch(index) {\n'
        for i,(owner,name,condition) in enumerate(entries):
            condition = condition or ('ENABLE_STATISTICS' if owner=='Size_Class_Use' else '')
            if condition: code += '#if '+condition.replace('RPMALLOC_FIRST_CLASS_HEAPS','BINDING_RPMALLOC_PUBLIC_HEAPS')+'\n'
            expression = ('sizeof(Public_'+owner+')' if function=='layout_size' else
                          'offsetof(Public_'+owner+','+name+')' if function=='layout_offset' else
                          'sizeof(((Public_'+owner+'*)0)->'+name+')')
            code += 'case '+str(i)+':return '+expression+';\n'
            if condition: code += '#endif\n'
        code += 'default: abort(); } }\n'
    expressions = dict(Heap_Size_Class='heap.size_class[0]',Span_Cache='heap.span_cache',
                       Span_Large_Cache='heap.span_large_cache[0]',Span_Use='heap.span_use[0]',
                       Size_Class_Use='heap.size_class_use[0]',Span='heap.span_reserve.*',Heap='heap.*')
    jai = '// Generated reviewed public record/C mirror ABI assertions.\n'
    for stats in (True,False):
        function = (('raw_' if public_heaps else 'raw_base_') if raw else '')+('statistics' if stats else 'no_statistics')
        bridge = (('Raw' if public_heaps else 'RawBase') if raw else '')+('Bridge' if stats else 'NoStatsBridge')
        def active(condition):
            if condition=='RPMALLOC_FIRST_CLASS_HEAPS': return public_heaps
            return not condition or condition=='ENABLE_THREAD_CACHE' or bool(stats)
        jai += 'canonical_layout_'+function+' :: (heap: *$T) {\n'
        for i,owner in enumerate(ORDER):
            if owner=='Size_Class_Use' and not stats: continue
            if owner=='Span_Use' and not stats: continue
            var='value'+str(i)
            jai += '    '+var+': type_of('+expressions[owner]+');\n'
            jai += '    assert(size_of(type_of('+var+')) == '+bridge+'.layout_size('+str(i)+'));\n'
            for j,(record,name,condition) in enumerate(rows):
                if record != owner or not active(condition): continue
                jai += '    assert(cast(u64)*'+var+'.'+name+' - cast(u64)*'+var+' == '+bridge+'.layout_offset('+str(j)+'));\n'
                jai += '    assert(size_of(type_of('+var+'.'+name+')) == '+bridge+'.layout_width('+str(j)+'));\n'
        jai += '}\n'
    return code,jai
