-- Installs the toolchain the way the Neovim plugin does: its own `jai.install`, its own stamped version and its
-- own pinned checksums. Run by .github/workflows/release-smoke.yml with the plugin on the runtimepath:
--
--   nvim --headless -u NONE -i NONE --cmd 'set rtp^=<plugin>' -l release_smoke.lua <tag> <install root>
--
-- Prints the folder holding the tools on success; exits 1 with the plugin's message otherwise.
local tag, root = arg[1], arg[2]
local install = require('jai.install')
local toolchain = require('jai.toolchain')

local version = tag:gsub('^v', '')
local stamped = require('jai.version')
if stamped ~= version then
  io.stderr:write(('the plugin is stamped for %s, expected %s\n'):format(stamped, version))
  os.exit(1)
end
local asset = install.asset_for(vim.uv.os_uname().sysname, vim.uv.os_uname().machine)
if not asset then
  io.stderr:write('no release archive for this machine\n')
  os.exit(1)
end

local finished, failure, dir = false, nil, nil
install.install({
  version = version,
  asset = asset,
  root = root,
  pinned = toolchain.manifest and toolchain.manifest() or nil,
  log = function(message)
    io.stderr:write(message .. '\n')
  end,
}, function(err, installed)
  finished, failure, dir = true, err, installed
end)
vim.wait(300000, function()
  return finished
end, 100)
if not finished or failure then
  io.stderr:write(('install failed: %s\n'):format(failure or 'timed out'))
  os.exit(1)
end
for _, tool in ipairs(install.TOOLS) do
  if not vim.uv.fs_stat(vim.fs.joinpath(dir, toolchain.executable_name(tool))) then
    io.stderr:write(('%s is missing from %s\n'):format(tool, dir))
    os.exit(1)
  end
end
io.stdout:write(dir .. '\n')
os.exit(0)
