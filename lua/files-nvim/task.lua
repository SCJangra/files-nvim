local a_util = require 'plenary.async.util'

local Buf = require 'files-nvim.buf'
local TaskGen = require 'files-nvim.task.task_gen'
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local utils = require 'files-nvim.utils'
local async_wrap = utils.async_wrap

local api = vim.api

local Task = Buf:new()

function Task:new(client)
  local t = {
    tasks = {},
    tg = TaskGen:new(client),
  }
  self.__index = self
  return setmetatable(t, self)
end

function Task:open_current()
  getmetatable(getmetatable(self).__index).__index.open_current(self)

  self:_setup_keymaps()
  self:_show_tasks()
end

function Task:open_split(rel, pos, size)
  getmetatable(getmetatable(self).__index).__index.open_split(self, rel, pos, size)

  self:_setup_keymaps()
  self:_show_tasks()
end

function Task:run(task)
  local t = {
    message = task.message,
  }

  local cancel, wait = task.run(function(lines)
    self:_show_prog(t, lines)
  end)

  t.cancel = cancel

  self:_insert_task(t)

  wait()

  self:_remove_task(t)
end

function Task:copy(files, dst, prog_interval, on_prog)
  return self:run(self.tg:copy(files, dst, prog_interval, on_prog))
end

function Task:move(files, dst, on_prog)
  return self:run(self.tg:move(files, dst, on_prog))
end

function Task:delete(files, on_prog)
  return self:run(self.tg:delete(files, on_prog))
end

function Task:_setup_keymaps()
  local gkm = uconf.keymaps

  self:map('n', gkm.quit, async_wrap(self.close, self))
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

    api.nvim_buf_set_lines(bufnr, i, k, false, { t.message })
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
    api.nvim_buf_set_lines(bufnr, row, row + 1, true, {})
  end
end

function Task:_show_tasks()
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  for k, t in ipairs(self.tasks) do
    local i = k - 1

    api.nvim_buf_set_lines(bufnr, i, k, false, { t.message })
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

  a_util.scheduler()

  local em = api.nvim_buf_get_extmark_by_id(bufnr, ns_id, extmark_id, { details = false })

  api.nvim_buf_set_extmark(bufnr, ns_id, em[1], em[2], {
    id = extmark_id,
    virt_lines = lines,
  })
end

return Task
