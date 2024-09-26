local api = vim.api

local setup = function()
  -- TODO: Highlight groups are defined for global namespace, use a plugin specific namespace.
  api.nvim_set_hl(0, 'FilesNvimDirectoryIcon', { fg = 'Orange', ctermfg = 'LightRed' })
end

return {
  setup = setup
}
