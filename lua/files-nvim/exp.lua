local Client = require 'files-nvim.client'
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local utils = require 'files-nvim.utils'
local Navigator = require 'files-nvim.exp.navigator'
local Buf = require 'files-nvim.buf'
local Task = require 'files-nvim.task'

local api = vim.api

local a_util = require 'plenary.async.util'
local Line = require 'nui.line'
local Text = require 'nui.text'

local CbAction = {
  Copy = 0,
}

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
          return string.format('%10s', utils.bytes_to_size(size))
        end,
        name = function(name)
          return string.format('%s', name)
        end,
      },
    },
    cb = {
      action = nil,
      files = nil,
    },
    task = Task:new(client),
  }

  self.__index = self

  return setmetatable(e, self)
end

function Exp:open_current()
  getmetatable(getmetatable(self).__index).__index.open_current(self)

  self:_setup()
end

function Exp:open_split(rel, pos, size)
  getmetatable(getmetatable(self).__index).__index.open_split(self, rel, pos, size)

  self:_setup()
end

function Exp:close()
  self.task:close()

  getmetatable(getmetatable(self).__index).__index.close(self)

  self.client:stop()
end

function Exp:_setup_keymaps()
  local km = uconf.exp.keymaps
  local gkm = uconf.keymaps
  local call_wrap_async = utils.call_wrap_async

  self:map('n', gkm.quit, call_wrap_async(self, self.close))
  self:map('n', km.open, call_wrap_async(self, self._open_current_file))
  self:map('n', km.next, call_wrap_async(self, self._nav, self.nav.next))
  self:map('n', km.prev, call_wrap_async(self, self._nav, self.nav.prev))
  self:map('n', km.up, call_wrap_async(self, self._nav, self.nav.up))
  self:map({ 'n', 'x' }, km.copy, call_wrap_async(self, self._copy_to_cb))
  self:map('n', km.paste, call_wrap_async(self, self._paste))
  self:map('n', km.show_tasks_split, call_wrap_async(self.task, self.task.open_split, 0, 'right', 40))
end

function Exp:_setup()
  local client = self.client

  local err = client:start()
  assert(not err, err)

  a_util.scheduler()
  local cwd = vim.fn.getcwd(self.winid)

  local err, dir = client:get_meta { 'Local', cwd }
  assert(not err, err)

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

function Exp:_copy_to_cb()
  local cur_files = self.current.files
  local cb = self.cb

  local range = self:get_sel_range()
  local action = CbAction.Copy
  local files = {}

  for i = range[1], range[2] do
    table.insert(files, cur_files[i])
  end

  cb.action = action
  cb.files = files
end

function Exp:_paste()
  local cb = self.cb
  local task = self.task
  local current = self.current

  if cb.action == CbAction.Copy then
    task:copy(cb.files, current.dir)
  end
end

return Exp
