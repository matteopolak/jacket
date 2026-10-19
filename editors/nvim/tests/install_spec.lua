local h = require('harness')
local T = h.set()
local install = require('jai.install')

local ZERO = ('0'):rep(64)
local GOOD = ('ab'):rep(32)

T['the manifest is parsed (shared fixture)'] = function()
  local fixture = vim.fs.joinpath(vim.g.jai_test_dir, '..', '..', 'fixtures', install.MANIFEST_FILE)
  local manifest = assert(install.parse_manifest(table.concat(vim.fn.readfile(fixture), '\n')))
  assert(manifest.version == '0.4.0')
  for _, asset in ipairs({ 'jacket-linux-x64.tar.gz', 'jacket-macos-arm64.tar.gz', 'jacket-windows-x64.zip' }) do
    assert(#manifest.sha256[asset] == 64, asset)
  end
end

T['a malformed manifest is rejected and bad hashes are dropped'] = function()
  assert(install.parse_manifest('not json') == nil)
  assert(install.parse_manifest('{"version": 4}') == nil)
  local manifest = assert(install.parse_manifest(vim.json.encode({ version = '1.0.0', sha256 = { a = GOOD:upper(), b = 'xyz' } })))
  assert(manifest.sha256.a == GOOD and manifest.sha256.b == nil, vim.inspect(manifest))
end

T['hash tool output is parsed'] = function()
  assert(install.parse_hash_output(GOOD .. '  file.tgz\n') == GOOD, 'sha256sum')
  assert(install.parse_hash_output('SHA2-256(file.tgz)= ' .. GOOD .. '\n') == GOOD, 'openssl')
  assert(install.parse_hash_output('SHA256 hash of file.tgz:\r\n' .. GOOD .. '\r\nCertUtil: -hashfile command completed successfully.\r\n') == GOOD, 'certutil')
  assert(install.parse_hash_output('nothing here') == nil)
end

T['verification accepts a match and rejects everything else'] = function()
  assert(install.verify(GOOD, GOOD:upper(), 'a'))
  local ok, err = install.verify(GOOD, ZERO, 'jacket-linux-x64.tar.gz')
  assert(not ok and err:find('does not match'), err)
  ok, err = install.verify(GOOD, nil, 'jacket-linux-x64.tar.gz')
  assert(not ok and err:find('no SHA%-256 checksum'), err)
end

T['the archive follows the shared installer vectors (tests/installers/vectors.json)'] = function()
  local file = vim.fs.joinpath(vim.g.jai_test_dir, '..', '..', '..', 'tests', 'installers', 'vectors.json')
  local vectors = vim.json.decode(table.concat(vim.fn.readfile(file), '\n'))
  for _, vector in ipairs(vectors.platforms) do
    local expected = vector.asset
    if expected == vim.NIL then
      expected = nil
    end
    assert(install.asset_for(vector.uname[1], vector.uname[2]) == expected, vim.inspect(vector))
  end
end

T['the archive is chosen by platform'] = function()
  assert(install.asset_for('Darwin', 'arm64') == 'jacket-macos-arm64.tar.gz')
  assert(install.asset_for('Darwin', 'x86_64') == 'jacket-macos-x64.tar.gz')
  assert(install.asset_for('Linux', 'aarch64') == 'jacket-linux-arm64.tar.gz')
  assert(install.asset_for('Linux', 'x86_64') == 'jacket-linux-x64.tar.gz')
  assert(install.asset_for('Windows_NT', 'AMD64') == 'jacket-windows-x64.zip')
  assert(install.asset_for('FreeBSD', 'x86_64') == nil)
  assert(install.asset_for('Linux', 'riscv64') == nil)
end

if h.is_windows() then
  return h.skipped(T, 'the fake release is built with sh and tar')
end

---@param dir string
---@param version string
---@param sums table<string, string>
local function write_manifest(dir, version, sums)
  vim.fn.writefile({ vim.json.encode({ version = version, sha256 = sums }) }, dir .. '/' .. install.MANIFEST_FILE)
end

---Builds a fake release (`<base>/jacket-linux-x64.tar.gz` and its manifest) holding the four tools.
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
  write_manifest(dir, '9.9.9', { ['jacket-linux-x64.tar.gz'] = hash })
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

T['an archive with the right checksum from the release manifest is unpacked into <root>/<version>'] = function()
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
end

T['pinned checksums are used instead of the release manifest'] = function()
  local base, hash = fake_release(ALL)
  os.remove(vim.uri_to_fname(base .. '/' .. install.MANIFEST_FILE))
  local root = h.tmpdir() .. '/jai'
  local pinned = { version = '9.9.9', sha256 = { ['jacket-linux-x64.tar.gz'] = hash } }
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base, pinned = pinned })
  assert(not err, err)
  local marker = vim.json.decode(table.concat(vim.fn.readfile(root .. '/9.9.9/.jai-installed.json'), '\n'))
  assert(marker.version == '9.9.9' and marker.asset == 'jacket-linux-x64.tar.gz' and marker.sha256 == hash, vim.inspect(marker))
end

T['a wrong checksum installs nothing'] = function()
  local base = fake_release(ALL)
  local root = h.tmpdir() .. '/jai'
  write_manifest(vim.uri_to_fname(base), '9.9.9', { ['jacket-linux-x64.tar.gz'] = ZERO })
  local err, dir = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('does not match its checksum'), tostring(err))
  assert(dir == nil)
  assert(#vim.fn.readdir(root) == 0, 'nothing is left behind: ' .. vim.inspect(vim.fn.readdir(root)))
end

T['an archive missing from the release manifest is refused'] = function()
  local base = fake_release(ALL)
  local root = h.tmpdir() .. '/jai'
  write_manifest(vim.uri_to_fname(base), '9.9.9', { ['jacket-other.tar.gz'] = GOOD })
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('no SHA%-256 checksum is known'), tostring(err))
end

T['an archive without a tool is refused'] = function()
  local base = fake_release({ 'jaic', 'jailsp' })
  local root = h.tmpdir() .. '/jai'
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(err and err:find('has no jailint'), tostring(err))
  assert(#vim.fn.readdir(root) == 0)
end

T['an unreachable release is an error, and older versions are removed after a success'] = function()
  local root = h.tmpdir() .. '/jai'
  local err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = 'file:///nonexistent-jai-release' })
  assert(err and err:find('could not download'), tostring(err))
  vim.fn.mkdir(root .. '/1.0.0', 'p')
  local base = fake_release(ALL)
  err = run_install({ version = '9.9.9', asset = 'jacket-linux-x64.tar.gz', root = root, base_url = base })
  assert(not err, err)
  assert(vim.deep_equal(vim.fn.readdir(root), { '9.9.9' }), vim.inspect(vim.fn.readdir(root)))
end

return T
