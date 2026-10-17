---Downloading the toolchain release: pick the archive for this machine, check its SHA-256, and
---unpack it into `<data>/jai/<version>`. Uses `curl` and `tar` (Windows 10+ ships both; its
---`tar` reads zip files).
local toolchain = require('jai.toolchain')

local M = {}

M.REPOSITORY = 'matteopolak/jacket'
M.RELEASES_URL = 'https://github.com/' .. M.REPOSITORY .. '/releases'
M.TOOLS = { 'jaic', 'jailsp', 'jailint', 'jaifmt' }

---The release archive for a machine (`uv.os_uname()` values), or nil without a prebuilt one.
---@param sysname string
---@param machine string
---@return string?
function M.asset_for(sysname, machine)
  local os = ({ Darwin = 'macos', Linux = 'linux', Windows_NT = 'windows' })[sysname]
  local m = machine:lower()
  local arch
  if m == 'x86_64' or m == 'amd64' or m == 'x64' then
    arch = 'x64'
  elseif m == 'arm64' or m == 'aarch64' then
    arch = 'arm64'
  end
  if not os or not arch then
    return nil
  end
  return ('jacket-%s-%s.%s'):format(os, arch, os == 'windows' and 'zip' or 'tar.gz')
end

---Parses `sha256sum` output: `<hex>  <name>` (or `<hex> *<name>`) per line.
---@param text string
---@return table<string, string>  lower-case hex by file name
function M.parse_sums(text)
  local sums = {}
  for line in text:gmatch('[^\r\n]+') do
    local hex, name = line:match('^(%x+)%s+%*?(.-)%s*$')
    if hex and #hex == 64 and name ~= '' then
      sums[name] = hex:lower()
    end
  end
  return sums
end

---The hash in the output of `sha256sum`, `shasum`, `openssl dgst` or `certutil -hashfile`.
---@param text string
---@return string?
function M.parse_hash_output(text)
  for line in text:gmatch('[^\r\n]+') do
    -- Old certutil spaces the hex out in pairs.
    local compact = line:gsub('%s', '')
    if #compact == 64 and not compact:find('%X') then
      return compact:lower()
    end
    for token in line:gmatch('%x+') do
      if #token == 64 then
        return token:lower()
      end
    end
  end
  return nil
end

---@param actual string
---@param expected string?
---@param asset string
---@return boolean ok
---@return string? err
function M.verify(actual, expected, asset)
  if not expected then
    return false, ('no SHA-256 checksum is known for %s, so it was not installed'):format(asset)
  end
  if actual:lower() ~= expected:lower() then
    return false, ('%s does not match its checksum (expected %s, got %s); it was not installed'):format(asset, expected, actual)
  end
  return true
end

---@param version string
---@return string
function M.base_url(version)
  return ('https://github.com/%s/releases/download/v%s'):format(M.REPOSITORY, version)
end

---@class jai.InstallOptions
---@field version string
---@field asset string
---@field pinned? { version: string?, sha256: table<string, string> }  checksums shipped with the plugin
---@field root? string  where versions live (default `<data>/jai`)
---@field base_url? string  release download URL (a `file://` folder in tests)
---@field log? fun(msg: string)

---Runs a command; never throws. Calls `cb` on the main loop.
---@param cmd string[]
---@param cb fun(out: { code: integer, stdout: string, stderr: string })
local function run(cmd, cb)
  local ok, err = pcall(vim.system, cmd, { text = true }, vim.schedule_wrap(cb))
  if not ok then
    vim.schedule(function()
      cb({ code = 127, stdout = '', stderr = tostring(err) })
    end)
  end
end

---@return string[][]  commands that print a file's SHA-256, in order of preference
local function hash_commands(file)
  if vim.uv.os_uname().sysname == 'Windows_NT' then
    return { { 'certutil', '-hashfile', file, 'SHA256' } }
  end
  return { { 'sha256sum', file }, { 'shasum', '-a', '256', file }, { 'openssl', 'dgst', '-sha256', file } }
end

---@param file string
---@param cb fun(hash: string?, err: string?)
function M.sha256_file(file, cb)
  local commands = hash_commands(file)
  local function try(i)
    if not commands[i] then
      return cb(nil, 'no tool to compute a SHA-256 was found (tried sha256sum, shasum, openssl)')
    end
    run(commands[i], function(out)
      local hash = out.code == 0 and M.parse_hash_output(out.stdout) or nil
      if hash then
        cb(hash)
      else
        try(i + 1)
      end
    end)
  end
  try(1)
