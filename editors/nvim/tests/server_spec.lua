-- End to end against a real jailsp: JAILSP=/path/to/jailsp.
local h = require('harness')

local jailsp = vim.env.JAILSP
if not jailsp or jailsp == '' then
  h.skip('JAILSP is not set')
end

local config = require('jai.config')
local jai = require('jai')

local dir = h.tmpdir()
local file = dir .. '/main.jai'
vim.fn.writefile({
  '// Adds two numbers.',
  '//',
  '// Returns their sum.',
  'add :: (a: int, b: int) -> int {',
  '    return a + b;',
  '}',
  '',
  'main :: () {',
  '    total := add(1, 2);',
  '    missing_name;',
  '}',
}, file)

config.set({ server = { path = jailsp } })
vim.lsp.enable('jailsp')
vim.cmd.edit(file)
local bufnr = vim.api.nvim_get_current_buf()

---@return vim.lsp.Client
local function client()
  return assert(vim.lsp.get_clients({ bufnr = bufnr, name = 'jailsp' })[1])
end

h.test('jailsp attaches to a Jai buffer', function()
  h.wait_for(15000, function()
    return #vim.lsp.get_clients({ bufnr = bufnr, name = 'jailsp' }) > 0
  end, 'jailsp to attach')
  local c = client()
  assert(c.server_info and c.server_info.name == 'jai-language-server', vim.inspect(c.server_info))
  assert(c.server_capabilities.hoverProvider)
  assert(c.server_capabilities.semanticTokensProvider)
end)

h.test('the error is published as a diagnostic', function()
  h.wait_for(30000, function()
    return #vim.diagnostic.get(bufnr) > 0
  end, 'diagnostics')
  local diagnostics = vim.diagnostic.get(bufnr)
  local on_line_10 = vim.tbl_filter(function(d)
    return d.lnum == 9
  end, diagnostics)
  assert(#on_line_10 > 0, vim.inspect(diagnostics))
end)

h.test('hover returns the doc comment', function()
  local result
  local params = { textDocument = { uri = vim.uri_from_bufnr(bufnr) }, position = { line = 8, character = 14 } }
  client():request('textDocument/hover', params, function(err, res)
    assert(not err, vim.inspect(err))
    result = res or false
  end, bufnr)
  h.wait_for(10000, function()
    return result ~= nil
  end, 'the hover response')
  assert(result, 'hover returned nothing')
  local text = type(result.contents) == 'table' and (result.contents.value or vim.inspect(result.contents)) or tostring(result.contents)
  assert(text:find('Adds two numbers', 1, true), text)
end)

h.test('semantic tokens are requested and highlight the buffer', function()
  h.wait_for(10000, function()
    local marks = vim.api.nvim_buf_get_extmarks(bufnr, -1, 0, -1, { details = true })
    for _, mark in ipairs(marks) do
      if (mark[4].hl_group or ''):find('@lsp.type', 1, true) then
        return true
      end
    end
  end, 'semantic token highlights')
end)

h.test('go to definition finds the procedure', function()
  local result
  local params = { textDocument = { uri = vim.uri_from_bufnr(bufnr) }, position = { line = 8, character = 14 } }
  client():request('textDocument/definition', params, function(_, res)
    result = res or false
  end, bufnr)
  h.wait_for(10000, function()
    return result ~= nil
  end, 'the definition response')
  local location = vim.islist(result) and result[1] or result
  assert(location, 'no definition')
  local range = location.range or location.targetSelectionRange
  assert(range.start.line == 3, vim.inspect(location))
end)

h.test(':Jai expand reports no expansion where there is none, without an error', function()
  vim.api.nvim_win_set_cursor(0, { 9, 14 })
  vim.cmd('Jai expand')
  vim.wait(500)
end)

h.test(':Jai restart brings the server back', function()
  local old = client().id
  local done = false
  jai.restart(function()
    done = true
  end)
  h.wait_for(10000, function()
    return done
  end, 'the restart')
  h.wait_for(15000, function()
    local c = vim.lsp.get_clients({ bufnr = bufnr, name = 'jailsp' })[1]
    return c and c.id ~= old
  end, 'a new client to attach')
end)

h.test('the server stops with the editor', function()
  vim.lsp.enable('jailsp', false)
  h.wait_for(10000, function()
    return #vim.lsp.get_clients({ name = 'jailsp' }) == 0
  end, 'jailsp to stop')
end)
