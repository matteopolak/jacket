// Generated from docs/specs/stdlib/modules/metal.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "Objective_C";

binding_objc_class :: (name: *u8) -> *void #foreign libobjc "objc_getClass";
binding_objc_selector :: (name: *u8) -> *void #foreign libobjc "sel_registerName";
binding_objc_send :: () #foreign libobjc "objc_msgSend";

#if CPU == .X64 {
    binding_objc_send_stret :: () #foreign libobjc "objc_msgSend_stret";
}

#library,system,link_always "Metal";
uint64_t :: u64;
uint32_t :: u32;
int32_t :: s32;
int64_t :: s64;
uint16_t :: u16;
uint8_t :: u8;
CGColorSpace :: struct {};
CGColorSpaceRef :: *CGColorSpace;
CFTimeInterval :: float64;
__IOSurface :: struct {};
IOSurfaceRef :: *__IOSurface;
binding_abort :: () #foreign libc "abort";
Metal :: #library,system "Metal";
libc :: #library,system "c";
libobjc :: #library,system "objc";
#scope_export

// Types and layouts

_CAMetalLayerPrivate :: struct {}

CAMetalDrawable :: struct {
    texture :: (self: *CAMetalDrawable) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("texture"));
    }
    layer :: (self: *CAMetalDrawable) -> *CAMetalLayer {
        Send :: #type (receiver: *void, op: *void) -> *CAMetalLayer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*CAMetalLayer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("layer"));
    }
}

CAMetalLayer :: struct {
    using calayer: CALayer;
    _priv: *_CAMetalLayerPrivate;
    nextDrawable :: (self: *CAMetalLayer) -> *CAMetalDrawable {
        Send :: #type (receiver: *void, op: *void) -> *CAMetalDrawable #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*CAMetalDrawable) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("nextDrawable"));
    }
    device :: (self: *CAMetalLayer) -> *MTLDevice {
        Send :: #type (receiver: *void, op: *void) -> *MTLDevice #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLDevice) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    setDevice :: (self: *CAMetalLayer, device: *MTLDevice) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(device)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDevice:"), device);
    }
    pixelFormat :: (self: *CAMetalLayer) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("pixelFormat"));
    }
    setPixelFormat :: (self: *CAMetalLayer, pixelFormat: MTLPixelFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setPixelFormat:"), pixelFormat);
    }
    framebufferOnly :: (self: *CAMetalLayer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("framebufferOnly"));
    }
    setFramebufferOnly :: (self: *CAMetalLayer, framebufferOnly: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(framebufferOnly)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFramebufferOnly:"), framebufferOnly);
    }
    drawableSize :: (self: *CAMetalLayer) -> CGSize {
        Send :: #type (receiver: *void, op: *void) -> CGSize #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CGSize) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("drawableSize"));
    }
    setDrawableSize :: (self: *CAMetalLayer, drawableSize: CGSize) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(drawableSize)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDrawableSize:"), drawableSize);
    }
    maximumDrawableCount :: (self: *CAMetalLayer) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maximumDrawableCount"));
    }
    setMaximumDrawableCount :: (self: *CAMetalLayer, maximumDrawableCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maximumDrawableCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaximumDrawableCount:"), maximumDrawableCount);
    }
    presentsWithTransaction :: (self: *CAMetalLayer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("presentsWithTransaction"));
    }
    setPresentsWithTransaction :: (self: *CAMetalLayer, presentsWithTransaction: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(presentsWithTransaction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setPresentsWithTransaction:"), presentsWithTransaction);
    }
    colorspace :: (self: *CAMetalLayer) -> CGColorSpaceRef {
        Send :: #type (receiver: *void, op: *void) -> CGColorSpaceRef #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CGColorSpaceRef) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("colorspace"));
    }
    setColorspace :: (self: *CAMetalLayer, colorspace: CGColorSpaceRef) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(colorspace)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setColorspace:"), colorspace);
    }
    wantsExtendedDynamicRangeContent :: (self: *CAMetalLayer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("wantsExtendedDynamicRangeContent"));
    }
    setWantsExtendedDynamicRangeContent :: (self: *CAMetalLayer, wantsExtendedDynamicRangeContent: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(wantsExtendedDynamicRangeContent)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setWantsExtendedDynamicRangeContent:"), wantsExtendedDynamicRangeContent);
    }
    displaySyncEnabled :: (self: *CAMetalLayer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("displaySyncEnabled"));
    }
    setDisplaySyncEnabled :: (self: *CAMetalLayer, displaySyncEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(displaySyncEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDisplaySyncEnabled:"), displaySyncEnabled);
    }
    allowsNextDrawableTimeout :: (self: *CAMetalLayer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("allowsNextDrawableTimeout"));
    }
    setAllowsNextDrawableTimeout :: (self: *CAMetalLayer, allowsNextDrawableTimeout: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(allowsNextDrawableTimeout)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAllowsNextDrawableTimeout:"), allowsNextDrawableTimeout);
    }
}

MTLArgument :: struct {
    nsobject: NSObject;
    name :: (self: *MTLArgument) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    type :: (self: *MTLArgument) -> MTLArgumentType {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("type"));
    }
    access :: (self: *MTLArgument) -> MTLArgumentAccess {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentAccess #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentAccess) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("access"));
    }
    index :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("index"));
    }
    isActive :: (self: *MTLArgument) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isActive"));
    }
    bufferAlignment :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferAlignment"));
    }
    bufferDataSize :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferDataSize"));
    }
    bufferDataType :: (self: *MTLArgument) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferDataType"));
    }
    bufferStructType :: (self: *MTLArgument) -> *MTLStructType {
        Send :: #type (receiver: *void, op: *void) -> *MTLStructType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStructType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferStructType"));
    }
    bufferPointerType :: (self: *MTLArgument) -> *MTLPointerType {
        Send :: #type (receiver: *void, op: *void) -> *MTLPointerType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPointerType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferPointerType"));
    }
    threadgroupMemoryAlignment :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("threadgroupMemoryAlignment"));
    }
    threadgroupMemoryDataSize :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("threadgroupMemoryDataSize"));
    }
    textureType :: (self: *MTLArgument) -> MTLTextureType {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureType"));
    }
    textureDataType :: (self: *MTLArgument) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureDataType"));
    }
    isDepthTexture :: (self: *MTLArgument) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isDepthTexture"));
    }
    arrayLength :: (self: *MTLArgument) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayLength"));
    }
}

using MTLArgumentAccess :: enum NSUInteger {
    MTLArgumentAccessReadOnly :: 0;
    MTLArgumentAccessReadWrite :: 1;
    MTLArgumentAccessWriteOnly :: 2;
}

using MTLArgumentBuffersTier :: enum NSUInteger {
    MTLArgumentBuffersTier1 :: 0;
    MTLArgumentBuffersTier2 :: 1;
}

MTLArgumentDescriptor :: struct {
    nsobject: NSObject;
    argumentDescriptor :: () -> *MTLArgumentDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLArgumentDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLArgumentDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLArgumentDescriptor"), binding_objc_selector("argumentDescriptor"));
    }
    dataType :: (self: *MTLArgumentDescriptor) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dataType"));
    }
    setDataType :: (self: *MTLArgumentDescriptor, dataType: MTLDataType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(dataType)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDataType:"), dataType);
    }
    index :: (self: *MTLArgumentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("index"));
    }
    setIndex :: (self: *MTLArgumentDescriptor, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setIndex:"), index);
    }
    arrayLength :: (self: *MTLArgumentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayLength"));
    }
    setArrayLength :: (self: *MTLArgumentDescriptor, arrayLength: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(arrayLength)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setArrayLength:"), arrayLength);
    }
    access :: (self: *MTLArgumentDescriptor) -> MTLArgumentAccess {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentAccess #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentAccess) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("access"));
    }
    setAccess :: (self: *MTLArgumentDescriptor, access: MTLArgumentAccess) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(access)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAccess:"), access);
    }
    textureType :: (self: *MTLArgumentDescriptor) -> MTLTextureType {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureType"));
    }
    setTextureType :: (self: *MTLArgumentDescriptor, textureType: MTLTextureType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textureType)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTextureType:"), textureType);
    }
    constantBlockAlignment :: (self: *MTLArgumentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("constantBlockAlignment"));
    }
    setConstantBlockAlignment :: (self: *MTLArgumentDescriptor, constantBlockAlignment: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(constantBlockAlignment)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setConstantBlockAlignment:"), constantBlockAlignment);
    }
}

MTLArgumentEncoder :: struct {
    setArgumentBuffer_offset :: (self: *MTLArgumentEncoder, argumentBuffer: id, offset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(argumentBuffer), a1: type_of(offset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setArgumentBuffer:offset:"), argumentBuffer, offset);
    }
    setArgumentBuffer_startOffset_arrayElement :: (self: *MTLArgumentEncoder, argumentBuffer: id, startOffset: NSUInteger, arrayElement: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(argumentBuffer), a1: type_of(startOffset), a2: type_of(arrayElement)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setArgumentBuffer:startOffset:arrayElement:"), argumentBuffer, startOffset, arrayElement);
    }
    setBuffer_offset_atIndex :: (self: *MTLArgumentEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBuffer:offset:atIndex:"), buffer, offset, index);
    }
    setBuffers_offsets_withRange :: (self: *MTLArgumentEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffers), a1: type_of(offsets), a2: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBuffers:offsets:withRange:"), buffers, offsets, range);
    }
    setTexture_atIndex :: (self: *MTLArgumentEncoder, texture: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTexture:atIndex:"), texture, index);
    }
    setTextures_withRange :: (self: *MTLArgumentEncoder, textures: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textures), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTextures:withRange:"), textures, range);
    }
    setSamplerState_atIndex :: (self: *MTLArgumentEncoder, sampler: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerState:atIndex:"), sampler, index);
    }
    setSamplerStates_withRange :: (self: *MTLArgumentEncoder, samplers: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerStates:withRange:"), samplers, range);
    }
    constantDataAtIndex :: (self: *MTLArgumentEncoder, index: NSUInteger) -> *void {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> *void #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*void) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("constantDataAtIndex:"), index);
    }
    setRenderPipelineState_atIndex :: (self: *MTLArgumentEncoder, pipeline: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pipeline), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRenderPipelineState:atIndex:"), pipeline, index);
    }
    setRenderPipelineStates_withRange :: (self: *MTLArgumentEncoder, pipelines: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pipelines), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRenderPipelineStates:withRange:"), pipelines, range);
    }
    setIndirectCommandBuffer_atIndex :: (self: *MTLArgumentEncoder, indirectCommandBuffer: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectCommandBuffer), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setIndirectCommandBuffer:atIndex:"), indirectCommandBuffer, index);
    }
    setIndirectCommandBuffers_withRange :: (self: *MTLArgumentEncoder, buffers: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffers), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setIndirectCommandBuffers:withRange:"), buffers, range);
    }
    newArgumentEncoderForBufferAtIndex :: (self: *MTLArgumentEncoder, index: NSUInteger) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newArgumentEncoderForBufferAtIndex:"), index);
    }
    device :: (self: *MTLArgumentEncoder) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    label :: (self: *MTLArgumentEncoder) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLArgumentEncoder, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    encodedLength :: (self: *MTLArgumentEncoder) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("encodedLength"));
    }
    alignment :: (self: *MTLArgumentEncoder) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("alignment"));
    }
}

using MTLArgumentType :: enum NSUInteger {
    MTLArgumentTypeBuffer :: 0;
    MTLArgumentTypeThreadgroupMemory :: 1;
    MTLArgumentTypeTexture :: 2;
    MTLArgumentTypeSampler :: 3;
}

MTLArrayType :: struct {
    mtltype: MTLType;
    elementStructType :: (self: *MTLArrayType) -> *MTLStructType {
        Send :: #type (receiver: *void, op: *void) -> *MTLStructType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStructType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementStructType"));
    }
    elementArrayType :: (self: *MTLArrayType) -> *MTLArrayType {
        Send :: #type (receiver: *void, op: *void) -> *MTLArrayType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLArrayType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementArrayType"));
    }
    elementTextureReferenceType :: (self: *MTLArrayType) -> *MTLTextureReferenceType {
        Send :: #type (receiver: *void, op: *void) -> *MTLTextureReferenceType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLTextureReferenceType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementTextureReferenceType"));
    }
    elementPointerType :: (self: *MTLArrayType) -> *MTLPointerType {
        Send :: #type (receiver: *void, op: *void) -> *MTLPointerType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPointerType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementPointerType"));
    }
    elementType :: (self: *MTLArrayType) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementType"));
    }
    arrayLength :: (self: *MTLArrayType) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayLength"));
    }
    stride :: (self: *MTLArrayType) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stride"));
    }
    argumentIndexStride :: (self: *MTLArrayType) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("argumentIndexStride"));
    }
}

MTLAttribute :: struct {
    nsobject: NSObject;
    name :: (self: *MTLAttribute) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    attributeIndex :: (self: *MTLAttribute) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributeIndex"));
    }
    attributeType :: (self: *MTLAttribute) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributeType"));
    }
    isActive :: (self: *MTLAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isActive"));
    }
    isPatchData :: (self: *MTLAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isPatchData"));
    }
    isPatchControlPointData :: (self: *MTLAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isPatchControlPointData"));
    }
}

MTLAttributeDescriptor :: struct {
    nsobject: NSObject;
    format :: (self: *MTLAttributeDescriptor) -> MTLAttributeFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLAttributeFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLAttributeFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("format"));
    }
    setFormat :: (self: *MTLAttributeDescriptor, format: MTLAttributeFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(format)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFormat:"), format);
    }
    offset :: (self: *MTLAttributeDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("offset"));
    }
    setOffset :: (self: *MTLAttributeDescriptor, offset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(offset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setOffset:"), offset);
    }
    bufferIndex :: (self: *MTLAttributeDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferIndex"));
    }
    setBufferIndex :: (self: *MTLAttributeDescriptor, bufferIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBufferIndex:"), bufferIndex);
    }
}

MTLAttributeDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLAttributeDescriptorArray, index: NSUInteger) -> *MTLAttributeDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> *MTLAttributeDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLAttributeDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), index);
    }
    setObject_atIndexedSubscript :: (self: *MTLAttributeDescriptorArray, attributeDesc: *MTLAttributeDescriptor, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attributeDesc), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), attributeDesc, index);
    }
}

using MTLAttributeFormat :: enum NSUInteger {
    MTLAttributeFormatInvalid :: 0;
    MTLAttributeFormatUChar2 :: 1;
    MTLAttributeFormatUChar3 :: 2;
    MTLAttributeFormatUChar4 :: 3;
    MTLAttributeFormatChar2 :: 4;
    MTLAttributeFormatChar3 :: 5;
    MTLAttributeFormatChar4 :: 6;
    MTLAttributeFormatUChar2Normalized :: 7;
    MTLAttributeFormatUChar3Normalized :: 8;
    MTLAttributeFormatUChar4Normalized :: 9;
    MTLAttributeFormatChar2Normalized :: 10;
    MTLAttributeFormatChar3Normalized :: 11;
    MTLAttributeFormatChar4Normalized :: 12;
    MTLAttributeFormatUShort2 :: 13;
    MTLAttributeFormatUShort3 :: 14;
    MTLAttributeFormatUShort4 :: 15;
    MTLAttributeFormatShort2 :: 16;
    MTLAttributeFormatShort3 :: 17;
    MTLAttributeFormatShort4 :: 18;
    MTLAttributeFormatUShort2Normalized :: 19;
    MTLAttributeFormatUShort3Normalized :: 20;
    MTLAttributeFormatUShort4Normalized :: 21;
    MTLAttributeFormatShort2Normalized :: 22;
    MTLAttributeFormatShort3Normalized :: 23;
    MTLAttributeFormatShort4Normalized :: 24;
    MTLAttributeFormatHalf2 :: 25;
    MTLAttributeFormatHalf3 :: 26;
    MTLAttributeFormatHalf4 :: 27;
    MTLAttributeFormatFloat :: 28;
    MTLAttributeFormatFloat2 :: 29;
    MTLAttributeFormatFloat3 :: 30;
    MTLAttributeFormatFloat4 :: 31;
    MTLAttributeFormatInt :: 32;
    MTLAttributeFormatInt2 :: 33;
    MTLAttributeFormatInt3 :: 34;
    MTLAttributeFormatInt4 :: 35;
    MTLAttributeFormatUInt :: 36;
    MTLAttributeFormatUInt2 :: 37;
    MTLAttributeFormatUInt3 :: 38;
    MTLAttributeFormatUInt4 :: 39;
    MTLAttributeFormatInt1010102Normalized :: 40;
    MTLAttributeFormatUInt1010102Normalized :: 41;
    MTLAttributeFormatUChar4Normalized_BGRA :: 42;
    MTLAttributeFormatUChar :: 45;
    MTLAttributeFormatChar :: 46;
    MTLAttributeFormatUCharNormalized :: 47;
    MTLAttributeFormatCharNormalized :: 48;
    MTLAttributeFormatUShort :: 49;
    MTLAttributeFormatShort :: 50;
    MTLAttributeFormatUShortNormalized :: 51;
    MTLAttributeFormatShortNormalized :: 52;
    MTLAttributeFormatHalf :: 53;
}

using MTLBarrierScope :: enum NSUInteger {
    MTLBarrierScopeBuffers :: 1;
    MTLBarrierScopeTextures :: 2;
    MTLBarrierScopeRenderTargets :: 4;
}

using MTLBlendFactor :: enum NSUInteger {
    MTLBlendFactorZero :: 0;
    MTLBlendFactorOne :: 1;
    MTLBlendFactorSourceColor :: 2;
    MTLBlendFactorOneMinusSourceColor :: 3;
    MTLBlendFactorSourceAlpha :: 4;
    MTLBlendFactorOneMinusSourceAlpha :: 5;
    MTLBlendFactorDestinationColor :: 6;
    MTLBlendFactorOneMinusDestinationColor :: 7;
    MTLBlendFactorDestinationAlpha :: 8;
    MTLBlendFactorOneMinusDestinationAlpha :: 9;
    MTLBlendFactorSourceAlphaSaturated :: 10;
    MTLBlendFactorBlendColor :: 11;
    MTLBlendFactorOneMinusBlendColor :: 12;
    MTLBlendFactorBlendAlpha :: 13;
    MTLBlendFactorOneMinusBlendAlpha :: 14;
    MTLBlendFactorSource1Color :: 15;
    MTLBlendFactorOneMinusSource1Color :: 16;
    MTLBlendFactorSource1Alpha :: 17;
    MTLBlendFactorOneMinusSource1Alpha :: 18;
}

using MTLBlendOperation :: enum NSUInteger {
    MTLBlendOperationAdd :: 0;
    MTLBlendOperationSubtract :: 1;
    MTLBlendOperationReverseSubtract :: 2;
    MTLBlendOperationMin :: 3;
    MTLBlendOperationMax :: 4;
}

