---Settings: defaults, then `vim.g.jai`, then `require('jai').setup()`. They mirror the VS Code
---extension's `jai.*` settings.
local M = {}

---@class jai.Config
---@field server { path: string }  Path to `jailsp`; empty looks it up.
---@field compiler { path: string }  Path to `jaic` (the Run/Build/Check commands).
---@field formatter { path: string }  Path to `jaifmt`.
---@field stdlib { path: string, prelude: string }  Library directory and prelude file, passed as `JAIC_STDLIB` and `JAIC_PRELUDE`.
---@field toolchain { auto_download: 'prompt'|'always'|'never', version: string }
---@field completion { auto_import: boolean, call_snippets: 'addParentheses'|'fillArguments'|'none' }  Offer names the file cannot see yet; what accepting a procedure inserts.
---@field inlay_hints { closing_brace_min_lines: integer }  Name the closing brace of procedures, structs and enums this many lines long (0: never).
---@field run { arguments: string[] }  Extra `jaic` arguments for Run/Build/Check.
---@field trusted boolean  The workspace is trusted: `jailsp` may run its compile-time code (`#run`, build metaprograms, computed `#insert`) and `:Jai run`/`build`/`check` are on.
---@field format_on_save boolean  Format Jai buffers with `jaifmt` before writing.
---@field lsp { enable: boolean }  Enable `jailsp` when the plugin loads.
local defaults = {
  server = { path = '' },
  compiler = { path = '' },
  formatter = { path = '' },
  stdlib = { path = '', prelude = '' },
  toolchain = { auto_download = 'prompt', version = '' },
  completion = { auto_import = true, call_snippets = 'addParentheses' },
  inlay_hints = { closing_brace_min_lines = 25 },
  run = { arguments = {} },
  trusted = true,
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
---@return { completion: { autoImport: boolean, callSnippets: string }, inlayHints: { closingBraceMinLines: integer }, trusted: boolean }
function M.server_settings()
  local cfg = M.get()
  return {
    completion = { autoImport = cfg.completion.auto_import, callSnippets = cfg.completion.call_snippets },
    inlayHints = { closingBraceMinLines = cfg.inlay_hints.closing_brace_min_lines },
    trusted = cfg.trusted,
  }
end

---Environment overrides for `jailsp` and `jaic`; empty settings inherit the process environment.
---@return table<string, string>
function M.tool_env()
  local stdlib = M.get().stdlib
  local env = {}
  for setting, variable in pairs({ path = 'JAIC_STDLIB', prelude = 'JAIC_PRELUDE' }) do
    local value = vim.trim(stdlib[setting])
    if value ~= '' then
      env[variable] = vim.fs.normalize(value)
    end
  end
  return env
end

return M
