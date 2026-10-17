struct ModelConversionProbe;
struct ModelConversionProbe {
    virtual operator bool() { return true; }
    virtual int following() { return 73; }
};