MTLBlitCommandEncoder :: struct {
    synchronizeResource :: (self: *MTLBlitCommandEncoder, resource: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resource)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("synchronizeResource:"), resource);
    }
    synchronizeTexture_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(slice), a2: type_of(level)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("synchronizeTexture:slice:level:"), texture, slice, level);
    }
    copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceTexture), a1: type_of(sourceSlice), a2: type_of(sourceLevel), a3: type_of(sourceOrigin), a4: type_of(sourceSize), a5: type_of(destinationTexture), a6: type_of(destinationSlice), a7: type_of(destinationLevel), a8: type_of(destinationOrigin)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromTexture:sourceSlice:sourceLevel:sourceOrigin:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:"), sourceTexture, sourceSlice, sourceLevel, sourceOrigin, sourceSize, destinationTexture, destinationSlice, destinationLevel, destinationOrigin);
    }
    copyFromBuffer_sourceOffset_sourceBytesPerRow_sourceBytesPerImage_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, sourceBytesPerRow: NSUInteger, sourceBytesPerImage: NSUInteger, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceBuffer), a1: type_of(sourceOffset), a2: type_of(sourceBytesPerRow), a3: type_of(sourceBytesPerImage), a4: type_of(sourceSize), a5: type_of(destinationTexture), a6: type_of(destinationSlice), a7: type_of(destinationLevel), a8: type_of(destinationOrigin)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:"), sourceBuffer, sourceOffset, sourceBytesPerRow, sourceBytesPerImage, sourceSize, destinationTexture, destinationSlice, destinationLevel, destinationOrigin);
    }
    copyFromBuffer_sourceOffset_sourceBytesPerRow_sourceBytesPerImage_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin_options :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, sourceBytesPerRow: NSUInteger, sourceBytesPerImage: NSUInteger, sourceSize: MTLSize, destinationTexture: id, destinationSlice: NSUInteger, destinationLevel: NSUInteger, destinationOrigin: MTLOrigin, options: MTLBlitOption) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceBuffer), a1: type_of(sourceOffset), a2: type_of(sourceBytesPerRow), a3: type_of(sourceBytesPerImage), a4: type_of(sourceSize), a5: type_of(destinationTexture), a6: type_of(destinationSlice), a7: type_of(destinationLevel), a8: type_of(destinationOrigin), a9: type_of(options)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromBuffer:sourceOffset:sourceBytesPerRow:sourceBytesPerImage:sourceSize:toTexture:destinationSlice:destinationLevel:destinationOrigin:options:"), sourceBuffer, sourceOffset, sourceBytesPerRow, sourceBytesPerImage, sourceSize, destinationTexture, destinationSlice, destinationLevel, destinationOrigin, options);
    }
    copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationBuffer: id, destinationOffset: NSUInteger, destinationBytesPerRow: NSUInteger, destinationBytesPerImage: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceTexture), a1: type_of(sourceSlice), a2: type_of(sourceLevel), a3: type_of(sourceOrigin), a4: type_of(sourceSize), a5: type_of(destinationBuffer), a6: type_of(destinationOffset), a7: type_of(destinationBytesPerRow), a8: type_of(destinationBytesPerImage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromTexture:sourceSlice:sourceLevel:sourceOrigin:sourceSize:toBuffer:destinationOffset:destinationBytesPerRow:destinationBytesPerImage:"), sourceTexture, sourceSlice, sourceLevel, sourceOrigin, sourceSize, destinationBuffer, destinationOffset, destinationBytesPerRow, destinationBytesPerImage);
    }
    copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage_options :: (self: *MTLBlitCommandEncoder, sourceTexture: id, sourceSlice: NSUInteger, sourceLevel: NSUInteger, sourceOrigin: MTLOrigin, sourceSize: MTLSize, destinationBuffer: id, destinationOffset: NSUInteger, destinationBytesPerRow: NSUInteger, destinationBytesPerImage: NSUInteger, options: MTLBlitOption) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceTexture), a1: type_of(sourceSlice), a2: type_of(sourceLevel), a3: type_of(sourceOrigin), a4: type_of(sourceSize), a5: type_of(destinationBuffer), a6: type_of(destinationOffset), a7: type_of(destinationBytesPerRow), a8: type_of(destinationBytesPerImage), a9: type_of(options)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromTexture:sourceSlice:sourceLevel:sourceOrigin:sourceSize:toBuffer:destinationOffset:destinationBytesPerRow:destinationBytesPerImage:options:"), sourceTexture, sourceSlice, sourceLevel, sourceOrigin, sourceSize, destinationBuffer, destinationOffset, destinationBytesPerRow, destinationBytesPerImage, options);
    }
    generateMipmapsForTexture :: (self: *MTLBlitCommandEncoder, texture: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("generateMipmapsForTexture:"), texture);
    }
    fillBuffer_range_value :: (self: *MTLBlitCommandEncoder, buffer: id, range: NSRange, value: uint8_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(range), a2: type_of(value)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("fillBuffer:range:value:"), buffer, range, value);
    }
    copyFromBuffer_sourceOffset_toBuffer_destinationOffset_size :: (self: *MTLBlitCommandEncoder, sourceBuffer: id, sourceOffset: NSUInteger, destinationBuffer: id, destinationOffset: NSUInteger, size: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceBuffer), a1: type_of(sourceOffset), a2: type_of(destinationBuffer), a3: type_of(destinationOffset), a4: type_of(size)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyFromBuffer:sourceOffset:toBuffer:destinationOffset:size:"), sourceBuffer, sourceOffset, destinationBuffer, destinationOffset, size);
    }
    updateFence :: (self: *MTLBlitCommandEncoder, fence: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("updateFence:"), fence);
    }
    waitForFence :: (self: *MTLBlitCommandEncoder, fence: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("waitForFence:"), fence);
    }
    optimizeContentsForGPUAccess :: (self: *MTLBlitCommandEncoder, texture: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("optimizeContentsForGPUAccess:"), texture);
    }
    optimizeContentsForGPUAccess_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(slice), a2: type_of(level)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("optimizeContentsForGPUAccess:slice:level:"), texture, slice, level);
    }
    optimizeContentsForCPUAccess :: (self: *MTLBlitCommandEncoder, texture: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("optimizeContentsForCPUAccess:"), texture);
    }
    optimizeContentsForCPUAccess_slice_level :: (self: *MTLBlitCommandEncoder, texture: id, slice: NSUInteger, level: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(slice), a2: type_of(level)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("optimizeContentsForCPUAccess:slice:level:"), texture, slice, level);
    }
    resetCommandsInBuffer_withRange :: (self: *MTLBlitCommandEncoder, buffer: id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("resetCommandsInBuffer:withRange:"), buffer, range);
    }
    copyIndirectCommandBuffer_sourceRange_destination_destinationIndex :: (self: *MTLBlitCommandEncoder, source: id, sourceRange: NSRange, destination: id, destinationIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(source), a1: type_of(sourceRange), a2: type_of(destination), a3: type_of(destinationIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("copyIndirectCommandBuffer:sourceRange:destination:destinationIndex:"), source, sourceRange, destination, destinationIndex);
    }
    optimizeIndirectCommandBuffer_withRange :: (self: *MTLBlitCommandEncoder, indirectCommandBuffer: id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectCommandBuffer), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("optimizeIndirectCommandBuffer:withRange:"), indirectCommandBuffer, range);
    }
}

using MTLBlitOption :: enum NSUInteger {
    MTLBlitOptionNone :: 0;
    MTLBlitOptionDepthFromDepthStencil :: 1;
    MTLBlitOptionStencilFromDepthStencil :: 2;
    MTLBlitOptionRowLinearPVRTC :: 4;
}

MTLBuffer :: struct {
    contents :: (self: *MTLBuffer) -> *void {
        Send :: #type (receiver: *void, op: *void) -> *void #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*void) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("contents"));
    }
    didModifyRange :: (self: *MTLBuffer, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("didModifyRange:"), range);
    }
    newTextureWithDescriptor_offset_bytesPerRow :: (self: *MTLBuffer, descriptor: *MTLTextureDescriptor, offset: NSUInteger, bytesPerRow: NSUInteger) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(offset), a2: type_of(bytesPerRow)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureWithDescriptor:offset:bytesPerRow:"), descriptor, offset, bytesPerRow);
    }
    addDebugMarker_range :: (self: *MTLBuffer, marker: *NSString, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(marker), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("addDebugMarker:range:"), marker, range);
    }
    removeAllDebugMarkers :: (self: *MTLBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("removeAllDebugMarkers"));
    }
    length :: (self: *MTLBuffer) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("length"));
    }
}

MTLBufferLayoutDescriptor :: struct {
    nsobject: NSObject;
    stride :: (self: *MTLBufferLayoutDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stride"));
    }
    setStride :: (self: *MTLBufferLayoutDescriptor, stride: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stride)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStride:"), stride);
    }
    stepFunction :: (self: *MTLBufferLayoutDescriptor) -> MTLStepFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLStepFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStepFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stepFunction"));
    }
    setStepFunction :: (self: *MTLBufferLayoutDescriptor, stepFunction: MTLStepFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stepFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStepFunction:"), stepFunction);
    }
    stepRate :: (self: *MTLBufferLayoutDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stepRate"));
    }
    setStepRate :: (self: *MTLBufferLayoutDescriptor, stepRate: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stepRate)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStepRate:"), stepRate);
    }
}

MTLBufferLayoutDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLBufferLayoutDescriptorArray, index: NSUInteger) -> *MTLBufferLayoutDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> *MTLBufferLayoutDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBufferLayoutDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), index);
    }
    setObject_atIndexedSubscript :: (self: *MTLBufferLayoutDescriptorArray, bufferDesc: *MTLBufferLayoutDescriptor, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferDesc), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), bufferDesc, index);
    }
}

MTLCaptureManager :: struct {
    nsobject: NSObject;
    _isCapturing: BOOL;
    _defaultCaptureScope: id;
    sharedCaptureManager :: () -> *MTLCaptureManager {
        Send :: #type (receiver: *void, op: *void) -> *MTLCaptureManager #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCaptureManager) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLCaptureManager"), binding_objc_selector("sharedCaptureManager"));
    }
    init :: (self: *MTLCaptureManager) -> *MTLCaptureManager {
        Send :: #type (receiver: *void, op: *void) -> *MTLCaptureManager #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCaptureManager) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("init"));
    }
    newCaptureScopeWithDevice :: (self: *MTLCaptureManager, device: id) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(device)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newCaptureScopeWithDevice:"), device);
    }
    newCaptureScopeWithCommandQueue :: (self: *MTLCaptureManager, commandQueue: id) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(commandQueue)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newCaptureScopeWithCommandQueue:"), commandQueue);
    }
    startCaptureWithDevice :: (self: *MTLCaptureManager, device: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(device)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("startCaptureWithDevice:"), device);
    }
    startCaptureWithCommandQueue :: (self: *MTLCaptureManager, commandQueue: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(commandQueue)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("startCaptureWithCommandQueue:"), commandQueue);
    }
    startCaptureWithScope :: (self: *MTLCaptureManager, captureScope: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(captureScope)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("startCaptureWithScope:"), captureScope);
    }
    stopCapture :: (self: *MTLCaptureManager) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("stopCapture"));
    }
    defaultCaptureScope :: (self: *MTLCaptureManager) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("defaultCaptureScope"));
    }
    setDefaultCaptureScope :: (self: *MTLCaptureManager, defaultCaptureScope: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(defaultCaptureScope)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDefaultCaptureScope:"), defaultCaptureScope);
    }
    isCapturing :: (self: *MTLCaptureManager) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isCapturing"));
    }
}

MTLCaptureScope :: struct {
    beginScope :: (self: *MTLCaptureScope) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("beginScope"));
    }
    endScope :: (self: *MTLCaptureScope) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("endScope"));
    }
    label :: (self: *MTLCaptureScope) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLCaptureScope, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLCaptureScope) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    commandQueue :: (self: *MTLCaptureScope) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("commandQueue"));
    }
}

MTLClearColor :: struct {
    red: float64;
    green: float64;
    blue: float64;
    alpha: float64;
}

using MTLColorWriteMask :: enum NSUInteger {
    MTLColorWriteMaskNone :: 0;
    MTLColorWriteMaskRed :: 8;
    MTLColorWriteMaskGreen :: 4;
    MTLColorWriteMaskBlue :: 2;
    MTLColorWriteMaskAlpha :: 1;
    MTLColorWriteMaskAll :: 15;
}

MTLCommandBuffer :: struct {
    enqueue :: (self: *MTLCommandBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("enqueue"));
    }
    commit :: (self: *MTLCommandBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("commit"));
    }
    presentDrawable :: (self: *MTLCommandBuffer, drawable: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(drawable)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("presentDrawable:"), drawable);
    }
    presentDrawable_atTime :: (self: *MTLCommandBuffer, drawable: id, presentationTime: CFTimeInterval) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(drawable), a1: type_of(presentationTime)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("presentDrawable:atTime:"), drawable, presentationTime);
    }
    waitUntilScheduled :: (self: *MTLCommandBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("waitUntilScheduled"));
    }
    waitUntilCompleted :: (self: *MTLCommandBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("waitUntilCompleted"));
    }
    blitCommandEncoder :: (self: *MTLCommandBuffer) -> *MTLBlitCommandEncoder {
        Send :: #type (receiver: *void, op: *void) -> *MTLBlitCommandEncoder #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBlitCommandEncoder) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("blitCommandEncoder"));
    }
    renderCommandEncoderWithDescriptor :: (self: *MTLCommandBuffer, renderPassDescriptor: *MTLRenderPassDescriptor) -> *MTLRenderCommandEncoder {
        Send :: #type (receiver: *void, op: *void, a0: type_of(renderPassDescriptor)) -> *MTLRenderCommandEncoder #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderCommandEncoder) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("renderCommandEncoderWithDescriptor:"), renderPassDescriptor);
    }
    computeCommandEncoder :: (self: *MTLCommandBuffer) -> *MTLComputeCommandEncoder {
        Send :: #type (receiver: *void, op: *void) -> *MTLComputeCommandEncoder #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLComputeCommandEncoder) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("computeCommandEncoder"));
    }
    computeCommandEncoderWithDispatchType :: (self: *MTLCommandBuffer, dispatchType: MTLDispatchType) -> *MTLComputeCommandEncoder {
        Send :: #type (receiver: *void, op: *void, a0: type_of(dispatchType)) -> *MTLComputeCommandEncoder #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLComputeCommandEncoder) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("computeCommandEncoderWithDispatchType:"), dispatchType);
    }
    encodeWaitForEvent_value :: (self: *MTLCommandBuffer, event: id, value: uint64_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(event), a1: type_of(value)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("encodeWaitForEvent:value:"), event, value);
    }
    encodeSignalEvent_value :: (self: *MTLCommandBuffer, event: id, value: uint64_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(event), a1: type_of(value)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("encodeSignalEvent:value:"), event, value);
    }
    parallelRenderCommandEncoderWithDescriptor :: (self: *MTLCommandBuffer, renderPassDescriptor: *MTLRenderPassDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(renderPassDescriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("parallelRenderCommandEncoderWithDescriptor:"), renderPassDescriptor);
    }
    pushDebugGroup :: (self: *MTLCommandBuffer, _string: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(_string)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("pushDebugGroup:"), _string);
    }
    popDebugGroup :: (self: *MTLCommandBuffer) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("popDebugGroup"));
    }
    device :: (self: *MTLCommandBuffer) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    commandQueue :: (self: *MTLCommandBuffer) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("commandQueue"));
    }
    retainedReferences :: (self: *MTLCommandBuffer) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("retainedReferences"));
    }
    label :: (self: *MTLCommandBuffer) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLCommandBuffer, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    kernelStartTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {
        Send :: #type (receiver: *void, op: *void) -> CFTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CFTimeInterval) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("kernelStartTime"));
    }
    kernelEndTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {
        Send :: #type (receiver: *void, op: *void) -> CFTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CFTimeInterval) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("kernelEndTime"));
    }
    GPUStartTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {
        Send :: #type (receiver: *void, op: *void) -> CFTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CFTimeInterval) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("GPUStartTime"));
    }
    GPUEndTime :: (self: *MTLCommandBuffer) -> CFTimeInterval {
        Send :: #type (receiver: *void, op: *void) -> CFTimeInterval #c_call;
        send: Send;

        #if CPU == .X64 && size_of(CFTimeInterval) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("GPUEndTime"));
    }
    status :: (self: *MTLCommandBuffer) -> MTLCommandBufferStatus {
        Send :: #type (receiver: *void, op: *void) -> MTLCommandBufferStatus #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCommandBufferStatus) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("status"));
    }
    error :: (self: *MTLCommandBuffer) -> *NSError {
        Send :: #type (receiver: *void, op: *void) -> *NSError #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSError) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("error"));
    }
}

using MTLCommandBufferError :: enum NSUInteger {
    MTLCommandBufferErrorNone :: 0;
    MTLCommandBufferErrorInternal :: 1;
    MTLCommandBufferErrorTimeout :: 2;
    MTLCommandBufferErrorPageFault :: 3;
    MTLCommandBufferErrorBlacklisted :: 4;
    MTLCommandBufferErrorNotPermitted :: 7;
    MTLCommandBufferErrorOutOfMemory :: 8;
    MTLCommandBufferErrorInvalidResource :: 9;
    MTLCommandBufferErrorMemoryless :: 10;
    MTLCommandBufferErrorDeviceRemoved :: 11;
}

using MTLCommandBufferStatus :: enum NSUInteger {
    MTLCommandBufferStatusNotEnqueued :: 0;
    MTLCommandBufferStatusEnqueued :: 1;
    MTLCommandBufferStatusCommitted :: 2;
    MTLCommandBufferStatusScheduled :: 3;
    MTLCommandBufferStatusCompleted :: 4;
    MTLCommandBufferStatusError :: 5;
}

MTLCommandEncoder :: struct {
    endEncoding :: (self: *MTLCommandEncoder) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("endEncoding"));
    }
    insertDebugSignpost :: (self: *MTLCommandEncoder, _string: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(_string)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("insertDebugSignpost:"), _string);
    }
    pushDebugGroup :: (self: *MTLCommandEncoder, _string: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(_string)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("pushDebugGroup:"), _string);
    }
    popDebugGroup :: (self: *MTLCommandEncoder) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("popDebugGroup"));
    }
    device :: (self: *MTLCommandEncoder) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    label :: (self: *MTLCommandEncoder) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLCommandEncoder, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
}

MTLCommandQueue :: struct {
    commandBuffer :: (self: *MTLCommandQueue) -> *MTLCommandBuffer {
        Send :: #type (receiver: *void, op: *void) -> *MTLCommandBuffer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCommandBuffer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("commandBuffer"));
    }
    commandBufferWithUnretainedReferences :: (self: *MTLCommandQueue) -> *MTLCommandBuffer {
        Send :: #type (receiver: *void, op: *void) -> *MTLCommandBuffer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCommandBuffer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("commandBufferWithUnretainedReferences"));
    }
    insertDebugCaptureBoundary :: (self: *MTLCommandQueue) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("insertDebugCaptureBoundary"));
    }
    label :: (self: *MTLCommandQueue) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLCommandQueue, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLCommandQueue) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
}

using MTLCompareFunction :: enum NSUInteger {
    MTLCompareFunctionNever :: 0;
    MTLCompareFunctionLess :: 1;
    MTLCompareFunctionEqual :: 2;
    MTLCompareFunctionLessEqual :: 3;
    MTLCompareFunctionGreater :: 4;
    MTLCompareFunctionNotEqual :: 5;
    MTLCompareFunctionGreaterEqual :: 6;
    MTLCompareFunctionAlways :: 7;
}

MTLCompileOptions :: struct {
    nsobject: NSObject;
    preprocessorMacros :: (self: *MTLCompileOptions) -> *NSDictionary {
        Send :: #type (receiver: *void, op: *void) -> *NSDictionary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSDictionary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("preprocessorMacros"));
    }
    setPreprocessorMacros :: (self: *MTLCompileOptions, preprocessorMacros: *NSDictionary) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(preprocessorMacros)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setPreprocessorMacros:"), preprocessorMacros);
    }
    fastMathEnabled :: (self: *MTLCompileOptions) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("fastMathEnabled"));
    }
    setFastMathEnabled :: (self: *MTLCompileOptions, fastMathEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fastMathEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFastMathEnabled:"), fastMathEnabled);
    }
    languageVersion :: (self: *MTLCompileOptions) -> MTLLanguageVersion {
        Send :: #type (receiver: *void, op: *void) -> MTLLanguageVersion #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLLanguageVersion) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("languageVersion"));
    }
    setLanguageVersion :: (self: *MTLCompileOptions, languageVersion: MTLLanguageVersion) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(languageVersion)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLanguageVersion:"), languageVersion);
    }
}

MTLComputeCommandEncoder :: struct {
    setComputePipelineState :: (self: *MTLComputeCommandEncoder, state: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(state)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setComputePipelineState:"), state);
    }
    setBytes_length_atIndex :: (self: *MTLComputeCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bytes), a1: type_of(length), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBytes:length:atIndex:"), bytes, length, index);
    }
    setBuffer_offset_atIndex :: (self: *MTLComputeCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBuffer:offset:atIndex:"), buffer, offset, index);
    }
    setBufferOffset_atIndex :: (self: *MTLComputeCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(offset), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBufferOffset:atIndex:"), offset, index);
    }
    setBuffers_offsets_withRange :: (self: *MTLComputeCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffers), a1: type_of(offsets), a2: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBuffers:offsets:withRange:"), buffers, offsets, range);
    }
    setTexture_atIndex :: (self: *MTLComputeCommandEncoder, texture: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTexture:atIndex:"), texture, index);
    }
    setTextures_withRange :: (self: *MTLComputeCommandEncoder, textures: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textures), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTextures:withRange:"), textures, range);
    }
    setSamplerState_atIndex :: (self: *MTLComputeCommandEncoder, sampler: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerState:atIndex:"), sampler, index);
    }
    setSamplerStates_withRange :: (self: *MTLComputeCommandEncoder, samplers: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerStates:withRange:"), samplers, range);
    }
    setSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLComputeCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(lodMinClamp), a2: type_of(lodMaxClamp), a3: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerState:lodMinClamp:lodMaxClamp:atIndex:"), sampler, lodMinClamp, lodMaxClamp, index);
    }
    setSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLComputeCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(lodMinClamps), a2: type_of(lodMaxClamps), a3: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplerStates:lodMinClamps:lodMaxClamps:withRange:"), samplers, lodMinClamps, lodMaxClamps, range);
    }
    setThreadgroupMemoryLength_atIndex :: (self: *MTLComputeCommandEncoder, length: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(length), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setThreadgroupMemoryLength:atIndex:"), length, index);
    }
    setStageInRegion :: (self: *MTLComputeCommandEncoder, region: MTLRegion) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(region)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStageInRegion:"), region);
    }
    setStageInRegionWithIndirectBuffer_indirectBufferOffset :: (self: *MTLComputeCommandEncoder, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectBuffer), a1: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStageInRegionWithIndirectBuffer:indirectBufferOffset:"), indirectBuffer, indirectBufferOffset);
    }
    dispatchThreadgroups_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, threadgroupsPerGrid: MTLSize, threadsPerThreadgroup: MTLSize) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(threadgroupsPerGrid), a1: type_of(threadsPerThreadgroup)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("dispatchThreadgroups:threadsPerThreadgroup:"), threadgroupsPerGrid, threadsPerThreadgroup);
    }
    dispatchThreadgroupsWithIndirectBuffer_indirectBufferOffset_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, indirectBuffer: id, indirectBufferOffset: NSUInteger, threadsPerThreadgroup: MTLSize) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectBuffer), a1: type_of(indirectBufferOffset), a2: type_of(threadsPerThreadgroup)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("dispatchThreadgroupsWithIndirectBuffer:indirectBufferOffset:threadsPerThreadgroup:"), indirectBuffer, indirectBufferOffset, threadsPerThreadgroup);
    }
    dispatchThreads_threadsPerThreadgroup :: (self: *MTLComputeCommandEncoder, threadsPerGrid: MTLSize, threadsPerThreadgroup: MTLSize) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(threadsPerGrid), a1: type_of(threadsPerThreadgroup)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("dispatchThreads:threadsPerThreadgroup:"), threadsPerGrid, threadsPerThreadgroup);
    }
    updateFence :: (self: *MTLComputeCommandEncoder, fence: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("updateFence:"), fence);
    }
    waitForFence :: (self: *MTLComputeCommandEncoder, fence: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("waitForFence:"), fence);
    }
    useResource_usage :: (self: *MTLComputeCommandEncoder, resource: id, usage: MTLResourceUsage) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resource), a1: type_of(usage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useResource:usage:"), resource, usage);
    }
    useResources_count_usage :: (self: *MTLComputeCommandEncoder, resources: *id, count: NSUInteger, usage: MTLResourceUsage) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resources), a1: type_of(count), a2: type_of(usage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useResources:count:usage:"), resources, count, usage);
    }
    useHeap :: (self: *MTLComputeCommandEncoder, heap: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(heap)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useHeap:"), heap);
    }
    useHeaps_count :: (self: *MTLComputeCommandEncoder, heaps: *id, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(heaps), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useHeaps:count:"), heaps, count);
    }
    memoryBarrierWithScope :: (self: *MTLComputeCommandEncoder, scope: MTLBarrierScope) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(scope)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("memoryBarrierWithScope:"), scope);
    }
    memoryBarrierWithResources_count :: (self: *MTLComputeCommandEncoder, resources: *id, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resources), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("memoryBarrierWithResources:count:"), resources, count);
    }
    dispatchType :: (self: *MTLComputeCommandEncoder) -> MTLDispatchType {
        Send :: #type (receiver: *void, op: *void) -> MTLDispatchType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDispatchType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dispatchType"));
    }
}

