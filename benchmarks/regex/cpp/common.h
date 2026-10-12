// Shared helpers for the C++ harnesses: argument parsing, cases.tsv, timing, result lines.
//
// Every harness is run as `harness CORPUS RUNS CASES_TSV [CASE...]` and prints
//     RESULT engine case status count check mbps compile_us note
// with tab separators, one line per case. See benchmarks/regex/cases.tsv for the case ids.
#pragma once
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <functional>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

struct Case {
    std::string id, pattern;
};

struct Harness {
    std::string corpus;
    int runs = 5;
    std::vector<Case> cases;
    std::vector<std::string> wanted;
};

inline double now_seconds() {
    using clock = std::chrono::steady_clock;
    return std::chrono::duration<double>(clock::now().time_since_epoch()).count();
}

inline Harness parse_args(int argc, char** argv) {
    if (argc < 4) {
        std::fprintf(stderr, "usage: %s CORPUS RUNS CASES_TSV [CASE...]\n", argv[0]);
        std::exit(2);
    }
    Harness h;
    std::ifstream in(argv[1], std::ios::binary);
    std::stringstream ss;
    ss << in.rdbuf();
    h.corpus = ss.str();
    h.runs = std::atoi(argv[2]);
    for (int i = 4; i < argc; i++) h.wanted.push_back(argv[i]);
    std::ifstream tsv(argv[3]);
    std::string line;
    while (std::getline(tsv, line)) {
        if (line.empty() || line[0] == '#') continue;
        size_t a = line.find('\t');
        size_t b = line.find('\t', a + 1);
        Case c{line.substr(0, a), line.substr(a + 1, b - a - 1)};
        if (!h.wanted.empty() && std::find(h.wanted.begin(), h.wanted.end(), c.id) == h.wanted.end()) continue;
        h.cases.push_back(c);
    }
    return h;
}

struct Outcome {
    uint64_t count = 0;
    uint64_t check = 0;
};

// Best wall time of `runs` calls of f, which returns the match count and checksum.
inline double best_of(int runs, Outcome& out, const std::function<Outcome()>& f) {
    double best = 1e30;
    for (int i = 0; i < runs; i++) {
        double t = now_seconds();
        out = f();
        best = std::min(best, now_seconds() - t);
    }
    return best;
}

inline void report(const char* engine, const std::string& id, const Outcome& o, double best, size_t bytes,
                   double compile_us, const std::string& note = "") {
    std::printf("RESULT\t%s\t%s\tok\t%llu\t%llu\t%.3f\t%.3f\t%s\n", engine, id.c_str(), (unsigned long long)o.count,
                (unsigned long long)o.check, (double)bytes / 1e6 / best, compile_us, note.c_str());
    std::fflush(stdout);
}

inline void report_na(const char* engine, const std::string& id, const std::string& why) {
    std::printf("RESULT\t%s\t%s\tna\t0\t0\t0\t0\t%s\n", engine, id.c_str(), why.c_str());
    std::fflush(stdout);
}

inline uint64_t to_u64(const char* p, size_t n) {
    uint64_t v = 0;
    for (size_t i = 0; i < n; i++) v = v * 10 + (uint64_t)(p[i] - '0');
    return v;
}
