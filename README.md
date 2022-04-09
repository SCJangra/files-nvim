# A file explorer for neovim

This is not (for now) a file tree like [nvim-tree.lua](https://github.com/kyazdani42/nvim-tree.lua),
[neo-tree.nvim](https://github.com/nvim-neo-tree/neo-tree.nvim) etc. This is supposed to be more like
external file explorers like thunar and pcmanfm.

This plugin will also support managing files in remote file storage services like [Mega](https://mega.io/)
and [Google Drive](https://drive.google.com/). This plugin uses the [files](https://github.com/SCJangra/files) crate
so as soon as support for remote files is implemented there, it will also be available here.

## Dependencies
- A linux os
- neovim >= 0.7.0
- wget
- [nui.nvim](https://github.com/MunifTanjim/nui.nvim)
- [plenary.nvim](https://github.com/nvim-lua/plenary.nvim)
- [nvim-web-devicons](https://github.com/kyazdani42/nvim-web-devicons)

## Install
``` lua
use {
    'SCJangra/files-nvim',
    requires = {
      'kyazdani42/nvim-web-devicons',
      'MunifTanjim/nui.nvim',
      'nvim-lua/plenary.nvim'
    },
    config = function() require('files-nvim').setup {} end,
    run = function(c)
      vim.api.nvim_command('!bash ' .. c.install_path .. '/install.sh ' .. vim.fn.stdpath 'data')
    end,
}
```

## Usage
Open explorer in current window  
`:lua require('files-nvim').open_current({ 'size' }, true)`

Open explorer in a split  
`:lua require('files-nvim').open_split({ 'size' }, 'win', 'left', 50)`

To select multiple files to copy/move/delete got to Visual Line Mode using 'V' (Shift-v).

## Default config
``` lua
{
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
```
