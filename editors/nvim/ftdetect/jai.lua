-- Neovim (up to at least 0.12) has no `*.jai` rule; register one unless a newer version has.
if vim.filetype.match({ filename = 'a.jai' }) == nil then
  vim.filetype.add({ extension = { jai = 'jai' } })
end