MTLComputePipelineDescriptor :: struct {
    nsobject: NSObject;
    reset :: (self: *MTLComputePipelineDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
    label :: (self: *MTLComputePipelineDescriptor) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLComputePipelineDescriptor, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    computeFunction :: (self: *MTLComputePipelineDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("computeFunction"));
    }
    setComputeFunction :: (self: *MTLComputePipelineDescriptor, computeFunction: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(computeFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setComputeFunction:"), computeFunction);
    }
    threadGroupSizeIsMultipleOfThreadExecutionWidth :: (self: *MTLComputePipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("threadGroupSizeIsMultipleOfThreadExecutionWidth"));
    }
    setThreadGroupSizeIsMultipleOfThreadExecutionWidth :: (self: *MTLComputePipelineDescriptor, threadGroupSizeIsMultipleOfThreadExecutionWidth: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(threadGroupSizeIsMultipleOfThreadExecutionWidth)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setThreadGroupSizeIsMultipleOfThreadExecutionWidth:"), threadGroupSizeIsMultipleOfThreadExecutionWidth);
    }
    maxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxTotalThreadsPerThreadgroup"));
    }
    setMaxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineDescriptor, maxTotalThreadsPerThreadgroup: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxTotalThreadsPerThreadgroup)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaxTotalThreadsPerThreadgroup:"), maxTotalThreadsPerThreadgroup);
    }
    stageInputDescriptor :: (self: *MTLComputePipelineDescriptor) -> *MTLStageInputOutputDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLStageInputOutputDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStageInputOutputDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stageInputDescriptor"));
    }
    setStageInputDescriptor :: (self: *MTLComputePipelineDescriptor, stageInputDescriptor: *MTLStageInputOutputDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stageInputDescriptor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStageInputDescriptor:"), stageInputDescriptor);
    }
    buffers :: (self: *MTLComputePipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLPipelineBufferDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPipelineBufferDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("buffers"));
    }
}

MTLComputePipelineReflection :: struct {
    nsobject: NSObject;
    arguments :: (self: *MTLComputePipelineReflection) -> *NSArray(*MTLArgument) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLArgument) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLArgument)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arguments"));
    }
}

MTLComputePipelineState :: struct {
    label :: (self: *MTLComputePipelineState) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    device :: (self: *MTLComputePipelineState) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    maxTotalThreadsPerThreadgroup :: (self: *MTLComputePipelineState) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxTotalThreadsPerThreadgroup"));
    }
    threadExecutionWidth :: (self: *MTLComputePipelineState) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("threadExecutionWidth"));
    }
    staticThreadgroupMemoryLength :: (self: *MTLComputePipelineState) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("staticThreadgroupMemoryLength"));
    }
}

using MTLCPUCacheMode :: enum NSUInteger {
    MTLCPUCacheModeDefaultCache :: 0;
    MTLCPUCacheModeWriteCombined :: 1;
}

using MTLCullMode :: enum NSUInteger {
    MTLCullModeNone :: 0;
    MTLCullModeFront :: 1;
    MTLCullModeBack :: 2;
}

using MTLDataType :: enum NSUInteger {
    MTLDataTypeNone :: 0;
    MTLDataTypeStruct :: 1;
    MTLDataTypeArray :: 2;
    MTLDataTypeFloat :: 3;
    MTLDataTypeFloat2 :: 4;
    MTLDataTypeFloat3 :: 5;
    MTLDataTypeFloat4 :: 6;
    MTLDataTypeFloat2x2 :: 7;
    MTLDataTypeFloat2x3 :: 8;
    MTLDataTypeFloat2x4 :: 9;
    MTLDataTypeFloat3x2 :: 10;
    MTLDataTypeFloat3x3 :: 11;
    MTLDataTypeFloat3x4 :: 12;
    MTLDataTypeFloat4x2 :: 13;
    MTLDataTypeFloat4x3 :: 14;
    MTLDataTypeFloat4x4 :: 15;
    MTLDataTypeHalf :: 16;
    MTLDataTypeHalf2 :: 17;
    MTLDataTypeHalf3 :: 18;
    MTLDataTypeHalf4 :: 19;
    MTLDataTypeHalf2x2 :: 20;
    MTLDataTypeHalf2x3 :: 21;
    MTLDataTypeHalf2x4 :: 22;
    MTLDataTypeHalf3x2 :: 23;
    MTLDataTypeHalf3x3 :: 24;
    MTLDataTypeHalf3x4 :: 25;
    MTLDataTypeHalf4x2 :: 26;
    MTLDataTypeHalf4x3 :: 27;
    MTLDataTypeHalf4x4 :: 28;
    MTLDataTypeInt :: 29;
    MTLDataTypeInt2 :: 30;
    MTLDataTypeInt3 :: 31;
    MTLDataTypeInt4 :: 32;
    MTLDataTypeUInt :: 33;
    MTLDataTypeUInt2 :: 34;
    MTLDataTypeUInt3 :: 35;
    MTLDataTypeUInt4 :: 36;
    MTLDataTypeShort :: 37;
    MTLDataTypeShort2 :: 38;
    MTLDataTypeShort3 :: 39;
    MTLDataTypeShort4 :: 40;
    MTLDataTypeUShort :: 41;
    MTLDataTypeUShort2 :: 42;
    MTLDataTypeUShort3 :: 43;
    MTLDataTypeUShort4 :: 44;
    MTLDataTypeChar :: 45;
    MTLDataTypeChar2 :: 46;
    MTLDataTypeChar3 :: 47;
    MTLDataTypeChar4 :: 48;
    MTLDataTypeUChar :: 49;
    MTLDataTypeUChar2 :: 50;
    MTLDataTypeUChar3 :: 51;
    MTLDataTypeUChar4 :: 52;
    MTLDataTypeBool :: 53;
    MTLDataTypeBool2 :: 54;
    MTLDataTypeBool3 :: 55;
    MTLDataTypeBool4 :: 56;
    MTLDataTypeTexture :: 58;
    MTLDataTypeSampler :: 59;
    MTLDataTypePointer :: 60;
    MTLDataTypeRenderPipeline :: 78;
    MTLDataTypeIndirectCommandBuffer :: 80;
}

using MTLDepthClipMode :: enum NSUInteger {
    MTLDepthClipModeClip :: 0;
    MTLDepthClipModeClamp :: 1;
}

MTLDepthStencilDescriptor :: struct {
    nsobject: NSObject;
    depthCompareFunction :: (self: *MTLDepthStencilDescriptor) -> MTLCompareFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLCompareFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCompareFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthCompareFunction"));
    }
    setDepthCompareFunction :: (self: *MTLDepthStencilDescriptor, depthCompareFunction: MTLCompareFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthCompareFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthCompareFunction:"), depthCompareFunction);
    }
    isDepthWriteEnabled :: (self: *MTLDepthStencilDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isDepthWriteEnabled"));
    }
    setDepthWriteEnabled :: (self: *MTLDepthStencilDescriptor, depthWriteEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthWriteEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthWriteEnabled:"), depthWriteEnabled);
    }
    frontFaceStencil :: (self: *MTLDepthStencilDescriptor) -> *MTLStencilDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLStencilDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStencilDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("frontFaceStencil"));
    }
    setFrontFaceStencil :: (self: *MTLDepthStencilDescriptor, frontFaceStencil: *MTLStencilDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(frontFaceStencil)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFrontFaceStencil:"), frontFaceStencil);
    }
    backFaceStencil :: (self: *MTLDepthStencilDescriptor) -> *MTLStencilDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLStencilDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStencilDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("backFaceStencil"));
    }
    setBackFaceStencil :: (self: *MTLDepthStencilDescriptor, backFaceStencil: *MTLStencilDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(backFaceStencil)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBackFaceStencil:"), backFaceStencil);
    }
    label :: (self: *MTLDepthStencilDescriptor) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLDepthStencilDescriptor, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
}

MTLDepthStencilState :: struct {
    label :: (self: *MTLDepthStencilState) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    device :: (self: *MTLDepthStencilState) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
}

MTLDevice :: struct {
    newCommandQueue :: (self: *MTLDevice) -> *MTLCommandQueue {
        Send :: #type (receiver: *void, op: *void) -> *MTLCommandQueue #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCommandQueue) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newCommandQueue"));
    }
    newCommandQueueWithMaxCommandBufferCount :: (self: *MTLDevice, maxCommandBufferCount: NSUInteger) -> *MTLCommandQueue {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxCommandBufferCount)) -> *MTLCommandQueue #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLCommandQueue) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newCommandQueueWithMaxCommandBufferCount:"), maxCommandBufferCount);
    }
    heapTextureSizeAndAlignWithDescriptor :: (self: *MTLDevice, desc: *MTLTextureDescriptor) -> MTLSizeAndAlign {
        Send :: #type (receiver: *void, op: *void, a0: type_of(desc)) -> MTLSizeAndAlign #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSizeAndAlign) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("heapTextureSizeAndAlignWithDescriptor:"), desc);
    }
    heapBufferSizeAndAlignWithLength_options :: (self: *MTLDevice, length: NSUInteger, options: MTLResourceOptions) -> MTLSizeAndAlign {
        Send :: #type (receiver: *void, op: *void, a0: type_of(length), a1: type_of(options)) -> MTLSizeAndAlign #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSizeAndAlign) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("heapBufferSizeAndAlignWithLength:options:"), length, options);
    }
    newHeapWithDescriptor :: (self: *MTLDevice, descriptor: *MTLHeapDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newHeapWithDescriptor:"), descriptor);
    }
    newBufferWithLength_options :: (self: *MTLDevice, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {
        Send :: #type (receiver: *void, op: *void, a0: type_of(length), a1: type_of(options)) -> *MTLBuffer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBuffer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newBufferWithLength:options:"), length, options);
    }
    newBufferWithBytes_length_options :: (self: *MTLDevice, pointer: *void, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pointer), a1: type_of(length), a2: type_of(options)) -> *MTLBuffer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBuffer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newBufferWithBytes:length:options:"), pointer, length, options);
    }
    newDepthStencilStateWithDescriptor :: (self: *MTLDevice, descriptor: *MTLDepthStencilDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newDepthStencilStateWithDescriptor:"), descriptor);
    }
    newTextureWithDescriptor :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureWithDescriptor:"), descriptor);
    }
    newTextureWithDescriptor_iosurface_plane :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor, iosurface: IOSurfaceRef, plane: NSUInteger) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(iosurface), a2: type_of(plane)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureWithDescriptor:iosurface:plane:"), descriptor, iosurface, plane);
    }
    newSharedTextureWithDescriptor :: (self: *MTLDevice, descriptor: *MTLTextureDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedTextureWithDescriptor:"), descriptor);
    }
    newSharedTextureWithHandle :: (self: *MTLDevice, sharedHandle: *MTLSharedTextureHandle) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sharedHandle)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedTextureWithHandle:"), sharedHandle);
    }
    newSamplerStateWithDescriptor :: (self: *MTLDevice, descriptor: *MTLSamplerDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSamplerStateWithDescriptor:"), descriptor);
    }
    newDefaultLibrary :: (self: *MTLDevice) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newDefaultLibrary"));
    }
    newDefaultLibraryWithBundle_error :: (self: *MTLDevice, bundle: *NSBundle, error: **NSError) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bundle), a1: type_of(error)) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newDefaultLibraryWithBundle:error:"), bundle, error);
    }
    newLibraryWithFile_error :: (self: *MTLDevice, filepath: *NSString, error: **NSError) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void, a0: type_of(filepath), a1: type_of(error)) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newLibraryWithFile:error:"), filepath, error);
    }
    newLibraryWithURL_error :: (self: *MTLDevice, url: *NSURL, error: **NSError) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void, a0: type_of(url), a1: type_of(error)) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newLibraryWithURL:error:"), url, error);
    }
    newLibraryWithData_error :: (self: *MTLDevice, data: dispatch_data_t, error: **NSError) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void, a0: type_of(data), a1: type_of(error)) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newLibraryWithData:error:"), data, error);
    }
    newLibraryWithSource_options_error :: (self: *MTLDevice, source: *NSString, options: *MTLCompileOptions, error: **NSError) -> *MTLLibrary {
        Send :: #type (receiver: *void, op: *void, a0: type_of(source), a1: type_of(options), a2: type_of(error)) -> *MTLLibrary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLLibrary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newLibraryWithSource:options:error:"), source, options, error);
    }
    newRenderPipelineStateWithDescriptor_error :: (self: *MTLDevice, descriptor: *MTLRenderPipelineDescriptor, error: **NSError) -> *MTLRenderPipelineState {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(error)) -> *MTLRenderPipelineState #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPipelineState) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newRenderPipelineStateWithDescriptor:error:"), descriptor, error);
    }
    newRenderPipelineStateWithDescriptor_options_reflection_error :: (self: *MTLDevice, descriptor: *MTLRenderPipelineDescriptor, options: MTLPipelineOption, reflection: **MTLRenderPipelineReflection, error: **NSError) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(options), a2: type_of(reflection), a3: type_of(error)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newRenderPipelineStateWithDescriptor:options:reflection:error:"), descriptor, options, reflection, error);
    }
    newComputePipelineStateWithFunction_error :: (self: *MTLDevice, computeFunction: id, error: **NSError) -> *MTLComputePipelineState {
        Send :: #type (receiver: *void, op: *void, a0: type_of(computeFunction), a1: type_of(error)) -> *MTLComputePipelineState #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLComputePipelineState) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newComputePipelineStateWithFunction:error:"), computeFunction, error);
    }
    newComputePipelineStateWithFunction_options_reflection_error :: (self: *MTLDevice, computeFunction: id, options: MTLPipelineOption, reflection: **MTLComputePipelineReflection, error: **NSError) -> *MTLComputePipelineState {
        Send :: #type (receiver: *void, op: *void, a0: type_of(computeFunction), a1: type_of(options), a2: type_of(reflection), a3: type_of(error)) -> *MTLComputePipelineState #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLComputePipelineState) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newComputePipelineStateWithFunction:options:reflection:error:"), computeFunction, options, reflection, error);
    }
    newComputePipelineStateWithDescriptor_options_reflection_error :: (self: *MTLDevice, descriptor: *MTLComputePipelineDescriptor, options: MTLPipelineOption, reflection: **MTLComputePipelineReflection, error: **NSError) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(options), a2: type_of(reflection), a3: type_of(error)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newComputePipelineStateWithDescriptor:options:reflection:error:"), descriptor, options, reflection, error);
    }
    newFence :: (self: *MTLDevice) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newFence"));
    }
    supportsFeatureSet :: (self: *MTLDevice, featureSet: MTLFeatureSet) -> BOOL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(featureSet)) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("supportsFeatureSet:"), featureSet);
    }
    supportsTextureSampleCount :: (self: *MTLDevice, sampleCount: NSUInteger) -> BOOL {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampleCount)) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("supportsTextureSampleCount:"), sampleCount);
    }
    minimumLinearTextureAlignmentForPixelFormat :: (self: *MTLDevice, format: MTLPixelFormat) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void, a0: type_of(format)) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("minimumLinearTextureAlignmentForPixelFormat:"), format);
    }
    minimumTextureBufferAlignmentForPixelFormat :: (self: *MTLDevice, format: MTLPixelFormat) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void, a0: type_of(format)) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("minimumTextureBufferAlignmentForPixelFormat:"), format);
    }
    getDefaultSamplePositions_count :: (self: *MTLDevice, positions: *MTLSamplePosition, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(positions), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("getDefaultSamplePositions:count:"), positions, count);
    }
    newArgumentEncoderWithArguments :: (self: *MTLDevice, arguments: *NSArray(*MTLArgumentDescriptor)) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(arguments)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newArgumentEncoderWithArguments:"), arguments);
    }
    newIndirectCommandBufferWithDescriptor_maxCommandCount_options :: (self: *MTLDevice, descriptor: *MTLIndirectCommandBufferDescriptor, maxCount: NSUInteger, options: MTLResourceOptions) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(descriptor), a1: type_of(maxCount), a2: type_of(options)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newIndirectCommandBufferWithDescriptor:maxCommandCount:options:"), descriptor, maxCount, options);
    }
    newEvent :: (self: *MTLDevice) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newEvent"));
    }
    newSharedEvent :: (self: *MTLDevice) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedEvent"));
    }
    newSharedEventWithHandle :: (self: *MTLDevice, sharedEventHandle: *MTLSharedEventHandle) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sharedEventHandle)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedEventWithHandle:"), sharedEventHandle);
    }
    name :: (self: *MTLDevice) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    registryID :: (self: *MTLDevice) -> uint64_t {
        Send :: #type (receiver: *void, op: *void) -> uint64_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint64_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("registryID"));
    }
    maxThreadsPerThreadgroup :: (self: *MTLDevice) -> MTLSize {
        Send :: #type (receiver: *void, op: *void) -> MTLSize #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSize) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxThreadsPerThreadgroup"));
    }
    isLowPower :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isLowPower"));
    }
    isHeadless :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isHeadless"));
    }
    isRemovable :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isRemovable"));
    }
    recommendedMaxWorkingSetSize :: (self: *MTLDevice) -> uint64_t {
        Send :: #type (receiver: *void, op: *void) -> uint64_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint64_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("recommendedMaxWorkingSetSize"));
    }
    isDepth24Stencil8PixelFormatSupported :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isDepth24Stencil8PixelFormatSupported"));
    }
    readWriteTextureSupport :: (self: *MTLDevice) -> MTLReadWriteTextureTier {
        Send :: #type (receiver: *void, op: *void) -> MTLReadWriteTextureTier #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLReadWriteTextureTier) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("readWriteTextureSupport"));
    }
    argumentBuffersSupport :: (self: *MTLDevice) -> MTLArgumentBuffersTier {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentBuffersTier #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentBuffersTier) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("argumentBuffersSupport"));
    }
    areRasterOrderGroupsSupported :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("areRasterOrderGroupsSupported"));
    }
    currentAllocatedSize :: (self: *MTLDevice) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("currentAllocatedSize"));
    }
    maxThreadgroupMemoryLength :: (self: *MTLDevice) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxThreadgroupMemoryLength"));
    }
    maxArgumentBufferSamplerCount :: (self: *MTLDevice) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxArgumentBufferSamplerCount"));
    }
    areProgrammableSamplePositionsSupported :: (self: *MTLDevice) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("areProgrammableSamplePositionsSupported"));
    }
    maxBufferLength :: (self: *MTLDevice) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxBufferLength"));
    }
}

MTLDispatchThreadgroupsIndirectArguments :: struct {
    threadgroupsPerGrid: [3] uint32_t;
}

using MTLDispatchType :: enum NSUInteger {
    MTLDispatchTypeSerial :: 0;
    MTLDispatchTypeConcurrent :: 1;
}

MTLDrawable :: struct {
    present :: (self: *MTLDrawable) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("present"));
    }
    presentAtTime :: (self: *MTLDrawable, presentationTime: CFTimeInterval) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(presentationTime)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("presentAtTime:"), presentationTime);
    }
}

MTLDrawIndexedPrimitivesIndirectArguments :: struct {
    indexCount: uint32_t;
    instanceCount: uint32_t;
    indexStart: uint32_t;
    baseVertex: int32_t;
    baseInstance: uint32_t;
}

MTLDrawPatchIndirectArguments :: struct {
    patchCount: uint32_t;
    instanceCount: uint32_t;
    patchStart: uint32_t;
    baseInstance: uint32_t;
}

MTLDrawPrimitivesIndirectArguments :: struct {
    vertexCount: uint32_t;
    instanceCount: uint32_t;
    vertexStart: uint32_t;
    baseInstance: uint32_t;
}

MTLEvent :: struct {
    device :: (self: *MTLEvent) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    label :: (self: *MTLEvent) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLEvent, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
}

