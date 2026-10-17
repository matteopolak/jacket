---SHA-256 checksums of the release archives, written by the release workflow when it publishes
---the plugin (`{ version = '0.9.0', sha256 = { ['jacket-linux-x64.tar.gz'] = '...' } }`). Empty in
---a development checkout, which then reads the release's own `SHA256SUMS`.
---@type { version: string?, sha256: table<string, string> }
return { sha256 = {} }
