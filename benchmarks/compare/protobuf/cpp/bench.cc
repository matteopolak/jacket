// Google's C++ protobuf on the five shapes of benches/protobuf.jai: a mixed message, 1 MiB of packed
// fixed32, 64 levels of nesting, 1,000 string-heavy log lines and 2,000 tracing spans, encode and decode.
// proto3, arena off, default options, code from protoc (../proto/shapes.proto). The Spans generator is
// the one of the Jai benchmark (xorshift64*), so the encoded sizes match.
//
// Each workload is timed with std::chrono: the iteration count is found by doubling until one sample takes
// 20 ms, then 20 samples of that many iterations are taken and the median and spread are printed, with
// MB/s and the allocations of one call (counted by replacing operator new in a pass of its own).
// Every iteration allocates and frees through malloc, as the library does by default; the Jai benchmark
// allocates from a bump pool instead.
//
// Built and run by build.py, inside the sandbox. Output, one line per cell, tab separated:
//   name  bytes  ns_per_iter(median)  ns_low  ns_high  MB_per_s  allocs  alloc_bytes
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <new>
#include <string>
#include <vector>

#include "shapes.pb.h"

// ------------------------------------------------------------------------------ allocation counting

static bool counting = false;
static int64_t allocation_count = 0;
static int64_t allocation_bytes = 0;

void* operator new(std::size_t size) {
    if (counting) {
        allocation_count += 1;
        allocation_bytes += static_cast<int64_t>(size);
    }
    void* pointer = std::malloc(size ? size : 1);
    if (!pointer) throw std::bad_alloc();
    return pointer;
}

void operator delete(void* pointer) noexcept { std::free(pointer); }
void operator delete(void* pointer, std::size_t) noexcept { std::free(pointer); }

// ------------------------------------------------------------------------------ the Spans generator

static uint64_t rng_state = 0x9E3779B97F4A7C15ull;

static uint64_t next() {
    uint64_t x = rng_state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    rng_state = x;
    return x * 0x2545F4914F6CDD1Dull;
}

static uint64_t below(uint64_t n) { return next() % n; }

static const char* const NAMES[] = {"GET /api/users/{id}", "POST /api/orders", "GET /api/orders/{id}", "db.query", "cache.get", "cache.set", "render.page", "auth.verify", "queue.publish", "queue.consume", "GET /healthz", "PUT /api/users/{id}"};
static const char* const KEYS[] = {"http.method", "http.status_code", "http.route", "db.system", "db.statement", "net.peer.name", "net.peer.port", "user.id", "retry.count", "error.type"};
static const char* const EVENT_NAMES[] = {"exception", "retry", "cache.miss", "message"};

static std::string text() {
    uint64_t length = 4 + below(21);
    std::string out(length, ' ');
    for (uint64_t i = 0; i < length; i++) out[i] = static_cast<char>('a' + below(26));
    return out;
}

static void attribute(Attribute* out) {
    out->set_key(KEYS[below(10)]);
    if (below(3) == 0) {
        out->set_number(static_cast<int64_t>(below(100000)));
    } else {
        out->set_value(text());
    }
}

static std::string random_bytes(int count) {
    std::string out(count, '\0');
    for (int i = 0; i < count; i++) out[i] = static_cast<char>(next() & 0xFF);
    return out;
}

static void make_spans(Spans* spans) {
    std::string trace;
    std::string previous;
    for (int i = 0; i < 2000; i++) {
        Span* span = spans->add_spans();
        if (i % 8 == 0) trace = random_bytes(16);
        span->set_trace_id(trace);
        std::string id = random_bytes(8);
        span->set_span_id(id);
        if (i % 8 != 0) span->set_parent_id(previous);
        previous = id;
        span->set_name(NAMES[below(12)]);
        span->set_kind(static_cast<Kind>(i % 4));
        uint64_t start = 1700000000000000000ull + static_cast<uint64_t>(i) * 1000 + below(1000);
        span->set_start_ns(start);
        span->set_end_ns(start + below(5000000));
        uint64_t attributes = 3 + below(6);
        for (uint64_t a = 0; a < attributes; a++) attribute(span->add_attributes());
        uint64_t events = below(3);
        for (uint64_t e = 0; e < events; e++) {
            Event* event = span->add_events();
            event->set_time_ns(start + below(1000000));
            event->set_name(EVENT_NAMES[below(4)]);
            uint64_t event_attributes = 1 + below(2);
            for (uint64_t a = 0; a < event_attributes; a++) attribute(event->add_attributes());
        }
        span->set_status(below(10) == 0 ? 2 : 0);
    }
}

