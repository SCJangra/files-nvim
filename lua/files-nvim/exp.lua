local Client = require 'files-nvim.client'

local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local exp = uconf.exp
local input_opts = exp.input_opts

local utils = require 'files-nvim.utils'
local async_wrap = utils.async_wrap
local wrap = utils.wrap
local async = utils.async
local Navigator = require 'files-nvim.exp.navigator'
local Buf = require 'files-nvim.buf'
local Task = require 'files-nvim.task'

local api = vim.api

local a_util = require 'plenary.async.util'
local Line = require 'nui.line'
local Text = require 'nui.text'
local Input = require 'nui.input'

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
  local km = exp.keymaps
  local gkm = uconf.keymaps

  self:map('n', gkm.quit, async_wrap(self.close, self))
  self:map('n', km.open, async_wrap(self._open_current_file, self))
  self:map('n', km.next, async_wrap(self._nav, self, self.nav.next))
  self:map('n', km.prev, async_wrap(self._nav, self, self.nav.prev))
  self:map('n', km.up, async_wrap(self._nav, self, self.nav.up))
  self:map({ 'n', 'x' }, km.copy, async_wrap(self._action, self, wrap(self._copy_to_cb, self, CbAction.Copy), 'Never'))
  self:map({ 'n', 'x' }, km.move, async_wrap(self._action, self, wrap(self._copy_to_cb, self, CbAction.Move), 'Never'))
  self:map({ 'n', 'x' }, km.delete, async_wrap(self._action, self, wrap(self._del_sel_files, self), 'Done'))
  self:map({ 'n', 'x' }, km.rename, wrap(self._show_rename_dialog, self))
  self:map(
    { 'n', 'x' },
    km.paste,
    async_wrap(self._action, self, wrap(self._paste, self), function()
      local action = self.cb.action
      if action == CbAction.Copy then
        return 'Progress'
      elseif action == CbAction.Move then
        return 'Done'
      end
    end)
  )
  self:map('n', km.show_tasks_split, async_wrap(self.task.open_split, self.task, 0, 'right', 40))
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

function Exp:_action(fn, update_on)
  local c = self.current
  local c_dir_id = c.dir.id
  local is_id_equal = utils.is_id_equal

  local pfn = function()
    if not is_id_equal(c_dir_id, c.dir.id) then
      return
    end

    async(self._refresh, self)
  end

  if type(update_on) == 'function' then
    update_on = update_on()
  end

  if update_on == 'Progress' then
    fn(pfn)
  elseif update_on == 'Done' then
    fn()
    if is_id_equal(c_dir_id, c.dir.id) then
      self:_refresh()
    end
  elseif update_on == 'Never' then
    fn()
  end
end

function Exp:_show_rename_dialog()
  local _, file = self:_get_current_file()

  local i = Input(input_opts.rename, {
    prompt = '',
    default_value = file.name,
    on_submit = function(new_name)
      async(self._action, self, wrap(self._rename, self, file, new_name), 'Done')
    end,
  })

  i:mount()
end

function Exp:_rename(file, new_name)
  local err, _ = self.client:rename(file.id, new_name)
  assert(not err, err)
end

return Exp
