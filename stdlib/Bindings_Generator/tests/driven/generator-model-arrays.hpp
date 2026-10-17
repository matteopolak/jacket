template<class T, int N, int M> struct ModelArrayBounds {
    T single[N];
    T matrix[N][M];
    T mixed[2][N + 1][M];
    T (*pointer)[N] = nullptr;
    T (*(*callbacks[N])())[M];
    T (*callback_arguments[N])(T (*)[M]);
    T (*(*callback_both[N])(T (*)[N + 1]))[M];
    int (*reference_callback)(const T&);
};