using MTLFeatureSet :: enum NSUInteger {
    MTLFeatureSet_iOS_GPUFamily1_v1 :: 0;
    MTLFeatureSet_iOS_GPUFamily2_v1 :: 1;
    MTLFeatureSet_iOS_GPUFamily1_v2 :: 2;
    MTLFeatureSet_iOS_GPUFamily2_v2 :: 3;
    MTLFeatureSet_iOS_GPUFamily3_v1 :: 4;
    MTLFeatureSet_iOS_GPUFamily1_v3 :: 5;
    MTLFeatureSet_iOS_GPUFamily2_v3 :: 6;
    MTLFeatureSet_iOS_GPUFamily3_v2 :: 7;
    MTLFeatureSet_iOS_GPUFamily1_v4 :: 8;
    MTLFeatureSet_iOS_GPUFamily2_v4 :: 9;
    MTLFeatureSet_iOS_GPUFamily3_v3 :: 10;
    MTLFeatureSet_iOS_GPUFamily4_v1 :: 11;
    MTLFeatureSet_iOS_GPUFamily1_v5 :: 12;
    MTLFeatureSet_iOS_GPUFamily2_v5 :: 13;
    MTLFeatureSet_iOS_GPUFamily3_v4 :: 14;
    MTLFeatureSet_iOS_GPUFamily4_v2 :: 15;
    MTLFeatureSet_macOS_GPUFamily1_v1 :: 10000;
    MTLFeatureSet_OSX_GPUFamily1_v1 :: 10000;
    MTLFeatureSet_macOS_GPUFamily1_v2 :: 10001;
    MTLFeatureSet_OSX_GPUFamily1_v2 :: 10001;
    MTLFeatureSet_macOS_ReadWriteTextureTier2 :: 10002;
    MTLFeatureSet_OSX_ReadWriteTextureTier2 :: 10002;
    MTLFeatureSet_macOS_GPUFamily1_v3 :: 10003;
    MTLFeatureSet_macOS_GPUFamily1_v4 :: 10004;
    MTLFeatureSet_macOS_GPUFamily2_v1 :: 10005;
    MTLFeatureSet_tvOS_GPUFamily1_v1 :: 30000;
    MTLFeatureSet_TVOS_GPUFamily1_v1 :: 30000;
    MTLFeatureSet_tvOS_GPUFamily1_v2 :: 30001;
    MTLFeatureSet_tvOS_GPUFamily1_v3 :: 30002;
    MTLFeatureSet_tvOS_GPUFamily1_v4 :: 30004;
}

MTLFence :: struct {
    device :: (self: *MTLFence) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    label :: (self: *MTLFence) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLFence, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
}

MTLFunction :: struct {
    newArgumentEncoderWithBufferIndex :: (self: *MTLFunction, bufferIndex: NSUInteger) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferIndex)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newArgumentEncoderWithBufferIndex:"), bufferIndex);
    }
    newArgumentEncoderWithBufferIndex_reflection :: (self: *MTLFunction, bufferIndex: NSUInteger, reflection: **MTLArgument) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferIndex), a1: type_of(reflection)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newArgumentEncoderWithBufferIndex:reflection:"), bufferIndex, reflection);
    }
    label :: (self: *MTLFunction) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLFunction, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLFunction) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    functionType :: (self: *MTLFunction) -> MTLFunctionType {
        Send :: #type (receiver: *void, op: *void) -> MTLFunctionType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLFunctionType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("functionType"));
    }
    patchType :: (self: *MTLFunction) -> MTLPatchType {
        Send :: #type (receiver: *void, op: *void) -> MTLPatchType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPatchType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("patchType"));
    }
    patchControlPointCount :: (self: *MTLFunction) -> NSInteger {
        Send :: #type (receiver: *void, op: *void) -> NSInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("patchControlPointCount"));
    }
    vertexAttributes :: (self: *MTLFunction) -> *NSArray(*MTLVertexAttribute) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLVertexAttribute) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLVertexAttribute)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("vertexAttributes"));
    }
    stageInputAttributes :: (self: *MTLFunction) -> *NSArray(*MTLAttribute) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLAttribute) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLAttribute)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stageInputAttributes"));
    }
    name :: (self: *MTLFunction) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    functionConstantsDictionary :: (self: *MTLFunction) -> *NSDictionary {
        Send :: #type (receiver: *void, op: *void) -> *NSDictionary #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSDictionary) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("functionConstantsDictionary"));
    }
}

MTLFunctionConstant :: struct {
    nsobject: NSObject;
    name :: (self: *MTLFunctionConstant) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    type :: (self: *MTLFunctionConstant) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("type"));
    }
    index :: (self: *MTLFunctionConstant) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("index"));
    }
    required :: (self: *MTLFunctionConstant) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("required"));
    }
}

MTLFunctionConstantValues :: struct {
    nsobject: NSObject;
    setConstantValue_type_atIndex :: (self: *MTLFunctionConstantValues, value: *void, type: MTLDataType, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(value), a1: type_of(type), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setConstantValue:type:atIndex:"), value, type, index);
    }
    setConstantValues_type_withRange :: (self: *MTLFunctionConstantValues, values: *void, type: MTLDataType, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(values), a1: type_of(type), a2: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setConstantValues:type:withRange:"), values, type, range);
    }
    setConstantValue_type_withName :: (self: *MTLFunctionConstantValues, value: *void, type: MTLDataType, name: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(value), a1: type_of(type), a2: type_of(name)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setConstantValue:type:withName:"), value, type, name);
    }
    reset :: (self: *MTLFunctionConstantValues) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
}

using MTLFunctionType :: enum NSUInteger {
    MTLFunctionTypeVertex :: 1;
    MTLFunctionTypeFragment :: 2;
    MTLFunctionTypeKernel :: 3;
}

MTLHeap :: struct {
    maxAvailableSizeWithAlignment :: (self: *MTLHeap, alignment: NSUInteger) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void, a0: type_of(alignment)) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxAvailableSizeWithAlignment:"), alignment);
    }
    newBufferWithLength_options :: (self: *MTLHeap, length: NSUInteger, options: MTLResourceOptions) -> *MTLBuffer {
        Send :: #type (receiver: *void, op: *void, a0: type_of(length), a1: type_of(options)) -> *MTLBuffer #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBuffer) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newBufferWithLength:options:"), length, options);
    }
    newTextureWithDescriptor :: (self: *MTLHeap, desc: *MTLTextureDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(desc)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureWithDescriptor:"), desc);
    }
    setPurgeableState :: (self: *MTLHeap, state: MTLPurgeableState) -> MTLPurgeableState {
        Send :: #type (receiver: *void, op: *void, a0: type_of(state)) -> MTLPurgeableState #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPurgeableState) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("setPurgeableState:"), state);
    }
    label :: (self: *MTLHeap) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLHeap, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLHeap) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    storageMode :: (self: *MTLHeap) -> MTLStorageMode {
        Send :: #type (receiver: *void, op: *void) -> MTLStorageMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStorageMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storageMode"));
    }
    cpuCacheMode :: (self: *MTLHeap) -> MTLCPUCacheMode {
        Send :: #type (receiver: *void, op: *void) -> MTLCPUCacheMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCPUCacheMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("cpuCacheMode"));
    }
    size :: (self: *MTLHeap) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("size"));
    }
    usedSize :: (self: *MTLHeap) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("usedSize"));
    }
    currentAllocatedSize :: (self: *MTLHeap) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("currentAllocatedSize"));
    }
}

MTLHeapDescriptor :: struct {
    nsobject: NSObject;
    size :: (self: *MTLHeapDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("size"));
    }
    setSize :: (self: *MTLHeapDescriptor, size: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(size)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSize:"), size);
    }
    storageMode :: (self: *MTLHeapDescriptor) -> MTLStorageMode {
        Send :: #type (receiver: *void, op: *void) -> MTLStorageMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStorageMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storageMode"));
    }
    setStorageMode :: (self: *MTLHeapDescriptor, storageMode: MTLStorageMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storageMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStorageMode:"), storageMode);
    }
    cpuCacheMode :: (self: *MTLHeapDescriptor) -> MTLCPUCacheMode {
        Send :: #type (receiver: *void, op: *void) -> MTLCPUCacheMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCPUCacheMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("cpuCacheMode"));
    }
    setCpuCacheMode :: (self: *MTLHeapDescriptor, cpuCacheMode: MTLCPUCacheMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(cpuCacheMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setCpuCacheMode:"), cpuCacheMode);
    }
}

using MTLIndexType :: enum NSUInteger {
    MTLIndexTypeUInt16 :: 0;
    MTLIndexTypeUInt32 :: 1;
}

MTLIndirectCommandBuffer :: struct {
    resetWithRange :: (self: *MTLIndirectCommandBuffer, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("resetWithRange:"), range);
    }
    indirectRenderCommandAtIndex :: (self: *MTLIndirectCommandBuffer, commandIndex: NSUInteger) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(commandIndex)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("indirectRenderCommandAtIndex:"), commandIndex);
    }
    size :: (self: *MTLIndirectCommandBuffer) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("size"));
    }
}

MTLIndirectCommandBufferDescriptor :: struct {
    nsobject: NSObject;
    commandTypes :: (self: *MTLIndirectCommandBufferDescriptor) -> MTLIndirectCommandType {
        Send :: #type (receiver: *void, op: *void) -> MTLIndirectCommandType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLIndirectCommandType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("commandTypes"));
    }
    setCommandTypes :: (self: *MTLIndirectCommandBufferDescriptor, commandTypes: MTLIndirectCommandType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(commandTypes)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setCommandTypes:"), commandTypes);
    }
    inheritPipelineState :: (self: *MTLIndirectCommandBufferDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("inheritPipelineState"));
    }
    setInheritPipelineState :: (self: *MTLIndirectCommandBufferDescriptor, inheritPipelineState: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(inheritPipelineState)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setInheritPipelineState:"), inheritPipelineState);
    }
    inheritBuffers :: (self: *MTLIndirectCommandBufferDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("inheritBuffers"));
    }
    setInheritBuffers :: (self: *MTLIndirectCommandBufferDescriptor, inheritBuffers: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(inheritBuffers)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setInheritBuffers:"), inheritBuffers);
    }
    maxVertexBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxVertexBufferBindCount"));
    }
    setMaxVertexBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor, maxVertexBufferBindCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxVertexBufferBindCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaxVertexBufferBindCount:"), maxVertexBufferBindCount);
    }
    maxFragmentBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxFragmentBufferBindCount"));
    }
    setMaxFragmentBufferBindCount :: (self: *MTLIndirectCommandBufferDescriptor, maxFragmentBufferBindCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxFragmentBufferBindCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaxFragmentBufferBindCount:"), maxFragmentBufferBindCount);
    }
}

using MTLIndirectCommandType :: enum NSUInteger {
    MTLIndirectCommandTypeDraw :: 1;
    MTLIndirectCommandTypeDrawIndexed :: 2;
    MTLIndirectCommandTypeDrawPatches :: 4;
    MTLIndirectCommandTypeDrawIndexedPatches :: 8;
}

MTLIndirectRenderCommand :: struct {
    setRenderPipelineState :: (self: *MTLIndirectRenderCommand, pipelineState: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pipelineState)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRenderPipelineState:"), pipelineState);
    }
    setVertexBuffer_offset_atIndex :: (self: *MTLIndirectRenderCommand, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexBuffer:offset:atIndex:"), buffer, offset, index);
    }
    setFragmentBuffer_offset_atIndex :: (self: *MTLIndirectRenderCommand, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentBuffer:offset:atIndex:"), buffer, offset, index);
    }
    drawPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_instanceCount_baseInstance_tessellationFactorBuffer_tessellationFactorBufferOffset_tessellationFactorBufferInstanceStride :: (self: *MTLIndirectRenderCommand, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchStart), a2: type_of(patchCount), a3: type_of(patchIndexBuffer), a4: type_of(patchIndexBufferOffset), a5: type_of(instanceCount), a6: type_of(baseInstance), a7: type_of(buffer), a8: type_of(offset), a9: type_of(instanceStride)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPatches:patchStart:patchCount:patchIndexBuffer:patchIndexBufferOffset:instanceCount:baseInstance:tessellationFactorBuffer:tessellationFactorBufferOffset:tessellationFactorBufferInstanceStride:"), numberOfPatchControlPoints, patchStart, patchCount, patchIndexBuffer, patchIndexBufferOffset, instanceCount, baseInstance, buffer, offset, instanceStride);
    }
    drawIndexedPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_instanceCount_baseInstance_tessellationFactorBuffer_tessellationFactorBufferOffset_tessellationFactorBufferInstanceStride :: (self: *MTLIndirectRenderCommand, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchStart), a2: type_of(patchCount), a3: type_of(patchIndexBuffer), a4: type_of(patchIndexBufferOffset), a5: type_of(controlPointIndexBuffer), a6: type_of(controlPointIndexBufferOffset), a7: type_of(instanceCount), a8: type_of(baseInstance), a9: type_of(buffer), a10: type_of(offset), a11: type_of(instanceStride)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPatches:patchStart:patchCount:patchIndexBuffer:patchIndexBufferOffset:controlPointIndexBuffer:controlPointIndexBufferOffset:instanceCount:baseInstance:tessellationFactorBuffer:tessellationFactorBufferOffset:tessellationFactorBufferInstanceStride:"), numberOfPatchControlPoints, patchStart, patchCount, patchIndexBuffer, patchIndexBufferOffset, controlPointIndexBuffer, controlPointIndexBufferOffset, instanceCount, baseInstance, buffer, offset, instanceStride);
    }
    drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance :: (self: *MTLIndirectRenderCommand, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(vertexStart), a2: type_of(vertexCount), a3: type_of(instanceCount), a4: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPrimitives:vertexStart:vertexCount:instanceCount:baseInstance:"), primitiveType, vertexStart, vertexCount, instanceCount, baseInstance);
    }
    drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance :: (self: *MTLIndirectRenderCommand, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseVertex: NSInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indexCount), a2: type_of(indexType), a3: type_of(indexBuffer), a4: type_of(indexBufferOffset), a5: type_of(instanceCount), a6: type_of(baseVertex), a7: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:baseVertex:baseInstance:"), primitiveType, indexCount, indexType, indexBuffer, indexBufferOffset, instanceCount, baseVertex, baseInstance);
    }
    reset :: (self: *MTLIndirectRenderCommand) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
}

using MTLLanguageVersion :: enum NSUInteger {
    MTLLanguageVersion1_0 :: 65536;
    MTLLanguageVersion1_1 :: 65537;
    MTLLanguageVersion1_2 :: 65538;
    MTLLanguageVersion2_0 :: 131072;
    MTLLanguageVersion2_1 :: 131073;
}

MTLLibrary :: struct {
    newFunctionWithName :: (self: *MTLLibrary, functionName: *NSString) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(functionName)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newFunctionWithName:"), functionName);
    }
    newFunctionWithName_constantValues_error :: (self: *MTLLibrary, name: *NSString, constantValues: *MTLFunctionConstantValues, error: **NSError) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(name), a1: type_of(constantValues), a2: type_of(error)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newFunctionWithName:constantValues:error:"), name, constantValues, error);
    }
    label :: (self: *MTLLibrary) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLLibrary, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLLibrary) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    functionNames :: (self: *MTLLibrary) -> *NSArray(*NSString) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*NSString) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*NSString)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("functionNames"));
    }
}

using MTLLibraryError :: enum NSUInteger {
    MTLLibraryErrorUnsupported :: 1;
    MTLLibraryErrorInternal :: 2;
    MTLLibraryErrorCompileFailure :: 3;
    MTLLibraryErrorCompileWarning :: 4;
    MTLLibraryErrorFunctionNotFound :: 5;
    MTLLibraryErrorFileNotFound :: 6;
}

using MTLLoadAction :: enum NSUInteger {
    MTLLoadActionDontCare :: 0;
    MTLLoadActionLoad :: 1;
    MTLLoadActionClear :: 2;
}

using MTLMultisampleDepthResolveFilter :: enum NSUInteger {
    MTLMultisampleDepthResolveFilterSample0 :: 0;
    MTLMultisampleDepthResolveFilterMin :: 1;
    MTLMultisampleDepthResolveFilterMax :: 2;
}

using MTLMultisampleStencilResolveFilter :: enum NSUInteger {
    MTLMultisampleStencilResolveFilterSample0 :: 0;
    MTLMultisampleStencilResolveFilterDepthResolvedSample :: 1;
}

using MTLMutability :: enum NSUInteger {
    MTLMutabilityDefault :: 0;
    MTLMutabilityMutable :: 1;
    MTLMutabilityImmutable :: 2;
}

MTLOrigin :: struct {
    x: NSUInteger;
    y: NSUInteger;
    z: NSUInteger;
}

MTLParallelRenderCommandEncoder :: struct {
    renderCommandEncoder :: (self: *MTLParallelRenderCommandEncoder) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("renderCommandEncoder"));
    }
    setColorStoreAction_atIndex :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction, colorAttachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction), a1: type_of(colorAttachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setColorStoreAction:atIndex:"), storeAction, colorAttachmentIndex);
    }
    setDepthStoreAction :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStoreAction:"), storeAction);
    }
    setStencilStoreAction :: (self: *MTLParallelRenderCommandEncoder, storeAction: MTLStoreAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilStoreAction:"), storeAction);
    }
    setColorStoreActionOptions_atIndex :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions, colorAttachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions), a1: type_of(colorAttachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setColorStoreActionOptions:atIndex:"), storeActionOptions, colorAttachmentIndex);
    }
    setDepthStoreActionOptions :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStoreActionOptions:"), storeActionOptions);
    }
    setStencilStoreActionOptions :: (self: *MTLParallelRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilStoreActionOptions:"), storeActionOptions);
    }
}

using MTLPatchType :: enum NSUInteger {
    MTLPatchTypeNone :: 0;
    MTLPatchTypeTriangle :: 1;
    MTLPatchTypeQuad :: 2;
}

MTLPipelineBufferDescriptor :: struct {
    nsobject: NSObject;
    mutability :: (self: *MTLPipelineBufferDescriptor) -> MTLMutability {
        Send :: #type (receiver: *void, op: *void) -> MTLMutability #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLMutability) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("mutability"));
    }
    setMutability :: (self: *MTLPipelineBufferDescriptor, mutability: MTLMutability) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(mutability)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMutability:"), mutability);
    }
}

MTLPipelineBufferDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLPipelineBufferDescriptorArray, bufferIndex: NSUInteger) -> *MTLPipelineBufferDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferIndex)) -> *MTLPipelineBufferDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPipelineBufferDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), bufferIndex);
    }
    setObject_atIndexedSubscript :: (self: *MTLPipelineBufferDescriptorArray, buffer: *MTLPipelineBufferDescriptor, bufferIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(bufferIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), buffer, bufferIndex);
    }
}

using MTLPipelineOption :: enum NSUInteger {
    MTLPipelineOptionNone :: 0;
    MTLPipelineOptionArgumentInfo :: 1;
    MTLPipelineOptionBufferTypeInfo :: 2;
}

