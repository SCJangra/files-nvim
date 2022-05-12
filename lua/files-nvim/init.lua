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
local uv = require('plenary.async').uv
local a_util = require 'plenary.async.util'

-- misc
local event = require 'files-nvim.event'
local fn = vim.fn
local api = vim.api

local exps = {}

local start_server = function()
  pconf.tmp = os.tmpname()

  local err, _ = uv.fs_unlink(pconf.tmp)
  assert(not err, err)

  local err, _ = uv.fs_mkdir(pconf.tmp, 448) -- 448 is 700 in octal
  assert(not err, err)

  a_util.scheduler()

  api.nvim_create_autocmd({ 'VimLeavePre' }, {
    command = '!rm -rf ' .. pconf.tmp,
  })

  local s, r = channel.oneshot()

  local started = false

  local job_id = fn.jobstart({ fn.stdpath 'data' .. '/files-ipc', pconf:get_socket_addr() }, {
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
    if not pconf.tmp then
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
