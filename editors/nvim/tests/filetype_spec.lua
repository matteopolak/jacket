local h = require('harness')

h.test('*.jai is detected as jai', function()
  assert(vim.filetype.match({ filename = 'a.jai' }) == 'jai')
  assert(vim.filetype.match({ filename = '/some/dir/main.jai' }) == 'jai')
end)

h.test('the ftplugin sets comments and four-space indentation', function()
  local dir = h.tmpdir()
  local file = dir .. '/a.jai'
  vim.fn.writefile({ 'main :: () {', '}' }, file)
  vim.cmd.edit(file)
  assert(vim.bo.filetype == 'jai', vim.bo.filetype)
  assert(vim.bo.commentstring == '// %s')
  assert(vim.bo.expandtab and vim.bo.shiftwidth == 4)
  vim.cmd('bwipeout!')
end)

---The syntax group (syntax group name) of the character at `line`, `col` (1-based).
---@param line integer
---@param col integer
---@return string
local function group_at(line, col)
  return vim.fn.synIDattr(vim.fn.synID(line, col, 1), 'name')
end

h.test('the syntax file highlights the Jai lexical classes', function()
  local lines = {
    'main :: () {', -- 1
    '    // comment', -- 2
    '    /* a /* nested */ still */ x := 1;', -- 3
    '    s := "text %";', -- 4
    '    n := 0xff;', -- 5
    '    #run foo();', -- 6
    'T :: struct { a: int; } @Note', -- 7
    'TEXT :: #string END', -- 8
    'in the here-string if 5', -- 9
    'END', -- 10
    'after := 2.5;', -- 11
  }
  local dir = h.tmpdir()
  vim.fn.writefile(lines, dir .. '/s.jai')
  vim.cmd.edit(dir .. '/s.jai')
  vim.cmd('syntax sync fromstart')
  assert(vim.bo.syntax == 'jai', vim.bo.syntax)
  assert(group_at(1, 1) == 'jaiProcDecl', 'proc decl: ' .. group_at(1, 1))
  assert(group_at(2, 6) == 'jaiLineComment')
  assert(group_at(3, 20) == 'jaiBlockComment', 'nested comment: ' .. group_at(3, 20))
  assert(group_at(3, 31) ~= 'jaiBlockComment', 'code after a block comment')
  assert(group_at(4, 12) == 'jaiString')
  assert(group_at(5, 12) == 'jaiNumber', 'hex: ' .. group_at(5, 12))
  assert(group_at(6, 6) == 'jaiRun', '#run: ' .. group_at(6, 6))
  assert(group_at(7, 1) == 'jaiTypeDecl', 'struct decl: ' .. group_at(7, 1))
  assert(group_at(7, 25) == 'jaiNote', 'note: ' .. group_at(7, 25))
  assert(group_at(9, 10) == 'jaiHereString', 'here-string body: ' .. group_at(9, 10))
  assert(group_at(9, 17) ~= 'jaiConditional', 'keyword inside a here-string')
  assert(group_at(11, 12) == 'jaiFloat', 'after the here-string: ' .. group_at(11, 12))
  vim.cmd('bwipeout!')
end)
