"""Independently authored convenience bodies for documented signatures.

This module deliberately does not infer behavior from legacy tests. Native
macro arithmetic follows the corresponding public C APIs.
"""
import re
from bitfields import body as bitfield_body


def simple_body(module, name, condition=''):
    if module == 'd3d12':
        bodies = {
            'D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING':'return cast(D3D12_SHADER_COMPONENT_MAPPING) Binding_Math.encode_components(Src0,Src1,Src2,Src3);',
            'D3D12_DECODE_SHADER_4_COMPONENT_MAPPING':'return cast(D3D12_SHADER_COMPONENT_MAPPING) Binding_Math.decode_component(ComponentToExtract,Mapping);',
            'D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING':'return D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(0,1,2,3);',
            'D3D12_ENCODE_BASIC_FILTER':'return cast(D3D12_FILTER) Binding_Math.encode_filter(cast(s32) min,cast(s32) mag,cast(s32) mip,cast(s32) reduction);',
            'D3D12_ENCODE_ANISOTROPIC_FILTER':'return cast(D3D12_FILTER) Binding_Math.encode_anisotropic(cast(s32) reduction);',
            'D3D12_DECODE_MIN_FILTER':'return cast(D3D12_FILTER_TYPE) Binding_Math.decode_minimum(cast(s32) D3D12Filter);',
            'D3D12_DECODE_MAG_FILTER':'return cast(D3D12_FILTER_TYPE) Binding_Math.decode_magnification(cast(s32) D3D12Filter);',
            'D3D12_DECODE_MIP_FILTER':'return cast(D3D12_FILTER_TYPE) Binding_Math.decode_mip(cast(s32) D3D12Filter);',
            'D3D12_DECODE_FILTER_REDUCTION':'return cast(D3D12_FILTER_REDUCTION_TYPE) Binding_Math.decode_reduction(cast(s32) D3D12Filter);',
            'D3D12_DECODE_IS_COMPARISON_FILTER':'return Binding_Math.is_comparison(cast(s32) D3D12Filter);',
            'D3D12_DECODE_IS_ANISOTROPIC_FILTER':'return Binding_Math.is_anisotropic(cast(s32) D3D12Filter);',
        }
        if name in bodies:
            return bodies[name]
    if module == 'Objective_C':
        bodies = {
            'NSMakePoint': 'return NSPoint.{x,y};',
            'NSMakeSize': 'return NSSize.{w,h};',
            'NSMakeRect': 'return NSRect.{NSPoint.{x,y},NSSize.{w,h}};',
            'init_objective_c_foundation': 'if !binding_objc_class("NSString") binding_abort();',
        }
        if name in bodies:
            return bodies[name]
    if module == 'Metal':
        bodies = {
            'init_metal': 'init_objective_c(); if !binding_objc_class("MTLTextureDescriptor") binding_abort();',
            'MTLOriginMake': 'return MTLOrigin.{x,y,z};',
            'MTLSizeMake': 'return MTLSize.{width,height,depth};',
            'MTLRegionMake1D': 'return MTLRegion.{MTLOrigin.{x,0,0},MTLSize.{width,1,1}};',
            'MTLRegionMake2D': 'return MTLRegion.{MTLOrigin.{x,y,0},MTLSize.{width,height,1}};',
            'MTLRegionMake3D': 'return MTLRegion.{MTLOrigin.{x,y,z},MTLSize.{width,height,depth}};',
            'MTLSamplePositionMake': 'return MTLSamplePosition.{x,y};',
            'MTLClearColorMake': 'return MTLClearColor.{red,green,blue,alpha};',
        }
        return bodies.get(name)
    if module == 'xxHash':
        bodies = {
            'XXH_versionNumber': 'return binding_XXH_versionNumber();',
            'native_XXH32': 'return binding_XXH32(input, cast(u64) len, seed);',
            'native_XXH64': 'return binding_XXH64(input, cast(u64) len, cast(u64) seed);',
            'XXH32_reset': 'if binding_XXH32_reset(state, seed) != 0 binding_abort();',
            'XXH64_reset': 'if binding_XXH64_reset(state, seed) != 0 binding_abort();',
            'XXH32_update': 'if len == 0 return; if binding_XXH32_update(state, input, cast(u64) len) != 0 binding_abort();',
            'XXH64_update': 'if len == 0 return; if binding_XXH64_update(state, input, cast(u64) len) != 0 binding_abort();',
            'XXH32_digest': 'return binding_XXH32_digest(state);',
            'XXH64_digest': 'return binding_XXH64_digest(state);',
        }
        return bodies.get(name)
    if module == 'freetype':
        flags = {'FT_HAS_HORIZONTAL': 'HORIZONTAL', 'FT_HAS_VERTICAL': 'VERTICAL',
                 'FT_HAS_KERNING': 'KERNING', 'FT_IS_SCALABLE': 'SCALABLE',
                 'FT_IS_SFNT': 'SFNT', 'FT_IS_FIXED_WIDTH': 'FIXED_WIDTH',
                 'FT_HAS_FIXED_SIZES': 'FIXED_SIZES', 'FT_HAS_GLYPH_NAMES': 'GLYPH_NAMES',
                 'FT_HAS_MULTIPLE_MASTERS': 'MULTIPLE_MASTERS', 'FT_IS_VARIATION': 'VARIATION',
                 'FT_IS_CID_KEYED': 'CID_KEYED', 'FT_IS_TRICKY': 'TRICKY',
                 'FT_HAS_COLOR': 'COLOR', 'FT_HAS_SVG': 'SVG', 'FT_HAS_SBIX': 'SBIX',
                 'FT_HAS_SBIX_OVERLAY': 'SBIX_OVERLAY'}
        if name in flags:
            return 'return (face.face_flags & FT_FACE_FLAG_' + flags[name] + ') != 0;'
        if name == 'FT_IS_NAMED_INSTANCE':
            return 'return (face.face_index & 0x7fff0000) != 0;'
    if module == 'Vulkan':
        expressions = {
            'VK_MAKE_VERSION': '(major << 22) | (minor << 12) | patch',
            'VK_VERSION_MAJOR': 'version >> 22', 'VK_VERSION_MINOR': '(version >> 12) & 0x3ff',
            'VK_VERSION_PATCH': 'version & 0xfff',
            'VK_MAKE_API_VERSION': '(variant << 29) | (major << 22) | (minor << 12) | patch',
            'VK_API_VERSION_VARIANT': 'version >> 29',
            'VK_API_VERSION_MAJOR': '(version >> 22) & 0x7f',
            'VK_API_VERSION_MINOR': '(version >> 12) & 0x3ff',
            'VK_API_VERSION_PATCH': 'version & 0xfff',
            'vk_unmake_version': 'VK_Version.{VK_VERSION_MAJOR(vk_version), VK_VERSION_MINOR(vk_version), VK_VERSION_PATCH(vk_version)}',
        }
        if name in expressions:
            return 'return ' + expressions[name] + ';'
    if module == 'Windows':
        bodies = {
            'SUCCEEDED': 'return hr >= 0;', 'FAILED': 'return hr < 0;',
            'TOUCH_COORD_TO_PIXEL': 'return cast(float) c / 100.0;',
            'HasOverlappedIoCompleted': 'return overlapped.Internal != 0x103;',
            'MAKELANGID': 'return (usSubLanguage << 10) | usPrimaryLanguage;',
            'operator ==': 'if a.Data1 != b.Data1 || a.Data2 != b.Data2 || a.Data3 != b.Data3 return false; for i: 0..7 if a.Data4[i] != b.Data4[i] return false; return true;',
            'safe_release': 'if obj != null obj.vtable.Release(obj);',
        }
        if name in bodies:
            return bodies[name]
        if name == 'safe_release_and_reset':
            return 'if <<b != null { safe_release(cast(*IUnknown) <<b); <<b = null; }'
    if module == 'POSIX':
        bodies = {'errno': 'return cast(OS_Error_Code) <<binding_errno_address();',
                  'set_errno': '<<binding_errno_address() = cast(s32) value;',
                  'IFTODT': 'return (mode & S_IFMT) >> 12;',
                  'DTTOIF': 'return dirtype << 12;'}
        if name in bodies:
            return bodies[name]
        if name.startswith('S_IS'):
            flag = {'S_ISDIR': 'S_IFDIR', 'S_ISCHR': 'S_IFCHR', 'S_ISBLK': 'S_IFBLK',
                    'S_ISREG': 'S_IFREG', 'S_ISFIFO': 'S_IFIFO', 'S_ISLNK': 'S_IFLNK',
                    'S_ISSOCK': 'S_IFSOCK'}.get(name)
            if flag:
                return 'return (mode & S_IFMT) == ' + flag + ';'
        waits = {'WEXITSTATUS': '(status >> 8) & 255', 'WTERMSIG': 'status & 127',
                 'WSTOPSIG': '(status >> 8) & 255', 'WIFEXITED': '(status & 127) == 0',
                 'WIFSIGNALED': '(status & 127) != 0 && (status & 127) != 127',
                 'WIFSTOPPED': '(status & 255) == 127', 'WIFCONTINUED': 'status == 0xffff',
                 'WCOREDUMP': 'status & 128'}
        if name in waits:
            if name == 'WIFCONTINUED' and '.MACOS' in condition:
                return 'return (status & 127) == 127 && (status >> 8) == 19;'
            if name == 'WIFSTOPPED' and '.MACOS' in condition:
                return 'return (status & 127) == 127 && (status >> 8) != 19;'
            if name == 'WSTOPSIG' and '.MACOS' in condition:
                return 'return status >> 8;'
            return 'return ' + waits[name] + ';'
        if name in ('stat', 'lstat'):
            return 'return syscall(SYS_' + name + ', path, buf);'
        if name == 'fstat':
            return 'return syscall(SYS_fstat, fd, buf);'
    if module == 'Linux':
        if name == 'io_uring_setup':
            return 'return cast(s32) syscall(425, entries, p);'
        if name == 'io_uring_register':
            return 'return cast(s32) syscall(427, fd, opcode, arg, nr_args);'
    if module == 'SDL':
        bodies = {
            'SDL_VERSION': 'x.major = SDL_MAJOR_VERSION; x.minor = SDL_MINOR_VERSION; x.patch = SDL_PATCHLEVEL;',
            'SDL_VERSIONNUM': 'return X*1000 + Y*100 + Z;',
            'SDL_WINDOWPOS_UNDEFINED_DISPLAY': 'return SDL_WINDOWPOS_UNDEFINED_MASK | x;',
            'SDL_WINDOWPOS_CENTERED_DISPLAY': 'return SDL_WINDOWPOS_CENTERED_MASK | x;',
            'SDL_LoadBMP': 'return SDL_LoadBMP_RW(SDL_RWFromFile(path, "rb"), 1);',
            'SDL_SCANCODE_TO_KEYCODE': 'return cast(s32) scancode | (1 << 30);',
            'SDL_BUTTON': 'return 1 << (x-1);',
            'SDL_PointInRect': 'return cast(SDL_bool) (p.x >= r.x && p.x < r.x+r.w && p.y >= r.y && p.y < r.y+r.h);',
            'SDL_RectEmpty': 'return cast(SDL_bool) (!r || r.w <= 0 || r.h <= 0);',
            'SDL_RectEquals': 'return cast(SDL_bool) (a != null && b != null && a.x == b.x && a.y == b.y && a.w == b.w && a.h == b.h);',
            'SDL_AUDIO_BITSIZE': 'return x & SDL_AUDIO_MASK_BITSIZE;',
            'SDL_AUDIO_ISFLOAT': 'return (x & SDL_AUDIO_MASK_DATATYPE) != 0;',
            'SDL_AUDIO_ISBIGENDIAN': 'return (x & SDL_AUDIO_MASK_ENDIAN) != 0;',
            'SDL_AUDIO_ISSIGNED': 'return (x & SDL_AUDIO_MASK_SIGNED) != 0;',
            'SDL_AUDIO_ISINT': 'return !SDL_AUDIO_ISFLOAT(x);',
            'SDL_AUDIO_ISLITTLEENDIAN': 'return !SDL_AUDIO_ISBIGENDIAN(x);',
            'SDL_AUDIO_ISUNSIGNED': 'return !SDL_AUDIO_ISSIGNED(x);',
        }
        return bodies.get(name)
    if module == 'macOS' and name == 'VM_MAKE_TAG':
        return 'return val << 24;'
    if module == 'iOS' and name == 'get_ios_target_triple':
        return 'if cpu == .ARM64 return "arm64-apple-ios"; return "x86_64-apple-ios";'
    if module == 'X11':
        bodies = {
            'DefaultScreen': 'return XDefaultScreen(display);',
            'ScreenOfDisplay': 'return XScreenOfDisplay(display, scr);',
            'DefaultScreenOfDisplay': 'return XDefaultScreenOfDisplay(display);',
            'DefaultRootWindow': 'return XDefaultRootWindow(display);',
            'DefaultGC': 'return XDefaultGC(display, scr);',
            'x_rgb_to_color': 'color: XColor; color.red = cast(u16) _r * 257; color.green = cast(u16) _g * 257; color.blue = cast(u16) _b * 257; color.flags = 7; return color;',
            'x_textwidth': 'return XTextWidth(font, msg.data, cast(s32) msg.count);',
        }
        return bodies.get(name)
    if module == 'telemetry3' and condition == '!ENABLED || OS != .WINDOWS':
        return 'return false;' if name == 'telemetry_enabled' else ''
    return None


