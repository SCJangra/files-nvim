local percent = require('files-nvim.utils').percent
local api = vim.api
local o = vim.o

local cmds = {
  editor = {
    top = function(size)
      size = percent(size, o.lines)
      return string.format('topleft %dsplit', size)
    end,
    bottom = function(size)
      size = percent(size, o.lines)
      return string.format('botright %dsplit', size)
    end,
    left = function(size)
      size = percent(size, o.columns)
      return string.format('vertical topleft %dsplit', size)
    end,
    right = function(size)
      size = percent(size, o.columns)
      return string.format('vertical botright %dsplit', size)
    end,
  },
  win = {
    top = function(size, winid)
      size = percent(size, api.nvim_win_get_height(winid))
      return string.format('aboveleft %dsplit', size)
    end,
    bottom = function(size, winid)
      size = percent(size, api.nvim_win_get_height(winid))
      return string.format('belowright %dsplit', size)
    end,
    left = function(size, winid)
      size = percent(size, api.nvim_win_get_width(winid))
      return string.format('vertical leftabove %dsplit', size)
    end,
    right = function(size, winid)
      size = percent(size, api.nvim_win_get_width(winid))
      return string.format('vertical rightbelow %dsplit', size)
    end,
  },
}

local new_split = function(cmd, bufnr)
  api.nvim_command(cmd)

  local winid = api.nvim_get_current_win()

  if bufnr then
    api.nvim_win_set_buf(winid, bufnr)
  end

  return winid
end

local split = {
  editor = {
    top = function(size, bufnr)
      return new_split(cmds.editor.top(size), bufnr)
    end,
    bottom = function(size, bufnr)
      return new_split(cmds.editor.bottom(size), bufnr)
    end,
    left = function(size, bufnr)
      return new_split(cmds.editor.left(size), bufnr)
    end,
    right = function(size, bufnr)
      return new_split(cmds.editor.right(size), bufnr)
    end,
  },
  win = {
    top = function(size, winid, bufnr)
      return new_split(cmds.win.top(size, winid), bufnr)
    end,
    bottom = function(size, winid, bufnr)
      return new_split(cmds.win.bottom(size, winid), bufnr)
    end,
    left = function(size, winid, bufnr)
      return new_split(cmds.win.left(size, winid), bufnr)
    end,
    right = function(size, winid, bufnr)
      return new_split(cmds.win.right(size, winid), bufnr)
    end,
  },
}

return split
