---Jai support for Neovim: starts `jailsp`, finds or downloads the toolchain, formats with
---`jaifmt`. Everything works without calling `setup()`.
local config = require('jai.config')
local install = require('jai.install')
local toolchain = require('jai.toolchain')

local M = {}

---The name of the LSP configuration in `lsp/jailsp.lua`.
M.SERVER = 'jailsp'

local DECLINED = 'jai-declined-toolchain'

---@param msg string
---@param level? integer
local function notify(msg, level)
  vim.notify('Jai: ' .. msg, level or vim.log.levels.INFO)
end

---Overrides settings (see `jai.Config`). Running servers get the new settings at once.
---@param opts? table
function M.setup(opts)
  config.set(opts)
  for _, client in ipairs(vim.lsp.get_clients({ name = M.SERVER })) do
    local settings = { jai = config.server_settings() }
    client.settings = settings
    client:notify('workspace/didChangeConfiguration', { settings = settings })
  end
end

---`cmd` of the LSP configuration: looks the server up when a Jai buffer needs it, so installing
---the toolchain later takes effect on the next start. Without a server it offers the download
---and stops (the editor then says the server is not running).
---@param dispatchers vim.lsp.rpc.Dispatchers
---@param cfg vim.lsp.ClientConfig
---@return vim.lsp.rpc.PublicClient
function M.server_cmd(dispatchers, cfg)
  local found, err = toolchain.find('jailsp')
  if not found then
    M.offer_install(err)
    error(err or 'jailsp was not found; run :Jai install', 0)
  end
  return vim.lsp.rpc.start({ found.path }, dispatchers, {
    cwd = cfg.cmd_cwd,
    env = vim.tbl_extend('force', config.tool_env(), cfg.cmd_env or {}),
    detached = cfg.detached,
  })
end

---@return string
local function declined_file()
  return vim.fs.joinpath(vim.fn.stdpath('state') --[[@as string]], DECLINED)
end

local offered = false

---Called when no `jailsp` exists. Follows `toolchain.auto_download`: asks first (once per
---session, and not again after "Don't ask again" for this version), downloads without asking
---for `always`, and only explains for `never`.
---@param problem? string  a configured path that does not exist: never answered with a download
function M.offer_install(problem)
  if offered then
    return
  end
  offered = true
  local mode = config.get().toolchain.auto_download
  local version = toolchain.wanted_version()
  vim.schedule(function()
    if problem then
      return notify(problem .. '. Fix the setting, or clear it to use the toolchain on PATH.', vim.log.levels.ERROR)
    end
    local hint = 'jailsp (the Jai language server) was not found, so there is no checking, completion or linting. Put the toolchain from '
      .. install.RELEASES_URL
      .. ' on PATH, set server.path, or run :Jai install.'
    local declined = vim.fn.filereadable(declined_file()) == 1 and vim.fn.readfile(declined_file())[1] == version
    if mode == 'never' or (mode == 'prompt' and declined) then
      return notify(hint, vim.log.levels.WARN)
    end
    if mode == 'always' then
      return M.install()
    end
    vim.ui.select({ 'Download', 'Not now', "Don't ask again" }, {
      prompt = ('Jai: no jailsp was found. Download jaic %s (jaic, jailsp, jailint, jaifmt, stdlib) from github.com/%s?'):format(version, install.REPOSITORY),
    }, function(choice)
      if choice == 'Download' then
        M.install()
      else
        if choice == "Don't ask again" then
          vim.fn.writefile({ version }, declined_file())
        end
        notify(hint, vim.log.levels.WARN)
      end
    end)
  end)
end

---Downloads the toolchain for this plugin's version, then restarts the server.
---@param opts? { on_done?: fun(err: string?, dir: string?) }
function M.install(opts)
  opts = opts or {}
  local sysname, machine = vim.uv.os_uname().sysname, vim.uv.os_uname().machine
  local asset = install.asset_for(sysname, machine)
  local version = toolchain.wanted_version()
  if not asset then
    local msg = ('there is no prebuilt toolchain for %s %s. Build jaic, jailsp and jailint from source (https://github.com/%s) and put them on PATH.'):format(sysname, machine, install.REPOSITORY)
    notify(msg, vim.log.levels.WARN)
    return opts.on_done and opts.on_done(msg)
  end
  local platform = asset:gsub('^jai%-', ''):gsub('%.tar%.gz$', ''):gsub('%.zip$', '')
  notify(('downloading jaic %s for %s...'):format(version, platform))
  install.install({
    version = version,
    asset = asset,
    pinned = require('jai.checksums'),
  }, function(err, dir)
    if err then
      notify('could not install the toolchain: ' .. err, vim.log.levels.ERROR)
    else
      notify('installed jaic ' .. version .. ' in ' .. dir)
      offered = false
      M.restart()
    end
    if opts.on_done then
      opts.on_done(err, dir)
    end
  end)
end

---Stops `jailsp` and starts it again for the open Jai buffers.
---@param cb? fun()
function M.restart(cb)
  vim.lsp.enable(M.SERVER, false)
  local waited = 0
  local function start()
    if #vim.lsp.get_clients({ name = M.SERVER }) > 0 and waited < 5000 then
      waited = waited + 50
      return vim.defer_fn(start, 50)
    end
    vim.lsp.enable(M.SERVER)
    -- enable() only attaches to open buffers once the editor has started.
    for _, buf in ipairs(vim.api.nvim_list_bufs()) do
      if vim.api.nvim_buf_is_loaded(buf) and vim.bo[buf].filetype == 'jai' and #vim.lsp.get_clients({ bufnr = buf, name = M.SERVER }) == 0 then
        pcall(vim.api.nvim_exec_autocmds, 'FileType', { group = 'nvim.lsp.enable', buffer = buf })
      end
    end
    if cb then
      cb()
    end
  end
  start()
end

return M
