local h = require('harness')
local install = require('jai.install')

local ZERO = ('0'):rep(64)
local GOOD = ('ab'):rep(32)

h.test('SHA256SUMS lines are parsed', function()
  local sums = install.parse_sums(table.concat({
    GOOD .. '  jacket-linux-x64.tar.gz',
    ZERO:upper() .. ' *jacket-windows-x64.zip',
    'not a checksum line',
    'abc123  short.tar.gz',
  }, '\n'))
  assert(sums['jacket-linux-x64.tar.gz'] == GOOD)
  assert(sums['jacket-windows-x64.zip'] == ZERO, 'binary-mode marker and upper case')
  assert(sums['short.tar.gz'] == nil, 'too short to be a SHA-256')
end)

h.test('hash tool output is parsed', function()
  assert(install.parse_hash_output(GOOD .. '  file.tgz\n') == GOOD, 'sha256sum')
  assert(install.parse_hash_output('SHA2-256(file.tgz)= ' .. GOOD .. '\n') == GOOD, 'openssl')
  assert(install.parse_hash_output('SHA256 hash of file.tgz:\r\n' .. GOOD .. '\r\nCertUtil: -hashfile command completed successfully.\r\n') == GOOD, 'certutil')
  assert(install.parse_hash_output('nothing here') == nil)
end)

h.test('verification accepts a match and rejects everything else', function()
  assert(install.verify(GOOD, GOOD:upper(), 'a'))
  local ok, err = install.verify(GOOD, ZERO, 'jacket-linux-x64.tar.gz')
  assert(not ok and err:find('does not match'), err)
  ok, err = install.verify(GOOD, nil, 'jacket-linux-x64.tar.gz')
  assert(not ok and err:find('no SHA%-256 checksum'), err)
end)

h.test('the archive is chosen by platform', function()
  assert(install.asset_for('Darwin', 'arm64') == 'jacket-macos-arm64.tar.gz')
  assert(install.asset_for('Darwin', 'x86_64') == 'jacket-macos-x64.tar.gz')
  assert(install.asset_for('Linux', 'aarch64') == 'jacket-linux-arm64.tar.gz')
  assert(install.asset_for('Linux', 'x86_64') == 'jacket-linux-x64.tar.gz')
  assert(install.asset_for('Windows_NT', 'AMD64') == 'jacket-windows-x64.zip')
  assert(install.asset_for('FreeBSD', 'x86_64') == nil)
  assert(install.asset_for('Linux', 'riscv64') == nil)
end)

if h.is_windows() then
  h.skip('the fake release is built with sh and tar')
end

---Builds a fake release (`<base>/jacket-linux-x64.tar.gz` and `SHA256SUMS`) holding the four tools.
---@param tools string[]
---@return string base  a file:// URL
---@return string hash  of the archive
local function fake_release(tools)
  local dir = h.tmpdir()
  local pack = dir .. '/pack/jacket-linux-x64'
  vim.fn.mkdir(pack, 'p')
  for _, tool in ipairs(tools) do
    h.script(pack .. '/' .. tool, 'echo fake ' .. tool)
  end
  vim.fn.mkdir(pack .. '/stdlib/prelude', 'p')
  vim.fn.writefile({ '' }, pack .. '/stdlib/prelude/module.jai')
  local made = vim.system({ 'tar', '-czf', dir .. '/jacket-linux-x64.tar.gz', '-C', dir .. '/pack', 'jacket-linux-x64' }):wait()
  assert(made.code == 0, made.stderr)
  local hash
  install.sha256_file(dir .. '/jacket-linux-x64.tar.gz', function(value)
    hash = value
  end)
  h.wait_for(5000, function()
    return hash
  end, 'the archive hash')
  vim.fn.writefile({ hash .. '  jacket-linux-x64.tar.gz' }, dir .. '/SHA256SUMS')
  return 'file://' .. dir, hash
end

---Runs an install and waits for it.
---@param opts jai.InstallOptions
---@return string? err
---@return string? dir
local function run_install(opts)
  local finished, result = false, {}
  install.install(opts, function(err, dir)
    finished, result = true, { err, dir }
  end)
  h.wait_for(15000, function()
    return finished
  end, 'the install')
  return result[1], result[2]
end

local ALL = { 'jaic', 'jailsp', 'jailint', 'jaifmt' }

h.test('an archive with the right checksum from SHA256SUMS is unpacked into <root>/<version>', function()
  local base = fake_release(ALL)
  local root = h.tmpdir() .. '/jai'
  local err, dir = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(not err, err)
  assert(dir == root .. '/9.9.9', dir)
  for _, tool in ipairs(ALL) do
    assert(vim.uv.fs_stat(dir .. '/' .. tool), tool)
  end
  assert(vim.uv.fs_stat(dir .. '/stdlib/prelude/module.jai'), 'the whole archive is unpacked, stdlib included')
  assert(#vim.fn.readdir(root) == 1, 'no staging folder is left: ' .. vim.inspect(vim.fn.readdir(root)))
end)

h.test('pinned checksums are used instead of SHA256SUMS', function()
  local base, hash = fake_release(ALL)
  os.remove(vim.uri_to_fname(base .. '/SHA256SUMS'))
  local root = h.tmpdir() .. '/jai'
  local pinned = { version = '9.9.9', sha256 = { ['jacket-linux-x64.tar.gz'] = hash } }
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base, pinned = pinned })
  assert(not err, err)
  local marker = vim.json.decode(table.concat(vim.fn.readfile(root .. '/9.9.9/.jai-installed.json'), '\n'))
  assert(marker.version == '9.9.9' and marker.asset == 'jacket-linux-x64.tar.gz' and marker.sha256 == hash, vim.inspect(marker))
end)

h.test('a wrong checksum installs nothing', function()
  local base = fake_release(ALL)
  local root = h.tmpdir() .. '/jai'
  vim.fn.writefile({ ZERO .. '  jacket-linux-x64.tar.gz' }, vim.uri_to_fname(base) .. '/SHA256SUMS')
  local err, dir = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('does not match its checksum'), tostring(err))
  assert(dir == nil)
  assert(#vim.fn.readdir(root) == 0, 'nothing is left behind: ' .. vim.inspect(vim.fn.readdir(root)))
end)

h.test('an archive missing from SHA256SUMS is refused', function()
  local base = fake_release(ALL)
  local root = h.tmpdir() .. '/jai'
  vim.fn.writefile({ GOOD .. '  jacket-other.tar.gz' }, vim.uri_to_fname(base) .. '/SHA256SUMS')
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('no SHA%-256 checksum is known'), tostring(err))
end)

h.test('an archive without a tool is refused', function()
  local base = fake_release({ 'jaic', 'jailsp' })
  local root = h.tmpdir() .. '/jai'
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('has no jailint'), tostring(err))
  assert(#vim.fn.readdir(root) == 0)
end)

h.test('an unreachable release is an error, and older versions are removed after a success', function()
  local root = h.tmpdir() .. '/jai'
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = 'file:///nonexistent-jai-release' })
  assert(err and err:find('could not download'), tostring(err))
  vim.fn.mkdir(root .. '/1.0.0', 'p')
  local base = fake_release(ALL)
  err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(not err, err)
  assert(vim.deep_equal(vim.fn.readdir(root), { '9.9.9' }), vim.inspect(vim.fn.readdir(root)))
end)
