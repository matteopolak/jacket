---Finding `jailsp`, `jaic`, `jaifmt` and `jailint`: the configured path, then `PATH`, then next
---to the compiler, then the toolchain this plugin downloaded.
local config = require('jai.config')

local M = {}

---@alias jai.Tool 'jaic'|'jailsp'|'jailint'|'jaifmt'
---@alias jai.Source 'setting'|'path'|'next to jaic'|'toolchain'

---@class jai.Found
---@field path string
---@field source jai.Source

---@type table<string, 'compiler'|'server'|'formatter'>
local SETTING = { jaic = 'compiler', jailsp = 'server', jaifmt = 'formatter' }

---@param tool jai.Tool
---@return string
function M.executable_name(tool)
  return vim.uv.os_uname().sysname == 'Windows_NT' and tool .. '.exe' or tool
end

---Where downloaded toolchains live: `<data>/jai/<version>`.
---@return string
function M.root()
  return vim.fs.joinpath(vim.fn.stdpath('data') --[[@as string]], 'jai')
end

---@param version string
---@return string
function M.version_dir(version)
  return vim.fs.joinpath(M.root(), version)
end

---The toolchain version the plugin wants: `toolchain.version`, else the stamped release.
---@return string
function M.wanted_version()
  local configured = config.get().toolchain.version
  return configured ~= '' and configured or require('jai.version')
end

---@param a string
---@param b string
---@return boolean  whether `a` is a newer version than `b`
local function newer(a, b)
  local function parts(v)
    local out = {}
    for n in v:gmatch('%d+') do
      out[#out + 1] = tonumber(n)
    end
    return out
  end
  local x, y = parts(a), parts(b)
  for i = 1, math.max(#x, #y) do
    if (x[i] or 0) ~= (y[i] or 0) then
      return (x[i] or 0) > (y[i] or 0)
    end
  end
  return false
end

---Installed toolchain versions, newest first.
---@return string[]
function M.installed_versions()
  local versions = {}
  local jailsp = M.executable_name('jailsp')
  local wanted = M.wanted_version()
  local expected
  local pinned_version = false
  local pins_ok, pins = pcall(require, 'jai.checksums')
  if pins_ok and pins.version == wanted then
    pinned_version = true
    local uname = vim.uv.os_uname()
    local asset = require('jai.install').asset_for(uname.sysname, uname.machine)
    expected = asset and pins.sha256[asset] or nil
  end
  for name, kind in vim.fs.dir(M.root()) do
    if kind == 'directory' and name:match('^%d+%.%d+%.%d+') and vim.uv.fs_stat(vim.fs.joinpath(M.root(), name, jailsp)) then
      local usable = true
      if name == wanted and pinned_version then
        local marker = vim.fs.joinpath(M.version_dir(name), '.jai-installed.json')
        local ok, data = pcall(function()
          return vim.json.decode(table.concat(vim.fn.readfile(marker), '\n'))
        end)
        usable = expected ~= nil and ok and type(data) == 'table' and data.version == name and data.sha256 == expected
      end
      if usable then versions[#versions + 1] = name end
    end
  end
  table.sort(versions, newer)
  return versions
end

---The folder of the downloaded toolchain to use: the wanted version, else the newest one.
---@return string?
function M.installed_dir()
  local wanted = M.wanted_version()
  local versions = M.installed_versions()
  if vim.list_contains(versions, wanted) then
    return M.version_dir(wanted)
  end
  return versions[1] and M.version_dir(versions[1]) or nil
end

---@param file string
---@return boolean
local function is_file(file)
  local stat = vim.uv.fs_stat(file)
  return stat ~= nil and stat.type == 'file'
end

---A configured path, or nil when the setting is empty or names a missing file (then with a message).
---@param tool jai.Tool
---@return string?, string?
local function configured(tool)
  local key = SETTING[tool]
  local value = key and config.get()[key].path or ''
  if value == '' then
    return nil
  end
  local path = vim.fs.normalize(value)
  if not is_file(path) then
    return nil, ('%s.path is set to %s, which does not exist'):format(key, path)
  end
  return path
end

---@param tool jai.Tool
---@param path string
---@return string?
local function beside(tool, path)
  local candidate = vim.fs.joinpath(vim.fs.dirname(path), M.executable_name(tool))
  return is_file(candidate) and candidate or nil
end

---@param tool jai.Tool
---@return string?
local function on_path(tool)
  local found = vim.fn.exepath(tool)
  return found ~= '' and found or nil
end

---Locates a tool. A configured path that does not exist is an error, never a silent fallback.
---@param tool jai.Tool
---@return jai.Found?, string?
function M.find(tool)
  local path, err = configured(tool)
  if err then
    return nil, err
  end
  if path then
    return { path = path, source = 'setting' }
  end
  path = on_path(tool)
  if path then
    return { path = path, source = 'path' }
  end
  local compiler = configured('jaic') or on_path('jaic')
  path = compiler and beside(tool, compiler)
  if path then
    return { path = path, source = 'next to jaic' }
  end
  local dir = M.installed_dir()
  path = dir and vim.fs.joinpath(dir, M.executable_name(tool))
  if path and is_file(path) then
    return { path = path, source = 'toolchain' }
  end
  return nil
end

return M