using MTLPixelFormat :: enum NSUInteger {
    MTLPixelFormatInvalid :: 0;
    MTLPixelFormatA8Unorm :: 1;
    MTLPixelFormatR8Unorm :: 10;
    MTLPixelFormatR8Unorm_sRGB :: 11;
    MTLPixelFormatR8Snorm :: 12;
    MTLPixelFormatR8Uint :: 13;
    MTLPixelFormatR8Sint :: 14;
    MTLPixelFormatR16Unorm :: 20;
    MTLPixelFormatR16Snorm :: 22;
    MTLPixelFormatR16Uint :: 23;
    MTLPixelFormatR16Sint :: 24;
    MTLPixelFormatR16Float :: 25;
    MTLPixelFormatRG8Unorm :: 30;
    MTLPixelFormatRG8Unorm_sRGB :: 31;
    MTLPixelFormatRG8Snorm :: 32;
    MTLPixelFormatRG8Uint :: 33;
    MTLPixelFormatRG8Sint :: 34;
    MTLPixelFormatB5G6R5Unorm :: 40;
    MTLPixelFormatA1BGR5Unorm :: 41;
    MTLPixelFormatABGR4Unorm :: 42;
    MTLPixelFormatBGR5A1Unorm :: 43;
    MTLPixelFormatR32Uint :: 53;
    MTLPixelFormatR32Sint :: 54;
    MTLPixelFormatR32Float :: 55;
    MTLPixelFormatRG16Unorm :: 60;
    MTLPixelFormatRG16Snorm :: 62;
    MTLPixelFormatRG16Uint :: 63;
    MTLPixelFormatRG16Sint :: 64;
    MTLPixelFormatRG16Float :: 65;
    MTLPixelFormatRGBA8Unorm :: 70;
    MTLPixelFormatRGBA8Unorm_sRGB :: 71;
    MTLPixelFormatRGBA8Snorm :: 72;
    MTLPixelFormatRGBA8Uint :: 73;
    MTLPixelFormatRGBA8Sint :: 74;
    MTLPixelFormatBGRA8Unorm :: 80;
    MTLPixelFormatBGRA8Unorm_sRGB :: 81;
    MTLPixelFormatRGB10A2Unorm :: 90;
    MTLPixelFormatRGB10A2Uint :: 91;
    MTLPixelFormatRG11B10Float :: 92;
    MTLPixelFormatRGB9E5Float :: 93;
    MTLPixelFormatBGR10A2Unorm :: 94;
    MTLPixelFormatBGR10_XR :: 554;
    MTLPixelFormatBGR10_XR_sRGB :: 555;
    MTLPixelFormatRG32Uint :: 103;
    MTLPixelFormatRG32Sint :: 104;
    MTLPixelFormatRG32Float :: 105;
    MTLPixelFormatRGBA16Unorm :: 110;
    MTLPixelFormatRGBA16Snorm :: 112;
    MTLPixelFormatRGBA16Uint :: 113;
    MTLPixelFormatRGBA16Sint :: 114;
    MTLPixelFormatRGBA16Float :: 115;
    MTLPixelFormatBGRA10_XR :: 552;
    MTLPixelFormatBGRA10_XR_sRGB :: 553;
    MTLPixelFormatRGBA32Uint :: 123;
    MTLPixelFormatRGBA32Sint :: 124;
    MTLPixelFormatRGBA32Float :: 125;
    MTLPixelFormatBC1_RGBA :: 130;
    MTLPixelFormatBC1_RGBA_sRGB :: 131;
    MTLPixelFormatBC2_RGBA :: 132;
    MTLPixelFormatBC2_RGBA_sRGB :: 133;
    MTLPixelFormatBC3_RGBA :: 134;
    MTLPixelFormatBC3_RGBA_sRGB :: 135;
    MTLPixelFormatBC4_RUnorm :: 140;
    MTLPixelFormatBC4_RSnorm :: 141;
    MTLPixelFormatBC5_RGUnorm :: 142;
    MTLPixelFormatBC5_RGSnorm :: 143;
    MTLPixelFormatBC6H_RGBFloat :: 150;
    MTLPixelFormatBC6H_RGBUfloat :: 151;
    MTLPixelFormatBC7_RGBAUnorm :: 152;
    MTLPixelFormatBC7_RGBAUnorm_sRGB :: 153;
    MTLPixelFormatPVRTC_RGB_2BPP :: 160;
    MTLPixelFormatPVRTC_RGB_2BPP_sRGB :: 161;
    MTLPixelFormatPVRTC_RGB_4BPP :: 162;
    MTLPixelFormatPVRTC_RGB_4BPP_sRGB :: 163;
    MTLPixelFormatPVRTC_RGBA_2BPP :: 164;
    MTLPixelFormatPVRTC_RGBA_2BPP_sRGB :: 165;
    MTLPixelFormatPVRTC_RGBA_4BPP :: 166;
    MTLPixelFormatPVRTC_RGBA_4BPP_sRGB :: 167;
    MTLPixelFormatEAC_R11Unorm :: 170;
    MTLPixelFormatEAC_R11Snorm :: 172;
    MTLPixelFormatEAC_RG11Unorm :: 174;
    MTLPixelFormatEAC_RG11Snorm :: 176;
    MTLPixelFormatEAC_RGBA8 :: 178;
    MTLPixelFormatEAC_RGBA8_sRGB :: 179;
    MTLPixelFormatETC2_RGB8 :: 180;
    MTLPixelFormatETC2_RGB8_sRGB :: 181;
    MTLPixelFormatETC2_RGB8A1 :: 182;
    MTLPixelFormatETC2_RGB8A1_sRGB :: 183;
    MTLPixelFormatASTC_4x4_sRGB :: 186;
    MTLPixelFormatASTC_5x4_sRGB :: 187;
    MTLPixelFormatASTC_5x5_sRGB :: 188;
    MTLPixelFormatASTC_6x5_sRGB :: 189;
    MTLPixelFormatASTC_6x6_sRGB :: 190;
    MTLPixelFormatASTC_8x5_sRGB :: 192;
    MTLPixelFormatASTC_8x6_sRGB :: 193;
    MTLPixelFormatASTC_8x8_sRGB :: 194;
    MTLPixelFormatASTC_10x5_sRGB :: 195;
    MTLPixelFormatASTC_10x6_sRGB :: 196;
    MTLPixelFormatASTC_10x8_sRGB :: 197;
    MTLPixelFormatASTC_10x10_sRGB :: 198;
    MTLPixelFormatASTC_12x10_sRGB :: 199;
    MTLPixelFormatASTC_12x12_sRGB :: 200;
    MTLPixelFormatASTC_4x4_LDR :: 204;
    MTLPixelFormatASTC_5x4_LDR :: 205;
    MTLPixelFormatASTC_5x5_LDR :: 206;
    MTLPixelFormatASTC_6x5_LDR :: 207;
    MTLPixelFormatASTC_6x6_LDR :: 208;
    MTLPixelFormatASTC_8x5_LDR :: 210;
    MTLPixelFormatASTC_8x6_LDR :: 211;
    MTLPixelFormatASTC_8x8_LDR :: 212;
    MTLPixelFormatASTC_10x5_LDR :: 213;
    MTLPixelFormatASTC_10x6_LDR :: 214;
    MTLPixelFormatASTC_10x8_LDR :: 215;
    MTLPixelFormatASTC_10x10_LDR :: 216;
    MTLPixelFormatASTC_12x10_LDR :: 217;
    MTLPixelFormatASTC_12x12_LDR :: 218;
    MTLPixelFormatGBGR422 :: 240;
    MTLPixelFormatBGRG422 :: 241;
    MTLPixelFormatDepth16Unorm :: 250;
    MTLPixelFormatDepth32Float :: 252;
    MTLPixelFormatStencil8 :: 253;
    MTLPixelFormatDepth24Unorm_Stencil8 :: 255;
    MTLPixelFormatDepth32Float_Stencil8 :: 260;
    MTLPixelFormatX32_Stencil8 :: 261;
    MTLPixelFormatX24_Stencil8 :: 262;
}

MTLPointerType :: struct {
    mtltype: MTLType;
    elementStructType :: (self: *MTLPointerType) -> *MTLStructType {
        Send :: #type (receiver: *void, op: *void) -> *MTLStructType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStructType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementStructType"));
    }
    elementArrayType :: (self: *MTLPointerType) -> *MTLArrayType {
        Send :: #type (receiver: *void, op: *void) -> *MTLArrayType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLArrayType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementArrayType"));
    }
    elementType :: (self: *MTLPointerType) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementType"));
    }
    access :: (self: *MTLPointerType) -> MTLArgumentAccess {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentAccess #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentAccess) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("access"));
    }
    alignment :: (self: *MTLPointerType) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("alignment"));
    }
    dataSize :: (self: *MTLPointerType) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dataSize"));
    }
    elementIsArgumentBuffer :: (self: *MTLPointerType) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("elementIsArgumentBuffer"));
    }
}

using MTLPrimitiveTopologyClass :: enum NSUInteger {
    MTLPrimitiveTopologyClassUnspecified :: 0;
    MTLPrimitiveTopologyClassPoint :: 1;
    MTLPrimitiveTopologyClassLine :: 2;
    MTLPrimitiveTopologyClassTriangle :: 3;
}

using MTLPrimitiveType :: enum NSUInteger {
    MTLPrimitiveTypePoint :: 0;
    MTLPrimitiveTypeLine :: 1;
    MTLPrimitiveTypeLineStrip :: 2;
    MTLPrimitiveTypeTriangle :: 3;
    MTLPrimitiveTypeTriangleStrip :: 4;
}

using MTLPurgeableState :: enum NSUInteger {
    MTLPurgeableStateKeepCurrent :: 1;
    MTLPurgeableStateNonVolatile :: 2;
    MTLPurgeableStateVolatile :: 3;
    MTLPurgeableStateEmpty :: 4;
}

MTLQuadTessellationFactorsHalf :: struct {
    edgeTessellationFactor: [4] uint16_t;
    insideTessellationFactor: [2] uint16_t;
}

using MTLReadWriteTextureTier :: enum NSUInteger {
    MTLReadWriteTextureTierNone :: 0;
    MTLReadWriteTextureTier1 :: 1;
    MTLReadWriteTextureTier2 :: 2;
}

MTLRegion :: struct {
    origin: MTLOrigin;
    size: MTLSize;
}

MTLRenderCommandEncoder :: struct {
    using #as mtlcommandencder: MTLCommandEncoder;
    setRenderPipelineState :: (self: *MTLRenderCommandEncoder, pipelineState: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pipelineState)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRenderPipelineState:"), pipelineState);
    }
    setVertexBytes_length_atIndex :: (self: *MTLRenderCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bytes), a1: type_of(length), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexBytes:length:atIndex:"), bytes, length, index);
    }
    setVertexBuffer_offset_atIndex :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexBuffer:offset:atIndex:"), buffer, offset, index);
    }
    setVertexBufferOffset_atIndex :: (self: *MTLRenderCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(offset), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexBufferOffset:atIndex:"), offset, index);
    }
    setVertexBuffers_offsets_withRange :: (self: *MTLRenderCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffers), a1: type_of(offsets), a2: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexBuffers:offsets:withRange:"), buffers, offsets, range);
    }
    setVertexTexture_atIndex :: (self: *MTLRenderCommandEncoder, texture: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexTexture:atIndex:"), texture, index);
    }
    setVertexTextures_withRange :: (self: *MTLRenderCommandEncoder, textures: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textures), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexTextures:withRange:"), textures, range);
    }
    setVertexSamplerState_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexSamplerState:atIndex:"), sampler, index);
    }
    setVertexSamplerStates_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexSamplerStates:withRange:"), samplers, range);
    }
    setVertexSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(lodMinClamp), a2: type_of(lodMaxClamp), a3: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexSamplerState:lodMinClamp:lodMaxClamp:atIndex:"), sampler, lodMinClamp, lodMaxClamp, index);
    }
    setVertexSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(lodMinClamps), a2: type_of(lodMaxClamps), a3: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexSamplerStates:lodMinClamps:lodMaxClamps:withRange:"), samplers, lodMinClamps, lodMaxClamps, range);
    }
    setViewport :: (self: *MTLRenderCommandEncoder, viewport: MTLViewport) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(viewport)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setViewport:"), viewport);
    }
    setViewports_count :: (self: *MTLRenderCommandEncoder, viewports: *MTLViewport, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(viewports), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setViewports:count:"), viewports, count);
    }
    setFrontFacingWinding :: (self: *MTLRenderCommandEncoder, frontFacingWinding: MTLWinding) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(frontFacingWinding)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFrontFacingWinding:"), frontFacingWinding);
    }
    setCullMode :: (self: *MTLRenderCommandEncoder, cullMode: MTLCullMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(cullMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setCullMode:"), cullMode);
    }
    setDepthClipMode :: (self: *MTLRenderCommandEncoder, depthClipMode: MTLDepthClipMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthClipMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthClipMode:"), depthClipMode);
    }
    setDepthBias_slopeScale_clamp :: (self: *MTLRenderCommandEncoder, depthBias: float32, slopeScale: float32, clamp: float32) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthBias), a1: type_of(slopeScale), a2: type_of(clamp)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthBias:slopeScale:clamp:"), depthBias, slopeScale, clamp);
    }
    setScissorRect :: (self: *MTLRenderCommandEncoder, rect: MTLScissorRect) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(rect)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setScissorRect:"), rect);
    }
    setScissorRects_count :: (self: *MTLRenderCommandEncoder, scissorRects: *MTLScissorRect, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(scissorRects), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setScissorRects:count:"), scissorRects, count);
    }
    setTriangleFillMode :: (self: *MTLRenderCommandEncoder, fillMode: MTLTriangleFillMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fillMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTriangleFillMode:"), fillMode);
    }
    setFragmentBytes_length_atIndex :: (self: *MTLRenderCommandEncoder, bytes: *void, length: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bytes), a1: type_of(length), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentBytes:length:atIndex:"), bytes, length, index);
    }
    setFragmentBuffer_offset_atIndex :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentBuffer:offset:atIndex:"), buffer, offset, index);
    }
    setFragmentBufferOffset_atIndex :: (self: *MTLRenderCommandEncoder, offset: NSUInteger, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(offset), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentBufferOffset:atIndex:"), offset, index);
    }
    setFragmentBuffers_offsets_withRange :: (self: *MTLRenderCommandEncoder, buffers: *id, offsets: *NSUInteger, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffers), a1: type_of(offsets), a2: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentBuffers:offsets:withRange:"), buffers, offsets, range);
    }
    setFragmentTexture_atIndex :: (self: *MTLRenderCommandEncoder, texture: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentTexture:atIndex:"), texture, index);
    }
    setFragmentTextures_withRange :: (self: *MTLRenderCommandEncoder, textures: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textures), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentTextures:withRange:"), textures, range);
    }
    setFragmentSamplerState_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentSamplerState:atIndex:"), sampler, index);
    }
    setFragmentSamplerStates_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentSamplerStates:withRange:"), samplers, range);
    }
    setFragmentSamplerState_lodMinClamp_lodMaxClamp_atIndex :: (self: *MTLRenderCommandEncoder, sampler: id, lodMinClamp: float32, lodMaxClamp: float32, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampler), a1: type_of(lodMinClamp), a2: type_of(lodMaxClamp), a3: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentSamplerState:lodMinClamp:lodMaxClamp:atIndex:"), sampler, lodMinClamp, lodMaxClamp, index);
    }
    setFragmentSamplerStates_lodMinClamps_lodMaxClamps_withRange :: (self: *MTLRenderCommandEncoder, samplers: *id, lodMinClamps: *float32, lodMaxClamps: *float32, range: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(samplers), a1: type_of(lodMinClamps), a2: type_of(lodMaxClamps), a3: type_of(range)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentSamplerStates:lodMinClamps:lodMaxClamps:withRange:"), samplers, lodMinClamps, lodMaxClamps, range);
    }
    setBlendColorRed_green_blue_alpha :: (self: *MTLRenderCommandEncoder, red: float32, green: float32, blue: float32, alpha: float32) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(red), a1: type_of(green), a2: type_of(blue), a3: type_of(alpha)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBlendColorRed:green:blue:alpha:"), red, green, blue, alpha);
    }
    setDepthStencilState :: (self: *MTLRenderCommandEncoder, depthStencilState: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthStencilState)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStencilState:"), depthStencilState);
    }
    setStencilReferenceValue :: (self: *MTLRenderCommandEncoder, referenceValue: uint32_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(referenceValue)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilReferenceValue:"), referenceValue);
    }
    setStencilFrontReferenceValue_backReferenceValue :: (self: *MTLRenderCommandEncoder, frontReferenceValue: uint32_t, backReferenceValue: uint32_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(frontReferenceValue), a1: type_of(backReferenceValue)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilFrontReferenceValue:backReferenceValue:"), frontReferenceValue, backReferenceValue);
    }
    setVisibilityResultMode_offset :: (self: *MTLRenderCommandEncoder, mode: MTLVisibilityResultMode, offset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(mode), a1: type_of(offset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVisibilityResultMode:offset:"), mode, offset);
    }
    setColorStoreAction_atIndex :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction, colorAttachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction), a1: type_of(colorAttachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setColorStoreAction:atIndex:"), storeAction, colorAttachmentIndex);
    }
    setDepthStoreAction :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStoreAction:"), storeAction);
    }
    setStencilStoreAction :: (self: *MTLRenderCommandEncoder, storeAction: MTLStoreAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilStoreAction:"), storeAction);
    }
    setColorStoreActionOptions_atIndex :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions, colorAttachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions), a1: type_of(colorAttachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setColorStoreActionOptions:atIndex:"), storeActionOptions, colorAttachmentIndex);
    }
    setDepthStoreActionOptions :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStoreActionOptions:"), storeActionOptions);
    }
    setStencilStoreActionOptions :: (self: *MTLRenderCommandEncoder, storeActionOptions: MTLStoreActionOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilStoreActionOptions:"), storeActionOptions);
    }
    drawPrimitives_vertexStart_vertexCount_instanceCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(vertexStart), a2: type_of(vertexCount), a3: type_of(instanceCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPrimitives:vertexStart:vertexCount:instanceCount:"), primitiveType, vertexStart, vertexCount, instanceCount);
    }
    drawPrimitives_vertexStart_vertexCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(vertexStart), a2: type_of(vertexCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPrimitives:vertexStart:vertexCount:"), primitiveType, vertexStart, vertexCount);
    }
    drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indexCount), a2: type_of(indexType), a3: type_of(indexBuffer), a4: type_of(indexBufferOffset), a5: type_of(instanceCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:"), primitiveType, indexCount, indexType, indexBuffer, indexBufferOffset, instanceCount);
    }
    drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indexCount), a2: type_of(indexType), a3: type_of(indexBuffer), a4: type_of(indexBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:"), primitiveType, indexCount, indexType, indexBuffer, indexBufferOffset);
    }
    drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, vertexStart: NSUInteger, vertexCount: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(vertexStart), a2: type_of(vertexCount), a3: type_of(instanceCount), a4: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPrimitives:vertexStart:vertexCount:instanceCount:baseInstance:"), primitiveType, vertexStart, vertexCount, instanceCount, baseInstance);
    }
    drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount_baseVertex_baseInstance :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexCount: NSUInteger, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseVertex: NSInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indexCount), a2: type_of(indexType), a3: type_of(indexBuffer), a4: type_of(indexBufferOffset), a5: type_of(instanceCount), a6: type_of(baseVertex), a7: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPrimitives:indexCount:indexType:indexBuffer:indexBufferOffset:instanceCount:baseVertex:baseInstance:"), primitiveType, indexCount, indexType, indexBuffer, indexBufferOffset, instanceCount, baseVertex, baseInstance);
    }
    drawPrimitives_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indirectBuffer), a2: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPrimitives:indirectBuffer:indirectBufferOffset:"), primitiveType, indirectBuffer, indirectBufferOffset);
    }
    drawIndexedPrimitives_indexType_indexBuffer_indexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, primitiveType: MTLPrimitiveType, indexType: MTLIndexType, indexBuffer: id, indexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(primitiveType), a1: type_of(indexType), a2: type_of(indexBuffer), a3: type_of(indexBufferOffset), a4: type_of(indirectBuffer), a5: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPrimitives:indexType:indexBuffer:indexBufferOffset:indirectBuffer:indirectBufferOffset:"), primitiveType, indexType, indexBuffer, indexBufferOffset, indirectBuffer, indirectBufferOffset);
    }
    textureBarrier :: (self: *MTLRenderCommandEncoder) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("textureBarrier"));
    }
    updateFence_afterStages :: (self: *MTLRenderCommandEncoder, fence: id, stages: MTLRenderStages) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence), a1: type_of(stages)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("updateFence:afterStages:"), fence, stages);
    }
    waitForFence_beforeStages :: (self: *MTLRenderCommandEncoder, fence: id, stages: MTLRenderStages) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fence), a1: type_of(stages)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("waitForFence:beforeStages:"), fence, stages);
    }
    setTessellationFactorBuffer_offset_instanceStride :: (self: *MTLRenderCommandEncoder, buffer: id, offset: NSUInteger, instanceStride: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(buffer), a1: type_of(offset), a2: type_of(instanceStride)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationFactorBuffer:offset:instanceStride:"), buffer, offset, instanceStride);
    }
    setTessellationFactorScale :: (self: *MTLRenderCommandEncoder, scale: float32) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(scale)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationFactorScale:"), scale);
    }
    drawPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchStart), a2: type_of(patchCount), a3: type_of(patchIndexBuffer), a4: type_of(patchIndexBufferOffset), a5: type_of(instanceCount), a6: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPatches:patchStart:patchCount:patchIndexBuffer:patchIndexBufferOffset:instanceCount:baseInstance:"), numberOfPatchControlPoints, patchStart, patchCount, patchIndexBuffer, patchIndexBufferOffset, instanceCount, baseInstance);
    }
    drawPatches_patchIndexBuffer_patchIndexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchIndexBuffer), a2: type_of(patchIndexBufferOffset), a3: type_of(indirectBuffer), a4: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawPatches:patchIndexBuffer:patchIndexBufferOffset:indirectBuffer:indirectBufferOffset:"), numberOfPatchControlPoints, patchIndexBuffer, patchIndexBufferOffset, indirectBuffer, indirectBufferOffset);
    }
    drawIndexedPatches_patchStart_patchCount_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_instanceCount_baseInstance :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchStart: NSUInteger, patchCount: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, instanceCount: NSUInteger, baseInstance: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchStart), a2: type_of(patchCount), a3: type_of(patchIndexBuffer), a4: type_of(patchIndexBufferOffset), a5: type_of(controlPointIndexBuffer), a6: type_of(controlPointIndexBufferOffset), a7: type_of(instanceCount), a8: type_of(baseInstance)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPatches:patchStart:patchCount:patchIndexBuffer:patchIndexBufferOffset:controlPointIndexBuffer:controlPointIndexBufferOffset:instanceCount:baseInstance:"), numberOfPatchControlPoints, patchStart, patchCount, patchIndexBuffer, patchIndexBufferOffset, controlPointIndexBuffer, controlPointIndexBufferOffset, instanceCount, baseInstance);
    }
    drawIndexedPatches_patchIndexBuffer_patchIndexBufferOffset_controlPointIndexBuffer_controlPointIndexBufferOffset_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, numberOfPatchControlPoints: NSUInteger, patchIndexBuffer: id, patchIndexBufferOffset: NSUInteger, controlPointIndexBuffer: id, controlPointIndexBufferOffset: NSUInteger, indirectBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(numberOfPatchControlPoints), a1: type_of(patchIndexBuffer), a2: type_of(patchIndexBufferOffset), a3: type_of(controlPointIndexBuffer), a4: type_of(controlPointIndexBufferOffset), a5: type_of(indirectBuffer), a6: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("drawIndexedPatches:patchIndexBuffer:patchIndexBufferOffset:controlPointIndexBuffer:controlPointIndexBufferOffset:indirectBuffer:indirectBufferOffset:"), numberOfPatchControlPoints, patchIndexBuffer, patchIndexBufferOffset, controlPointIndexBuffer, controlPointIndexBufferOffset, indirectBuffer, indirectBufferOffset);
    }
    useResource_usage :: (self: *MTLRenderCommandEncoder, resource: id, usage: MTLResourceUsage) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resource), a1: type_of(usage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useResource:usage:"), resource, usage);
    }
    useResources_count_usage :: (self: *MTLRenderCommandEncoder, resources: *id, count: NSUInteger, usage: MTLResourceUsage) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resources), a1: type_of(count), a2: type_of(usage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useResources:count:usage:"), resources, count, usage);
    }
    useHeap :: (self: *MTLRenderCommandEncoder, heap: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(heap)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useHeap:"), heap);
    }
    useHeaps_count :: (self: *MTLRenderCommandEncoder, heaps: *id, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(heaps), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("useHeaps:count:"), heaps, count);
    }
    executeCommandsInBuffer_withRange :: (self: *MTLRenderCommandEncoder, indirectCommandBuffer: id, executionRange: NSRange) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectCommandBuffer), a1: type_of(executionRange)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("executeCommandsInBuffer:withRange:"), indirectCommandBuffer, executionRange);
    }
    executeCommandsInBuffer_indirectBuffer_indirectBufferOffset :: (self: *MTLRenderCommandEncoder, indirectCommandbuffer: id, indirectRangeBuffer: id, indirectBufferOffset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indirectCommandbuffer), a1: type_of(indirectRangeBuffer), a2: type_of(indirectBufferOffset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("executeCommandsInBuffer:indirectBuffer:indirectBufferOffset:"), indirectCommandbuffer, indirectRangeBuffer, indirectBufferOffset);
    }
    memoryBarrierWithScope_afterStages_beforeStages :: (self: *MTLRenderCommandEncoder, scope: MTLBarrierScope, after: MTLRenderStages, before: MTLRenderStages) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(scope), a1: type_of(after), a2: type_of(before)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("memoryBarrierWithScope:afterStages:beforeStages:"), scope, after, before);
    }
    memoryBarrierWithResources_count_afterStages_beforeStages :: (self: *MTLRenderCommandEncoder, resources: *id, count: NSUInteger, after: MTLRenderStages, before: MTLRenderStages) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resources), a1: type_of(count), a2: type_of(after), a3: type_of(before)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("memoryBarrierWithResources:count:afterStages:beforeStages:"), resources, count, after, before);
    }
}

