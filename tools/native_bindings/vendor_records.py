"""Shared strict LP64 C record translation and active ABI assertion generation."""
import hashlib
import subprocess

SCALARS = {'int':('s32',4,4), 'short':('s16',2,2),
           'unsigned int':('u32',4,4), 'long':('s64',8,8),
           'unsigned long':('u64',8,8), 'void *':('*void',8,8)}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def record_shape(record):
    if not record.get('completeDefinition') or record.get('tagUsed') != 'struct':
        raise ValueError('A complete ordinary C struct is required')
    if any(n['kind'] not in ('FieldDecl','FullComment') for n in record.get('inner',[])):
        raise ValueError('Unsupported nested/attributed C record; never flatten or omit it')
    fields, offset, alignment = [], 0, 1
    for field in record['inner']:
        if field['kind'] != 'FieldDecl':
            continue
        if not field.get('name') or field.get('isBitfield') or field.get('inner'):
            raise ValueError('Unsupported anonymous, bitfield or attributed C field')
        ty = field['type']
        canonical = ty.get('desugaredQualType',ty['qualType'])
        if canonical not in SCALARS:
            raise ValueError('Unreviewed LP64 field type: '+canonical)
        jai, size, align = SCALARS[canonical]
        offset = (offset+align-1)//align*align
        fields.append({'name':field['name'],'native_type':ty['qualType'],
                       'canonical_c_type':canonical,'jai_type':jai,
                       'offset':offset,'size':size,'alignment':align})
        offset += size
        alignment = max(alignment,align)
    if not fields:
        raise ValueError('No complete fields in native C record')
    name = record['name']
    return {'name':name,'code':name+' :: struct {\n'+
            '\n'.join('    '+f['name']+': '+f['jai_type']+';' for f in fields)+'\n}',
            'size':(offset+alignment-1)//alignment*alignment,'alignment':alignment,'fields':fields}


def assertions(shapes):
    lines = ['_Static_assert(sizeof(void*) == 8 && sizeof(long) == 8 && sizeof(int) == 4 && sizeof(short) == 2, "LP64 scalar ABI");']
    for shape in shapes:
        name = shape['name']
        for expression, expected in [('sizeof(struct '+name+')',shape['size']),
                                     ('_Alignof(struct '+name+')',shape['alignment'])]:
            lines.append('_Static_assert('+expression+' == '+str(expected)+', "'+name+' layout");')
        for field in shape['fields']:
            member = field['name']
            lines.append('_Static_assert(offsetof(struct '+name+', '+member+') == '+str(field['offset'])+', "'+name+'.'+member+' offset");')
            lines.append('_Static_assert(sizeof(((struct '+name+'*)0)->'+member+') == '+str(field['size'])+', "'+name+'.'+member+' size");')
    return '\n'.join(lines)+'\n'


def run(command):
    return subprocess.run([str(x) for x in command],check=True,capture_output=True,text=True)
