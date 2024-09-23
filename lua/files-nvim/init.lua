local plugin = require 'files_nvim'
local i = require('nvim-web-devicons')

return {
  setup = function(opts)
    opts.icons = {
      file_name = i.get_icons_by_filename(),
      extension = i.get_icons_by_extension()
    }

    local config = plugin.get_config()
    config = vim.tbl_deep_extend('force', config, opts)
    plugin.set_config(config)
  end,
}
