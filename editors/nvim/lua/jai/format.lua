---Formatting with `jaifmt --stdin`. jailsp has no `textDocument/formatting`, so the plugin runs
---the formatter itself and applies the result as line edits, which keeps folds and marks
---outside the changed lines.
local toolchain = require('jai.toolchain')

local M = {}

M.CONFIG_FILE = 'jaifmt.toml'

---The nearest `jaifmt.toml` in `dir` or above it.
---@param dir string
---@return string?
function M.find_config(dir)
  local found = vim.fs.find(M.CONFIG_FILE, { path = dir, upward = true, type = 'file', limit = 1 })[1]
  return found
end

---The `ignore = [...]` globs of a `jaifmt.toml` (the one-line string array jaifmt accepts).
---@param toml string
---@return string[]
function M.ignore_globs(toml)
  for raw in toml:gmatch('[^\r\n]+') do
    local list = raw:match('^%s*ignore%s*=%s*%[(.*)%]%s*#?.*$')
    if list then
      local globs = {}
      for glob in list:gmatch('"([^"]*)"') do
        globs[#globs + 1] = glob
      end
      return globs
    end
  end
  return {}
end

---Whether the config's `ignore` globs cover `file`. A glob naming a directory covers
---everything below it.
---@param config_path string
---@param globs string[]
---@param file string
---@return boolean
function M.is_ignored(config_path, globs, file)
  local base = vim.fs.dirname(config_path)
  local relative = vim.fs.relpath(base, file)
  if not relative or relative:sub(1, 3) == '../' then
    return false
  end
  relative = relative:gsub('\\', '/')
  local prefixes, at = {}, 0
  while true do
    at = relative:find('/', at + 1, true)
    prefixes[#prefixes + 1] = at and relative:sub(1, at - 1) or relative
    if not at then
      break
    end
  end
  for _, glob in ipairs(globs) do
    local pattern = vim.glob.to_lpeg((glob:gsub('^%./', '')))
    for _, prefix in ipairs(prefixes) do
      if pattern:match(prefix) then
        return true
      end
    end
  end
  return false
end

---Replaces the lines of `bufnr` that differ from `formatted`.
---@param bufnr integer
---@param formatted string[]
local function apply(bufnr, formatted)
  local before = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local hunks = vim.text.diff(table.concat(before, '\n') .. '\n', table.concat(formatted, '\n') .. '\n', { result_type = 'indices' }) --[[@as integer[][] ]]
  for i = #hunks, 1, -1 do
    local start_a, count_a, start_b, count_b = unpack(hunks[i])
    -- A zero count means an insertion *after* line `start`.
    local first = count_a == 0 and start_a or start_a - 1
    local replacement = count_b == 0 and {} or vim.list_slice(formatted, start_b, start_b + count_b - 1)
    vim.api.nvim_buf_set_lines(bufnr, first, first + count_a, false, replacement)
  end
end

---Formats a buffer with `jaifmt`, waiting for it. Buffers ignored by `jaifmt.toml` are left alone.
---@param bufnr? integer
---@return boolean changed
---@return string? err
function M.format(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local name = vim.api.nvim_buf_get_name(bufnr)
  local dir = name ~= '' and vim.fs.dirname(name) or vim.uv.cwd() --[[@as string]]
  local formatter = toolchain.find('jaifmt')
  if not formatter then
    return false, 'jaifmt was not found: set formatter.path, put jaifmt on PATH, or run :Jai install'
  end
  local config_path = M.find_config(dir)
  if config_path and name ~= '' then
    local ok, lines = pcall(vim.fn.readfile, config_path)
    if ok and M.is_ignored(config_path, M.ignore_globs(table.concat(lines, '\n')), name) then
      return false
    end
  end
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local text = table.concat(lines, '\n') .. '\n'
  local ok, job = pcall(vim.system, { formatter.path, '--stdin' }, { stdin = text, cwd = dir, text = true })
  if not ok then
    return false, 'could not run jaifmt: ' .. tostring(job)
  end
  local out = job:wait(10000)
  if out.code ~= 0 then
    -- Unbalanced brackets or text that does not lex: jaifmt leaves it alone and says why.
    local reason = vim.trim(out.stderr or ''):gsub('%s*\n%s*', ' ')
    return false, 'jaifmt: ' .. (reason ~= '' and reason or ('exit status ' .. out.code))
  end
  if out.stdout == text then
    return false
  end
  local formatted = vim.split(out.stdout:gsub('\n$', ''), '\n', { plain = true })
  apply(bufnr, formatted)
  return true
end

return M
