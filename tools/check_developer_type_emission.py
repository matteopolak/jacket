#!/usr/bin/env python3
"""Type workflow entry point; all prior type/fragment regressions run in private scratch.

The unified runner also verifies canonical bridges and macro values. It retains
native C++ fragment effects, generated Jai consumers, negatives, format and lint.
CARGO_TARGET_DIR selects the actual source-built worker tools; no shared output.
"""

from check_developer_public_bridge import main


if __name__ == "__main__":
    main()
