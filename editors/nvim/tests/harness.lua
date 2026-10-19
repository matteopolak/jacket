-- A tiny test harness: `test(name, body)` runs now and records the result.
local M = {}

---A new test set for a spec file: `local T = h.set()`, `T['name'] = function() ... end`, `return T`.
---@param opts? table  mini.test set options (hooks, parametrize)
---@return table
function M.set(opts)
  return require('mini.test').new_set(opts)
end

---Marks the whole spec file skipped: `if cond then return h.skipped(T, 'why') end` before the rest
---of the file's set-up runs.
---@param set table
---@param why string
---@return table
function M.skipped(set, why)
  set['skipped: ' .. why] = function()
    require('mini.test').skip(why)
  end
  return set
end

---Waits (processing events) until `cond()` is truthy.
---@param timeout integer  milliseconds
---@param cond fun(): any
---@param message string
function M.wait_for(timeout, cond, message)
  local ok = vim.wait(timeout, function()
    return cond() and true or false
  end, 20)
  assert(ok, 'timed out waiting for ' .. message)
end

---A fresh temporary directory.
---@return string
function M.tmpdir()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, 'p')
  return dir
end

---Writes an executable shell script (Unix only).
---@param path string
---@param body string
function M.script(path, body)
  vim.fn.writefile({ '#!/bin/sh', body }, path)
  vim.uv.fs_chmod(path, tonumber('755', 8))
end

---@return boolean
function M.is_windows()
  return vim.uv.os_uname().sysname == 'Windows_NT'
end

return M
