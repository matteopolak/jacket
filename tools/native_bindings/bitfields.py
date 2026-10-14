"""Native bitfield accessors, with shifts from pinned public C SDK headers."""
import re


def body(parts):
    name, params, args, tail = parts
    if not name.startswith(('get_', 'set_')) or not params:
        return None
    match = re.match(r'fields:\s*\*?(\w+)', params[0])
    if not match:
        return None
    record = match[1]
    member = name[4:]
    data_type = 'u32'
    storage = 'fields.__bitfield'
    mapping = {}
    prefix, suffix = '', ''
    if record == 'D3D11_VIDEO_PROCESSOR_COLOR_SPACE':
        mapping = {'Usage':(0,1), 'RGB_Range':(1,1), 'YCbCr_Matrix':(2,1),
                   'YCbCr_xvYCC':(3,1), 'Nominal_Range':(4,2), 'Reserved':(6,26)}
    elif record == '__MIDL___MIDL_itf_d3d11_0000_0034_0001':
        mapping = {'ProtectionEnabled':(0,1), 'OverlayOrFullscreenRequired':(1,1), 'Reserved':(2,30)}
    elif record == 'wait_t':
        mapping = {'w_T_w_Termsig':(0,7),'w_T_w_Coredump':(7,1),'w_T_w_Retcode':(8,8),
                   'w_T_w_Filler':(16,16),'w_S_w_Stopval':(0,8),'w_S_w_Stopsig':(8,8),'w_S_w_Filler':(16,16)}
        storage = 'fields.' + ('w_T' if member.startswith('w_T') else 'w_S') + '.__bitfield'
    elif record in ('__darwin_fp_control', '__darwin_fp_status'):
        data_type = 'u16'
        mapping = {'__invalid':(0,1),'__denorm':(1,1),'__zdiv':(2,1),'__ovrfl':(3,1),
                   '__undfl':(4,1),'__precis':(5,1)}
        mapping.update({'__pc':(8,2),'__rc':(10,2)} if record.endswith('control') else
                       {'__stkflt':(6,1),'__errsumm':(7,1),'__c0':(8,1),'__c1':(9,1),
                        '__c2':(10,1),'__tos':(11,3),'__c3':(14,1),'__busy':(15,1)})
    elif record in ('mach_port_qos', 'mach_port_qos_t'):
        mapping = {'name':(0,1),'prealloc':(1,1),'pad1':(2,30)}
    elif record == 'ImFontGlyph':
        mapping = {'Colored':(0,1),'Visible':(1,1),'Codepoint':(2,30)}
    elif record == 'D3D12_RAYTRACING_INSTANCE_DESC':
        mapping = {'InstanceID':(0,24),'InstanceMask':(24,8),
                   'InstanceContributionToHitGroupIndex':(0,24),'Flags':(24,8)}
        offset = 4 if member in ('InstanceContributionToHitGroupIndex','Flags') else 0
        storage = 'word'
        prefix = 'word := '+ ' | '.join('((cast(u32) fields.__bitfield['+str(offset+i)+']) << '+str(i*8)+')' for i in range(4))+'; '
        suffix = ' for i: 0..3 fields.__bitfield['+str(offset)+'+i] = cast,trunc(u8) (word >> (i*8));'
    elif record == 'mach_msg_type_descriptor_t':
        mapping = {'pad3':(0,24),'type':(24,8)}
    elif record == 'mach_msg_port_descriptor_t':
        mapping = {'pad2':(0,16),'disposition':(16,8),'type':(24,8)}
    elif record.startswith('mach_msg_ool_') and 'descriptor' in record:
        mapping = {'deallocate':(0,8),'copy':(8,8),'pad1':(16,8),'disposition':(16,8),'type':(24,8)}
    if member not in mapping:
        return None
    shift, width = mapping[member]
    mask = (1 << width) - 1
    if name.startswith('set_'):
        value_type = params[1].split(':',1)[1].strip()
        converted = 'value' if value_type == data_type else '(cast('+data_type+') value)'
        return prefix+storage+' = ('+storage+' & ~(cast('+data_type+') '+str(mask << shift)+')) | (('+converted+' & '+str(mask)+') << '+str(shift)+');'+suffix
    result = '('+storage+' >> '+str(shift)+') & '+str(mask)
    return_type = re.search(r'->\s*(\w+)',tail)[1]
    return prefix+('return ('+result+') != 0;' if return_type=='bool' else 'return cast('+return_type+') ('+result+');')
