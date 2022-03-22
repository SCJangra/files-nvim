local Client = require 'files-nvim.client'
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local pconf = conf.pconf
local utils = require 'files-nvim.utils'
local Navigator = require 'files-nvim.exp.navigator'
local Buf = require 'files-nvim.buf'

local keymap = vim.keymap
local api = vim.api

local a_util = require 'plenary.async.util'
local Line = require 'nui.line'
local Text = require 'nui.text'

local Exp = Buf:new()

function Exp:new(fields)
  local client = Client:new()
  local e = {
    client = client,
    nav = Navigator:new(client),
    current = {
      dir = nil,
      files = nil,
    },
    fields = {
      active = fields or uconf.exp.fields,
      fmt = {
        size = function(size)
          return string.format('%7s %2s', utils.bytes_to_size(size))
        end,
        name = function(name)
          return string.format('%s', name)
        end,
      },
    },
  }

  self.__index = self

  return setmetatable(e, self)
end

function Exp:open_current()
  getmetatable(getmetatable(self).__index).__index.open_current(self)

  self:_setup()
end

function Exp:close()
  getmetatable(getmetatable(self).__index).__index.close(self)

  self.client:stop()
end

function Exp:_setup_keymaps()
  local km = uconf.exp.keymaps
  local gkm = uconf.keymaps
  local call_wrap_async = utils.call_wrap_async

  self:_map(gkm.quit, call_wrap_async(self, self.close))
  self:_map(km.open, call_wrap_async(self, self._open_current_file))
  self:_map(km.next, call_wrap_async(self, self._nav, self.nav.next))
  self:_map(km.prev, call_wrap_async(self, self._nav, self.nav.prev))
  self:_map(km.up, call_wrap_async(self, self._nav, self.nav.up))
end

function Exp:_setup()
  local client = self.client

  local err = client:start()
  assert(not err, err)

  a_util.scheduler()
  local cwd = vim.fn.getcwd(self.winid)

  local err, dir = client:get_meta { 'Local', cwd }
  assert(not err, vim.inspect(err))

  self:_nav(self.nav.nav, dir)
  self:_setup_keymaps()
end

function Exp:_nav(fun, ...)
  local dir, files = fun(self.nav, ...)

  if not dir or not files then
    return
  end

  self:_set(dir, files)
end

function Exp:_map(lhs, rhs)
  local opts = vim.tbl_deep_extend('force', pconf.map_opts, { buffer = self.bufnr })

  keymap.set('n', lhs, rhs, opts)
end

function Exp:_to_line(file)
  local hl = uconf.exp.hl
  local fields = self.fields.active
  local fmt = self.fields.fmt
  local line = {}

  for _, f in ipairs(fields) do
    if f == 'name' then
      goto continue
    end

    table.insert(line, Text(fmt[f](file[f]), hl[f]))
    table.insert(line, Text ' ')

    ::continue::
  end

  table.insert(line, Text(fmt.name(file.name), hl.name))

  return Line(line)
end

function Exp:_get_current_file()
  local index = api.nvim_win_get_cursor(self.winid)[1]
  return index, self.current.files[index]
end

function Exp:_open_current_file()
  local _, file = self:_get_current_file()

  if file.file_type == 'Dir' then
    self:_nav(self.nav.nav, file)
  end
end

function Exp:_set(dir, files)
  a_util.scheduler()

  local cur = self.current
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  api.nvim_buf_clear_namespace(bufnr, ns_id, 0, -1)
  api.nvim_buf_set_lines(bufnr, 0, -1, true, {})

  for i, f in ipairs(files) do
    self:_to_line(f):render(bufnr, ns_id, i, i + 1)
  end

  cur.dir = dir
  cur.files = files
end

return Exp
