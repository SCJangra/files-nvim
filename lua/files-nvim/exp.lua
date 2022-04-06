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
local event = require 'files-nvim.event'

local api = vim.api
local confirm = vim.fn.confirm

local a_util = require 'plenary.async.util'
local Line = require 'nui.line'
local Text = require 'nui.text'
local Input = require 'nui.input'

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
    active_tasks = {
      copy = {},
      move = {},
      delete = {},
    },
  }

  self.__index = self

  return setmetatable(e, self)
end

function Exp:open_current(listed)
  getmetatable(getmetatable(self).__index).__index.open_current(self, listed)

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
  event:broadcast('exp_closed', self)
end

function Exp:_setup_keymaps()
  local km = exp.keymaps
  local gkm = uconf.keymaps

  self:map('n', gkm.quit, async_wrap(self.close, self))
  self:map('n', km.open, async_wrap(self._open_current_file, self))
  self:map('n', km.next, async_wrap(self._nav, self, self.nav.next))
  self:map('n', km.prev, async_wrap(self._nav, self, self.nav.prev))
  self:map('n', km.up, async_wrap(self._nav, self, self.nav.up))
  self:map('n', km.show_tasks_split, wrap(self.task.open_split, self.task, 0, 'right', 40))
  self:map({ 'n', 'x' }, km.copy, async_wrap(self._copy_to_cb, self, 'Copy'))
  self:map({ 'n', 'x' }, km.move, async_wrap(self._copy_to_cb, self, 'Move'))
  self:map({ 'n', 'x' }, km.delete, async_wrap(self._del_sel_files, self))
  self:map('n', km.paste, async_wrap(self._paste, self))
  self:map('n', km.rename, wrap(self._show_rename_dialog, self))
  self:map('n', km.create_file, wrap(self._show_create_dialog, self, 'File'))
  self:map('n', km.create_dir, wrap(self._show_create_dialog, self, 'Dir'))
end

function Exp:_setup_fs_events()
  local tasks = self.active_tasks
  local copy = tasks.copy
  local move = tasks.move
  local delete = tasks.delete

  local c = self.current
  local is_id_equal = utils.is_id_equal

  event:on('copy_start', function(id, files, dest)
    copy[id] = {
      files = files,
      dest = dest,
    }
  end)
  event:on('copy_prog', function(id, _)
    if not is_id_equal(c.dir.id, copy[id].dest.id) then
      return
    end

    async(self._refresh, self)
  end)
  event:on('copy_end', function(id)
    copy[id] = nil
  end)

  event:on('move_start', function(id, files, dest)
    move[id] = {
      files = files,
      dest = dest,
    }
  end)
  event:on('move_end', function(id)
    if not is_id_equal(c.dir.id, move[id].dest.id) then
      return
    end

    async(self._refresh, self)
    move[id] = nil
  end)

  event:on('delete_start', function(id, files, dir)
    delete[id] = {
      files = files,
      dir = dir,
    }
  end)
  event:on('delete_end', function(id)
    if not is_id_equal(c.dir.id, delete[id].dir.id) then
      return
    end

    async(self._refresh, self)
    delete[id] = nil
  end)

  event:on('renamed', function(_, _, dir)
    if not is_id_equal(dir.id, c.dir.id) then
      return
    end

    async(self._refresh, self)
  end)
  event:on('created', function(_, _, dir)
    if not is_id_equal(dir.id, c.dir.id) then
      return
    end

    async(self._refresh, self)
  end)
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
  self:_setup_fs_events()
  self:set_name 'FilesNvim'
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
  else
    self:_open_file(file)
  end
end

function Exp:_open_file(file)
  if file.id[1] ~= 'Local' then
    print 'Currently cannot open remote files'
    return
  end

  local err, mime = self.client:get_mime(file.id)
  assert(not err, err)

  if not utils.is_text(mime) then
    print 'Not a text file'
    return
  end

  a_util.scheduler()

  local winid = self.prev_winid
  local path = file.id[2]
  local bufnr = utils.is_open(path)

  api.nvim_set_current_win(winid)

  if bufnr then
    api.nvim_win_set_buf(winid, bufnr)
  else
    api.nvim_command('edit ' .. path)
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

function Exp:_copy_to_cb(action)
  local cb = self.cb

  local files = self:_get_sel_files()

  cb.action = action
  cb.files = files
end

function Exp:_paste()
  local cb = self.cb
  local c = self.current

  if cb.action == 'Copy' then
    self.task:copy(cb.files, c.dir, uconf.task.cp_interval)
  elseif cb.action == 'Move' then
    self.task:move(cb.files, c.dir)
  end
end

function Exp:_del_sel_files()
  local files = self:_get_sel_files()
  local dir = self.current.dir

  local choice = confirm(string.format('Delete %d items?', #files), 'Yes\nNo', 2)

  if choice ~= 1 then
    return
  end
  self.task:delete(files, dir)
end

function Exp:_show_rename_dialog()
  local _, file = self:_get_current_file()

  local i = Input(input_opts.rename, {
    prompt = '',
    default_value = file.name,
    on_submit = async_wrap(self._rename, self, file, self.current.dir),
  })

  i:mount()
end

function Exp:_rename(file, dir, new_name)
  local client = self.client

  local err, id = client:rename(file.id, new_name)
  assert(not err, err)

  local err, new_file = client:get_meta(id)
  assert(not err, err)

  event:broadcast('renamed', file, new_file, dir)
end

function Exp:_show_create_dialog(type)
  local opts

  if type == 'File' then
    opts = input_opts.create_file
  elseif type == 'Dir' then
    opts = input_opts.create_dir
  else
    return
  end

  local i = Input(opts, {
    prompt = '',
    default_value = '',
    on_submit = async_wrap(self._create, self, type, self.current.dir),
  })

  i:mount()
end

function Exp:_create(type, dir, name)
  local client = self.client
  local err, id

  if type == 'File' then
    err, id = client:create_file(name, dir.id)
  elseif type == 'Dir' then
    err, id = client:create_dir(name, dir.id)
  else
    return
  end
  assert(not err, err)

  local err, file = client:get_meta(id)
  assert(not err, err)

  event:broadcast('created', type, file, dir)
end

return Exp
