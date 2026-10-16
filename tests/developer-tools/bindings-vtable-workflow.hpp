struct IndependentDispatch {
    int value = 10;
    virtual int apply(int input) { return value + input; }
    virtual int retained_position(int input) { return value - input; }
};
