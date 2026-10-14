#!/usr/bin/env python3
"""Inventory every pinned installed SDL3 header, including inactive preprocessor branches.

The lexical discovery pass is independent of the binding's Clang AST pass. It records source
spelling and location, then reconciles each candidate with target AST declarations/exports.
It does not derive ABI types from C text. Unknown core declarations are generation errors.
"""
from __future__ import annotations
import hashlib
import re
from pathlib import Path

GROUPS = (
    ('foundation', 'stdinc endian bits cpuinfo atomic guid'),
    ('lifecycle', 'init error log assert main version platform platform_defines system power properties'),
    ('resources', 'thread mutex timer time iostream asyncio storage filesystem loadso process locale misc'),
    ('interaction', 'events keyboard keycode scancode mouse touch pen gamepad joystick sensor haptic hidapi clipboard dialog messagebox tray'),
    ('presentation', 'video surface pixels blendmode rect render camera metal vulkan'),
    ('gpu', 'gpu'),
    ('audio', 'audio'),
)
GROUP_NAMES = [name for name,_ in GROUPS]
HEADER_GROUP = {f'SDL_{header}.h':name for name,headers in GROUPS for header in headers.split()}


def group(header):
    return HEADER_GROUP.get(header,'foundation')


def ordering(name,kind,header):
    role=1
    if kind=='procedure':
        operation=name.removeprefix('SDL_')
        if operation.startswith(('Create','Acquire','Open','Init','Load','Start','Enter','Run')):role=0
        elif operation.startswith(('Destroy','Release','Close','Quit','Unload','Stop','Exit')):role=5
        elif operation.startswith(('Set','Add','Attach','Bind','Upload','Enable','Disable')):role=1
        elif operation.startswith(('Get','Is','Has','Was','Query','Read','Map')):role=2
        else:role=3
    return (GROUP_NAMES.index(group(header)),
            {'enum':0,'alias':1,'record':2,'callback':3,'procedure':4,'constant':5}[kind],role,name)


def scrub(text):
    # Preserve offsets and strings while removing C comments, including documentation examples.
    return re.sub(r'/\*.*?\*/|//[^\n]*|"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)*\'',
                  lambda m:m[0] if m[0].startswith(('"',"'")) else re.sub(r'[^\n]',' ',m[0]),text,flags=re.S)


