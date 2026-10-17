#pragma once

struct Point {
    int x;
    int y;
    double weight;
    // Defined in the header: inline, so the library has no symbol for it.
    Point() : x(0), y(0), weight(1.0) {}
    int sum() const;
    inline int twice_x() const { return x * 2; }
};

struct Point3 : Point {
    char tag;
    int z;
    int total() const;
};

inline int header_only_helper(int a) { return a + 1; }
int library_helper(int a);
