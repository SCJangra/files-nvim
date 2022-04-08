local a_util = require 'plenary.async.util'

local pconf = require('files-nvim.config').pconf
local split = require 'files-nvim.utils.split'

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

function Buf:open_current(listed)
  self:open_in(api.nvim_get_current_win(), listed)
end

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

function Buf:map(modes, lhs, rhs)
  local opts = vim.tbl_deep_extend('force', pconf.map_opts, { buffer = self.bufnr })

  keymap.set(modes, lhs, rhs, opts)
end

function Buf:get_sel_range()
  a_util.scheduler()

  local mode = api.nvim_get_mode().mode

  local range = {}

  if mode == 'n' then
    local line = api.nvim_win_get_cursor(self.winid)[1]
    table.insert(range, line)
    table.insert(range, line)
  elseif mode == 'V' then
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

function Buf:set_name(name)
  api.nvim_buf_set_name(self.bufnr, name)
end

function Buf:set_buf_opts(opts)
  local bufnr = self.bufnr

  for k, v in pairs(opts) do
    api.nvim_buf_set_option(bufnr, k, v)
  end
end

function Buf:set_win_opts(opts)
  local winid = self.winid

  for k, v in pairs(opts) do
    api.nvim_win_set_option(winid, k, v)
  end
end

function Buf:get_buf_opts(opts)
  local bufnr = self.bufnr
  local o = {}

  for _, v in ipairs(opts) do
    o[v] = api.nvim_buf_get_option(bufnr, v)
  end

  return o
end

function Buf:get_win_opts(opts)
  local winid = self.winid
  local o = {}

  for _, v in ipairs(opts) do
    o[v] = api.nvim_win_get_option(winid, v)
  end

  return o
end

function Buf:with_buf_opts(opts, fun)
  local backup_opts = self:get_buf_opts(vim.tbl_keys(opts))

  self:set_buf_opts(opts)
  fun()
  self:set_buf_opts(backup_opts)
end

function Buf:with_win_opts(opts, fun)
  local backup_opts = self:get_win_opts(vim.tbl_keys(opts))

  self:set_win_opts(opts)
  fun()
  self:set_win_opts(backup_opts)
end

function Buf:_setup_win_opts()
  self:set_win_opts {
    number = false,
    signcolumn = 'no',
  }
end

return Buf