def discover(path):
    text=path.read_text(); clean=scrub(text)
    items={}
    conditions={};stack=[]
    for number,line in enumerate(clean.splitlines(),1):
        conditions[number]=list(stack)
        directive=re.match(r'\s*#\s*(if|ifdef|ifndef|elif|else|endif)\b(.*)',line)
        if not directive:continue
        command,expr=directive.groups();expr=expr.strip()
        if command in {'if','ifdef','ifndef'}:stack.append(command+' '+expr)
        elif command=='elif' and stack:stack[-1]='elif '+expr
        elif command=='else' and stack:stack[-1]='else of ('+stack[-1]+')'
        elif command=='endif' and stack:stack.pop()
    def add(kind,name,start,end):
        key=(kind,name)
        spelling=' '.join(clean[start:end].split())
        item={'kind':kind,'name':name,'line':text.count('\n',0,start)+1,'spelling':spelling}
        item['conditions']=conditions.get(item['line'],[])
        # Distinct conditional spellings remain auditable (same C API name, several branches).
        if key not in items:items[key]=item
        elif spelling!=items[key]['spelling']:
            items[key].setdefault('conditional_spellings',[])
            if spelling not in items[key]['conditional_spellings']:items[key]['conditional_spellings'].append(spelling)
    for m in re.finditer(r'^[ \t]*#[ \t]*define[ \t]+(\w+)(\([^\n)]*\))?((?:[^\n\\]|\\(?!\n)|\\\n)*)',clean,re.M):
        add('function_macro' if m[2] else 'object_macro',m[1],m.start(),m.end())
    clean=re.sub(r'^[ \t]*#[^\n]*(?:(?<=\\)\n[^\n]*)*',lambda m:re.sub(r'[^\n]',' ',m[0]),clean,flags=re.M)
    for m in re.finditer(r'\bVK_DEFINE_(?:NON_DISPATCHABLE_)?HANDLE\s*\(\s*(\w+)\s*\)',clean):
        add('typedef',m[1],m.start(),m.end())
    # SDL's public functions consistently use SDLCALL; include SDLMAIN declarations and inline
    # helpers. This scans inactive OS/wiki branches too, independently of preprocessing.
    for m in re.finditer(r'\b(?:extern\s+(?:SDL_DECLSPEC|SDLMAIN_DECLSPEC)\b[^;{}]*?\bSDLCALL\s+|(?:SDL_FORCE_INLINE|SDL_INLINE)\b[^;{}]*?\s+)((?:SDL_|SDLTest_)\w+)\s*\([^;{}]*\)',clean,re.S):
        add('procedure',m[1],m.start(),m.end())
    for m in re.finditer(r'\b(?:GLAPI|GL_APICALL|EGLAPI|KHRONOS_APICALL)\b[^;{}]*?\b((?:gl|egl)\w+)\s*\([^;{}]*\)',clean,re.S):
        add('procedure',m[1],m.start(),m.end())
    for m in re.finditer(r'\bextern\b[^;(){}]*\b(\w+)\s*(?:\[[^]]*\])?\s*;',clean,re.S):
        add('variable',m[1],m.start(),m.end())
    # Typedefs end at a semicolon at brace depth zero. Callback signatures may have nested
    # parameter declarators, but no record body is confused with a separate typedef.
    for m in re.finditer(r'\btypedef\b',clean):
        depth=0;end=m.end()
        while end<len(clean):
            c=clean[end]
            if c=='{':depth+=1
            elif c=='}':depth-=1
            elif c==';' and depth==0:break
            end+=1
        decl=clean[m.start():end]
        callback=re.search(r'\(\s*(?:(?:SDLCALL|APIENTRY|EGLAPIENTRY|GL_APIENTRY)?\s*\*|[A-Z_]*APIENTRYP)\s*(\w+)\s*\)',decl)
        if re.match(r'typedef\s+(?:struct|union|enum)\b',decl) and '{' in decl:callback=None
        final=re.search(r'\b(\w+)\s*(?:\[[^]]*\])?\s*$',decl)
        if callback:add('callback',callback[1],m.start(),end+1)
        elif final:add('typedef',final[1],m.start(),end+1)
        else:raise ValueError(f'unrecognized typedef in {path}:{text.count(chr(10),0,m.start())+1}: {decl[:120]}')
    for m in re.finditer(r'\b(struct|union|enum)\s+(\w+)\s*(?=\{|;)',clean):
        add(m[1],m[2],m.start(),m.end())
    # Materialized enum members: identifiers at the beginning of enum entries. Parenthesis
    # depth excludes commas in SDL_DEFINE_PIXELFORMAT and similar initializer expressions.
    for m in re.finditer(r'\benum\s*(?:\w+\s*)?\{',clean):
        depth=0;start=m.end();i=start
        while i<len(clean):
            c=clean[i]
            if c=='(':depth+=1
            elif c==')':depth-=1
            if c in ',}' and depth==0:
                entry=clean[start:i];name=re.match(r'\s*(\w+)',entry)
                if name:add('enumerator',name[1],start,i)
                start=i+1
                if c=='}':break
            i+=1
    return sorted(items.values(),key=lambda n:(n['kind'],n['name']))


# These C APIs have no runtime symbol to import. They are application callbacks/entrypoints.
APPLICATION = {'SDL_main','SDL_AppInit','SDL_AppIterate','SDL_AppEvent','SDL_AppQuit'}
# Exact non-desktop symbols, from SDL_system/main.h, not a broad name-prefix exemption.
OTHER_OS = set('''SDL_iOSAnimationCallback SDL_SetiOSAnimationCallback SDL_SetiOSEventPump
SDL_GetAndroidJNIEnv SDL_GetAndroidActivity SDL_GetAndroidSDKVersion SDL_IsChromebook SDL_IsDeXMode
SDL_SendAndroidBackButton SDL_GetAndroidInternalStoragePath SDL_GetAndroidExternalStorageState
SDL_GetAndroidExternalStoragePath SDL_GetAndroidCachePath SDL_RequestAndroidPermissionCallback
SDL_RequestAndroidPermission SDL_ShowAndroidToast SDL_SendAndroidMessage SDL_IsTablet SDL_IsTV
SDL_OnApplicationDidChangeStatusBarOrientation SDL_GetGDKTaskQueue SDL_GetGDKDefaultUser
SDL_GDKSuspendComplete SDL_GDKSuspendGPU SDL_GDKResumeGPU XTaskQueueObject XTaskQueueHandle XUser XUserHandle'''.split())


