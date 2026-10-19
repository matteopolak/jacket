-- Runs every tests/*_spec.lua with mini.test (https://github.com/echasnovski/mini.test, pinned in
-- .github/pins.json and fetched by tools/dev/dev.py nvim); exits non-zero when one fails.
local dir = vim.g.jai_test_dir
package.path = dir .. '/?.lua;' .. package.path

local MiniTest = require('mini.test')
local specs = vim.fn.glob(dir .. '/*_spec.lua', false, true)
table.sort(specs)
MiniTest.run({
  collect = {
    find_files = function()
      return specs
    end,
  },
  execute = { reporter = MiniTest.gen_reporter.stdout({ group_depth = 1 }) },
})
