"""Python uuid baseline: same input, count and best-of-five rule as bench.jai."""
import timeit
import uuid

TEXT = "919108f7-52d1-4320-9bac-f847db4148a8"
VALUE = uuid.UUID(TEXT)
for label, statement in (
    ("parse", "uuid.UUID(TEXT)"),
    ("format", "str(VALUE)"),
    ("v4", "uuid.uuid4()"),
    ("v7", "uuid.uuid7()"),
):
    if label == "v7" and not hasattr(uuid, "uuid7"):
        print("v7: unavailable in this Python")
        continue
    seconds = min(timeit.repeat(statement, globals=globals(), number=200000, repeat=5))
    print(f"{label}: {seconds * 1e9 / 200000:.1f} ns/op")
