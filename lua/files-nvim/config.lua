local uconf = {
  exp = {
    fields = { 'size' },
    hl = {
      size = 'FilesNvimExpFileSize',
      name = 'FilesNvimExpFileName',
    },
    keymaps = {
      next = 'l',
      prev = 'h',
      up = 'H',
      open = '<CR>',
    },
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
end

local get_config = function()
  return uconf
end

return {
  set_config = set_config,
  get_config = get_config,
  pconf = pconf,
}
