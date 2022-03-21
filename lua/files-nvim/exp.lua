local Client = require 'files-nvim.client'
local conf = require 'files-nvim.config'
local uconf = conf.get_config()
local pconf = conf.pconf
local utils = require 'files-nvim.utils'
local Navigator = require 'files-nvim.exp.navigator'

local keymap = vim.keymap
local api = vim.api

local a_util = require 'plenary.async.util'
local Line = require 'nui.line'
local Text = require 'nui.text'

local Exp = {}

function Exp:new(fields)
  local client = Client:new()
  local e = {
    client = client,
    bufnr = nil,
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
    ns_id = api.nvim_create_namespace '',
  }

  return setmetatable(e, { __index = self })
end

function Exp:open_current()
  if self.bufnr then
    return
  end

  local bufnr = api.nvim_create_buf(true, true)
  api.nvim_win_set_buf(0, bufnr)
  self.bufnr = bufnr

  self:_map(
    uconf.keymaps.quit,
    utils.wrap(function()
      self:close()
    end)
  )
  self:_setup()
end

function Exp:close(winid)
  local bufnr = self.bufnr

  if not bufnr then
    return
  end

  if winid then
    api.nvim_win_close(winid, false)
  end

  api.nvim_buf_clear_namespace(bufnr, self.ns_id, 0, -1)
  api.nvim_buf_delete(bufnr, { force = true })
  self.bufnr = nil

  self.client:stop()
end

function Exp:_setup_keymaps()
  -- local bufnr = self.bufnr
  -- local km = conf.keymaps
end

function Exp:_setup(winid)
  local client = self.client

  local err = client:start()
  assert(not err, err)

  a_util.scheduler()
  local cwd = vim.fn.getcwd(winid or 0)

  local err, dir = client:get_meta { 'Local', cwd }
  assert(not err, vim.inspect(err))

  self:_nav(dir)
end

function Exp:_nav(dir)
  local cur = self.current
  local bufnr = self.bufnr
  local ns_id = self.ns_id

  local dir, files = self.nav:nav(dir)

  cur.dir = dir
  cur.files = files

  a_util.scheduler()

  for i, f in ipairs(files) do
    self:_to_line(f):render(bufnr, ns_id, i, i + 1)
  end
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

return Exp
