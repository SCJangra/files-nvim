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
    ns_id = api.nvim_create_namespace '',
    is_win_owned = false,
  }

  self.__index = self

  return setmetatable(b, self)
end

function Buf:open_current()
  if self.bufnr then
    return
  end

  a_util.scheduler()

  local winid = api.nvim_get_current_win()
  local bufnr = api.nvim_create_buf(true, true)

  api.nvim_win_set_buf(winid, bufnr)

  self.winid = winid
  self.bufnr = bufnr
end

function Buf:open_split(rel, pos, size)
  if self.bufnr then
    return
  end

  a_util.scheduler()
  local bufnr = api.nvim_create_buf(false, true)
  local winid

  if type(rel) == 'number' then
    winid = split.win[pos](size, rel, bufnr)
  elseif rel == 'win' then
    winid = split.win[pos](size, 0, bufnr)
  elseif rel == 'editor' then
    winid = split.editor[pos](size, bufnr)
  else
    assert(false, "Invalid value for 'rel'")
  end

  self.winid = winid
  self.bufnr = bufnr
end

function Buf:close()
  local bufnr = self.bufnr
  local winid = self.winid

  if not bufnr then
    return
  end

  if self.is_win_owned then
    api.nvim_win_close(winid, false)
    self.winid = nil
  end

  api.nvim_buf_clear_namespace(bufnr, self.ns_id, 0, -1)
  api.nvim_buf_delete(bufnr, { force = true })
  self.bufnr = nil
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

return Buf
