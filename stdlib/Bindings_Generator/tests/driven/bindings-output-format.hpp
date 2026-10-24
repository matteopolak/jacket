struct VMExecOperand {
    unsigned char** section;
    unsigned int type : 8;   // kind
    unsigned int field : 24; // field index
    unsigned int offset;
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
