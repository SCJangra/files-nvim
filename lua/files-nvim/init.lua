local prefix = function(icon)
  return {
    icon = icon.icon,
    name = 'DevIcon' .. icon.name
  }
end

local setup = function(opts)
  local plugin = require 'files_nvim'
  local i = require('nvim-web-devicons')
  local default = i.get_default_icon()

  if not i.has_loaded() then i.setup() end

  require('files-nvim.hl').setup()

  local icons = {
    file_name = vim.tbl_map(prefix, i.get_icons_by_filename()),
    extension = vim.tbl_map(prefix, i.get_icons_by_extension()),
    default = prefix(default),
  }

  local config = vim.tbl_deep_extend('force', plugin.get_config(), { icons = icons }, opts)
  plugin.set_config(config)
end

return {
  setup = setup,
}
