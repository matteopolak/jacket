---The `jai/compileTimeLimit` notification: `jailsp` stopped compile-time code at a limit. The
---message says where and how to raise the limit (the same wording the VS Code popup has).
local config = require('jai.config')

local M = {}

---@class jai.LimitNotice
---@field kind 'budget'|'time'
---@field uri string
---@field range { start: { line: integer, character: integer } }
---@field message string

---Notices already shown; cleared when a buffer is written, since the server repeats a notice
---after every check that hits the limit.
---@type table<string, true>
local seen = {}

---@param notice jai.LimitNotice
---@return string
local function place(notice)
  local file = vim.fn.fnamemodify(vim.uri_to_fname(notice.uri), ':~:.')
  return ('%s:%d'):format(file, notice.range.start.line + 1)
end

---The text shown for `notice`.
---@param notice jai.LimitNotice
---@return string
function M.text(notice)
  local limits = config.get().compile_time
  if notice.kind == 'time' then
    return ('checking stopped after %s s, so the results are partial (compile-time code at %s was running). '
      .. "Raise compile_time.check_time_limit in require('jai').setup() (0 for no limit), or run :Jai costs to see what is slow."):format(
      limits.check_time_limit,
      place(notice)
    )
  end
  return ('the compile-time code at %s ran past its budget of %d basic blocks and was stopped; the rest of the program was still checked. '
    .. "Raise compile_time.evaluation_budget in require('jai').setup() (0 for no limit), or run :Jai costs to see what is slow."):format(
    place(notice),
    limits.evaluation_budget
  )
end

---Shows a notice once until the next write.
---@param notice jai.LimitNotice
function M.notice(notice)
  local key = ('%s|%s|%d:%d'):format(notice.kind, notice.uri, notice.range.start.line, notice.range.start.character)
  if seen[key] then
    return
  end
  seen[key] = true
  if not M.group then
    M.group = vim.api.nvim_create_augroup('jai-limit-notices', { clear = true })
    vim.api.nvim_create_autocmd('BufWritePost', {
      group = M.group,
      callback = function()
        seen = {}
      end,
    })
  end
  vim.schedule(function()
    vim.notify('Jai: ' .. M.text(notice), vim.log.levels.WARN)
  end)
end

---Forgets what was shown (for tests).
function M.reset()
  seen = {}
end

return M
