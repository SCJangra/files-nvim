local plugin = require 'files_nvim'

return {
  setup = function(opts)
    local config = plugin.get_config()
    config = vim.tbl_deep_extend('force', config, opts)
    plugin.set_config(config)
  end,
}
