local uconf = {
  -- Config for the explorer.
  exp = {
    icons = {
      dir = '',
      default = '',
    },
    -- Set the fields that are shown in the explorer.
    -- Currently supported fields are: { 'size' }.
    -- The name of the file is always shown whether it is set in the fields or not.
    fields = { 'size' },
    -- Set highlight groups here
    hl = {
      size = 'FilesNvimExpFileSize', -- hl for file sizes
      name = 'FilesNvimExpFileName', -- hl for file names
    },
    keymaps = {
      next = 'l', -- go to the next directory
      prev = 'h', -- go to previous directory
      up = 'H', -- go up one directory
      open = '<CR>', -- open file/directory under cursor
      copy = 'y', -- copy selected files to clipboard
      -- Copy selected files to clipboard and set the action to 'Move'.
      -- When the files are pasted using 'paste' map, these files will be
      -- moved to the new directory instead of copying.
      -- Trying to move files between different mount points, hard drives, partitions etc
      -- will cause an error.
      move = 'x',
      paste = 'p', -- paste files from the clipboard in the current directory
      delete = 'd', -- delete selected files
      rename = 'r', -- rename the file under cursor
      create_file = 'af', -- create a new file in current directory
      create_dir = 'ad', -- create a new directory in current directory
      show_tasks_split = 'ts', -- open task viewer/manager
    },
    -- Theese are the options for the floating dialog boxes
    -- that are opened when creating and renaming files.
    -- Look at these links for all supported options:
    --   https://github.com/MunifTanjim/nui.nvim/tree/main/lua/nui/popup
    --   https://github.com/MunifTanjim/nui.nvim/tree/main/lua/nui/input
    input_opts = {
      rename = {
        relative = 'win',
        position = '50%',
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
  -- config for task viewer/manager
  task = {
    -- how often to update the copy progress
    cp_interval = 500, -- update every 500 milliseconds
    hl = {
      prog_key = 'FilesNvimProgKey',
      prog_val = 'FilesNvimProgVal',
    },
  },
  -- These keymaps apply to all windows/buffers that are opened by this plugin
  keymaps = {
    quit = 'q', -- quit current buffer/window
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
