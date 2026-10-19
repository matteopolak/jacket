---Formatting with `jaifmt --stdin`. jailsp has no `textDocument/formatting`, so the plugin runs
---the formatter itself and applies the result as line edits, which keeps folds and marks
---outside the changed lines.
local toolchain = require('jai.toolchain')

local M = {}

---Whether the `[format]` ignore list of the config that applies to `file` covers it, asked of
---`jaifmt --is-ignored` (exit 0 ignored, 1 formatted) so the globs have one implementation. Any
---other outcome counts as not ignored; formatting then reports the problem.
---@param formatter string  the jaifmt executable
---@param file string  absolute path
---@return boolean
function M.is_ignored(formatter, file)
  local ok, job = pcall(vim.system, { formatter, '--is-ignored', file })
  return ok and job:wait(10000).code == 0
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

---Formats a buffer with `jaifmt`, waiting for it. Buffers the config's ignore list covers are left alone.
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
  if name ~= '' and M.is_ignored(formatter.path, name) then
    return false
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
