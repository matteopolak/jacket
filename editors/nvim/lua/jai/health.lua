---`:checkhealth jai`.
local config = require('jai.config')
local install = require('jai.install')
local toolchain = require('jai.toolchain')

local M = {}

---The first line of `<tool> --version`, or nil.
---@param path string
---@return string?
local function version_of(path)
  local ok, job = pcall(vim.system, { path, '--version' }, { text = true, env = config.tool_env() })
  if not ok then
    return nil
  end
  local out = job:wait(5000)
  local text = vim.trim(out.stdout ~= '' and out.stdout or out.stderr or '')
  return out.code == 0 and text:match('[^\r\n]+') or nil
end

function M.check()
  local health = vim.health
  health.start('Neovim')
  if vim.fn.has('nvim-0.11') == 1 then
    local v = vim.version()
    health.ok(('Neovim %d.%d.%d (0.11+ is required)'):format(v.major, v.minor, v.patch))
  else
    health.error('Neovim 0.11 or newer is required (vim.lsp.config / vim.lsp.enable)')
  end
  if vim.fn.executable('curl') == 1 and vim.fn.executable('tar') == 1 then
    health.ok('curl and tar are available for :Jai install')
  else
    health.warn('curl or tar is missing, so :Jai install cannot download the toolchain')
  end

  health.start('Toolchain')
  local wanted = toolchain.wanted_version()
  health.info('plugin version (toolchain it downloads): ' .. wanted)
  local installed = toolchain.installed_versions()
  health.info('downloaded in ' .. toolchain.root() .. ': ' .. (#installed > 0 and table.concat(installed, ', ') or 'nothing'))
  if not install.asset_for(vim.uv.os_uname().sysname, vim.uv.os_uname().machine) then
    health.warn('there is no prebuilt toolchain for this platform; build from source and put the tools on PATH')
  end
  for _, tool in ipairs({ 'jailsp', 'jaic', 'jaifmt', 'jailint' }) do
    local found, err = toolchain.find(tool)
    if found then
      local version = version_of(found.path)
      health.ok(('%s: %s (%s)%s'):format(tool, found.path, found.source, version and ', ' .. version or ''))
    elseif err then
      health.error(('%s: %s'):format(tool, err))
    elseif tool == 'jailsp' then
      health.error('jailsp not found: put it on PATH, set server.path, or run :Jai install')
    else
      health.warn(('%s not found: put it on PATH or run :Jai install'):format(tool))
    end
  end

  health.start('Language server')
  if vim.lsp.config and vim.lsp.config[ 'jailsp' ] then
    health.ok('vim.lsp.config.jailsp is defined')
  else
    health.error('vim.lsp.config.jailsp is not defined; is the plugin on the runtimepath?')
  end
  if vim.lsp.is_enabled and vim.lsp.is_enabled('jailsp') then
    health.ok('vim.lsp.enable("jailsp") is on')
  else
    health.warn('jailsp is not enabled: call vim.lsp.enable("jailsp") (the plugin does by default)')
  end
  local clients = vim.lsp.get_clients({ name = 'jailsp' })
  if #clients == 0 then
    health.info('no jailsp client is running (open a .jai file to start it)')
  end
  for _, client in ipairs(clients) do
    local info = client.server_info
    health.ok(('attached: client %d, %s %s, root %s, %d buffer(s)'):format(client.id, info and info.name or 'jailsp', info and info.version or '', client.root_dir or '(none)', vim.tbl_count(client.attached_buffers)))
  end
end

return M
