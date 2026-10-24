// The shape of a COM-style SDK header: a namespace of interfaces over one base with calling-convention macros.
#define SLANG_NO_THROW
#define SLANG_MCALL

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
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL queryInterface(SlangUUID const& uuid, void** outObject) = 0;
    virtual SLANG_NO_THROW unsigned SLANG_MCALL addRef() = 0;
    virtual SLANG_NO_THROW unsigned SLANG_MCALL release() = 0;
};

namespace slang {

struct ISession : public ISlangUnknown {
    virtual SLANG_NO_THROW int SLANG_MCALL getIndex() = 0;
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL describe(SessionDesc* outDesc) = 0;
};

struct IGlobalSession : public ISlangUnknown {
    virtual SLANG_NO_THROW SlangResult SLANG_MCALL createSession(SessionDesc const& desc, ISession** outSession) = 0;
    virtual SLANG_NO_THROW const char* SLANG_MCALL getBuildTagString() = 0;
};

}
