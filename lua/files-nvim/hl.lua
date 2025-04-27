local api = vim.api

local setup = function()
  -- TODO: Highlight groups are defined for global namespace, use a plugin specific namespace.
  api.nvim_set_hl(0, 'FilesNvimDirectoryIcon', { fg = 'Orange', ctermfg = 'LightRed' })
  api.nvim_set_hl(0, 'FilesNvimCut', { fg = 'Orange', ctermfg = 'LightRed' })
  api.nvim_set_hl(0, 'FilesNvimCopy', { fg = 'Green', ctermfg = 'Green' })
  api.nvim_set_hl(0, 'FilesNvimTaskHead', { fg = 'White', ctermfg = 'White', bg = 'Orange', ctermbg = 'LightRed' })
  api.nvim_set_hl(0, 'FilesNvimTaskBody', { fg = 'White', ctermfg = 'White', bg = 'Orange', ctermbg = 'LightRed' })
end

return {
  setup = setup,
}
