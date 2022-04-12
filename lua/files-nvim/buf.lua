-- Dependencies
local a_util = require 'plenary.async.util'

-- Config
local pconf = require('files-nvim.config').pconf
local split = require 'files-nvim.utils.split'

-- Neovim builtin
local api = vim.api
local keymap = vim.keymap

local Buf = {}

function Buf:new()
  local b = {
    bufnr = nil,
    winid = nil,
    prev_winid = nil,
    ns_id = api.nvim_create_namespace '',
  }

  self.__index = self

  return setmetatable(b, self)
end

--- Create a new buffer and show it in the given window
-- @tparam number winid - window to show the buffer in
-- @tparam boolean listed - whether the buffer is listed or not
function Buf:open_in(winid, listed)
  if self.bufnr then
    return
  end

  if not self.prev_winid then
    self.prev_winid = api.nvim_get_current_win()
  end

  local bufnr = api.nvim_create_buf(listed, true)
  api.nvim_win_set_buf(winid, bufnr)
  api.nvim_set_current_win(winid)

  self.winid = winid
  self.bufnr = bufnr

  self:set_buf_opts { modifiable = false }
end

--- Create a new buffer and show it in the current window
-- @tparam boolean listed - whether the buffer is listed or not
function Buf:open_current(listed)
  self:open_in(api.nvim_get_current_win(), listed)
end

--- Create a new buffer and show it in a new split
-- @tparam number|string rel - tells whether the split is opened relative to a window or the editor.
--   If this is a number, it is treated as a winid, and the split is opened relative to this window.
--   If this is `'win'`, the split is opened relative to current window.
--   If this is `'editor'`, the split is opened relative to the editor.
-- @tparam string pos - position of the split, valid values are `'left'`, `'right'`, `'top'`, `'bottom'`.
-- @tparam number size - size of the split in percentage relative to the given win or the editor.
function Buf:open_split(rel, pos, size)
  self.prev_winid = api.nvim_get_current_win()

  local winid

  if type(rel) == 'number' then
    winid = split.win[pos](size, rel)
  elseif rel == 'win' then
    winid = split.win[pos](size, 0)
  elseif rel == 'editor' then
    winid = split.editor[pos](size)
  else
    assert(false, "Invalid value for 'rel'")
  end

  self:open_in(winid, false)
  self:_setup_win_opts()
end

--- Close the buffer.
function Buf:close()
  local bufnr = self.bufnr

  if not bufnr then
    return
  end

  api.nvim_buf_clear_namespace(bufnr, self.ns_id, 0, -1)
  api.nvim_buf_delete(bufnr, { force = true })
  self.bufnr = nil
  self.prev_winid = nil
end

--- Set keymaps for the buffer.
function Buf:map(modes, lhs, rhs)
  local opts = vim.tbl_deep_extend('force', pconf.map_opts, { buffer = self.bufnr })

  keymap.set(modes, lhs, rhs, opts)
end

--- Set the name of the buffer.
function Buf:set_name(name)
  api.nvim_buf_set_name(self.bufnr, name)
end

--- Set the given options to the buffer
-- @tparam table opts - options to set
function Buf:set_buf_opts(opts)
  local bufnr = self.bufnr

  for k, v in pairs(opts) do
    api.nvim_buf_set_option(bufnr, k, v)
  end
end

--- Set the given options to the window this buffer was opened in
-- @tparam table opts - options to set
function Buf:set_win_opts(opts)
  local winid = self.winid

  for k, v in pairs(opts) do
    api.nvim_win_set_option(winid, k, v)
  end
end

--- Get the values of the given options
-- @tparam table opts - options
function Buf:get_buf_opts(opts)
  local bufnr = self.bufnr
  local o = {}

  for _, v in ipairs(opts) do
    o[v] = api.nvim_buf_get_option(bufnr, v)
  end

  return o
end

--- Get the values of the given options
-- @tparam table opts - options
function Buf:get_win_opts(opts)
  local winid = self.winid
  local o = {}

  for _, v in ipairs(opts) do
    o[v] = api.nvim_win_get_option(winid, v)
  end

  return o
end

--- Run the given function in context of the given buffer options
-- @tparam table opts - options
-- @tparam function fun - function to run
function Buf:with_buf_opts(opts, fun)
  local backup_opts = self:get_buf_opts(vim.tbl_keys(opts))

  self:set_buf_opts(opts)
  fun()
  self:set_buf_opts(backup_opts)
end

--- Run the given function in context of the given window options
-- @tparam table opts - options
-- @tparam function fun - function to run
function Buf:with_win_opts(opts, fun)
  local backup_opts = self:get_win_opts(vim.tbl_keys(opts))

  self:set_win_opts(opts)
  fun()
  self:set_win_opts(backup_opts)
end

--- Returns the item under cursor
-- @tparam array items - the array from which to retrive the item
-- @return the item under cursor
-- @treturn number - index of the item in the given items array
function Buf:get_current_item(items)
  local index = api.nvim_win_get_cursor(self.winid)[1]
  return items[index], index
end

--- Returns currently selected items
-- @tparam array items - the array from which to retrive the items
-- @return the selected items
function Buf:get_sel_items(items)
  local r = self:_get_sel_range()

  local sel_items = {}

  for i = r[1], r[2] do
    table.insert(sel_items, items[i])
  end

  return sel_items
end

--- Returns the range of lines that are currently selected
-- @treturn {number,number} - the range of current selection
function Buf:_get_sel_range()
  a_util.scheduler()

  local mode = api.nvim_get_mode().mode
  mode = string.byte(mode)

  local normal = 110
  local visual = 118
  local visual_line = 86
  local visual_block = 22

  local range = {}

  if mode == normal then
    local line = api.nvim_win_get_cursor(self.winid)[1]
    table.insert(range, line)
    table.insert(range, line)
  elseif mode == visual or mode == visual_line or mode == visual_block then
    api.nvim_input '<esc>'

    -- This second scheduler call is required, because the marks '<' and '>' are
    -- updated in the next event loop iteration after escaping the visual mode.
    a_util.scheduler()

    local range_start = api.nvim_buf_get_mark(self.bufnr, '<')[1]
    local range_end = api.nvim_buf_get_mark(self.bufnr, '>')[1]

    table.insert(range, range_start)
    table.insert(range, range_end)
  end

  return range
end

--- Set initial options for the window
function Buf:_setup_win_opts()
  self:set_win_opts {
    number = false,
    signcolumn = 'no',
  }
end

return Buf
