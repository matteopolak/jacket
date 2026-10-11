-- A tiny test harness: `test(name, body)` runs now and records the result.
local M = {
  passed = 0,
  failed = 0,
  skipped = 0,
  ---@type string[]
  failures = {},
  ---@type string
  file = '',
}

---Defines a test; an error (or failed assert) in `body` fails it.
---@param name string
---@param body fun()
function M.test(name, body)
  local ok, err = xpcall(body, debug.traceback)
  if ok then
    M.passed = M.passed + 1
    io.stdout:write(('  ok   %s: %s\n'):format(M.file, name))
  else
    M.failed = M.failed + 1
    M.failures[#M.failures + 1] = ('%s: %s\n%s'):format(M.file, name, tostring(err))
    io.stdout:write(('  FAIL %s: %s\n'):format(M.file, name))
  end
end

---Skips the rest of a spec file.
---@param why string
function M.skip(why)
  M.skipped = M.skipped + 1
  io.stdout:write(('  skip %s: %s\n'):format(M.file, why))
  error({ skip = true })
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
