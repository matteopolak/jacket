-- Buffer settings for Jai files: comments, jaifmt's default indentation, format on save.
vim.bo.commentstring = '// %s'
vim.bo.comments = 's1:/*,mb:*,ex:*/,://'
vim.bo.formatoptions = vim.bo.formatoptions:gsub('[ao]', '') .. 'cqj'
-- jaifmt's defaults: four spaces.
vim.bo.expandtab = true
vim.bo.shiftwidth = 4
vim.bo.softtabstop = 4
vim.bo.tabstop = 4
vim.bo.cindent = true
vim.bo.cinoptions = 'l1,(s,m1'
vim.bo.suffixesadd = '.jai'
vim.b.undo_ftplugin = 'setl commentstring< comments< formatoptions< expandtab< shiftwidth< softtabstop< tabstop< cindent< cinoptions< suffixesadd<'

if require('jai.config').get().format_on_save then
  local group = vim.api.nvim_create_augroup('JaiFormatOnSave' .. vim.api.nvim_get_current_buf(), { clear = true })
  vim.api.nvim_create_autocmd('BufWritePre', {
    group = group,
    buffer = 0,
    desc = 'Format with jaifmt',
    callback = function(args)
      local _, err = require('jai.format').format(args.buf)
      if err then
        vim.notify('Jai: ' .. err, vim.log.levels.WARN)
      end
    end,
  })
end