def macro_reason(name,spelling):
    if name in {'SDL_FILE','SDL_ASSERT_FILE','SDL_FUNCTION','SDL_LINE'}:return 'Call-site source metadata; freezing a generator location would be incorrect.'
    if '_renamed_' in spelling or '_removed' in spelling:return 'SDL2 migration poison/old-name compatibility macro; SDL3 canonical names are bound.'
    if name.startswith('SDL_assert') or name in {'SDL_enabled_assert','SDL_disabled_assert','SDL_AssertBreakpoint','SDL_TriggerBreakpoint'}:
        return 'Call-site assertion/control-flow or debugger trap macro; use the bound SDL assertion handler/report API as appropriate.'
    if name in {'SDL_CompilerBarrier','SDL_CPUPauseInstruction','SDL_MemoryBarrierAcquire','SDL_MemoryBarrierRelease'}:
        return 'Compiler/instruction macro requiring caller-context semantics; use the exported SDL_MemoryBarrierAcquireFunction/ReleaseFunction for thread fences.'
    if name in {'SDL_stack_alloc','SDL_stack_free'}:return 'Caller-stack allocation and paired lifetime macro; use bound SDL_malloc/SDL_free for heap storage.'
    if name in {'SDL_arraysize','SDL_zero','SDL_zerop','SDL_zeroa','SDL_copyp','SDL_min','SDL_max','SDL_clamp','SDL_static_cast','SDL_const_cast','SDL_reinterpret_cast','SDL_UINT64_C','SDL_SINT64_C','SDL_iconv_wchar_utf8'}:
        return 'Type/token-dependent C convenience; corresponding Jai operators/array count, or SDL_memset/memcpy/iconv_string, preserve the operation without a false fixed C signature.'
    annotations=set("""SDL_ACQUIRE SDL_ACQUIRED_AFTER SDL_ACQUIRED_BEFORE SDL_ACQUIRE_SHARED SDL_ALIGNED
SDL_ALLOC_SIZE SDL_ALLOC_SIZE2 SDL_ASSERT_CAPABILITY SDL_ASSERT_SHARED_CAPABILITY SDL_CAPABILITY
SDL_COMPILE_TIME_ASSERT SDL_ELF_NOTE_DLOPEN SDL_EXCLUDES SDL_GUARDED_BY SDL_INOUT_Z_CAP SDL_IN_BYTECAP
SDL_OUT_BYTECAP SDL_OUT_CAP SDL_OUT_Z_BYTECAP SDL_OUT_Z_CAP SDL_PRINTF_VARARG_FUNC SDL_PRINTF_VARARG_FUNCV
SDL_PT_GUARDED_BY SDL_RELEASE SDL_RELEASE_GENERIC SDL_RELEASE_SHARED SDL_REQUIRES SDL_REQUIRES_SHARED
SDL_RETURN_CAPABILITY SDL_SCANF_VARARG_FUNC SDL_SCANF_VARARG_FUNCV SDL_STRINGIFY_ARG
SDL_THREAD_ANNOTATION_ATTRIBUTE__ SDL_TRY_ACQUIRE SDL_TRY_ACQUIRE_SHARED SDL_WPRINTF_VARARG_FUNC
SDL_WPRINTF_VARARG_FUNCV SDL_ANALYZER_NORETURN SDL_ASSERT_LEVEL SDL_BeginThreadFunction SDL_EndThreadFunction
SDL_DECLSPEC SDL_DEPRECATED SDL_FALLTHROUGH SDL_FORCE_INLINE SDL_FUNCTION SDL_INCLUDE_STDBOOL_H SDL_INLINE
SDL_MALLOC SDL_NODISCARD SDL_NORETURN SDL_NO_THREAD_SAFETY_ANALYSIS SDL_RESTRICT SDL_SCOPED_CAPABILITY
SDL_UNUSED SDL_DLN0TE SDL_DLNOTE_UNIQUE_NAME SDL_PRINTF_FORMAT_STRING SDL_SCANF_FORMAT_STRING""".split())
    if name in annotations or name.endswith(('_h_','_h')) or name.startswith(('SDL_DISABLE_','SDL_PLATFORM_','SDL_HAS_','SDL_MAIN_','SDL_DLNOTE_','SDL_ELF_NOTE_')):
        return 'C compiler annotation/build configuration or header guard; not a callable runtime API.'
    raise ValueError(f'Unclassified macro: {name}: {spelling}')


