// The shape of a COM-style SDK header: a namespace of interfaces over one base with calling-convention macros.
#define SLANG_NO_THROW
#define SLANG_MCALL
// The interface-identity macro of such headers expands to an inline static member inside the struct.
#define SLANG_COM_INTERFACE(a, b, c, ...) \
public: \
    inline static const SlangUUID& getTypeGuid() \
    { \
        static const SlangUUID guid = {a, b, c, __VA_ARGS__}; \
        return guid; \
    }

typedef int SlangResult;

struct SlangUUID {
    unsigned data1;
    unsigned short data2;
    unsigned short data3;
    unsigned char data4[8];
};

struct SessionDesc {
    int flags;
    int targetCount;
};

struct ISlangUnknown {
    SLANG_COM_INTERFACE(0x00000000, 0x0000, 0x0000, {0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46})
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL queryInterface(SlangUUID const& uuid, void** outObject) = 0;
    virtual SLANG_NO_THROW unsigned SLANG_MCALL addRef() = 0;
    virtual SLANG_NO_THROW unsigned SLANG_MCALL release() = 0;
};

namespace slang {

struct ISession : public ISlangUnknown {
    SLANG_COM_INTERFACE(0x67618701, 0xd116, 0x468f, {0xab, 0x3b, 0x47, 0x4b, 0xed, 0xce, 0x0e, 0x3d})

    virtual SLANG_NO_THROW int SLANG_MCALL getIndex() = 0;
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL describe(SessionDesc* outDesc) = 0;
};

struct IGlobalSession : public ISlangUnknown {
    SLANG_COM_INTERFACE(0xc140b5fd, 0x0c78, 0x452e, {0xba, 0x7c, 0x1a, 0x1e, 0x70, 0xc7, 0xf7, 0x1c})

    /** Create a new session for loading and compiling code.
     */
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL
    createSession(SessionDesc const& desc, ISession** outSession) = 0;
    virtual SLANG_NO_THROW const char* SLANG_MCALL getBuildTagString() = 0;
};

}
