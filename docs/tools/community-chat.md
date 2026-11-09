# Community chat

The project chat is a Zulip organization at https://jacket.zulipchat.com. Sign-up is open at `/register/`.

## Channels

`announcements` (admins post), `help`, `compiler`, `stdlib`, `tools`, `playground`, `show-and-tell`, `off-topic`, `test`. Only admins create channels, bots and custom emoji.

## Code blocks

The default code-block language is `jai`. Zulip's code playgrounds for `jai` and `jacket` add a button that opens `https://matteopolak.com/playground/jacket?code={code}`. The playground (in the portfolio repository) reads the raw `?code=` query and opens it as `main.jai`.

## Release announcements

`.github/workflows/zulip-release.yml` runs on `release: released`, so a stable release posts once to `announcements` (topic "Releases") and prereleases and drafts post nothing. `tools/ci/announce_release.py` reads the release with `gh api` and posts its notes (repository links rewritten to the tagged tree) and a collapsed "Downloads" list of every asset with its size. A repository webhook is not used: it can only filter on the whole `release` event, which also sends created, published and edited actions.

Preview a past release without publishing anything: run the workflow by hand (Actions, "announce release on Zulip") with a tag; it posts to `test` by default. Locally, `GITHUB_REPOSITORY=matteopolak/jacket tools/ci/announce_release.py --tag v0.33.0 --dry-run` prints the message.

## Configuration

- Repository secrets `ZULIP_BOT_EMAIL` and `ZULIP_BOT_KEY`: the Zulip bot "Releases" (a generic bot). Rotate the key in Zulip (Settings, Bots) and update the secret with `gh secret set ZULIP_BOT_KEY`.
- The bot is a member of `announcements`' "who can post" group, next to administrators. Without that it cannot post there.