def reconcile(src,reports,equivalents):
    headers={};unexplained=[]
    for path in sorted((src/'include/SDL3').glob('*.h')):
        name=path.name;items=discover(path)
        if name.startswith('SDL_test'):category='auxiliary_test_library'
        elif name in {'SDL_egl.h','SDL_intrin.h'} or name.startswith(('SDL_opengl','SDL_opengles')):category='external_graphics_or_intrinsics'
        elif name in {'SDL_begin_code.h','SDL_close_code.h','SDL_main_impl.h','SDL_copying.h','SDL_oldnames.h','SDL_revision.h'}:category='compiler_configuration_or_entry_glue'
        else:category='core'
        for item in items:
            symbol=item['name'];kind=item['kind'];targets={}
            for target,r in reports.items():
                emitted=equivalents[target]
                # Enum members remain inside their canonical enum declaration.
                if symbol in emitted:
                    targets[target]={'status':'generated','equivalent':emitted[symbol]}
                elif symbol in r['excluded_functions']:
                    targets[target]={'status':'excluded','reason':r['excluded_functions'][symbol]}
                elif symbol in r['wrapped_function_macros']:
                    targets[target]={'status':'wrapped','equivalent':f'SDL3/{symbol} (typed native macro wrapper)'}
                elif kind.endswith('macro') and symbol in r['excluded_object_macros']:
                    targets[target]={'status':'nonruntime','reason':macro_reason(symbol,item['spelling'])}
                elif kind=='function_macro' and symbol in r['unsupported_function_macros']:
                    targets[target]={'status':'excluded','reason':macro_reason(symbol,item['spelling'])}
            if targets:
                item['targets']=targets
                item['remaining_targets_reason']='Preprocessor branch inactive on these release targets; declaration equivalences are listed for every active extracted target.'
            elif symbol=='SDL_joystick_lock':item.update(status='nonruntime',reason='Header explicitly declares this only for Clang thread-safety analysis and says it is not an exported SDL symbol.')
            elif symbol=='SDL_KernelMemoryBarrierFunc':item.update(status='excluded',reason='Linux ARMv5/ARMv6 32-bit kernel barrier helper; absent on all six 64-bit release targets.')
            elif symbol in {'int8_t','int16_t','int32_t','int64_t','uint8_t','uint16_t','uint32_t','uint64_t','intptr_t','uintptr_t'}:item.update(status='nonruntime',reason='ISO C integer typedef fallback for pre-VS2010 compilers; SDL Uint/Sint typedefs are generated for modern release targets.')
            elif symbol in APPLICATION:item.update(status='application_supplied',reason='Application defines this entrypoint/callback; importing it as an SDL foreign export would be incorrect.')
            elif symbol in OTHER_OS:item.update(status='excluded',reason='Conditional Android/iOS/GDK API is unavailable on all six desktop release targets.')
            elif category=='auxiliary_test_library':item.update(status='excluded',reason='SDL3_test auxiliary test framework, separate from the packaged SDL3 core runtime; not SDL_image/mixer/ttf/net.')
            elif category=='external_graphics_or_intrinsics':item.update(status='excluded',reason='External EGL/OpenGL/OpenGL ES/Khronos or CPU-intrinsic definition, not an SDL3 core library export. SDL window/GL/EGL/Metal/Vulkan integration APIs are bound.')
            elif kind.endswith('macro') and category=='core':
                inactive={
                    'SDL_atomic.h':{'SDL_MEMORY_BARRIER_USES_FUNCTION'},
                    'SDL_endian.h':{'HAS_BROKEN_BSWAP','HAS_BUILTIN_BSWAP16','HAS_BUILTIN_BSWAP32','HAS_BUILTIN_BSWAP64','__PRFCHWINTRIN_H'},
                    'SDL_main.h':{'SDL_PS2_SKIP_IOP_RESET','SDLMAIN_DECLSPEC','SDL_MAIN_AVAILABLE','SDL_MAIN_EXPORTED','SDL_MAIN_NEEDED','SDL_MAIN_USE_CALLBACKS','SDL_PLATFORM_PRIVATE_MAIN','main'},
                    'SDL_stdinc.h':{'SDL_NOLONGLONG','__bool_true_false_are_defined','alloca','bool','false','true'},
                    'SDL_vulkan.h':{'VK_DEFINE_HANDLE','VK_DEFINE_NON_DISPATCHABLE_HANDLE','NO_SDL_VULKAN_TYPEDEFS'},
                }
                if name=='SDL_platform_defines.h':item.update(status='nonruntime',reason='Platform/compiler detection and SDK availability preprocessor vocabulary; Jai OS/CPU select supported release profiles.')
                elif symbol in {'SDL_ANDROID_EXTERNAL_STORAGE_READ','SDL_ANDROID_EXTERNAL_STORAGE_WRITE'}:item.update(status='excluded',reason='Android-only external-storage flags; absent on all six desktop release targets.')
                elif symbol in inactive.get(name,set()):item.update(status='nonruntime',reason='Conditional C compiler/entrypoint/type-generation configuration; standard types/operators or generated Vulkan handles carry the supported operation.')
                else:raise ValueError(f'Unexplained inactive core macro: {name}:{symbol}')
            elif kind.endswith('macro'):item.update(status='nonruntime',reason='Header-specific compiler/entrypoint/foreign-API preprocessor vocabulary; see the header category.')
            elif category=='compiler_configuration_or_entry_glue':item.update(status='nonruntime',reason='C compiler/entrypoint glue or generated build metadata; SDL runtime entry integration uses SDL_RunApp/SDL_SetMainReady/SDL_EnterAppMainCallbacks.')
            else:
                item.update(status='unexplained')
                unexplained.append(f'{name}:{item["line"]} {kind} {symbol}')
        for item in items:
            if 'targets' in item:
                grouped={}
                for target,entry in item.pop('targets').items():
                    entry=dict(entry)
                    if 'equivalent' in entry:entry['equivalent']=entry['equivalent'].replace(target,'{profile}')
                    key=tuple(sorted(entry.items()))
                    grouped.setdefault(key,{**entry,'profiles':[]})['profiles'].append(target)
                item['target_groups']=list(grouped.values())
                if sum(len(g['profiles']) for g in grouped.values())==len(reports):item.pop('remaining_targets_reason',None)
        for item in items:item['_conditions']=tuple(item.pop('conditions'))
        condition_values=sorted({item['_conditions'] for item in items})
        condition_ids={value:f'c{i}' for i,value in enumerate(condition_values)}
        # Re-read conditions from source items is unnecessary: keep their stable tuple while packing.
        for item in items:item['conditions_ref']=condition_ids[tuple(item.pop('_conditions'))]
        headers[name]={'condition_groups':{condition_ids[value]:list(value) for value in condition_values},'sha256' :hashlib.sha256(path.read_bytes()).hexdigest(),'category':category,'declarations':items}
    if unexplained:raise ValueError('Unexplained installed-header declarations:\n'+'\n'.join(unexplained))
    gpu=headers['SDL_gpu.h']['declarations']
    if not gpu or any(i.get('status') in {'excluded','unexplained'} for i in gpu if i['kind'] in {'procedure','typedef','callback','struct','union','enum','enumerator'} and i['name'] not in OTHER_OS):
        raise ValueError('GPU core declaration coverage is incomplete')
    reasons={}
    def intern_reasons(value):
        if isinstance(value,dict):
            if 'reason' in value:
                reason=value.pop('reason');key=hashlib.sha256(reason.encode()).hexdigest()[:16]
                reasons[key]=reason;value['reason_ref']=key
            for child in value.values():intern_reasons(child)
        elif isinstance(value,list):
            for child in value:intern_reasons(child)
    intern_reasons(headers)
    return {'format':1,'reasons':reasons,'source_version':'3.4.18','discovery':'Comment-aware lexical discovery over every installed header, reconciled with six Clang ASTs and actual generated declaration names. ABI types come only from Clang.',
            'organization':GROUP_NAMES,'unexplained':[],'headers':headers}
