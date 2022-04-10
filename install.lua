local install = function(c)
  local os = jit.os
  local bin_url

  if os == 'OSX' then
    bin_url = 'https://github.com/SCJangra/files-ipc/releases/download/unstable/files-ipc-macos'
  elseif os == 'Linux' then
    bin_url = 'https://github.com/SCJangra/files-ipc/releases/download/unstable/files-ipc-linux'
  else
    assert(false, string.format('OS %s is not supported', os))
  end

  local command = string.format('!bash %s/install.sh %s %s/files-ipc', c.install_path, bin_url, vim.fn.stdpath 'data')

  vim.api.nvim_command(command)
end

return install
