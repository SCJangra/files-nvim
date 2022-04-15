-- Config
local hl = require('files-nvim.config').get_config().task.hl

-- misc
local utils = require 'files-nvim.utils'

local lines = {}

function lines.copy_all(prog)
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
end

function lines.mv_all(prog)
  return {
    {
      { '  Progress: ', hl.prog_key },
      { string.format('%d / %d', prog.done, prog.total), hl.prog_val },
    },
  }
end

function lines.delete_all(prog)
  return {
    {
      { '  Progress: ', hl.prog_key },
      { string.format('%d / %d', prog.done, prog.total), hl.prog_val },
    },
  }
end

return lines
