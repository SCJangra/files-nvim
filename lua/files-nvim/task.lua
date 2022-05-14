-- Dependencies
local a_util = require 'plenary.async.util'

-- Classes
local Buf = require 'files-nvim.buf'

-- Config
local conf = require 'files-nvim.config'
local uconf = conf.get_config()

-- misc
local utils = require 'files-nvim.utils'
local async, wrap = utils.async, utils.wrap
local event = require 'files-nvim.event'
local lines = require 'files-nvim.task.prog_lines'

-- Neovim builtin
local api = vim.api
local notify = vim.notify
local l = vim.log.levels
local schedule = vim.schedule
local fn = vim.fn

local Task = Buf:new()

function Task:new(client)
  local t = {
    tasks = {},
    client = client,
  }
  self.__index = self
  return setmetatable(t, self)
end

function Task:open_current()
  getmetatable(getmetatable(self).__index).__index.open_current(self)

  self:set_name 'Tasks'
  self:_setup_keymaps()
  self:_show_tasks()
end

function Task:open_split(rel, pos, size)
  getmetatable(getmetatable(self).__index).__index.open_split(self, rel, pos, size)

  self:set_name 'Tasks'
  self:_setup_keymaps()
  self:_show_tasks()
end

function Task:copy(files, dst, prog_interval)
  local msg = string.format('Copy %d items to %s %s', #files, utils.get_icon(dst), dst.name)
  local prog_fn = function(_)
    event.dir_modified:broadcast(dst.id)
  end
  local fn = wrap(self.client.copy_all, self.client, files, dst, prog_interval)

  self:_run(msg, fn, lines.copy_all, nil, prog_fn, nil)
end

function Task:move(files, dst)
  local msg = string.format('Move %d files to %s %s', #files, utils.get_icon(dst), dst.name)
  local fn = wrap(self.client.mv_all, self.client, files, dst)

  self:_run(msg, fn, lines.prog)
  event.dir_modified:broadcast(dst.id)
end

function Task:delete(files, dir)
  local msg = string.format('Delete %d items from %s %s', #files, utils.get_icon(dir), dir.name)
  local fn = wrap(self.client.delete_all, self.client, files)

  self:_run(msg, fn, lines.prog)
  event.dir_modified:broadcast(dir.id)
end

function Task:rename(rns, dir)
  local msg = string.format('Rename %d files', #rns)
  local fn = wrap(self.client.rename_all, self.client, rns)

  self:_run(msg, fn, lines.prog)
  event.dir_modified:broadcast(dir.id)
end

function Task:_run(msg, fn, lines_fn, start_fn, prog_fn, end_fn)
  local t = {
    message = msg,
  }

  local err, _, cancel, wait = fn(function(err, prog)
    if err then
      schedule(function()
        notify(err, l.ERROR)
      end)
      return
    end

    if prog_fn then
      prog_fn(prog)
    end

    self:_show_prog(t, lines_fn(prog))
  end)
  assert(not err, err)

  t.cancel = cancel

  if start_fn then
    start_fn()
  end
  self:_insert_task(t)

  wait()

  if end_fn then
    end_fn()
  end
  self:_remove_task(t)
end

function Task:_setup_keymaps()
  getmetatable(getmetatable(self).__index).__index._setup_keymaps(self)

  local km = uconf.task.keymaps

  self:map({ 'n', 'x' }, km.cancel, wrap(async, self._cancel_sel_tasks, self))
end

function Task:_insert_task(t)
  local tasks = self.tasks
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  table.insert(tasks, t)

  t.index = #tasks

  if bufnr then
    local k = #tasks
    local i = k - 1

    a_util.scheduler()

    self:with_buf_opts({ modifiable = true }, function()
      api.nvim_buf_set_lines(bufnr, i, k, false, { t.message })
    end)

    local extmark_id = api.nvim_buf_set_extmark(bufnr, ns_id, i, 0, {})
    t.extmark_id = extmark_id
  end
end

function Task:_remove_task(t)
  local tasks = self.tasks
  local bufnr = self.bufnr
  local ns_id = self.ns_id
  local extmark_id = t.extmark_id

  table.remove(tasks, t.index)

  for i = t.index, #tasks do
    tasks[i].index = i
  end

  if bufnr and extmark_id then
    a_util.scheduler()

    local em = api.nvim_buf_get_extmark_by_id(bufnr, ns_id, extmark_id, { details = false })
    local row = em[1]

    api.nvim_buf_del_extmark(bufnr, ns_id, extmark_id)

    self:with_buf_opts({ modifiable = true }, function()
      api.nvim_buf_set_lines(bufnr, row, row + 1, true, {})
    end)
  end
end

function Task:_show_tasks()
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  for k, t in ipairs(self.tasks) do
    local i = k - 1

    self:with_buf_opts({ modifiable = true }, function()
      api.nvim_buf_set_lines(bufnr, i, k, false, { t.message })
    end)

    local extmark_id = api.nvim_buf_set_extmark(bufnr, ns_id, i, 0, {})

    t.extmark_id = extmark_id
  end
end

function Task:_show_prog(t, lines)
  local bufnr = self.bufnr
  local extmark_id = t.extmark_id
  local ns_id = self.ns_id

  if not bufnr or not extmark_id then
    return
  end

  schedule(function()
    local em = api.nvim_buf_get_extmark_by_id(bufnr, ns_id, extmark_id, { details = false })

    api.nvim_buf_set_extmark(bufnr, ns_id, em[1], em[2], {
      id = extmark_id,
      virt_lines = lines,
    })
  end)
end

function Task:_cancel_sel_tasks()
  local tasks = self:get_sel_items(self.tasks)

  local choice = fn.confirm(string.format('Cancel %d tasks?', #tasks), 'Yes\nNo', 2)

  if choice ~= 1 then
    return
  end

  for _, t in ipairs(tasks) do
    t.cancel()
  end
end

return Task
