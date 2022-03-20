local conf = require 'files-nvim.config'

local setup = function(c)
  conf.set_config(c)
end

return {
  setup = setup,
}
