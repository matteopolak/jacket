"""One native domain inventory for generated imports and real C compilation."""
from config import RPMALLOC_SDK_BUILD_OPTIONS

PROFILES = []
for raw in (False, True):
    for statistics in (True, False):
        suffix = ('_raw' if raw else '')+('' if statistics else '_nostats')
        prefix = 'binding_rpmalloc'+suffix+'_'
        PROFILES.append(dict(raw=raw, statistics=statistics,
                             condition=('RAW' if raw else '!RAW')+' && '+('STATISTICS' if statistics else '!STATISTICS'),
                             sdk='binding_rpmalloc_sdk'+suffix,
                             callbacks='binding_rpmalloc_callbacks'+suffix,
                             symbol_prefix=prefix if suffix else '',
                             facade_prefix=prefix+'facade_', callback_prefix=prefix,
                             options=dict(RPMALLOC_SDK_BUILD_OPTIONS, ENABLE_STATISTICS=int(statistics),
                                          BINDING_RPMALLOC_RAW=int(raw))))


def select_blocks(render):
    return '\n'.join('#if '+p['condition']+' {\n'+render(p)+'\n}' for p in PROFILES)+'\n'


def parameters():
    return '#module_parameters (STATISTICS := true, RAW := false)();\n'


HEAP_SIGNATURES = (
    ('enter', '()'), ('leave', '()'), ('pause', '() -> u32'), ('resume', '(depth: u32)'),
    ('reset', '()'), ('install', '()'), ('thread_mutation', '() -> *void'),
    ('end_mutation', '(token: *void)'), ('thread_finalized', '(token: *void)'),
    ('require_quiescent', '()'), ('valid_request', '(size: u64, alignment: u64) -> s32'),
    ('free_any', '(memory: *void)'), ('native_heap', '(heap: *void) -> *SDK.rpmalloc_heap_t'),
    ('layout_size', '(index: u32) -> u64'), ('layout_offset', '(index: u32) -> u64'),
    ('layout_width', '(index: u32) -> u64'), ('acquire', '() -> *void'),
    ('release', '(heap: *void)'), ('alloc', '(heap: *void, alignment: u64, size: u64) -> *void'),
    ('calloc', '(heap: *void, alignment: u64, num: u64, size: u64) -> *void'),
    ('realloc', '(heap: *void, memory: *void, alignment: u64, size: u64, flags: u32) -> *void'),
    ('free', '(heap: *void, memory: *void)'), ('free_all', '(heap: *void)'),
    ('set_current', '(heap: *void)'), ('contains', '(heap: *void, memory: *void, span: *void) -> s32'),
)


def heap_module():
    code = '// Generated private typed SDK ownership boundary; public records never reach C SDK calls.\n'+parameters()
    code += select_blocks(lambda p: '\n'.join(name+' :: '+signature+' #foreign sdk "'+p['facade_prefix']+name+'";'
                                              for name, signature in HEAP_SIGNATURES))
    code += '#scope_file\nSDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW);\n'
    return code+select_blocks(lambda p: 'sdk :: #library,system "'+p['sdk']+'";')


def callbacks_module():
    code = '// Generated private callback ABI; matching native profile owns the SDK instance.\n'+parameters()
    code += 'SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW);\n'
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
