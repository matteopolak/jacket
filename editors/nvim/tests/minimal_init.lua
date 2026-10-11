-- Headless test setup: only this plugin on the runtimepath, no user config.
--   nvim --headless --clean -u tests/minimal_init.lua -c 'luafile tests/run.lua'
-- Set JAILSP to a built jailsp to run the end-to-end tests too.
local here = vim.fs.dirname(vim.fs.abspath(debug.getinfo(1, 'S').source:sub(2)))
local plugin = vim.fs.dirname(here)

vim.opt.runtimepath:prepend(plugin)
vim.g.jai_test_dir = here
-- Never prompt or download; the server specs enable jailsp themselves.
vim.g.jai = { lsp = { enable = false }, toolchain = { auto_download = 'never' } }
vim.cmd('filetype plugin indent on')
vim.cmd('syntax on')
vim.cmd('runtime! ftdetect/jai.lua')
vim.cmd('runtime! plugin/jai.lua')