MTLRenderPassAttachmentDescriptor :: struct {
    nsobject: NSObject;
    texture :: (self: *MTLRenderPassAttachmentDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("texture"));
    }
    setTexture :: (self: *MTLRenderPassAttachmentDescriptor, texture: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(texture)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTexture:"), texture);
    }
    level :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("level"));
    }
    setLevel :: (self: *MTLRenderPassAttachmentDescriptor, level: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(level)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLevel:"), level);
    }
    slice :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("slice"));
    }
    setSlice :: (self: *MTLRenderPassAttachmentDescriptor, slice: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(slice)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSlice:"), slice);
    }
    depthPlane :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthPlane"));
    }
    setDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor, depthPlane: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthPlane)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthPlane:"), depthPlane);
    }
    resolveTexture :: (self: *MTLRenderPassAttachmentDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resolveTexture"));
    }
    setResolveTexture :: (self: *MTLRenderPassAttachmentDescriptor, resolveTexture: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resolveTexture)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setResolveTexture:"), resolveTexture);
    }
    resolveLevel :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resolveLevel"));
    }
    setResolveLevel :: (self: *MTLRenderPassAttachmentDescriptor, resolveLevel: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resolveLevel)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setResolveLevel:"), resolveLevel);
    }
    resolveSlice :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resolveSlice"));
    }
    setResolveSlice :: (self: *MTLRenderPassAttachmentDescriptor, resolveSlice: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resolveSlice)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setResolveSlice:"), resolveSlice);
    }
    resolveDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resolveDepthPlane"));
    }
    setResolveDepthPlane :: (self: *MTLRenderPassAttachmentDescriptor, resolveDepthPlane: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resolveDepthPlane)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setResolveDepthPlane:"), resolveDepthPlane);
    }
    loadAction :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLLoadAction {
        Send :: #type (receiver: *void, op: *void) -> MTLLoadAction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLLoadAction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("loadAction"));
    }
    setLoadAction :: (self: *MTLRenderPassAttachmentDescriptor, loadAction: MTLLoadAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(loadAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLoadAction:"), loadAction);
    }
    storeAction :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLStoreAction {
        Send :: #type (receiver: *void, op: *void) -> MTLStoreAction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStoreAction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storeAction"));
    }
    setStoreAction :: (self: *MTLRenderPassAttachmentDescriptor, storeAction: MTLStoreAction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeAction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStoreAction:"), storeAction);
    }
    storeActionOptions :: (self: *MTLRenderPassAttachmentDescriptor) -> MTLStoreActionOptions {
        Send :: #type (receiver: *void, op: *void) -> MTLStoreActionOptions #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStoreActionOptions) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storeActionOptions"));
    }
    setStoreActionOptions :: (self: *MTLRenderPassAttachmentDescriptor, storeActionOptions: MTLStoreActionOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storeActionOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStoreActionOptions:"), storeActionOptions);
    }
}

MTLRenderPassColorAttachmentDescriptor :: struct {
    using #as mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor;
    clearColor :: (self: *MTLRenderPassColorAttachmentDescriptor) -> MTLClearColor {
        Send :: #type (receiver: *void, op: *void) -> MTLClearColor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLClearColor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("clearColor"));
    }
    setClearColor :: (self: *MTLRenderPassColorAttachmentDescriptor, clearColor: MTLClearColor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(clearColor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setClearColor:"), clearColor);
    }
}

MTLRenderPassColorAttachmentDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLRenderPassColorAttachmentDescriptorArray, attachmentIndex: NSUInteger) -> *MTLRenderPassColorAttachmentDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attachmentIndex)) -> *MTLRenderPassColorAttachmentDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPassColorAttachmentDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), attachmentIndex);
    }
    setObject_atIndexedSubscript :: (self: *MTLRenderPassColorAttachmentDescriptorArray, attachment: *MTLRenderPassColorAttachmentDescriptor, attachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attachment), a1: type_of(attachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), attachment, attachmentIndex);
    }
}

MTLRenderPassDepthAttachmentDescriptor :: struct {
    mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor;
    clearDepth :: (self: *MTLRenderPassDepthAttachmentDescriptor) -> float64 {
        Send :: #type (receiver: *void, op: *void) -> float64 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(float64) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("clearDepth"));
    }
    setClearDepth :: (self: *MTLRenderPassDepthAttachmentDescriptor, clearDepth: float64) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(clearDepth)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setClearDepth:"), clearDepth);
    }
    depthResolveFilter :: (self: *MTLRenderPassDepthAttachmentDescriptor) -> MTLMultisampleDepthResolveFilter {
        Send :: #type (receiver: *void, op: *void) -> MTLMultisampleDepthResolveFilter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLMultisampleDepthResolveFilter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthResolveFilter"));
    }
    setDepthResolveFilter :: (self: *MTLRenderPassDepthAttachmentDescriptor, depthResolveFilter: MTLMultisampleDepthResolveFilter) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthResolveFilter)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthResolveFilter:"), depthResolveFilter);
    }
}

MTLRenderPassDescriptor :: struct {
    nsobject: NSObject;
    renderPassDescriptor :: () -> *MTLRenderPassDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLRenderPassDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPassDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLRenderPassDescriptor"), binding_objc_selector("renderPassDescriptor"));
    }
    setSamplePositions_count :: (self: *MTLRenderPassDescriptor, positions: *MTLSamplePosition, count: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(positions), a1: type_of(count)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSamplePositions:count:"), positions, count);
    }
    getSamplePositions_count :: (self: *MTLRenderPassDescriptor, positions: *MTLSamplePosition, count: NSUInteger) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void, a0: type_of(positions), a1: type_of(count)) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("getSamplePositions:count:"), positions, count);
    }
    colorAttachments :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassColorAttachmentDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLRenderPassColorAttachmentDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPassColorAttachmentDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("colorAttachments"));
    }
    depthAttachment :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassDepthAttachmentDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLRenderPassDepthAttachmentDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPassDepthAttachmentDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthAttachment"));
    }
    setDepthAttachment :: (self: *MTLRenderPassDescriptor, depthAttachment: *MTLRenderPassDepthAttachmentDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthAttachment)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthAttachment:"), depthAttachment);
    }
    stencilAttachment :: (self: *MTLRenderPassDescriptor) -> *MTLRenderPassStencilAttachmentDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLRenderPassStencilAttachmentDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPassStencilAttachmentDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stencilAttachment"));
    }
    setStencilAttachment :: (self: *MTLRenderPassDescriptor, stencilAttachment: *MTLRenderPassStencilAttachmentDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stencilAttachment)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilAttachment:"), stencilAttachment);
    }
    visibilityResultBuffer :: (self: *MTLRenderPassDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("visibilityResultBuffer"));
    }
    setVisibilityResultBuffer :: (self: *MTLRenderPassDescriptor, visibilityResultBuffer: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(visibilityResultBuffer)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVisibilityResultBuffer:"), visibilityResultBuffer);
    }
    renderTargetArrayLength :: (self: *MTLRenderPassDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("renderTargetArrayLength"));
    }
    setRenderTargetArrayLength :: (self: *MTLRenderPassDescriptor, renderTargetArrayLength: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(renderTargetArrayLength)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRenderTargetArrayLength:"), renderTargetArrayLength);
    }
}

MTLRenderPassStencilAttachmentDescriptor :: struct {
    mtlrenderpassattachmentdescriptor: MTLRenderPassAttachmentDescriptor;
    clearStencil :: (self: *MTLRenderPassStencilAttachmentDescriptor) -> uint32_t {
        Send :: #type (receiver: *void, op: *void) -> uint32_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint32_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("clearStencil"));
    }
    setClearStencil :: (self: *MTLRenderPassStencilAttachmentDescriptor, clearStencil: uint32_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(clearStencil)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setClearStencil:"), clearStencil);
    }
    stencilResolveFilter :: (self: *MTLRenderPassStencilAttachmentDescriptor) -> MTLMultisampleStencilResolveFilter {
        Send :: #type (receiver: *void, op: *void) -> MTLMultisampleStencilResolveFilter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLMultisampleStencilResolveFilter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stencilResolveFilter"));
    }
    setStencilResolveFilter :: (self: *MTLRenderPassStencilAttachmentDescriptor, stencilResolveFilter: MTLMultisampleStencilResolveFilter) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stencilResolveFilter)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilResolveFilter:"), stencilResolveFilter);
    }
}

MTLRenderPipelineColorAttachmentDescriptor :: struct {
    nsobject: NSObject;
    pixelFormat :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("pixelFormat"));
    }
    setPixelFormat :: (self: *MTLRenderPipelineColorAttachmentDescriptor, pixelFormat: MTLPixelFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setPixelFormat:"), pixelFormat);
    }
    isBlendingEnabled :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isBlendingEnabled"));
    }
    setBlendingEnabled :: (self: *MTLRenderPipelineColorAttachmentDescriptor, blendingEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(blendingEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBlendingEnabled:"), blendingEnabled);
    }
    sourceRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendFactor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendFactor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sourceRGBBlendFactor"));
    }
    setSourceRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, sourceRGBBlendFactor: MTLBlendFactor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceRGBBlendFactor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSourceRGBBlendFactor:"), sourceRGBBlendFactor);
    }
    destinationRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendFactor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendFactor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("destinationRGBBlendFactor"));
    }
    setDestinationRGBBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, destinationRGBBlendFactor: MTLBlendFactor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(destinationRGBBlendFactor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDestinationRGBBlendFactor:"), destinationRGBBlendFactor);
    }
    rgbBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendOperation {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendOperation #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendOperation) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("rgbBlendOperation"));
    }
    setRgbBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor, rgbBlendOperation: MTLBlendOperation) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(rgbBlendOperation)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRgbBlendOperation:"), rgbBlendOperation);
    }
    sourceAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendFactor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendFactor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sourceAlphaBlendFactor"));
    }
    setSourceAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, sourceAlphaBlendFactor: MTLBlendFactor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sourceAlphaBlendFactor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSourceAlphaBlendFactor:"), sourceAlphaBlendFactor);
    }
    destinationAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendFactor {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendFactor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendFactor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("destinationAlphaBlendFactor"));
    }
    setDestinationAlphaBlendFactor :: (self: *MTLRenderPipelineColorAttachmentDescriptor, destinationAlphaBlendFactor: MTLBlendFactor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(destinationAlphaBlendFactor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDestinationAlphaBlendFactor:"), destinationAlphaBlendFactor);
    }
    alphaBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLBlendOperation {
        Send :: #type (receiver: *void, op: *void) -> MTLBlendOperation #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLBlendOperation) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("alphaBlendOperation"));
    }
    setAlphaBlendOperation :: (self: *MTLRenderPipelineColorAttachmentDescriptor, alphaBlendOperation: MTLBlendOperation) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(alphaBlendOperation)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAlphaBlendOperation:"), alphaBlendOperation);
    }
    writeMask :: (self: *MTLRenderPipelineColorAttachmentDescriptor) -> MTLColorWriteMask {
        Send :: #type (receiver: *void, op: *void) -> MTLColorWriteMask #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLColorWriteMask) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("writeMask"));
    }
    setWriteMask :: (self: *MTLRenderPipelineColorAttachmentDescriptor, writeMask: MTLColorWriteMask) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(writeMask)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setWriteMask:"), writeMask);
    }
}

MTLRenderPipelineColorAttachmentDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLRenderPipelineColorAttachmentDescriptorArray, attachmentIndex: NSUInteger) -> *MTLRenderPipelineColorAttachmentDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attachmentIndex)) -> *MTLRenderPipelineColorAttachmentDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPipelineColorAttachmentDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), attachmentIndex);
    }
    setObject_atIndexedSubscript :: (self: *MTLRenderPipelineColorAttachmentDescriptorArray, attachment: *MTLRenderPipelineColorAttachmentDescriptor, attachmentIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attachment), a1: type_of(attachmentIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), attachment, attachmentIndex);
    }
}

MTLRenderPipelineDescriptor :: struct {
    nsobject: NSObject;
    reset :: (self: *MTLRenderPipelineDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
    label :: (self: *MTLRenderPipelineDescriptor) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLRenderPipelineDescriptor, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    vertexFunction :: (self: *MTLRenderPipelineDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("vertexFunction"));
    }
    setVertexFunction :: (self: *MTLRenderPipelineDescriptor, vertexFunction: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(vertexFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexFunction:"), vertexFunction);
    }
    fragmentFunction :: (self: *MTLRenderPipelineDescriptor) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("fragmentFunction"));
    }
    setFragmentFunction :: (self: *MTLRenderPipelineDescriptor, fragmentFunction: id) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(fragmentFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFragmentFunction:"), fragmentFunction);
    }
    vertexDescriptor :: (self: *MTLRenderPipelineDescriptor) -> *MTLVertexDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLVertexDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("vertexDescriptor"));
    }
    setVertexDescriptor :: (self: *MTLRenderPipelineDescriptor, vertexDescriptor: *MTLVertexDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(vertexDescriptor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setVertexDescriptor:"), vertexDescriptor);
    }
    sampleCount :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sampleCount"));
    }
    setSampleCount :: (self: *MTLRenderPipelineDescriptor, sampleCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampleCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSampleCount:"), sampleCount);
    }
    rasterSampleCount :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("rasterSampleCount"));
    }
    setRasterSampleCount :: (self: *MTLRenderPipelineDescriptor, rasterSampleCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(rasterSampleCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRasterSampleCount:"), rasterSampleCount);
    }
    isAlphaToCoverageEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isAlphaToCoverageEnabled"));
    }
    setAlphaToCoverageEnabled :: (self: *MTLRenderPipelineDescriptor, alphaToCoverageEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(alphaToCoverageEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAlphaToCoverageEnabled:"), alphaToCoverageEnabled);
    }
    isAlphaToOneEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isAlphaToOneEnabled"));
    }
    setAlphaToOneEnabled :: (self: *MTLRenderPipelineDescriptor, alphaToOneEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(alphaToOneEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAlphaToOneEnabled:"), alphaToOneEnabled);
    }
    isRasterizationEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isRasterizationEnabled"));
    }
    setRasterizationEnabled :: (self: *MTLRenderPipelineDescriptor, rasterizationEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(rasterizationEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRasterizationEnabled:"), rasterizationEnabled);
    }
    colorAttachments :: (self: *MTLRenderPipelineDescriptor) -> *MTLRenderPipelineColorAttachmentDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLRenderPipelineColorAttachmentDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLRenderPipelineColorAttachmentDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("colorAttachments"));
    }
    depthAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthAttachmentPixelFormat"));
    }
    setDepthAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor, depthAttachmentPixelFormat: MTLPixelFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthAttachmentPixelFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthAttachmentPixelFormat:"), depthAttachmentPixelFormat);
    }
    stencilAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stencilAttachmentPixelFormat"));
    }
    setStencilAttachmentPixelFormat :: (self: *MTLRenderPipelineDescriptor, stencilAttachmentPixelFormat: MTLPixelFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stencilAttachmentPixelFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilAttachmentPixelFormat:"), stencilAttachmentPixelFormat);
    }
    inputPrimitiveTopology :: (self: *MTLRenderPipelineDescriptor) -> MTLPrimitiveTopologyClass {
        Send :: #type (receiver: *void, op: *void) -> MTLPrimitiveTopologyClass #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPrimitiveTopologyClass) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("inputPrimitiveTopology"));
    }
    setInputPrimitiveTopology :: (self: *MTLRenderPipelineDescriptor, inputPrimitiveTopology: MTLPrimitiveTopologyClass) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(inputPrimitiveTopology)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setInputPrimitiveTopology:"), inputPrimitiveTopology);
    }
    tessellationPartitionMode :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationPartitionMode {
        Send :: #type (receiver: *void, op: *void) -> MTLTessellationPartitionMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTessellationPartitionMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tessellationPartitionMode"));
    }
    setTessellationPartitionMode :: (self: *MTLRenderPipelineDescriptor, tessellationPartitionMode: MTLTessellationPartitionMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationPartitionMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationPartitionMode:"), tessellationPartitionMode);
    }
    maxTessellationFactor :: (self: *MTLRenderPipelineDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxTessellationFactor"));
    }
    setMaxTessellationFactor :: (self: *MTLRenderPipelineDescriptor, maxTessellationFactor: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxTessellationFactor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaxTessellationFactor:"), maxTessellationFactor);
    }
    isTessellationFactorScaleEnabled :: (self: *MTLRenderPipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isTessellationFactorScaleEnabled"));
    }
    setTessellationFactorScaleEnabled :: (self: *MTLRenderPipelineDescriptor, tessellationFactorScaleEnabled: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationFactorScaleEnabled)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationFactorScaleEnabled:"), tessellationFactorScaleEnabled);
    }
    tessellationFactorFormat :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationFactorFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLTessellationFactorFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTessellationFactorFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tessellationFactorFormat"));
    }
    setTessellationFactorFormat :: (self: *MTLRenderPipelineDescriptor, tessellationFactorFormat: MTLTessellationFactorFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationFactorFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationFactorFormat:"), tessellationFactorFormat);
    }
    tessellationControlPointIndexType :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationControlPointIndexType {
        Send :: #type (receiver: *void, op: *void) -> MTLTessellationControlPointIndexType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTessellationControlPointIndexType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tessellationControlPointIndexType"));
    }
    setTessellationControlPointIndexType :: (self: *MTLRenderPipelineDescriptor, tessellationControlPointIndexType: MTLTessellationControlPointIndexType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationControlPointIndexType)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationControlPointIndexType:"), tessellationControlPointIndexType);
    }
    tessellationFactorStepFunction :: (self: *MTLRenderPipelineDescriptor) -> MTLTessellationFactorStepFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLTessellationFactorStepFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTessellationFactorStepFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tessellationFactorStepFunction"));
    }
    setTessellationFactorStepFunction :: (self: *MTLRenderPipelineDescriptor, tessellationFactorStepFunction: MTLTessellationFactorStepFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationFactorStepFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationFactorStepFunction:"), tessellationFactorStepFunction);
    }
    tessellationOutputWindingOrder :: (self: *MTLRenderPipelineDescriptor) -> MTLWinding {
        Send :: #type (receiver: *void, op: *void) -> MTLWinding #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLWinding) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tessellationOutputWindingOrder"));
    }
    setTessellationOutputWindingOrder :: (self: *MTLRenderPipelineDescriptor, tessellationOutputWindingOrder: MTLWinding) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tessellationOutputWindingOrder)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTessellationOutputWindingOrder:"), tessellationOutputWindingOrder);
    }
    vertexBuffers :: (self: *MTLRenderPipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLPipelineBufferDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPipelineBufferDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("vertexBuffers"));
    }
    fragmentBuffers :: (self: *MTLRenderPipelineDescriptor) -> *MTLPipelineBufferDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLPipelineBufferDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPipelineBufferDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("fragmentBuffers"));
    }
    supportIndirectCommandBuffers :: (self: *MTLRenderPipelineDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("supportIndirectCommandBuffers"));
    }
    setSupportIndirectCommandBuffers :: (self: *MTLRenderPipelineDescriptor, supportIndirectCommandBuffers: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(supportIndirectCommandBuffers)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSupportIndirectCommandBuffers:"), supportIndirectCommandBuffers);
    }
}

