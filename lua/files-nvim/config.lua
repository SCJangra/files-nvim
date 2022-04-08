local uconf = {
  exp = {
    fields = { 'size' },
    hl = {
      size = 'FilesNvimExpFileSize',
      name = 'FilesNvimExpFileName',
      prog_key = 'FilesNvimProgKey',
      prog_val = 'FilesNvimProgVal',
    },
    keymaps = {
      next = 'l',
      prev = 'h',
      up = 'H',
      open = '<CR>',
      copy = 'y',
      move = 'x',
      paste = 'p',
      delete = 'd',
      rename = 'r',
      create_file = 'af',
      create_dir = 'ad',
      show_tasks_split = 'ts',
    },
    input_opts = {
      rename = {
        relative = 'cursor',
        position = {
          row = 1,
          col = 0,
        },
        size = '80%',
        border = {
          style = 'rounded',
          text = {
            top = ' Rename ',
            top_align = 'left',
          },
        },
      },
      create_file = {
        relative = 'win',
        position = '50%',
        size = '80%',
        border = {
          style = 'rounded',
          text = {
            top = ' Create File ',
            top_align = 'left',
          },
        },
      },
      create_dir = {
        relative = 'win',
        position = '50%',
        size = '80%',
        border = {
          style = 'rounded',
          text = {
            top = ' Create Dir ',
            top_align = 'left',
          },
        },
      },
    },
  },
  task = {
    cp_interval = 500,
  },
  keymaps = {
    quit = 'q',
  },
}

local pconf = {
  map_opts = {
    noremap = true,
    silent = true,
  },
}

local set_config = function(c)
  uconf = vim.tbl_deep_extend('force', uconf, c)

  for k, _ in pairs(package.loaded) do
    if k:match '^files%-nvim' and not k:match '^files%-nvim.config' then
      package.loaded[k] = nil
    end
  end
end

local get_config = function()
  return uconf
end

return {
  set_config = set_config,
  get_config = get_config,
  pconf = pconf,
}
