local hl = require('files-nvim.config').get_config().exp.hl
local utils = require 'files-nvim.utils'

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
}

local TaskGen = {}

function TaskGen:new(client)
  local tg = {
    client = client,
  }
  self.__index = self
  return setmetatable(tg, self)
end

function TaskGen:_task(name, args)
  local t = {}
  t.run = function(on_prog)
    local err, cancel, wait = self.client.subscribe(self.client, name, args, function(err, prog)
      assert(not err, err)
      on_prog(lines.copy(prog))
    end)
    assert(not err, err)

    return cancel, wait
  end

  return t
end

function TaskGen:copy(files, dst, prog_interval)
  local t = self:_task('copy', { files, dst, prog_interval })
  t.message = string.format('Copy %d items to %s', #files, dst.name)
  return t
end

return TaskGen
