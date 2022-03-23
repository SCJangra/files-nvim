local a_util = require 'plenary.async.util'

local pconf = require('files-nvim.config').pconf

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

function Buf:map(lhs, rhs)
  local opts = vim.tbl_deep_extend('force', pconf.map_opts, { buffer = self.bufnr })

  keymap.set('n', lhs, rhs, opts)
end

return Buf
