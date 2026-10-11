-- Runs every tests/*_spec.lua and exits non-zero when one fails.
local dir = vim.g.jai_test_dir
package.path = dir .. '/?.lua;' .. package.path
local harness = require('harness')

local specs = vim.fn.glob(dir .. '/*_spec.lua', false, true)
table.sort(specs)
for _, spec in ipairs(specs) do
  harness.file = vim.fs.basename(spec):gsub('_spec%.lua$', '')
  local ok, err = pcall(dofile, spec)
  if not ok and not (type(err) == 'table' and err.skip) then
    harness.failed = harness.failed + 1
    harness.failures[#harness.failures + 1] = spec .. ': ' .. tostring(err)
  end
end

io.stdout:write(('\n%d passed, %d failed, %d skipped\n'):format(harness.passed, harness.failed, harness.skipped))
for _, failure in ipairs(harness.failures) do
  io.stdout:write('\n' .. failure .. '\n')
end
vim.cmd(harness.failed > 0 and 'cquit 1' or 'qa!')
