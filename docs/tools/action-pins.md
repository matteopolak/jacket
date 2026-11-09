# Action pins

Every GitHub Action is pinned by commit SHA with a version comment, and `tools/ci/check_action_pins.py` keeps them current. Dependabot is not used.

```
uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
```

## Check

```
PYTHONPATH=tools python3 tools/ci/check_action_pins.py
```

It reads every `uses:` in `.github/workflows/*.yml` and `.github/actions/*/action.yml` (local `./` uses are skipped) and asks the GitHub API about each repository. A pin fails when:

- it is not a 40-character SHA, or has no `# vX.Y.Z` comment;
- the SHA is not the commit the commented tag points to (annotated tags are followed);
- the release is younger than 14 days, the same rule as [cargo dependencies](development.md#dependency-policy).

It also lists pins with a newer stable release that is at least 14 days old, and marks a new major version. Pre-releases and drafts are ignored.

| Exit | Meaning |
| --- | --- |
| 0 | up to date |
| 1 | a pin breaks the rules above |
| 2 | updates are available; nothing is wrong |
| 3 | GitHub could not be reached or answered unexpectedly |

`--policy` checks only the rules (what CI runs); `--refresh` ignores cached answers.

## Update

```
PYTHONPATH=tools python3 tools/ci/check_action_pins.py --update
```

This rewrites the SHA and comment of each outdated pin, in every file where the action appears, to the newest release at least 14 days old. A new major version is applied too, so read the diff. Commit it as `ci: update pinned actions`.

## Hook and CI

`.githooks/pre-push` runs the check. Rule breaks and lookup failures block the push; available updates are printed and do not. Skip it with `git push --no-verify` or `JAI_HOOK_SKIP=actions`. CI runs `--policy` in the `dependency-hygiene` job, so a bad pin cannot land.

## Configuration

- `GITHUB_TOKEN` or `GH_TOKEN` raises the API limit from 60 requests an hour; set one if the check reports a rate limit.
- Answers are cached for six hours in `target/action-pins-cache.json`.
- Only each repository's newest 100 releases are read.

The 14-day constant and the cache helpers are shared with `check_dependency_age.py` in `tools/toolkit/age_policy.py`. Tests: `tools/ci/test_check_action_pins.py` (recorded answers, no network).
