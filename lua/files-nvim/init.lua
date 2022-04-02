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

return {
  setup = setup,
  open = open,
}
