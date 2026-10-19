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

return T
