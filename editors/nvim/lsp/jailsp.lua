-- The jailsp language server, for `vim.lsp.enable('jailsp')` (Neovim 0.11+). Needs no setup
-- call; override anything with `vim.lsp.config('jailsp', { ... })`. How the binary is found:
-- docs/tools/neovim-plugin.md.
local config = require('jai.config')

---@type vim.lsp.Config
return {
  cmd = require('jai').server_cmd,
  -- `jacket.toml` too: jailsp reports its problems and completes its keys and paths.
  filetypes = { 'jai', 'toml' },
  root_dir = function(bufnr, on_dir)
    local is_toml = vim.bo[bufnr].filetype == 'toml'
    if is_toml and vim.fs.basename(vim.api.nvim_buf_get_name(bufnr)) ~= 'jacket.toml' then
      return
    end
    on_dir(vim.fs.root(bufnr, { 'jacket.toml', '.git' }))
  end,
  handlers = {
    -- A compile-time limit stopped code while checking: say how to raise it.
    ['jai/compileTimeLimit'] = function(_, notice)
      require('jai.limits').notice(notice)
    end,
  },
  before_init = function(params, cfg)
    -- Read when the server starts, so `setup()` and `vim.g.jai` may come later than this file.
    local settings = config.server_settings()
    params.initializationOptions = settings
    cfg.settings = { jai = settings }
  end,
}
