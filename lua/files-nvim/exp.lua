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
  Move = 1,
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
  local cwa = utils.call_wrap_async

  self:map('n', gkm.quit, cwa(self, self.close))
  self:map('n', km.open, cwa(self, self._open_current_file))
  self:map('n', km.next, cwa(self, self._nav, self.nav.next))
  self:map('n', km.prev, cwa(self, self._nav, self.nav.prev))
  self:map('n', km.up, cwa(self, self._nav, self.nav.up))
  self:map({ 'n', 'x' }, km.copy, cwa(self, self._action, self._copy_to_cb, 'Never', { CbAction.Copy }))
  self:map({ 'n', 'x' }, km.move, cwa(self, self._action, self._copy_to_cb, 'Never', { CbAction.Move }))
  self:map({ 'n', 'x' }, km.delete, cwa(self, self._action, self._del_sel_files, 'Done'))
  self:map(
    'n',
    km.paste,
    cwa(self, self._action, self._paste, function()
      local action = self.cb.action
      if action == CbAction.Copy then
        return 'Progress'
      elseif action == CbAction.Move then
        return 'Done'
      end
    end)
  )
  self:map('n', km.show_tasks_split, cwa(self.task, self.task.open_split, 0, 'right', 40))
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

function Exp:_refresh()
  local dir = self.current.dir
  local winid = self.winid

  local err, files = self.client:list_meta(dir.id)
  assert(not err, err)

  a_util.scheduler()

  local cursor_pos = api.nvim_win_get_cursor(winid)

  self:_set(dir, files)

  local line_count = api.nvim_buf_line_count(self.bufnr)

  if cursor_pos[1] <= line_count then
    api.nvim_win_set_cursor(winid, cursor_pos)
  else
    api.nvim_win_set_cursor(winid, { line_count, cursor_pos[2] })
  end
end

function Exp:_get_sel_files()
  local cur_files = self.current.files

  local range = self:get_sel_range()
  local files = {}

  for i = range[1], range[2] do
    table.insert(files, cur_files[i])
  end

  return files
end

function Exp:_del_sel_files()
  a_util.scheduler()

  local files = self:_get_sel_files()
  local choice = vim.fn.confirm(string.format('Delete %d items?', #files), 'Yes\nNo', 2)

  if choice ~= 1 then
    return
  end

  self.task:delete(files)
end

function Exp:_copy_to_cb(action)
  local cb = self.cb

  local files = self:_get_sel_files()

  cb.action = action
  cb.files = files
end

function Exp:_paste(on_prog)
  local cb = self.cb
  local task = self.task
  local dir = self.current.dir

  if cb.action == CbAction.Copy then
    task:copy(cb.files, dir, nil, on_prog)
  elseif cb.action == CbAction.Move then
    task:move(cb.files, dir, on_prog)
  end
end

function Exp:_action(fn, update_on, args)
  local c = self.current
  local c_dir_id = c.dir.id
  local is_id_equal = utils.is_id_equal
  local call_async = utils.call_async
  local args = args or {}

  local pfn = function()
    if not is_id_equal(c_dir_id, c.dir.id) then
      return
    end

    call_async(self, self._refresh)
  end

  if type(update_on) == 'function' then
    update_on = update_on()
  end

  if update_on == 'Progress' then
    table.insert(args, pfn)
    fn(self, unpack(args))
  elseif update_on == 'Done' then
    fn(self, unpack(args))
    if is_id_equal(c_dir_id, c.dir.id) then
      self:_refresh()
    end
  elseif update_on == 'Never' then
    fn(self, unpack(args))
  end
end

return Exp
