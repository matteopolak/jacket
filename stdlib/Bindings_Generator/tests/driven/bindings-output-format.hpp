#include <stdint.h>
#define SLANG_PTR_IS_32 0
enum OperandDataType : uint8_t { OperandDataType_None, OperandDataType_Int };
struct VMExecOperand {
    uint8_t** section; // Pointer to the section start pointer.
    #if SLANG_PTR_IS_32
    uint32_t padding;
    #endif
    uint32_t type : 8; // type of the operand data.
    uint32_t size : 24;
    uint32_t offset;
    void* getPtr() const { return *section + offset; }
    OperandDataType getType() const { return (OperandDataType)type; }
};
struct Flags { int a : 3; unsigned b : 1; bool c : 1; };
struct SessionDesc { int x; };
struct ISession;
struct IUnknownish {
    virtual int queryInterface(int const& guid, void** out) = 0;
    virtual unsigned addRef() = 0;
    virtual unsigned release() = 0;
};
struct IGlobalSession : public IUnknownish {
    virtual int createSession(SessionDesc const& desc, ISession** outSession) = 0;
    virtual int scale(int a) = 0;
    virtual int scale(float a) = 0;
};
struct ISession : public IUnknownish {
    virtual int loadModule(const char* name) = 0;
};
