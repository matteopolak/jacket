local h = require('harness')
local T = h.set()
local format = require('jai.format')

if not vim.env.JAIFMT or vim.env.JAIFMT == '' then
  return h.skipped(T, 'set JAIFMT to a jaifmt executable')
end

T['the ignore list is asked of jaifmt --is-ignored'] = function()
  local jaifmt = vim.env.JAIFMT
  local dir = vim.fn.resolve(vim.fn.tempname())
  vim.fn.mkdir(dir .. '/gen/deep', 'p')
  vim.fn.writefile({ '[format]', 'ignore = ["gen/*.jai", "vendor"]' }, dir .. '/jacket.toml')
  assert(format.is_ignored(jaifmt, dir .. '/gen/a.jai'))
  assert(format.is_ignored(jaifmt, dir .. '/vendor/x/a.jai'), 'a directory glob covers what is below')
  assert(not format.is_ignored(jaifmt, dir .. '/gen/deep/a.jai'), '* stays in one component')
  assert(not format.is_ignored(jaifmt, dir .. '/src/a.jai'))
  assert(not format.is_ignored('/no/such/jaifmt', dir .. '/gen/a.jai'))
end

return T
