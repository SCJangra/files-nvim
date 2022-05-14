-- Classes
local Buf = require 'files-nvim.buf'

-- Misc
local utils = require 'files-nvim.utils'
local wrap, async = utils.wrap, utils.async
local api = vim.api

-- Config
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local exp = uconf.exp

local Ren = Buf:new()

function Ren:new(task)
  local r = {
    task = task,
  }

  self.__index = self

  return setmetatable(r, self)
end

function Ren:open_current()
  getmetatable(getmetatable(self).__index).__index.open_current(self)

  self:set_buf_opts { modifiable = true }
  self:_setup_keymaps()
end

function Ren:open_split(rel, pos, size)
  getmetatable(getmetatable(self).__index).__index.open_split(self, rel, pos, size)

  self:set_buf_opts { modifiable = true }
  self:_setup_keymaps()
end

function Ren:set_files(files, dir)
  self.files = files
  self.dir = dir

  local lines = {}
  for k, f in ipairs(files) do
    lines[k] = f.name
  end

  api.nvim_buf_set_lines(self.bufnr, 0, -1, true, lines)
end

function Ren:_close_and_rename()
  local names = api.nvim_buf_get_lines(self.bufnr, 0, -1, true)

  local r = {}

  for k, f in ipairs(self.files) do
    local n = names[k]

    if n and n ~= '' and n ~= f.name then
      table.insert(r, { f, n })
    end
  end

  async(self.task.rename, self.task, r, self.dir)
  self:close()
end

function Ren:_setup_keymaps()
  getmetatable(getmetatable(self).__index).__index._setup_keymaps(self)

  local km = exp.keymaps

  self:map('n', km.close_and_rename, wrap(self._close_and_rename, self))
end

return Ren
