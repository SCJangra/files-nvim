local conf = require 'files-nvim.config'
local Exp = require 'files-nvim.exp'
local event = require 'files-nvim.event'

local run = require('plenary.async').run

local exps = {}

local setup = function(c)
  conf.set_config(c)
end

local open = function(mode, fields, ...)
  local args = { ... }

  run(function()
    local e = Exp:new(fields)
    e['open_' .. mode](e, unpack(args))
    exps[e] = true
  end)
end

event:on('exp_closed', function(e)
  exps[e] = nil
end)

local open_current = function(fields, listed)
  open('current', fields, listed)
end

local open_split = function(fields, rel, pos, size)
  open('split', fields, rel, pos, size)
end

local install = function(c)
  local os = jit.os
  local bin_url

  if os == 'OSX' then
    bin_url = 'https://github.com/SCJangra/files-ipc/releases/download/unstable/files-ipc-macos'
  elseif os == 'Linux' then
    bin_url = 'https://github.com/SCJangra/files-ipc/releases/download/unstable/files-ipc-linux'
  else
    assert(false, string.format('OS %s is not supported', os))
  end

  local command = string.format('!bash %s/install.sh %s %s/files-ipc', c.install_path, bin_url, vim.fn.stdpath 'data')

  vim.api.nvim_command(command)
end

return {
  setup = setup,
  open_split = open_split,
  open_current = open_current,
  install = install,
}
