"""One native domain inventory for generated imports and real C compilation."""
from itertools import product
from config import RPMALLOC_SDK_BUILD_OPTIONS

PROFILES = []
for raw, public_heaps in ((False, True), (True, False), (True, True)):
    for statistics in (True, False):
        for global_cache, thread_cache, assertions, validation in product((True, False), repeat=4):
            suffix = (("_raw" if public_heaps else "_raw_base") if raw else "")+("" if statistics else "_nostats")
            suffix += "" if global_cache else "_no_global_cache"
            suffix += "" if thread_cache else "_no_thread_cache"
            suffix += "" if assertions else "_no_asserts"
            suffix += "" if validation else "_no_validate_args"
            prefix = "binding_rpmalloc"+suffix+"_"
            condition = ("RAW && "+("PUBLIC_HEAPS" if public_heaps else "!PUBLIC_HEAPS") if raw else "!RAW")
            condition += " && "+("STATISTICS" if statistics else "!STATISTICS")
            condition += " && "+("GLOBAL_CACHE" if global_cache else "!GLOBAL_CACHE")
            condition += " && "+("THREAD_CACHE" if thread_cache else "!THREAD_CACHE")
            condition += " && "+("ASSERTS" if assertions else "!ASSERTS")
            condition += " && "+("VALIDATE_ARGS" if validation else "!VALIDATE_ARGS")
            PROFILES.append(dict(raw=raw, public_heaps=public_heaps, statistics=statistics,
                                 global_cache=global_cache, thread_cache=thread_cache,
                                 assertions=assertions, validation=validation, condition=condition,
                                 sdk="binding_rpmalloc_sdk"+suffix,
                                 callbacks="binding_rpmalloc_callbacks"+suffix,
                                 symbol_prefix=prefix if suffix else "",
                                 facade_prefix=prefix+"facade_", callback_prefix=prefix,
                                 options=dict(RPMALLOC_SDK_BUILD_OPTIONS, ENABLE_STATISTICS=int(statistics),
                                              ENABLE_GLOBAL_CACHE=int(global_cache), ENABLE_THREAD_CACHE=int(thread_cache),
                                              ENABLE_ASSERTS=int(assertions), ENABLE_VALIDATE_ARGS=int(validation),
                                              BINDING_RPMALLOC_RAW=int(raw),
                                              BINDING_RPMALLOC_PUBLIC_HEAPS=int(public_heaps))))


def select_blocks(render):
    return '\n'.join('#if '+p['condition']+' {\n'+render(p)+'\n}' for p in PROFILES)+'\n'


def parameters():
    return '#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true)();\n'


HEAP_SIGNATURES = (
    ('build_flags', '() -> u32'), ('enter', '()'), ('leave', '()'), ('pause', '() -> u32'), ('resume', '(depth: u32)'),
    ('reset', '()'), ('install', '()'), ('thread_mutation', '() -> *void'),
    ('end_mutation', '(token: *void)'), ('thread_finalized', '(token: *void)'),
    ('require_quiescent', '()'), ('valid_request', '(size: u64, alignment: u64) -> s32'),
    ('free_any', '(memory: *void)'), ('native_heap', '(heap: *void) -> *SDK.rpmalloc_heap_t'),
    ('layout_size', '(index: u32) -> u64'), ('layout_offset', '(index: u32) -> u64'),
    ('layout_width', '(index: u32) -> u64'), ('acquire', '() -> *void'),
    ('current', '() -> *void'),
    ('release', '(heap: *void)'), ('alloc', '(heap: *void, alignment: u64, size: u64) -> *void'),
    ('calloc', '(heap: *void, alignment: u64, num: u64, size: u64) -> *void'),
    ('realloc', '(heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void'),
    ('free', '(heap: *void, memory: *void)'), ('free_all', '(heap: *void)'),
    ('set_current', '(heap: *void)'), ('contains', '(heap: *void, memory: *void, span: *void) -> s32'),
)
SNAPSHOT_SIGNATURES = (
    ('snapshot_create', '() -> *void'), ('snapshot_release', '(snapshot: *void)'),
    ('snapshot_count', '(snapshot: *void) -> u64'),
    ('snapshot_header', '(snapshot: *void, heap: u64, field: u32) -> u64'),
    ('snapshot_counters', '(snapshot: *void, heap: u64, family: u32) -> *s32'),
    ('snapshot_global', '(snapshot: *void) -> *SDK.rpmalloc_global_statistics_t'),
)


def heap_module():
    code = '// Generated private typed SDK ownership boundary; public records never reach C SDK calls.\n'+parameters()
    code += select_blocks(lambda p: '\n'.join(name+' :: '+signature+' #foreign sdk "'+p['facade_prefix']+name+'";'
                                              for name, signature in HEAP_SIGNATURES+(SNAPSHOT_SIGNATURES if p['statistics'] else ())))
    code += '#scope_file\nSDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS);\n'
    return code+select_blocks(lambda p: 'sdk :: #library,system "'+p['sdk']+'";')


def callbacks_module():
    code = '// Generated private callback ABI; matching native profile owns the SDK instance.\n'+parameters()
    code += 'SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS);\n'
    code += select_blocks(lambda p: 'initialize :: (config: *SDK.rpmalloc_config_t, name: *u8, name_length: u64, huge_name: *u8, huge_name_length: u64) -> s32 #foreign bridge "'+p['callback_prefix']+'initialize";\n'
                          'finalize :: () #foreign bridge "'+p['callback_prefix']+'finalize";')
    return code+'#scope_file\n'+select_blocks(lambda p: 'bridge :: #library,system "'+p['callbacks']+'";')


def c_definitions(profile, functions):
    definitions = ['-D'+key+'='+str(value) for key, value in profile['options'].items()]
    definitions += ['-DBINDING_RPMALLOC_FACADE_PREFIX='+profile['facade_prefix']]
    if profile['symbol_prefix']:
        definitions += ['-D'+f['name']+'='+profile['symbol_prefix']+f['name'] for f in functions]
    definitions += ['-Dbinding_rpmalloc_initialize='+profile['callback_prefix']+'initialize',
                    '-Dbinding_rpmalloc_finalize='+profile['callback_prefix']+'finalize']
    return definitions
