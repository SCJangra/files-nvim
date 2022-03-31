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
    return {
      {
        { '  Progress: ', hl.prog_key },
        { string.format('%d / %d', done, total), hl.prog_val },
      },
    }
  end,
  delete = function(prog)
    return {
      {
        { '  Progress: ', hl.prog_key },
        { string.format('%d / %d', prog.done, prog.total), hl.prog_val },
      },
    }
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

function TaskGen:_task(method, args, prog_cal_fn, on_prog)
  local run = function(on_lines)
    local err, cancel, wait = self.client:subscribe(method, args, function(err, prog)
      if err then
        vim.schedule(function()
          notify(err, l.ERROR)
        end)
        return
      end

      if on_prog then
        on_prog(prog)
      end

      if on_lines and prog_cal_fn then
        on_lines(lines[method](prog_cal_fn(prog)))
      else
        on_lines(lines[method](prog))
      end
    end)

    assert(not err, err)
    return cancel, wait
  end

  return run
end

function TaskGen:copy(files, dst, prog_interval, on_prog)
  return {
    message = string.format('Copy %d items to %s', #files, dst.name),
    run = self:_task('copy', { files, dst, prog_interval }, nil, on_prog),
  }
end

function TaskGen:move(files, dst, on_prog)
  local total = #files
  local done = 0

  return {
    message = string.format('Move %d items to %s', #files, dst.name),
    run = self:_task('move', { files, dst }, function(_)
      done = done + 1
      return total, done
    end, on_prog),
  }
end

function TaskGen:delete(files, on_prog)
  return {
    message = string.format('Delete %d items', #files),
    run = self:_task('delete', { files }, nil, on_prog),
  }
end

return TaskGen