MTLRenderPipelineReflection :: struct {
    nsobject: NSObject;
    vertexArguments :: (self: *MTLRenderPipelineReflection) -> *NSArray(*MTLArgument) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLArgument) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLArgument)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("vertexArguments"));
    }
    fragmentArguments :: (self: *MTLRenderPipelineReflection) -> *NSArray(*MTLArgument) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLArgument) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLArgument)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("fragmentArguments"));
    }
}

MTLRenderPipelineState :: struct {
    label :: (self: *MTLRenderPipelineState) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    device :: (self: *MTLRenderPipelineState) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    supportIndirectCommandBuffers :: (self: *MTLRenderPipelineState) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("supportIndirectCommandBuffers"));
    }
}

using MTLRenderStages :: enum NSUInteger {
    MTLRenderStageVertex :: 1;
    MTLRenderStageFragment :: 2;
}

MTLResource :: struct {
    setPurgeableState :: (self: *MTLResource, state: MTLPurgeableState) -> MTLPurgeableState {
        Send :: #type (receiver: *void, op: *void, a0: type_of(state)) -> MTLPurgeableState #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPurgeableState) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("setPurgeableState:"), state);
    }
    makeAliasable :: (self: *MTLResource) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("makeAliasable"));
    }
    isAliasable :: (self: *MTLResource) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isAliasable"));
    }
    label :: (self: *MTLResource) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLResource, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
    device :: (self: *MTLResource) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    cpuCacheMode :: (self: *MTLResource) -> MTLCPUCacheMode {
        Send :: #type (receiver: *void, op: *void) -> MTLCPUCacheMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCPUCacheMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("cpuCacheMode"));
    }
    storageMode :: (self: *MTLResource) -> MTLStorageMode {
        Send :: #type (receiver: *void, op: *void) -> MTLStorageMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStorageMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storageMode"));
    }
    heap :: (self: *MTLResource) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("heap"));
    }
    allocatedSize :: (self: *MTLResource) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("allocatedSize"));
    }
}

using MTLResourceOptions :: enum NSUInteger {
    MTLResourceCPUCacheModeDefaultCache :: 0;
    MTLResourceCPUCacheModeWriteCombined :: 1;
    MTLResourceStorageModeShared :: 0;
    MTLResourceStorageModeManaged :: 16;
    MTLResourceStorageModePrivate :: 32;
    MTLResourceStorageModeMemoryless :: 48;
    MTLResourceHazardTrackingModeUntracked :: 256;
    MTLResourceOptionCPUCacheModeDefault :: 0;
    MTLResourceOptionCPUCacheModeWriteCombined :: 1;
}

using MTLResourceUsage :: enum NSUInteger {
    MTLResourceUsageRead :: 1;
    MTLResourceUsageWrite :: 2;
    MTLResourceUsageSample :: 4;
}

MTLSamplePosition :: struct {
    x: float32;
    y: float32;
}

using MTLSamplerAddressMode :: enum NSUInteger {
    MTLSamplerAddressModeClampToEdge :: 0;
    MTLSamplerAddressModeMirrorClampToEdge :: 1;
    MTLSamplerAddressModeRepeat :: 2;
    MTLSamplerAddressModeMirrorRepeat :: 3;
    MTLSamplerAddressModeClampToZero :: 4;
    MTLSamplerAddressModeClampToBorderColor :: 5;
}

using MTLSamplerBorderColor :: enum NSUInteger {
    MTLSamplerBorderColorTransparentBlack :: 0;
    MTLSamplerBorderColorOpaqueBlack :: 1;
    MTLSamplerBorderColorOpaqueWhite :: 2;
}

MTLSamplerDescriptor :: struct {
    nsobject: NSObject;
    minFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMinMagFilter {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerMinMagFilter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerMinMagFilter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("minFilter"));
    }
    setMinFilter :: (self: *MTLSamplerDescriptor, minFilter: MTLSamplerMinMagFilter) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(minFilter)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMinFilter:"), minFilter);
    }
    magFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMinMagFilter {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerMinMagFilter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerMinMagFilter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("magFilter"));
    }
    setMagFilter :: (self: *MTLSamplerDescriptor, magFilter: MTLSamplerMinMagFilter) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(magFilter)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMagFilter:"), magFilter);
    }
    mipFilter :: (self: *MTLSamplerDescriptor) -> MTLSamplerMipFilter {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerMipFilter #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerMipFilter) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("mipFilter"));
    }
    setMipFilter :: (self: *MTLSamplerDescriptor, mipFilter: MTLSamplerMipFilter) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(mipFilter)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMipFilter:"), mipFilter);
    }
    maxAnisotropy :: (self: *MTLSamplerDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("maxAnisotropy"));
    }
    setMaxAnisotropy :: (self: *MTLSamplerDescriptor, maxAnisotropy: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(maxAnisotropy)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMaxAnisotropy:"), maxAnisotropy);
    }
    sAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerAddressMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerAddressMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sAddressMode"));
    }
    setSAddressMode :: (self: *MTLSamplerDescriptor, sAddressMode: MTLSamplerAddressMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sAddressMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSAddressMode:"), sAddressMode);
    }
    tAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerAddressMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerAddressMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("tAddressMode"));
    }
    setTAddressMode :: (self: *MTLSamplerDescriptor, tAddressMode: MTLSamplerAddressMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(tAddressMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTAddressMode:"), tAddressMode);
    }
    rAddressMode :: (self: *MTLSamplerDescriptor) -> MTLSamplerAddressMode {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerAddressMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerAddressMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("rAddressMode"));
    }
    setRAddressMode :: (self: *MTLSamplerDescriptor, rAddressMode: MTLSamplerAddressMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(rAddressMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setRAddressMode:"), rAddressMode);
    }
    borderColor :: (self: *MTLSamplerDescriptor) -> MTLSamplerBorderColor {
        Send :: #type (receiver: *void, op: *void) -> MTLSamplerBorderColor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLSamplerBorderColor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("borderColor"));
    }
    setBorderColor :: (self: *MTLSamplerDescriptor, borderColor: MTLSamplerBorderColor) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(borderColor)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBorderColor:"), borderColor);
    }
    normalizedCoordinates :: (self: *MTLSamplerDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("normalizedCoordinates"));
    }
    setNormalizedCoordinates :: (self: *MTLSamplerDescriptor, normalizedCoordinates: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(normalizedCoordinates)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setNormalizedCoordinates:"), normalizedCoordinates);
    }
    lodMinClamp :: (self: *MTLSamplerDescriptor) -> float32 {
        Send :: #type (receiver: *void, op: *void) -> float32 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(float32) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("lodMinClamp"));
    }
    setLodMinClamp :: (self: *MTLSamplerDescriptor, lodMinClamp: float32) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(lodMinClamp)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLodMinClamp:"), lodMinClamp);
    }
    lodMaxClamp :: (self: *MTLSamplerDescriptor) -> float32 {
        Send :: #type (receiver: *void, op: *void) -> float32 #c_call;
        send: Send;

        #if CPU == .X64 && size_of(float32) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("lodMaxClamp"));
    }
    setLodMaxClamp :: (self: *MTLSamplerDescriptor, lodMaxClamp: float32) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(lodMaxClamp)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLodMaxClamp:"), lodMaxClamp);
    }
    lodAverage :: (self: *MTLSamplerDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("lodAverage"));
    }
    setLodAverage :: (self: *MTLSamplerDescriptor, lodAverage: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(lodAverage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLodAverage:"), lodAverage);
    }
    compareFunction :: (self: *MTLSamplerDescriptor) -> MTLCompareFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLCompareFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCompareFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("compareFunction"));
    }
    setCompareFunction :: (self: *MTLSamplerDescriptor, compareFunction: MTLCompareFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(compareFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setCompareFunction:"), compareFunction);
    }
    supportArgumentBuffers :: (self: *MTLSamplerDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("supportArgumentBuffers"));
    }
    setSupportArgumentBuffers :: (self: *MTLSamplerDescriptor, supportArgumentBuffers: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(supportArgumentBuffers)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSupportArgumentBuffers:"), supportArgumentBuffers);
    }
    label :: (self: *MTLSamplerDescriptor) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    setLabel :: (self: *MTLSamplerDescriptor, label: *NSString) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(label)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setLabel:"), label);
    }
}

using MTLSamplerMinMagFilter :: enum NSUInteger {
    MTLSamplerMinMagFilterNearest :: 0;
    MTLSamplerMinMagFilterLinear :: 1;
}

using MTLSamplerMipFilter :: enum NSUInteger {
    MTLSamplerMipFilterNotMipmapped :: 0;
    MTLSamplerMipFilterNearest :: 1;
    MTLSamplerMipFilterLinear :: 2;
}

MTLSamplerState :: struct {
    label :: (self: *MTLSamplerState) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
    device :: (self: *MTLSamplerState) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
}

MTLScissorRect :: struct {
    x: NSUInteger;
    y: NSUInteger;
    width: NSUInteger;
    height: NSUInteger;
}

MTLSharedEvent :: struct {
    newSharedEventHandle :: (self: *MTLSharedEvent) -> *MTLSharedEventHandle {
        Send :: #type (receiver: *void, op: *void) -> *MTLSharedEventHandle #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLSharedEventHandle) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedEventHandle"));
    }
    signaledValue :: (self: *MTLSharedEvent) -> uint64_t {
        Send :: #type (receiver: *void, op: *void) -> uint64_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint64_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("signaledValue"));
    }
    setSignaledValue :: (self: *MTLSharedEvent, signaledValue: uint64_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(signaledValue)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSignaledValue:"), signaledValue);
    }
}

MTLSharedEventHandle :: struct {
    nsobject: NSObject;
    _priv: *MTLSharedEventHandlePrivate;
    label :: (self: *MTLSharedEventHandle) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
}

MTLSharedEventHandlePrivate :: struct {}

MTLSharedEventListener :: struct {
    nsobject: NSObject;
    init :: (self: *MTLSharedEventListener) -> *MTLSharedEventListener {
        Send :: #type (receiver: *void, op: *void) -> *MTLSharedEventListener #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLSharedEventListener) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("init"));
    }
    initWithDispatchQueue :: (self: *MTLSharedEventListener, dispatchQueue: dispatch_queue_t) -> *MTLSharedEventListener {
        Send :: #type (receiver: *void, op: *void, a0: type_of(dispatchQueue)) -> *MTLSharedEventListener #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLSharedEventListener) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("initWithDispatchQueue:"), dispatchQueue);
    }
    dispatchQueue :: (self: *MTLSharedEventListener) -> dispatch_queue_t {
        Send :: #type (receiver: *void, op: *void) -> dispatch_queue_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(dispatch_queue_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dispatchQueue"));
    }
}

MTLSharedTextureHandle :: struct {
    nsobject: NSObject;
    _priv: *MTLSharedTextureHandlePrivate;
    device :: (self: *MTLSharedTextureHandle) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("device"));
    }
    label :: (self: *MTLSharedTextureHandle) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("label"));
    }
}

MTLSharedTextureHandlePrivate :: struct {}

MTLSize :: struct {
    width: NSUInteger;
    height: NSUInteger;
    depth: NSUInteger;
}

MTLSizeAndAlign :: struct {
    size: NSUInteger;
    align: NSUInteger;
}

MTLStageInputOutputDescriptor :: struct {
    nsobject: NSObject;
    stageInputOutputDescriptor :: () -> *MTLStageInputOutputDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLStageInputOutputDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStageInputOutputDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLStageInputOutputDescriptor"), binding_objc_selector("stageInputOutputDescriptor"));
    }
    reset :: (self: *MTLStageInputOutputDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
    layouts :: (self: *MTLStageInputOutputDescriptor) -> *MTLBufferLayoutDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLBufferLayoutDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLBufferLayoutDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("layouts"));
    }
    attributes :: (self: *MTLStageInputOutputDescriptor) -> *MTLAttributeDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLAttributeDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLAttributeDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributes"));
    }
    indexType :: (self: *MTLStageInputOutputDescriptor) -> MTLIndexType {
        Send :: #type (receiver: *void, op: *void) -> MTLIndexType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLIndexType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("indexType"));
    }
    setIndexType :: (self: *MTLStageInputOutputDescriptor, indexType: MTLIndexType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indexType)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setIndexType:"), indexType);
    }
    indexBufferIndex :: (self: *MTLStageInputOutputDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("indexBufferIndex"));
    }
    setIndexBufferIndex :: (self: *MTLStageInputOutputDescriptor, indexBufferIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(indexBufferIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setIndexBufferIndex:"), indexBufferIndex);
    }
}

MTLStageInRegionIndirectArguments :: struct {
    stageInOrigin: [3] uint32_t;
    stageInSize: [3] uint32_t;
}

MTLStencilDescriptor :: struct {
    nsobject: NSObject;
    stencilCompareFunction :: (self: *MTLStencilDescriptor) -> MTLCompareFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLCompareFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCompareFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stencilCompareFunction"));
    }
    setStencilCompareFunction :: (self: *MTLStencilDescriptor, stencilCompareFunction: MTLCompareFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stencilCompareFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilCompareFunction:"), stencilCompareFunction);
    }
    stencilFailureOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {
        Send :: #type (receiver: *void, op: *void) -> MTLStencilOperation #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStencilOperation) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stencilFailureOperation"));
    }
    setStencilFailureOperation :: (self: *MTLStencilDescriptor, stencilFailureOperation: MTLStencilOperation) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stencilFailureOperation)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStencilFailureOperation:"), stencilFailureOperation);
    }
    depthFailureOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {
        Send :: #type (receiver: *void, op: *void) -> MTLStencilOperation #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStencilOperation) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthFailureOperation"));
    }
    setDepthFailureOperation :: (self: *MTLStencilDescriptor, depthFailureOperation: MTLStencilOperation) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthFailureOperation)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthFailureOperation:"), depthFailureOperation);
    }
    depthStencilPassOperation :: (self: *MTLStencilDescriptor) -> MTLStencilOperation {
        Send :: #type (receiver: *void, op: *void) -> MTLStencilOperation #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStencilOperation) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depthStencilPassOperation"));
    }
    setDepthStencilPassOperation :: (self: *MTLStencilDescriptor, depthStencilPassOperation: MTLStencilOperation) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depthStencilPassOperation)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepthStencilPassOperation:"), depthStencilPassOperation);
    }
    readMask :: (self: *MTLStencilDescriptor) -> uint32_t {
        Send :: #type (receiver: *void, op: *void) -> uint32_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint32_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("readMask"));
    }
    setReadMask :: (self: *MTLStencilDescriptor, readMask: uint32_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(readMask)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setReadMask:"), readMask);
    }
    writeMask :: (self: *MTLStencilDescriptor) -> uint32_t {
        Send :: #type (receiver: *void, op: *void) -> uint32_t #c_call;
        send: Send;

        #if CPU == .X64 && size_of(uint32_t) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("writeMask"));
    }
    setWriteMask :: (self: *MTLStencilDescriptor, writeMask: uint32_t) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(writeMask)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setWriteMask:"), writeMask);
    }
}

using MTLStencilOperation :: enum NSUInteger {
    MTLStencilOperationKeep :: 0;
    MTLStencilOperationZero :: 1;
    MTLStencilOperationReplace :: 2;
    MTLStencilOperationIncrementClamp :: 3;
    MTLStencilOperationDecrementClamp :: 4;
    MTLStencilOperationInvert :: 5;
    MTLStencilOperationIncrementWrap :: 6;
    MTLStencilOperationDecrementWrap :: 7;
}

using MTLStepFunction :: enum NSUInteger {
    MTLStepFunctionConstant :: 0;
    MTLStepFunctionPerVertex :: 1;
    MTLStepFunctionPerInstance :: 2;
    MTLStepFunctionPerPatch :: 3;
    MTLStepFunctionPerPatchControlPoint :: 4;
    MTLStepFunctionThreadPositionInGridX :: 5;
    MTLStepFunctionThreadPositionInGridY :: 6;
    MTLStepFunctionThreadPositionInGridXIndexed :: 7;
    MTLStepFunctionThreadPositionInGridYIndexed :: 8;
}

using MTLStorageMode :: enum NSUInteger {
    MTLStorageModeShared :: 0;
    MTLStorageModeManaged :: 1;
    MTLStorageModePrivate :: 2;
    MTLStorageModeMemoryless :: 3;
}

using MTLStoreAction :: enum NSUInteger {
    MTLStoreActionDontCare :: 0;
    MTLStoreActionStore :: 1;
    MTLStoreActionMultisampleResolve :: 2;
    MTLStoreActionStoreAndMultisampleResolve :: 3;
    MTLStoreActionUnknown :: 4;
    MTLStoreActionCustomSampleDepthStore :: 5;
}

using MTLStoreActionOptions :: enum NSUInteger {
    MTLStoreActionOptionNone :: 0;
    MTLStoreActionOptionCustomSamplePositions :: 1;
}

MTLStructMember :: struct {
    nsobject: NSObject;
    structType :: (self: *MTLStructMember) -> *MTLStructType {
        Send :: #type (receiver: *void, op: *void) -> *MTLStructType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStructType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("structType"));
    }
    arrayType :: (self: *MTLStructMember) -> *MTLArrayType {
        Send :: #type (receiver: *void, op: *void) -> *MTLArrayType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLArrayType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayType"));
    }
    textureReferenceType :: (self: *MTLStructMember) -> *MTLTextureReferenceType {
        Send :: #type (receiver: *void, op: *void) -> *MTLTextureReferenceType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLTextureReferenceType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureReferenceType"));
    }
    pointerType :: (self: *MTLStructMember) -> *MTLPointerType {
        Send :: #type (receiver: *void, op: *void) -> *MTLPointerType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLPointerType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("pointerType"));
    }
    name :: (self: *MTLStructMember) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    offset :: (self: *MTLStructMember) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("offset"));
    }
    dataType :: (self: *MTLStructMember) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dataType"));
    }
    argumentIndex :: (self: *MTLStructMember) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("argumentIndex"));
    }
}

MTLStructType :: struct {
    mtltype: MTLType;
    memberByName :: (self: *MTLStructType, name: *NSString) -> *MTLStructMember {
        Send :: #type (receiver: *void, op: *void, a0: type_of(name)) -> *MTLStructMember #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLStructMember) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("memberByName:"), name);
    }
    members :: (self: *MTLStructType) -> *NSArray(*MTLStructMember) {
        Send :: #type (receiver: *void, op: *void) -> *NSArray(*MTLStructMember) #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSArray(*MTLStructMember)) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("members"));
    }
}

using MTLTessellationControlPointIndexType :: enum NSUInteger {
    MTLTessellationControlPointIndexTypeNone :: 0;
    MTLTessellationControlPointIndexTypeUInt16 :: 1;
    MTLTessellationControlPointIndexTypeUInt32 :: 2;
}

using MTLTessellationFactorFormat :: enum NSUInteger {
    MTLTessellationFactorFormatHalf :: 0;
}

using MTLTessellationFactorStepFunction :: enum NSUInteger {
    MTLTessellationFactorStepFunctionConstant :: 0;
    MTLTessellationFactorStepFunctionPerPatch :: 1;
    MTLTessellationFactorStepFunctionPerInstance :: 2;
    MTLTessellationFactorStepFunctionPerPatchAndPerInstance :: 3;
}

using MTLTessellationPartitionMode :: enum NSUInteger {
    MTLTessellationPartitionModePow2 :: 0;
    MTLTessellationPartitionModeInteger :: 1;
    MTLTessellationPartitionModeFractionalOdd :: 2;
    MTLTessellationPartitionModeFractionalEven :: 3;
}

