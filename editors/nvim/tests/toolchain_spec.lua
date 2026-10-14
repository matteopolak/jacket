local h = require('harness')

if h.is_windows() then
  h.skip('uses shell-script stand-ins for the tools')
end

local config = require('jai.config')
local toolchain = require('jai.toolchain')
local install = require('jai.install')

local saved_path = vim.env.PATH
local saved_data = vim.env.XDG_DATA_HOME

---Runs `body` with an empty PATH and an empty data directory.
---@param body fun(data: string)
local function isolated(body)
  local data = h.tmpdir()
  vim.env.XDG_DATA_HOME = data
  vim.env.PATH = h.tmpdir() .. ':/usr/bin:/bin'
  vim.g.jai = { lsp = { enable = false }, toolchain = { auto_download = 'never' } }
  config.set(nil)
  local ok, err = pcall(body, data)
  vim.env.PATH, vim.env.XDG_DATA_HOME = saved_path, saved_data
  assert(ok, err)
end

---@param dir string
---@param names string[]
local function make_tools(dir, names)
  vim.fn.mkdir(dir, 'p')
  for _, name in ipairs(names) do
    h.script(dir .. '/' .. name, 'echo ' .. name .. ' 1.2.3')
  end
end

h.test('the lsp config is defined by the plugin and needs no setup', function()
  local cfg = vim.lsp.config['jailsp']
  assert(cfg, 'vim.lsp.config.jailsp')
  assert(vim.deep_equal(cfg.filetypes, { 'jai' }))
  assert(type(cfg.cmd) == 'function')
  assert(vim.tbl_contains(cfg.root_markers, 'jai.toml'))
end)

h.test('nothing is found when nothing is installed', function()
  isolated(function()
    assert(toolchain.find('jailsp') == nil)
  end)
end)

h.test('a tool on PATH is found', function()
  isolated(function()
    local bin = h.tmpdir()
    make_tools(bin, { 'jailsp' })
    vim.env.PATH = bin .. ':' .. vim.env.PATH
    local found = assert(toolchain.find('jailsp'))
    assert(found.source == 'path' and vim.fs.normalize(found.path) == vim.fs.normalize(bin .. '/jailsp'), vim.inspect(found))
  end)
end)

h.test('the configured path beats PATH, and a missing one is an error', function()
  isolated(function()
    local bin, mine = h.tmpdir(), h.tmpdir()
    make_tools(bin, { 'jailsp' })
    make_tools(mine, { 'jailsp' })
    vim.env.PATH = bin .. ':' .. vim.env.PATH
    config.set({ server = { path = mine .. '/jailsp' } })
    local found = assert(toolchain.find('jailsp'))
    assert(found.source == 'setting' and found.path == vim.fs.normalize(mine .. '/jailsp'))
    config.set({ server = { path = mine .. '/nope' } })
    local none, err = toolchain.find('jailsp')
    assert(none == nil and err and err:find('does not exist'), tostring(err))
    config.set(nil)
  end)
end)

h.test('tools are found next to jaic', function()
  isolated(function()
    local bin = h.tmpdir()
    make_tools(bin, { 'jaic', 'jaifmt' })
    config.set({ compiler = { path = bin .. '/jaic' } })
    local found = assert(toolchain.find('jaifmt'))
    assert(found.source == 'next to jaic', vim.inspect(found))
    config.set(nil)
  end)
end)

h.test('the downloaded toolchain is the last resort; the wanted version wins', function()
  isolated(function()
    local version = toolchain.wanted_version()
    make_tools(toolchain.version_dir('0.0.1'), { 'jailsp' })
    local old = assert(toolchain.find('jailsp'))
    assert(old.source == 'toolchain' and old.path:find('0.0.1', 1, true), old.path)
    make_tools(toolchain.version_dir(version), { 'jailsp' })
    local wanted = assert(toolchain.find('jailsp'))
    assert(wanted.path:find(version, 1, true), wanted.path)
    assert(toolchain.installed_versions()[1] == version, 'newest first')
  end)
end)

h.test('a cached toolchain with an obsolete pinned checksum is not selected', function()
  isolated(function()
    local version = toolchain.wanted_version()
    local uname = vim.uv.os_uname()
    local asset = assert(install.asset_for(uname.sysname, uname.machine))
    local dir = toolchain.version_dir(version)
    make_tools(dir, { 'jailsp' })
    vim.fn.writefile({ vim.json.encode({ version = version, asset = asset, sha256 = ('a'):rep(64) }) }, dir .. '/.jai-installed.json')
    local prior = package.loaded['jai.checksums']
    package.loaded['jai.checksums'] = { version = version, sha256 = { [asset] = ('b'):rep(64) } }
    local ok, versions = pcall(toolchain.installed_versions)
    package.loaded['jai.checksums'] = prior
    assert(ok, versions)
    assert(not vim.list_contains(versions, version), vim.inspect(versions))
  end)
end)

h.test('missing or malformed cache markers do not break checksum validation', function()
  isolated(function()
    local version = toolchain.wanted_version()
    local uname = vim.uv.os_uname()
    local asset = assert(install.asset_for(uname.sysname, uname.machine))
    local dir = toolchain.version_dir(version)
    make_tools(dir, { 'jailsp' })
    local prior = package.loaded['jai.checksums']
    package.loaded['jai.checksums'] = { version = version, sha256 = { [asset] = ('b'):rep(64) } }
    for _, contents in ipairs({ false, '{ invalid json' }) do
      local marker = dir .. '/.jai-installed.json'
      os.remove(marker)
      if contents then vim.fn.writefile({ contents }, marker) end
      local ok, versions = pcall(toolchain.installed_versions)
      assert(ok, versions)
      assert(not vim.list_contains(versions, version), vim.inspect(versions))
    end
    package.loaded['jai.checksums'] = prior
  end)
end)

h.test('the server command starts the resolved binary', function()
  isolated(function()
    local bin = h.tmpdir()
    make_tools(bin, { 'jailsp' })
    config.set({ server = { path = bin .. '/jailsp' }, stdlib = { path = '/stdlib/here' } })
    local started
    local real = vim.lsp.rpc.start
    ---@diagnostic disable-next-line: duplicate-set-field
    vim.lsp.rpc.start = function(cmd, _, params)
      started = { cmd = cmd, env = params.env }
      return {}
    end
    require('jai').server_cmd({}, {})
    vim.lsp.rpc.start = real
    config.set(nil)
    assert(started.cmd[1] == vim.fs.normalize(bin .. '/jailsp'), vim.inspect(started))
    assert(started.env.JAIC_STDLIB == '/stdlib/here')
  end)
end)

h.test('the server command explains a missing jailsp instead of starting', function()
  isolated(function()
    local ok, err = pcall(require('jai').server_cmd, {}, {})
    assert(not ok and tostring(err):find('jailsp was not found', 1, true), tostring(err))
  end)
end)

h.test(':checkhealth jai runs', function()
  vim.cmd('checkhealth jai')
  local text = table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), '\n')
  vim.cmd('bwipeout!')
  assert(text:find('Neovim', 1, true) and text:find('jailsp', 1, true), text)
end)
