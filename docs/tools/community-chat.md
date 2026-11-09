# Community chat

The project chat is a Zulip organization at https://jacket.zulipchat.com. Sign-up is open at `/register/`.

## Channels

`announcements` (admins post), `help`, `compiler`, `stdlib`, `tools`, `playground`, `show-and-tell`, `off-topic`, `test`. Only admins create channels, bots and custom emoji.

## Code blocks

The default code-block language is `jai`. Zulip's code playgrounds for `jai` and `jacket` add a button that opens `https://matteopolak.com/playground/jacket?code={code}`. The playground (in the portfolio repository) reads the raw `?code=` query and opens it as `main.jai`.

## Release announcements

`.github/workflows/zulip-release.yml` runs on `release: released`, so a stable release posts once to `announcements` (topic "Releases"), and prereleases and drafts post nothing. It forwards the event to Zulip's GitHub endpoint for a bot named "GitHub". The endpoint URL, which contains the bot's key, is the repository secret `ZULIP_RELEASE_WEBHOOK`.

To rotate the key, regenerate the bot's key in Zulip (Settings, Bots) and update the secret with `gh secret set ZULIP_RELEASE_WEBHOOK`. The URL is `https://jacket.zulipchat.com/api/v1/external/github?api_key=<key>&stream=announcements&topic=Releases`.

A repository webhook is not used: it can only filter on the whole `release` event, which also sends created, published and edited actions.
