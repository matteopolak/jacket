-- Loaded with the plugin: `:Jai` and `jailsp` itself (the filetype is in ftdetect/). All of it
-- works without a setup call; see docs/tools/neovim-plugin.md.
if vim.g.loaded_jai_plugin or vim.fn.has('nvim-0.11') == 0 then
  return
end
vim.g.loaded_jai_plugin = true

vim.api.nvim_create_user_command('Jai', function(opts)
  require('jai.commands').run(opts)
end, {
  nargs = 1,
  desc = 'Jai toolchain: install, restart, fix, lint, format, expand, polymorphs, run, build, check, health',
  complete = function(lead)
    return require('jai.commands').complete(lead)
  end,
})

-- The server's custom semantic token type has no default highlight.
vim.api.nvim_set_hl(0, '@lsp.type.formatSpecifier.jai', { link = 'SpecialChar', default = true })

if require('jai.config').get().lsp.enable then
  vim.lsp.enable('jailsp')
end