MTLTexture :: struct {
    using MTLResource;
    getBytes_bytesPerRow_bytesPerImage_fromRegion_mipmapLevel_slice :: (self: *MTLTexture, pixelBytes: *void, bytesPerRow: NSUInteger, bytesPerImage: NSUInteger, region: MTLRegion, level: NSUInteger, slice: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelBytes), a1: type_of(bytesPerRow), a2: type_of(bytesPerImage), a3: type_of(region), a4: type_of(level), a5: type_of(slice)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("getBytes:bytesPerRow:bytesPerImage:fromRegion:mipmapLevel:slice:"), pixelBytes, bytesPerRow, bytesPerImage, region, level, slice);
    }
    replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage :: (self: *MTLTexture, region: MTLRegion, level: NSUInteger, slice: NSUInteger, pixelBytes: *void, bytesPerRow: NSUInteger, bytesPerImage: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(region), a1: type_of(level), a2: type_of(slice), a3: type_of(pixelBytes), a4: type_of(bytesPerRow), a5: type_of(bytesPerImage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("replaceRegion:mipmapLevel:slice:withBytes:bytesPerRow:bytesPerImage:"), region, level, slice, pixelBytes, bytesPerRow, bytesPerImage);
    }
    getBytes_bytesPerRow_fromRegion_mipmapLevel :: (self: *MTLTexture, pixelBytes: *void, bytesPerRow: NSUInteger, region: MTLRegion, level: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelBytes), a1: type_of(bytesPerRow), a2: type_of(region), a3: type_of(level)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("getBytes:bytesPerRow:fromRegion:mipmapLevel:"), pixelBytes, bytesPerRow, region, level);
    }
    replaceRegion_mipmapLevel_withBytes_bytesPerRow :: (self: *MTLTexture, region: MTLRegion, level: NSUInteger, pixelBytes: *void, bytesPerRow: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(region), a1: type_of(level), a2: type_of(pixelBytes), a3: type_of(bytesPerRow)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("replaceRegion:mipmapLevel:withBytes:bytesPerRow:"), region, level, pixelBytes, bytesPerRow);
    }
    newTextureViewWithPixelFormat :: (self: *MTLTexture, pixelFormat: MTLPixelFormat) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureViewWithPixelFormat:"), pixelFormat);
    }
    newTextureViewWithPixelFormat_textureType_levels_slices :: (self: *MTLTexture, pixelFormat: MTLPixelFormat, textureType: MTLTextureType, levelRange: NSRange, sliceRange: NSRange) -> id {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat), a1: type_of(textureType), a2: type_of(levelRange), a3: type_of(sliceRange)) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newTextureViewWithPixelFormat:textureType:levels:slices:"), pixelFormat, textureType, levelRange, sliceRange);
    }
    newSharedTextureHandle :: (self: *MTLTexture) -> *MTLSharedTextureHandle {
        Send :: #type (receiver: *void, op: *void) -> *MTLSharedTextureHandle #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLSharedTextureHandle) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("newSharedTextureHandle"));
    }
    rootResource :: (self: *MTLTexture) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("rootResource"));
    }
    parentTexture :: (self: *MTLTexture) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("parentTexture"));
    }
    parentRelativeLevel :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("parentRelativeLevel"));
    }
    parentRelativeSlice :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("parentRelativeSlice"));
    }
    buffer :: (self: *MTLTexture) -> id {
        Send :: #type (receiver: *void, op: *void) -> id #c_call;
        send: Send;

        #if CPU == .X64 && size_of(id) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("buffer"));
    }
    bufferOffset :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferOffset"));
    }
    bufferBytesPerRow :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferBytesPerRow"));
    }
    iosurface :: (self: *MTLTexture) -> IOSurfaceRef {
        Send :: #type (receiver: *void, op: *void) -> IOSurfaceRef #c_call;
        send: Send;

        #if CPU == .X64 && size_of(IOSurfaceRef) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("iosurface"));
    }
    iosurfacePlane :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("iosurfacePlane"));
    }
    textureType :: (self: *MTLTexture) -> MTLTextureType {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureType"));
    }
    pixelFormat :: (self: *MTLTexture) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("pixelFormat"));
    }
    width :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("width"));
    }
    height :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("height"));
    }
    depth :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depth"));
    }
    mipmapLevelCount :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("mipmapLevelCount"));
    }
    sampleCount :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sampleCount"));
    }
    arrayLength :: (self: *MTLTexture) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayLength"));
    }
    usage :: (self: *MTLTexture) -> MTLTextureUsage {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureUsage #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureUsage) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("usage"));
    }
    isShareable :: (self: *MTLTexture) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isShareable"));
    }
    isFramebufferOnly :: (self: *MTLTexture) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isFramebufferOnly"));
    }
    allowGPUOptimizedContents :: (self: *MTLTexture) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("allowGPUOptimizedContents"));
    }
}

MTLTextureDescriptor :: struct {
    nsobject: NSObject;
    texture2DDescriptorWithPixelFormat_width_height_mipmapped :: (pixelFormat: MTLPixelFormat, width: NSUInteger, height: NSUInteger, mipmapped: BOOL) -> *MTLTextureDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat), a1: type_of(width), a2: type_of(height), a3: type_of(mipmapped)) -> *MTLTextureDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLTextureDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLTextureDescriptor"), binding_objc_selector("texture2DDescriptorWithPixelFormat:width:height:mipmapped:"), pixelFormat, width, height, mipmapped);
    }
    textureCubeDescriptorWithPixelFormat_size_mipmapped :: (pixelFormat: MTLPixelFormat, size: NSUInteger, mipmapped: BOOL) -> *MTLTextureDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat), a1: type_of(size), a2: type_of(mipmapped)) -> *MTLTextureDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLTextureDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLTextureDescriptor"), binding_objc_selector("textureCubeDescriptorWithPixelFormat:size:mipmapped:"), pixelFormat, size, mipmapped);
    }
    textureBufferDescriptorWithPixelFormat_width_resourceOptions_usage :: (pixelFormat: MTLPixelFormat, width: NSUInteger, resourceOptions: MTLResourceOptions, usage: MTLTextureUsage) -> *MTLTextureDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat), a1: type_of(width), a2: type_of(resourceOptions), a3: type_of(usage)) -> *MTLTextureDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLTextureDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLTextureDescriptor"), binding_objc_selector("textureBufferDescriptorWithPixelFormat:width:resourceOptions:usage:"), pixelFormat, width, resourceOptions, usage);
    }
    textureType :: (self: *MTLTextureDescriptor) -> MTLTextureType {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureType"));
    }
    setTextureType :: (self: *MTLTextureDescriptor, textureType: MTLTextureType) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(textureType)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setTextureType:"), textureType);
    }
    pixelFormat :: (self: *MTLTextureDescriptor) -> MTLPixelFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLPixelFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLPixelFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("pixelFormat"));
    }
    setPixelFormat :: (self: *MTLTextureDescriptor, pixelFormat: MTLPixelFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(pixelFormat)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setPixelFormat:"), pixelFormat);
    }
    width :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("width"));
    }
    setWidth :: (self: *MTLTextureDescriptor, width: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(width)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setWidth:"), width);
    }
    height :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("height"));
    }
    setHeight :: (self: *MTLTextureDescriptor, height: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(height)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setHeight:"), height);
    }
    depth :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("depth"));
    }
    setDepth :: (self: *MTLTextureDescriptor, depth: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(depth)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setDepth:"), depth);
    }
    mipmapLevelCount :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("mipmapLevelCount"));
    }
    setMipmapLevelCount :: (self: *MTLTextureDescriptor, mipmapLevelCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(mipmapLevelCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setMipmapLevelCount:"), mipmapLevelCount);
    }
    sampleCount :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("sampleCount"));
    }
    setSampleCount :: (self: *MTLTextureDescriptor, sampleCount: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(sampleCount)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setSampleCount:"), sampleCount);
    }
    arrayLength :: (self: *MTLTextureDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("arrayLength"));
    }
    setArrayLength :: (self: *MTLTextureDescriptor, arrayLength: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(arrayLength)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setArrayLength:"), arrayLength);
    }
    resourceOptions :: (self: *MTLTextureDescriptor) -> MTLResourceOptions {
        Send :: #type (receiver: *void, op: *void) -> MTLResourceOptions #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLResourceOptions) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("resourceOptions"));
    }
    setResourceOptions :: (self: *MTLTextureDescriptor, resourceOptions: MTLResourceOptions) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(resourceOptions)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setResourceOptions:"), resourceOptions);
    }
    cpuCacheMode :: (self: *MTLTextureDescriptor) -> MTLCPUCacheMode {
        Send :: #type (receiver: *void, op: *void) -> MTLCPUCacheMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLCPUCacheMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("cpuCacheMode"));
    }
    setCpuCacheMode :: (self: *MTLTextureDescriptor, cpuCacheMode: MTLCPUCacheMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(cpuCacheMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setCpuCacheMode:"), cpuCacheMode);
    }
    storageMode :: (self: *MTLTextureDescriptor) -> MTLStorageMode {
        Send :: #type (receiver: *void, op: *void) -> MTLStorageMode #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLStorageMode) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("storageMode"));
    }
    setStorageMode :: (self: *MTLTextureDescriptor, storageMode: MTLStorageMode) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(storageMode)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStorageMode:"), storageMode);
    }
    usage :: (self: *MTLTextureDescriptor) -> MTLTextureUsage {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureUsage #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureUsage) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("usage"));
    }
    setUsage :: (self: *MTLTextureDescriptor, usage: MTLTextureUsage) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(usage)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setUsage:"), usage);
    }
    allowGPUOptimizedContents :: (self: *MTLTextureDescriptor) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("allowGPUOptimizedContents"));
    }
    setAllowGPUOptimizedContents :: (self: *MTLTextureDescriptor, allowGPUOptimizedContents: BOOL) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(allowGPUOptimizedContents)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setAllowGPUOptimizedContents:"), allowGPUOptimizedContents);
    }
}

MTLTextureReferenceType :: struct {
    mtltype: MTLType;
    textureDataType :: (self: *MTLTextureReferenceType) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureDataType"));
    }
    textureType :: (self: *MTLTextureReferenceType) -> MTLTextureType {
        Send :: #type (receiver: *void, op: *void) -> MTLTextureType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLTextureType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("textureType"));
    }
    access :: (self: *MTLTextureReferenceType) -> MTLArgumentAccess {
        Send :: #type (receiver: *void, op: *void) -> MTLArgumentAccess #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLArgumentAccess) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("access"));
    }
    isDepthTexture :: (self: *MTLTextureReferenceType) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isDepthTexture"));
    }
}

using MTLTextureType :: enum NSUInteger {
    MTLTextureType1D :: 0;
    MTLTextureType1DArray :: 1;
    MTLTextureType2D :: 2;
    MTLTextureType2DArray :: 3;
    MTLTextureType2DMultisample :: 4;
    MTLTextureTypeCube :: 5;
    MTLTextureTypeCubeArray :: 6;
    MTLTextureType3D :: 7;
    MTLTextureType2DMultisampleArray :: 8;
    MTLTextureTypeTextureBuffer :: 9;
}

using MTLTextureUsage :: enum NSUInteger {
    MTLTextureUsageUnknown :: 0;
    MTLTextureUsageShaderRead :: 1;
    MTLTextureUsageShaderWrite :: 2;
    MTLTextureUsageRenderTarget :: 4;
    MTLTextureUsagePixelFormatView :: 16;
}

using MTLTriangleFillMode :: enum NSUInteger {
    MTLTriangleFillModeFill :: 0;
    MTLTriangleFillModeLines :: 1;
}

MTLTriangleTessellationFactorsHalf :: struct {
    edgeTessellationFactor: [3] uint16_t;
    insideTessellationFactor: uint16_t;
}

MTLType :: struct {
    nsobject: NSObject;
    dataType :: (self: *MTLType) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("dataType"));
    }
}

MTLVertexAttribute :: struct {
    nsobject: NSObject;
    name :: (self: *MTLVertexAttribute) -> *NSString {
        Send :: #type (receiver: *void, op: *void) -> *NSString #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*NSString) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("name"));
    }
    attributeIndex :: (self: *MTLVertexAttribute) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributeIndex"));
    }
    attributeType :: (self: *MTLVertexAttribute) -> MTLDataType {
        Send :: #type (receiver: *void, op: *void) -> MTLDataType #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLDataType) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributeType"));
    }
    isActive :: (self: *MTLVertexAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isActive"));
    }
    isPatchData :: (self: *MTLVertexAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isPatchData"));
    }
    isPatchControlPointData :: (self: *MTLVertexAttribute) -> BOOL {
        Send :: #type (receiver: *void, op: *void) -> BOOL #c_call;
        send: Send;

        #if CPU == .X64 && size_of(BOOL) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("isPatchControlPointData"));
    }
}

MTLVertexAttributeDescriptor :: struct {
    nsobject: NSObject;
    format :: (self: *MTLVertexAttributeDescriptor) -> MTLVertexFormat {
        Send :: #type (receiver: *void, op: *void) -> MTLVertexFormat #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLVertexFormat) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("format"));
    }
    setFormat :: (self: *MTLVertexAttributeDescriptor, format: MTLVertexFormat) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(format)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setFormat:"), format);
    }
    offset :: (self: *MTLVertexAttributeDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("offset"));
    }
    setOffset :: (self: *MTLVertexAttributeDescriptor, offset: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(offset)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setOffset:"), offset);
    }
    bufferIndex :: (self: *MTLVertexAttributeDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("bufferIndex"));
    }
    setBufferIndex :: (self: *MTLVertexAttributeDescriptor, bufferIndex: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferIndex)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setBufferIndex:"), bufferIndex);
    }
}

MTLVertexAttributeDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLVertexAttributeDescriptorArray, index: NSUInteger) -> *MTLVertexAttributeDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> *MTLVertexAttributeDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexAttributeDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), index);
    }
    setObject_atIndexedSubscript :: (self: *MTLVertexAttributeDescriptorArray, attributeDesc: *MTLVertexAttributeDescriptor, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(attributeDesc), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), attributeDesc, index);
    }
}

MTLVertexBufferLayoutDescriptor :: struct {
    nsobject: NSObject;
    stride :: (self: *MTLVertexBufferLayoutDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stride"));
    }
    setStride :: (self: *MTLVertexBufferLayoutDescriptor, stride: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stride)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStride:"), stride);
    }
    stepFunction :: (self: *MTLVertexBufferLayoutDescriptor) -> MTLVertexStepFunction {
        Send :: #type (receiver: *void, op: *void) -> MTLVertexStepFunction #c_call;
        send: Send;

        #if CPU == .X64 && size_of(MTLVertexStepFunction) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stepFunction"));
    }
    setStepFunction :: (self: *MTLVertexBufferLayoutDescriptor, stepFunction: MTLVertexStepFunction) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stepFunction)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStepFunction:"), stepFunction);
    }
    stepRate :: (self: *MTLVertexBufferLayoutDescriptor) -> NSUInteger {
        Send :: #type (receiver: *void, op: *void) -> NSUInteger #c_call;
        send: Send;

        #if CPU == .X64 && size_of(NSUInteger) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("stepRate"));
    }
    setStepRate :: (self: *MTLVertexBufferLayoutDescriptor, stepRate: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(stepRate)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setStepRate:"), stepRate);
    }
}

MTLVertexBufferLayoutDescriptorArray :: struct {
    nsobject: NSObject;
    objectAtIndexedSubscript :: (self: *MTLVertexBufferLayoutDescriptorArray, index: NSUInteger) -> *MTLVertexBufferLayoutDescriptor {
        Send :: #type (receiver: *void, op: *void, a0: type_of(index)) -> *MTLVertexBufferLayoutDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexBufferLayoutDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("objectAtIndexedSubscript:"), index);
    }
    setObject_atIndexedSubscript :: (self: *MTLVertexBufferLayoutDescriptorArray, bufferDesc: *MTLVertexBufferLayoutDescriptor, index: NSUInteger) -> () {
        Send :: #type (receiver: *void, op: *void, a0: type_of(bufferDesc), a1: type_of(index)) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("setObject:atIndexedSubscript:"), bufferDesc, index);
    }
}

MTLVertexDescriptor :: struct {
    nsobject: NSObject;
    vertexDescriptor :: () -> *MTLVertexDescriptor {
        Send :: #type (receiver: *void, op: *void) -> *MTLVertexDescriptor #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexDescriptor) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(binding_objc_class("MTLVertexDescriptor"), binding_objc_selector("vertexDescriptor"));
    }
    reset :: (self: *MTLVertexDescriptor) -> () {
        Send :: #type (receiver: *void, op: *void) -> () #c_call;
        send: Send;
        send = cast(Send) binding_objc_send;
        send(cast(*void) self, binding_objc_selector("reset"));
    }
    layouts :: (self: *MTLVertexDescriptor) -> *MTLVertexBufferLayoutDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLVertexBufferLayoutDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexBufferLayoutDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("layouts"));
    }
    attributes :: (self: *MTLVertexDescriptor) -> *MTLVertexAttributeDescriptorArray {
        Send :: #type (receiver: *void, op: *void) -> *MTLVertexAttributeDescriptorArray #c_call;
        send: Send;

        #if CPU == .X64 && size_of(*MTLVertexAttributeDescriptorArray) > 16 {
            send = cast(Send) binding_objc_send_stret;
        } else {
            send = cast(Send) binding_objc_send;
        }

        return send(cast(*void) self, binding_objc_selector("attributes"));
    }
}

using MTLVertexFormat :: enum NSUInteger {
    MTLVertexFormatInvalid :: 0;
    MTLVertexFormatUChar2 :: 1;
    MTLVertexFormatUChar3 :: 2;
    MTLVertexFormatUChar4 :: 3;
    MTLVertexFormatChar2 :: 4;
    MTLVertexFormatChar3 :: 5;
    MTLVertexFormatChar4 :: 6;
    MTLVertexFormatUChar2Normalized :: 7;
    MTLVertexFormatUChar3Normalized :: 8;
    MTLVertexFormatUChar4Normalized :: 9;
    MTLVertexFormatChar2Normalized :: 10;
    MTLVertexFormatChar3Normalized :: 11;
    MTLVertexFormatChar4Normalized :: 12;
    MTLVertexFormatUShort2 :: 13;
    MTLVertexFormatUShort3 :: 14;
    MTLVertexFormatUShort4 :: 15;
    MTLVertexFormatShort2 :: 16;
    MTLVertexFormatShort3 :: 17;
    MTLVertexFormatShort4 :: 18;
    MTLVertexFormatUShort2Normalized :: 19;
    MTLVertexFormatUShort3Normalized :: 20;
    MTLVertexFormatUShort4Normalized :: 21;
    MTLVertexFormatShort2Normalized :: 22;
    MTLVertexFormatShort3Normalized :: 23;
    MTLVertexFormatShort4Normalized :: 24;
    MTLVertexFormatHalf2 :: 25;
    MTLVertexFormatHalf3 :: 26;
    MTLVertexFormatHalf4 :: 27;
    MTLVertexFormatFloat :: 28;
    MTLVertexFormatFloat2 :: 29;
    MTLVertexFormatFloat3 :: 30;
    MTLVertexFormatFloat4 :: 31;
    MTLVertexFormatInt :: 32;
    MTLVertexFormatInt2 :: 33;
    MTLVertexFormatInt3 :: 34;
    MTLVertexFormatInt4 :: 35;
    MTLVertexFormatUInt :: 36;
    MTLVertexFormatUInt2 :: 37;
    MTLVertexFormatUInt3 :: 38;
    MTLVertexFormatUInt4 :: 39;
    MTLVertexFormatInt1010102Normalized :: 40;
    MTLVertexFormatUInt1010102Normalized :: 41;
    MTLVertexFormatUChar4Normalized_BGRA :: 42;
    MTLVertexFormatUChar :: 45;
    MTLVertexFormatChar :: 46;
    MTLVertexFormatUCharNormalized :: 47;
    MTLVertexFormatCharNormalized :: 48;
    MTLVertexFormatUShort :: 49;
    MTLVertexFormatShort :: 50;
    MTLVertexFormatUShortNormalized :: 51;
    MTLVertexFormatShortNormalized :: 52;
    MTLVertexFormatHalf :: 53;
}

using MTLVertexStepFunction :: enum NSUInteger {
    MTLVertexStepFunctionConstant :: 0;
    MTLVertexStepFunctionPerVertex :: 1;
    MTLVertexStepFunctionPerInstance :: 2;
    MTLVertexStepFunctionPerPatch :: 3;
    MTLVertexStepFunctionPerPatchControlPoint :: 4;
}

MTLViewport :: struct {
    originX: float64;
    originY: float64;
    width: float64;
    height: float64;
    znear: float64;
    zfar: float64;
}

using MTLVisibilityResultMode :: enum NSUInteger {
    MTLVisibilityResultModeDisabled :: 0;
    MTLVisibilityResultModeBoolean :: 1;
    MTLVisibilityResultModeCounting :: 2;
}

using MTLWinding :: enum NSUInteger {
    MTLWindingClockwise :: 0;
    MTLWindingCounterClockwise :: 1;
}

// Aliases and constants

MTLAutoreleasedArgument :: *MTLArgument;

MTLAutoreleasedComputePipelineReflection :: *MTLComputePipelineReflection;

MTLAutoreleasedRenderPipelineReflection :: *MTLRenderPipelineReflection;

MTLDeviceNotificationName :: *NSString;

// Native storage

MTLCommandBufferErrorDomain: *NSString;

MTLDeviceRemovalRequestedNotification: MTLDeviceNotificationName;

MTLDeviceWasAddedNotification: MTLDeviceNotificationName;

MTLDeviceWasRemovedNotification: MTLDeviceNotificationName;

MTLLibraryErrorDomain: *NSString;

// Native entry points

MTLCopyAllDevices :: () -> *NSArray(id) #foreign Metal;

MTLCreateSystemDefaultDevice :: () -> id #foreign Metal;

MTLRemoveDeviceObserver :: (observer: id) -> () #foreign Metal;

// Convenience operations

init_metal :: () {
    init_objective_c();
    if !binding_objc_class("MTLTextureDescriptor") binding_abort();
}

MTLClearColorMake :: (red: float64, green: float64, blue: float64, alpha: float64) -> MTLClearColor {
    return MTLClearColor.{ red, green, blue, alpha };
}

MTLOriginMake :: inline (x: NSUInteger, y: NSUInteger, z: NSUInteger) -> MTLOrigin {
    return MTLOrigin.{ x, y, z };
}

MTLRegionMake1D :: inline (x: NSUInteger, width: NSUInteger) -> MTLRegion {
    return MTLRegion.{ MTLOrigin.{ x, 0, 0 }, MTLSize.{ width, 1, 1 } };
}

MTLRegionMake2D :: inline (x: NSUInteger, y: NSUInteger, width: NSUInteger, height: NSUInteger) -> MTLRegion {
    return MTLRegion.{ MTLOrigin.{ x, y, 0 }, MTLSize.{ width, height, 1 } };
}

MTLRegionMake3D :: inline (x: NSUInteger, y: NSUInteger, z: NSUInteger, width: NSUInteger, height: NSUInteger, depth: NSUInteger) -> MTLRegion {
    return MTLRegion.{ MTLOrigin.{ x, y, z }, MTLSize.{ width, height, depth } };
}

MTLSamplePositionMake :: inline (x: float, y: float) -> MTLSamplePosition {
    return MTLSamplePosition.{ x, y };
}

MTLSizeMake :: inline (width: NSUInteger, height: NSUInteger, depth: NSUInteger) -> MTLSize {
    return MTLSize.{ width, height, depth };
}
