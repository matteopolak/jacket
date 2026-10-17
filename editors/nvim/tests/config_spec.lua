local h = require('harness')
local config = require('jai.config')

h.test('the workspace is trusted unless the user says otherwise', function()
  config.set({})
  assert(config.get().trusted == true)
  assert(config.server_settings().trusted == true)
end)

h.test('trusted = false reaches the server settings', function()
  config.set({ trusted = false })
  assert(config.server_settings().trusted == false)
  config.set({})
end)
