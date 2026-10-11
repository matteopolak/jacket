-- The jailsp language server, for `vim.lsp.enable('jailsp')` (Neovim 0.11+). Needs no setup
-- call; override anything with `vim.lsp.config('jailsp', { ... })`. How the binary is found:
-- docs/tools/neovim-plugin.md.
local config = require('jai.config')

---@type vim.lsp.Config
return {
  cmd = require('jai').server_cmd,
  filetypes = { 'jai' },
  root_markers = { 'jai.toml', 'jailint.toml', 'jaifmt.toml', '.git' },
  before_init = function(params, cfg)
    -- Read when the server starts, so `setup()` and `vim.g.jai` may come later than this file.
    local settings = config.server_settings()
    params.initializationOptions = settings
    cfg.settings = { jai = settings }
  end,
}