end

---@return string
local function tar_program()
  if vim.uv.os_uname().sysname == 'Windows_NT' then
    return vim.fs.joinpath(vim.env.SystemRoot or 'C:\\Windows', 'System32', 'tar.exe')
  end
  return 'tar'
end

---@param dir string
local function remove(dir)
  vim.fn.delete(dir, 'rf')
end

---Downloads, verifies and unpacks a release. `done(nil, dir)` when `dir` holds the tools,
---`done(err)` otherwise (nothing is left behind then).
---@param opts jai.InstallOptions
---@param done fun(err: string?, dir: string?)
function M.install(opts, done)
  local log = opts.log or function() end
  local root = opts.root or toolchain.root()
  local base = opts.base_url or M.base_url(opts.version)
  local asset = opts.asset
  vim.fn.mkdir(root, 'p')
  local staging = vim.fs.joinpath(root, ('.staging-%s-%x'):format(opts.version, vim.uv.hrtime() % 0xffffff))
  vim.fn.mkdir(staging, 'p')

  local function fail(err)
    remove(staging)
    done(err)
  end

  local function curl(url, dest, cb)
    run({ 'curl', '--fail', '--location', '--silent', '--show-error', '--retry', '2', '--output', dest, url }, function(out)
      if out.code ~= 0 then
        local detail = vim.trim(out.stderr)
        return cb(('could not download %s%s; check the connection, or install the toolchain by hand from %s'):format(url, detail ~= '' and ' (' .. detail .. ')' or '', M.RELEASES_URL))
      end
      cb(nil)
    end)
  end

  local function unpack(archive, actual_hash)
    local unpacked = vim.fs.joinpath(staging, 'unpacked')
    vim.fn.mkdir(unpacked, 'p')
    run({ tar_program(), '-xf', archive, '-C', unpacked, '--strip-components=1' }, function(out)
      if out.code ~= 0 then
        return fail(('could not unpack %s: %s'):format(asset, vim.trim(out.stderr)))
      end
      for _, tool in ipairs(M.TOOLS) do
        if not vim.uv.fs_stat(vim.fs.joinpath(unpacked, toolchain.executable_name(tool))) then
          return fail(('%s has no %s'):format(asset, toolchain.executable_name(tool)))
        end
      end
      vim.fn.writefile({ vim.json.encode({ version = opts.version, asset = asset, sha256 = actual_hash }) }, vim.fs.joinpath(unpacked, '.jai-installed.json'))
      local target = vim.fs.joinpath(root, opts.version)
      remove(target)
      local renamed, err = vim.uv.fs_rename(unpacked, target)
      if not renamed then
        return fail('could not move the toolchain into place: ' .. tostring(err))
      end
      remove(staging)
      -- Older versions and leftovers of interrupted downloads.
      for name in vim.fs.dir(root) do
        if name ~= opts.version and (name:match('^%d+%.%d+%.%d+') or name:match('^%.staging%-')) then
          remove(vim.fs.joinpath(root, name))
        end
      end
      done(nil, target)
    end)
  end

  local function download_and_check(expected)
    local archive = vim.fs.joinpath(staging, asset)
    log(('downloading %s/%s'):format(base, asset))
    curl(base .. '/' .. asset, archive, function(err)
      if err then
        return fail(err)
      end
      M.sha256_file(archive, function(actual, hash_err)
        if not actual then
          return fail(hash_err)
        end
        local ok, verify_err = M.verify(actual, expected, asset)
        if not ok then
          return fail(verify_err)
        end
        log(('%s: SHA-256 %s (verified)'):format(asset, actual))
        unpack(archive, actual)
      end)
    end)
  end

  local pinned = opts.pinned
  if pinned and pinned.version == opts.version then
    if not pinned.sha256[asset] then
      return fail(('jaic %s has no %s release archive'):format(opts.version, asset))
    end
    return download_and_check(pinned.sha256[asset])
  end
  log('no checksums are pinned for ' .. opts.version .. '; using the release SHA256SUMS')
  local sums_file = vim.fs.joinpath(staging, 'SHA256SUMS')
  curl(base .. '/SHA256SUMS', sums_file, function(err)
    if err then
      return fail(err)
    end
    local lines = vim.fn.readfile(sums_file)
    download_and_check(M.parse_sums(table.concat(lines, '\n'))[asset])
  end)
end

return M
