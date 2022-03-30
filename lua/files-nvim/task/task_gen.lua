local hl = require('files-nvim.config').get_config().exp.hl
local utils = require 'files-nvim.utils'

local l = vim.log.levels
local notify = vim.notify

local lines = {
  copy = function(prog)
    local f = prog.files
    local s = prog.size
    local c = prog.current
    local prog_key = hl.prog_key
    local prog_val = hl.prog_val
    local bts = utils.bytes_to_size

    local l = {
      {
        { '  Files: ', prog_key },
        { string.format('%d / %d', f.done, f.total), prog_val },
      },
      {
        { '  Size: ', prog_key },
        { string.format('%s / %s', bts(s.done), bts(s.total)), prog_val },
      },
      {
        { '  Current: ', prog_key },
        { c.name, prog_val },
      },
      {
        { '    Size: ', prog_key },
        { string.format('%s / %s', bts(c.prog.done), bts(c.prog.total)), prog_val },
      },
    }

    return l
  end,
  move = function(total, done)
    local prog_key = hl.prog_key
    local prog_val = hl.prog_val

    local l = {
      {
        { '  Progress: ', prog_key },
        { string.format('%d / %d', done, total), prog_val },
      },
    }
    return l
  end,
}

local TaskGen = {}

function TaskGen:new(client)
  local tg = {
    client = client,
  }
  self.__index = self
  return setmetatable(tg, self)
end

function TaskGen:copy(files, dst, prog_interval, on_prog)
  return {
    message = string.format('Copy %d items to %s', #files, dst.name),
    run = function(on_prog1)
      local err, cancel, wait = self.client:copy(files, dst, prog_interval, function(err, prog)
        assert(not err, err)
        on_prog(prog)
        on_prog1(lines.copy(prog))
      end)

      assert(not err, err)
      return cancel, wait
    end,
  }
end

function TaskGen:move(files, dst, on_prog)
  local total = #files
  local done = 0

  return {
    message = string.format('Move %d items to %s', #files, dst.name),
    run = function(on_prog1)
      local err, cancel, wait = self.client:move(files, dst, function(err, m)
        if err then
          vim.schedule(function()
            notify(err, l.ERROR)
          end)
          return
        end

        done = done + 1
        on_prog(m)
        on_prog1(lines.move(total, done))
      end)

      assert(not err, err)
      return cancel, wait
    end,
  }
end

return TaskGen
