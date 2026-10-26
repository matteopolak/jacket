local h = require('harness')
local T = h.set()
local config = require('jai.config')

T['the workspace is trusted unless the user says otherwise'] = function()
  config.set({})
  assert(config.get().trusted == true)
  assert(config.server_settings().trusted == true)
end

T['trusted = false reaches the server settings'] = function()
  config.set({ trusted = false })
  assert(config.server_settings().trusted == false)
  config.set({})
end

T['the compile-time limits default to the VS Code values'] = function()
  config.set({})
  local c = config.server_settings().compileTime
  assert(c.evaluationBudget == 200000000, vim.inspect(c))
  assert(c.checkTimeLimit == 10, vim.inspect(c))
  assert(c.costLensBlocks == 1000000, vim.inspect(c))
  assert(c.limitNotices == true, vim.inspect(c))
end

T['compile_time overrides reach the server settings'] = function()
  config.set({ compile_time = { evaluation_budget = 5, check_time_limit = 0 } })
  local c = config.server_settings().compileTime
  assert(c.evaluationBudget == 5 and c.checkTimeLimit == 0 and c.costLensBlocks == 1000000, vim.inspect(c))
  config.set({})
end

T['a limit notice says where it happened and how to raise the limit'] = function()
  local limits = require('jai.limits')
  config.set({ compile_time = { evaluation_budget = 1000 } })
  local notice = { kind = 'budget', uri = 'file:///tmp/main.jai', range = { start = { line = 2, character = 0 } }, message = 'x' }
  local text = limits.text(notice)
  assert(text:find('main.jai:3', 1, true), text)
  assert(text:find('1000 basic blocks', 1, true), text)
  assert(text:find('compile_time.evaluation_budget', 1, true), text)
  local time = limits.text(vim.tbl_extend('force', notice, { kind = 'time' }))
  assert(time:find('compile_time.check_time_limit', 1, true), time)
  config.set({})
end

T['a limit notice is shown once until a write'] = function()
  local limits = require('jai.limits')
  limits.reset()
  local shown = {}
  local original = vim.notify
  vim.notify = function(msg, level)
    shown[#shown + 1] = { msg, level }
  end
  local notice = { kind = 'time', uri = 'file:///tmp/main.jai', range = { start = { line = 0, character = 0 } }, message = 'x' }
  limits.notice(notice)
  limits.notice(notice)
  vim.wait(100, function()
    return #shown > 0
  end)
  assert(#shown == 1, vim.inspect(shown))
  assert(shown[1][2] == vim.log.levels.WARN)
  vim.api.nvim_exec_autocmds('BufWritePost', { group = 'jai-limit-notices' })
  limits.notice(notice)
  vim.wait(100, function()
    return #shown > 1
  end)
  assert(#shown == 2, vim.inspect(shown))
  vim.notify = original
end

return T
