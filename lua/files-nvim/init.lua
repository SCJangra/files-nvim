---@diagnostic disable: unused-local

-- Config
local conf = require 'files-nvim.config'
local pconf = conf.pconf

-- Classes
local Exp = require 'files-nvim.exp'
local Client = require 'files-nvim.client'

-- Dependencies
local run = require('plenary.async').run
local channel = require('plenary.async.control').channel

-- misc
local event = require 'files-nvim.event'
local fn = vim.fn

local exps = {}

local start_server = function()
  pconf.socket = os.tmpname()

  local s, r = channel.oneshot()

  local started = false

  local job_id = fn.jobstart({ fn.stdpath 'data' .. '/files-ipc', pconf.socket }, {
    on_stdout = function()
      if started then
        return
      end

      started = true
      run(function()
        s()
      end)
    end,
  })
  assert(job_id ~= 0, 'Invalid arguments to ipc server')
  assert(job_id ~= -1, 'server binary is not executable')

  r()
end

local setup = function(c)
  conf.set_config(c)
end

local open = function(mode, fields, ...)
  local args = { ... }

  run(function()
    if not pconf.socket then
      start_server()
    end

    local e = Exp:new(fields)
    e['open_' .. mode](e, unpack(args))
    exps[e] = true
  end)
end

event.exp_closed:add(function(e)
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
