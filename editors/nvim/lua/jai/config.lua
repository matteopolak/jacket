---Settings: defaults, then `vim.g.jai`, then `require('jai').setup()`. They mirror the VS Code
---extension's `jai.*` settings.
local M = {}

---@class jai.Config
---@field server { path: string }  Path to `jailsp`; empty looks it up.
---@field compiler { path: string }  Path to `jaic` (the Run/Build/Check commands).
---@field formatter { path: string }  Path to `jaifmt`.
---@field stdlib { path: string }  Standard library directory, passed as `JAIC_STDLIB`.
---@field toolchain { auto_download: 'prompt'|'always'|'never', version: string }
---@field completion { auto_import: boolean }  Offer names the file cannot see yet.
---@field run { arguments: string[] }  Extra `jaic` arguments for Run/Build/Check.
---@field format_on_save boolean  Format Jai buffers with `jaifmt` before writing.
---@field lsp { enable: boolean }  Enable `jailsp` when the plugin loads.
local defaults = {
  server = { path = '' },
  compiler = { path = '' },
  formatter = { path = '' },
  stdlib = { path = '' },
  toolchain = { auto_download = 'prompt', version = '' },
  completion = { auto_import = true },
  run = { arguments = {} },
  format_on_save = false,
  lsp = { enable = true },
}

---@type table
local overrides = {}

---Replaces the options given to `setup()`.
---@param opts? table
function M.set(opts)
  overrides = opts or {}
end

---The effective settings.
---@return jai.Config
function M.get()
  local user = type(vim.g.jai) == 'table' and vim.g.jai or {}
  return vim.tbl_deep_extend('force', defaults, user, overrides)
end

---What `jailsp` reads as `initializationOptions` and as the `jai` section of
---`workspace/didChangeConfiguration`.
---@return { completion: { autoImport: boolean } }
function M.server_settings()
  return { completion = { autoImport = M.get().completion.auto_import } }
end

---Environment for `jailsp` and `jaic`: `JAIC_STDLIB` from `stdlib.path`.
---@return table<string, string>
function M.tool_env()
  local stdlib = M.get().stdlib.path
  if stdlib == '' then
    return {}
  end
  return { JAIC_STDLIB = vim.fs.normalize(stdlib) }
end

return M
