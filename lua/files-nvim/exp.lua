-- Classes
local Client = require 'files-nvim.client'
local Navigator = require 'files-nvim.exp.navigator'
local Buf = require 'files-nvim.buf'
local Task = require 'files-nvim.task'

-- Config
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local exp = uconf.exp
local input_opts = exp.input_opts

-- misc
local utils = require 'files-nvim.utils'
local async, wrap = utils.async, utils.wrap
local event = require 'files-nvim.event'

-- Neovim builtin
local api = vim.api
local fn = vim.fn

-- Dependencies
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

    space = Text ' ',
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
  event.exp_closed:broadcast(self)
end

function Exp:_setup_keymaps()
  getmetatable(getmetatable(self).__index).__index._setup_keymaps(self)

  local km = exp.keymaps
  local gkm = uconf.keymaps

  self:map('n', gkm.quit, wrap(async, self.close, self))
  self:map('n', km.open, wrap(async, self._open_current_file, self))
  self:map('n', km.next, wrap(async, self._nav, self, self.nav.next))
  self:map('n', km.prev, wrap(async, self._nav, self, self.nav.prev))
  self:map('n', km.up, wrap(async, self._nav, self, self.nav.up))
  self:map('n', km.show_tasks_split, wrap(self.task.open_split, self.task, 0, 'right', 40))
  self:map({ 'n', 'x' }, km.copy, wrap(async, self._copy_to_cb, self, 'Copy'))
  self:map({ 'n', 'x' }, km.move, wrap(async, self._copy_to_cb, self, 'Move'))
  self:map({ 'n', 'x' }, km.delete, wrap(async, self._del_sel_files, self))
  self:map('n', km.paste, wrap(async, self._paste, self))
  self:map('n', km.rename, wrap(self._show_rename_dialog, self))
  self:map('n', km.create_file, wrap(self._show_create_dialog, self, 'File'))
  self:map('n', km.create_dir, wrap(self._show_create_dialog, self, 'Dir'))
end

function Exp:_setup_fs_events()
  local c = self.current
  local is_id_equal = utils.is_id_equal

  event.dir_modified:add(function(dir_id)
    if not is_id_equal(c.dir.id, dir_id) then
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

--- Navigate the explorer using the given function
-- @tparam function fun - the function to use for navigation
-- @tparam varargs ... - arguments that are passed to the navigation function
function Exp:_nav(fun, ...)
  local dir, files = fun(self.nav, ...)

  if not dir or not files then
    return
  end

  self:_view(dir, files)
end

function Exp:_to_line(file)
  local hl = uconf.exp.hl
  local fields = self.fields.active
  local fmt = self.fields.fmt
  local line = {}
  local space = self.space

  for _, f in ipairs(fields) do
    if f == 'name' then
      goto continue
    end

    table.insert(line, Text(fmt[f](file[f]), hl[f]))
    table.insert(line, space)

    ::continue::
  end

  table.insert(line, utils.get_icon(file))
  table.insert(line, space)
  table.insert(line, Text(fmt.name(file.name), hl.name))

  return Line(line)
end

function Exp:_open_current_file()
  local file = self:get_current_item(self.current.files)

  if file.file_type == 'Dir' then
    self:_nav(self.nav.nav, file)
  else
    self:_open_file(file)
  end
end

function Exp:_open_file(file)
  if file.id[1] == 'Local' then
    self:_open_local(file)
  else
    assert(false, 'Currently cannot open remote files')
  end
end

--- Use this function to open a local file
-- @tparam table file - file to open
function Exp:_open_local(file)
  local err, mime = self.client:get_mime(file.id)
  assert(not err, err)

  a_util.scheduler()

  if not utils.is_text(mime) then
    local os = jit.os

    if os == 'Linux' then
      fn.jobstart({ 'xdg-open', file.id[2] }, { detach = true })
    elseif os == 'OSX' then
      fn.jobstart({ 'open', file.id[2] }, { detach = true })
    else
      assert(false, 'Cannot open files in current os!')
    end
  else
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
end

--- View the given files in the explorer
-- @tparam table dir - parent directory of the files
-- @tparam {table,...} - files to view
function Exp:_view(dir, files)
  a_util.scheduler()

  local cur = self.current
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  api.nvim_buf_clear_namespace(bufnr, ns_id, 0, -1)

  self:with_buf_opts({ modifiable = true }, function()
    api.nvim_buf_set_lines(bufnr, 0, -1, true, {})
    for i, f in ipairs(files) do
      self:_to_line(f):render(bufnr, ns_id, i, i + 1)
    end
  end)

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

  self:_view(dir, files)

  local line_count = api.nvim_buf_line_count(self.bufnr)

  if cursor_pos[1] <= line_count then
    api.nvim_win_set_cursor(winid, cursor_pos)
  else
    api.nvim_win_set_cursor(winid, { line_count, cursor_pos[2] })
  end
end

--- Copy selected files to clipboard
-- @tparam string action - action associated with these files, can be either `'Copy'` or `'Move'`
function Exp:_copy_to_cb(action)
  local cb = self.cb

  local files = self:get_sel_items(self.current.files)

  cb.action = action
  cb.files = files
end

--- Paste files from clipboard to the current directory
function Exp:_paste()
  local cb = self.cb
  local c = self.current

  if cb.action == 'Copy' then
    self.task:copy(cb.files, c.dir, uconf.task.cp_interval)
  elseif cb.action == 'Move' then
    self.task:move(cb.files, c.dir)
  end
end

--- Delete selected files
function Exp:_del_sel_files()
  local files = self:get_sel_items(self.current.files)
  local dir = self.current.dir

  local choice = fn.confirm(string.format('Delete %d items?', #files), 'Yes\nNo', 2)

  if choice ~= 1 then
    return
  end
  self.task:delete(files, dir)
end

function Exp:_show_rename_dialog()
  local file = self:get_current_item(self.current.files)

  local i = Input(input_opts.rename, {
    prompt = '',
    default_value = file.name,
    on_submit = wrap(async, self._rename, self, file, self.current.dir),
  })

  i:mount()
end

function Exp:_rename(file, dir, new_name)
  self.task:rename({ { file, new_name } }, dir)
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
    on_submit = wrap(async, self._create, self, type, self.current.dir),
  })

  i:mount()
end

function Exp:_create(type, dir, name)
  local client = self.client
  local err

  if type == 'File' then
    err = client:create_file(name, dir.id)
  elseif type == 'Dir' then
    err = client:create_dir(name, dir.id)
  else
    return
  end
  assert(not err, err)

  event.dir_modified:broadcast(dir.id)
end

return Exp