// ------------------------------------------------------------------------------ measuring

using Clock = std::chrono::steady_clock;

static double time_batch(const std::function<void()>& work, int64_t iterations) {
    auto start = Clock::now();
    for (int64_t i = 0; i < iterations; i++) work();
    return std::chrono::duration<double, std::nano>(Clock::now() - start).count() / static_cast<double>(iterations);
}

static void measure(const char* name, size_t bytes, const std::function<void()>& work) {
    int64_t iterations = 1;
    while (true) {
        auto start = Clock::now();
        for (int64_t i = 0; i < iterations; i++) work();
        double elapsed = std::chrono::duration<double, std::milli>(Clock::now() - start).count();
        if (elapsed >= 20.0) break;
        iterations *= 2;
    }
    std::vector<double> samples;
    for (int s = 0; s < 20; s++) samples.push_back(time_batch(work, iterations));
    std::sort(samples.begin(), samples.end());
    double median = (samples[9] + samples[10]) / 2;

    counting = true;
    allocation_count = 0;
    allocation_bytes = 0;
    work();
    counting = false;

    std::printf("%s\t%zu\t%.1f\t%.1f\t%.1f\t%.0f\t%lld\t%lld\n", name, bytes, median, samples.front(), samples.back(),
                static_cast<double>(bytes) * 1000.0 / median, static_cast<long long>(allocation_count),
                static_cast<long long>(allocation_bytes));
    std::fflush(stdout);
}

template <typename T>
static void cell(const char* workload, const T& value) {
    std::string bytes;
    value.SerializeToString(&bytes);
    std::string encode_name = std::string("encode_") + workload + "__cpp";
    std::string decode_name = std::string("decode_") + workload + "__cpp";
    // The decode must give back what was encoded, byte for byte.
    {
        T again;
        if (!again.ParseFromString(bytes)) { std::fprintf(stderr, "%s: cannot parse its own output\n", workload); std::exit(1); }
        std::string second;
        again.SerializeToString(&second);
        if (second != bytes) { std::fprintf(stderr, "%s: parse and serialize changed the bytes\n", workload); std::exit(1); }
    }
    measure(encode_name.c_str(), bytes.size(), [&]() {
        std::string out;
        value.SerializeToString(&out);
        asm volatile("" : : "r"(out.data()) : "memory");
    });
    measure(decode_name.c_str(), bytes.size(), [&]() {
        T parsed;
        bool ok = parsed.ParseFromArray(bytes.data(), static_cast<int>(bytes.size()));
        asm volatile("" : : "r"(&parsed), "r"(ok) : "memory");
    });
}

int main(int argc, char** argv) {
    std::string only = argc > 1 ? argv[1] : "";

    Mixed mixed;
    mixed.set_id(123456);
    mixed.set_name("an API response");
    mixed.mutable_child()->set_x(42);
    mixed.mutable_child()->set_label("nested");
    for (int i = 0; i < 4; i++) {
        Leaf* leaf = mixed.add_children();
        leaf->set_x(42);
        leaf->set_label("nested");
    }
    mixed.set_active(true);

    Packed packed;
    for (uint32_t i = 0; i < 262144; i++) packed.add_numbers(i);

    Deep deep;
    {
        Deep* node = &deep;
        for (int i = 0; i < 64; i++) node = node->mutable_next();
        node->set_value("leaf");
    }

    Logs logs;
    for (int i = 0; i < 1000; i++) {
        Log* log = logs.add_entries();
        log->set_time("2026-11-02T12:00:00Z");
        log->set_level("INFO");
        log->set_source("api/request");
        log->set_text("Request completed successfully with a useful response");
    }

    Spans spans;
    make_spans(&spans);

    std::string spans_bytes;
    spans.SerializeToString(&spans_bytes);
    std::fprintf(stderr, "spans encoded bytes: %zu\n", spans_bytes.size());

    if (only.empty() || only == "mixed") cell("mixed", mixed);
    if (only.empty() || only == "packed") cell("packed", packed);
    if (only.empty() || only == "deep") cell("deep", deep);
    if (only.empty() || only == "logs") cell("logs", logs);
    if (only.empty() || only == "spans") cell("spans", spans);
    return 0;
}
