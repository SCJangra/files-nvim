-- Config
local conf = require 'files-nvim.config'

-- Classes
local Exp = require 'files-nvim.exp'

-- Dependencies
local run = require('plenary.async').run

-- misc
local event = require 'files-nvim.event'

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

return {
  setup = setup,
  open_split = open_split,
  open_current = open_current,
}
