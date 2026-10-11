local h = require('harness')
local commands = require('jai.commands')
local config = require('jai.config')
local format = require('jai.format')

h.test('jaifmt.toml ignore globs are read', function()
  local toml = '# c\nindent_width = 4\nignore = ["tests/corpus/**", "fuzz/**"] # comment\n'
  assert(vim.deep_equal(format.ignore_globs(toml), { 'tests/corpus/**', 'fuzz/**' }))
  assert(vim.deep_equal(format.ignore_globs('indent_width = 4'), {}))
end)

h.test('ignored files are those the globs cover, directories included', function()
  local cfg = '/proj/jaifmt.toml'
  local globs = { 'tests/corpus/**', 'vendor', './gen/*.jai' }
  assert(format.is_ignored(cfg, globs, '/proj/tests/corpus/a/b.jai'))
  assert(format.is_ignored(cfg, globs, '/proj/vendor/deep/x.jai'), 'a directory glob covers what is below')
  assert(format.is_ignored(cfg, globs, '/proj/gen/x.jai'))
  assert(not format.is_ignored(cfg, globs, '/proj/gen/sub/x.jai'), '* stays in one component')
  assert(not format.is_ignored(cfg, globs, '/proj/src/main.jai'))
  assert(not format.is_ignored(cfg, globs, '/elsewhere/tests/corpus/a.jai'))
end)

h.test('jailint output becomes quickfix items', function()
  local items = commands.parse_jailint(table.concat({
    'warning[index_only_loop]: `i` counts through `names` to index it',
    '  --> list.jai:21:5',
    '   |',
    '21 |     for i: 0..names.count - 1 {',
    'error[bool_comparison]: comparison with `true`',
    ' --> src/main.jai:3:12',
  }, '\n'))
  assert(#items == 2, vim.inspect(items))
  assert(items[1].filename == 'list.jai' and items[1].lnum == 21 and items[1].col == 5 and items[1].type == 'W')
  assert(items[1].text:find('index_only_loop', 1, true))
  assert(items[2].filename == 'src/main.jai' and items[2].type == 'E')
end)

if h.is_windows() then
  h.skip('jaifmt is stood in for by a shell script')
end

---Buffer lines of a saved file formatted by a stand-in `jaifmt` that strips trailing blanks.
---@param dir string
---@param lines string[]
---@return string[] result
---@return boolean changed
---@return string? err
local function format_file(dir, lines)
  local bin = h.tmpdir()
  h.script(bin .. '/jaifmt', [[cat | sed 's/[ \t]*$//']])
  config.set({ formatter = { path = bin .. '/jaifmt' } })
  local file = dir .. '/a.jai'
  vim.fn.writefile(lines, file)
  vim.cmd.edit(file)
  local changed, err = format.format(0)
  local result = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  vim.cmd('bwipeout!')
  config.set(nil)
  return result, changed, err
end

h.test('format applies only the changed lines', function()
  local result, changed, err = format_file(h.tmpdir(), { 'a :: 1;   ', 'b :: 2;', 'c :: 3;  ', 'd :: 4;' })
  assert(not err, err)
  assert(changed)
  assert(vim.deep_equal(result, { 'a :: 1;', 'b :: 2;', 'c :: 3;', 'd :: 4;' }), vim.inspect(result))
end)

h.test('format reports nothing to do for formatted text', function()
  local result, changed = format_file(h.tmpdir(), { 'a :: 1;' })
  assert(not changed and vim.deep_equal(result, { 'a :: 1;' }))
end)

h.test('files ignored by jaifmt.toml are left alone', function()
  local dir = h.tmpdir()
  vim.fn.writefile({ 'ignore = ["a.jai"]' }, dir .. '/jaifmt.toml')
  local result, changed = format_file(dir, { 'a :: 1;   ' })
  assert(not changed and result[1] == 'a :: 1;   ', vim.inspect(result))
end)

h.test('a formatter failure leaves the buffer alone and says why', function()
  local bin = h.tmpdir()
  h.script(bin .. '/jaifmt', 'echo "<stdin>:1:2: error: unbalanced" >&2; exit 2')
  config.set({ formatter = { path = bin .. '/jaifmt' } })
  vim.cmd.enew()
  vim.api.nvim_buf_set_lines(0, 0, -1, false, { 'x := {' })
  local changed, err = format.format(0)
  vim.cmd('bwipeout!')
  config.set(nil)
  assert(not changed and err and err:find('unbalanced'), tostring(err))
end)

h.test('format_on_save formats when the buffer is written', function()
  local bin = h.tmpdir()
  h.script(bin .. '/jaifmt', [[cat | sed 's/[ \t]*$//']])
  config.set({ formatter = { path = bin .. '/jaifmt' }, format_on_save = true })
  local file = h.tmpdir() .. '/save.jai'
  vim.fn.writefile({ 'a :: 1;   ' }, file)
  vim.cmd.edit(file)
  vim.cmd('silent write')
  local on_disk = vim.fn.readfile(file)
  vim.cmd('bwipeout!')
  config.set(nil)
  assert(vim.deep_equal(on_disk, { 'a :: 1;' }), vim.inspect(on_disk))
end)