def forwarding_body(module, declaration, declarations, parts, parse):
    bits = bitfield_body(parts)
    if bits is not None:
        return bits
    name, params, args, tail = parts
    prefix = 'return ' if '->' in tail and not re.search(r'->\s*(?:void|\(\))', tail) else ''
    if module == 'ImGui' and any(p.split(':',1)[0].strip() == 'fmt' and ': string' in p for p in params) and '__args' in args:
        for native in declarations:
            if native.name != name+'_CFormat' or native.condition != declaration.condition:
                continue
            target = parse(native.code)
            if not target or len(target[1]) != len(params):
                continue
            fixed, compatible = [], True
            for public, raw, argument in zip(params,target[1],args):
                public_type = public.split(':',1)[1].split('=',1)[0].strip()
                raw_type = raw.split(':',1)[1].split('=',1)[0].strip()
                if argument == '__args':
                    continue
                if argument == 'fmt':
                    fixed.append('.{kind=6,value=cast(u64) cformat}')
                elif raw_type.startswith('*') and public_type == raw_type[1:]:
                    fixed.append('.{kind=6,value=cast(u64) *'+argument+'}')
                elif raw_type == public_type and raw_type.startswith('*'):
                    fixed.append('.{kind=6,value=cast(u64) '+argument+'}')
                elif raw_type == public_type and public_type == 'TreeNodeFlags':
                    fixed.append('.{kind=1,value=cast(u64) '+argument+'}')
                else:
                    compatible = False
                    break
            if compatible:
                setup = 'cformat := Native.terminated_copy(fmt); defer Native.release(cformat); '
                setup += 'fixed: ['+str(len(fixed))+']Varargs.Argument = .['+', '.join(fixed)+']; '
                # The overloaded string/pointer ID variants need a typed
                # procedure address to select their exact native signature.
                signature = native.code.split('::',1)[1].split('#foreign',1)[0].strip()
                setup += 'Native_Format :: #type '+signature+' #c_call; function: Native_Format = '+native.name+'; '
                invoke = 'invoke_bool' if prefix else 'invoke_void'
                return setup+prefix+'Varargs.'+invoke+'(cast(*void) function,fixed,__args);'
    if module == 'Vulkan' and re.search(r'->\s*\[\]', tail):
        element = re.search(r'->\s*\[\]\s*(\w+)', tail)[1]
        has_result = 'VkResult' in tail
        native_declarations = list(declarations)
        for pfn in declarations:
            if pfn.name == 'PFN_' + name and pfn.condition == declaration.condition:
                from spec import Declaration
                signature = pfn.code.split('#type', 1)[1].replace('#c_call', '#foreign libvulkan')
                native_declarations.append(Declaration(name + ' :: ' + signature, pfn.condition))
        for raw in native_declarations:
            if raw.name != name or '#foreign' not in raw.code:
                continue
            if raw.condition != declaration.condition:
                continue
            target = parse(raw.code)
            if not target or len(target[1]) != len(params)+2:
                continue
            native_name = name if raw in declarations else 'binding_' + name
            first_call = native_name + '(' + ', '.join(args + ['*count', 'null']) + ')'
            second_call = native_name + '(' + ', '.join(args + ['*count', 'values.data']) + ')'
            body = 'count: u32; '
            if has_result:
                body += 'status := ' + first_call + '; if status != .SUCCESS || count == 0 return .[], status; '
            else:
                body += first_call + '; if count == 0 return .[]; '
            body += 'values, base := Binding_Arrays.allocate_values(cast(s64) count, ' + element + '); '
            body += ('if !base return .[], .ERROR_OUT_OF_HOST_MEMORY; ' if has_result else 'if !base binding_abort(); ')
            if has_result:
                body += 'status = ' + second_call + '; if cast,trunc(s32) status < 0 { Binding_Arrays.release_values(base); return .[], status; } '
            else:
                body += second_call + '; '
            body += 'values.count = count; return values' + (', status' if has_result else '') + ';'
            return body
    if module == 'Linux' and name == 'io_uring_enter':
        signal = 'null' if len(args) == 4 else args[-1]
        size = 'size_of(io_uring_getevents_arg)' if 'io_uring_getevents_arg' in params[-1] else '8'
        return 'return cast(s32) syscall(426, fd, to_submit, min_complete, flags, ' + signal + ', ' + size + ');'
    if module == 'Curl' and name.startswith('curl_m'):
        fixed = []
        for arg in args:
            if arg == '__args':
                continue
            value = 'cformat' if arg == 'format' else arg
            kind = 4 if arg == 'maxlength' else 6
            converted = value if arg == 'maxlength' else 'cast(u64) '+value
            fixed.append('.{kind='+str(kind)+', value='+converted+'}')
        output = 'cformat := Native.terminated_copy(format); defer Native.release(cformat); '
        output += 'fixed: ['+str(len(fixed))+']Varargs.Argument = .['+', '.join(fixed)+']; '
        invoke = 'invoke_pointer' if name=='curl_maprintf' else 'invoke_int'
        return output + 'return Varargs.'+invoke+'(cast(*void) '+name+'_CFormat, fixed, __args);'
    if module == 'Objective_C':
        if name == 'init_objective_c':
            return 'if !binding_objc_class("NSObject") binding_abort();'
        if name == 'init_objective_c_selector_struct':
            return '''for member: type_info(T).members {
                if (member.flags & .CONSTANT) != 0 continue;
                if member.type != type_info(Selector) binding_abort();
                text := Native.terminated_copy(member.name);
                for i: 0..member.name.count-1 if text[i] == #char "_" text[i] = #char ":";
                slot := cast(*Selector) (cast(*u8) selectors + member.offset_in_bytes);
                <<slot = binding_objc_selector(text);
                Native.release(text);
            }'''
        if name == 'selector':
            return 'return sel_registerName(sel_str);'
        if name == 'objc_new':
            return 'return objc_init(objc_alloc(type));'
        if name == 'class' and params[0].startswith('$type'):
            return 'name := Native.terminated_copy(type_info(type).name); defer Native.release(name); return cast(Class) binding_objc_class(name);'
        if name == 'class':
            return 'return object_getClass(self);'
        if name == 'superclass':
            return 'return class_getSuperclass(object_getClass(self));'
        if name == 'objc_finalize_class':
            return 'objc_registerClassPair(my_class);'
        if name == 'to_string':
            return 'text := NSString.UTF8String(str); if !text return ""; return .{data=text, count=cast(s64) binding_strlen(text)};'
        messages = {'objc_alloc': 'alloc', 'objc_init': 'init', 'objc_copy': 'copy',
                    'retain': 'retain', 'autorelease': 'autorelease', 'release': 'release',
                    'isEqual': 'isEqual:'}
        if name in messages:
            if name == 'objc_alloc':
                receiver = 'class' if not params[0].startswith('$') else 'binding_objc_class(binding_name)'
                call_args = []
            else:
                receiver, call_args = 'self', args[1:]
            result_type = re.search(r'->\s*([^#]+)', tail)[1].strip()
            c_result = 'void' if name == 'release' else 'BOOL' if name == 'isEqual' else result_type
            signature = 'Send :: #type (receiver: *void, op: *void' + ''.join(', a'+str(i)+': type_of('+arg+')' for i,arg in enumerate(call_args)) + ') -> ' + c_result + ' #c_call; '
            if name == 'objc_alloc' and params[0].startswith('$'):
                signature += 'binding_name := Native.terminated_copy(type_info(type).name); defer Native.release(binding_name); '
            signature += 'send := cast(Send) binding_objc_send; '
            call = 'send(' + ', '.join(['cast(*void) ' + receiver, 'binding_objc_selector("'+messages[name]+'")']+call_args) + ')'
            return signature + (call+'; return null;' if name=='release' else 'return '+call+' != NO;' if name=='isEqual' else 'return '+call+';')
        if name == 'objc_msgSend_typed':
            return 'Send :: #type (target: *void, op: Selector) -> *void #c_call; send := cast(Send) binding_objc_send; return send(target, selector);'
    # A value overload forwarding to its documented pointer overload. Comparing
    # every parameter prevents accidentally selecting a different native overload.
    for native in declarations:
        if native.name != name or '#foreign' not in native.code:
            continue
        if native.condition and declaration.condition and native.condition != declaration.condition:
            continue
        target = parse(native.code)
        if not target:
            continue
        call_args, changed, compatible, setup = [], False, True, []
        native_index = 0
        for arg, public_param in zip(args, params):
            def ty(p):
                return p.split(':', 1)[1].split('=', 1)[0].strip() if ':' in p else ''
            if native_index >= len(target[1]):
                compatible = False
                break
            native_param = target[1][native_index]
            public_type, native_type = ty(public_param), ty(native_param)
            next_param = target[1][native_index+1] if native_index+1 < len(target[1]) else ''
            if public_type == 'string' and native_type == '*u8' and ty(next_param) == '*u8' and re.search(r'\b\w*(?:end|End)\s*:',next_param):
                call_args.extend([arg+'.data',arg+'.data + '+arg+'.count'])
                changed = True
                native_index += 1
            elif native_type == public_type:
                call_args.append(arg)
            elif native_type == '*' + public_type:
                call_args.append('*' + arg)
                changed = True
            elif module == 'Objective_C' and public_type == 'string' and native_type == '*u8':
                setup.append('binding_'+arg+' := Native.terminated_copy('+arg+'); defer Native.release(binding_'+arg+'); ')
                call_args.append('binding_'+arg)
                changed = True
            else:
                compatible = False
                break
            native_index += 1
        if changed and compatible and native_index == len(target[1]):
            return ''.join(setup) + prefix + name + '(' + ', '.join(call_args) + ');'
    return None
