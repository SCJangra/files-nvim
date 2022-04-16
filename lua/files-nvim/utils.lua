--- Here are some utility functions that are used throughout the project.

-- Dependencies
local run = require('plenary.async').run
local icons = require 'nvim-web-devicons'
local Text = require 'nui.text'

-- Config
local exp_icons = require('files-nvim.config').get_config().exp.icons

-- Neovim builtin
local api = vim.api

local round = function(num, idp)
  return tonumber(string.format('%.' .. (idp or 0) .. 'f', num))
end

--- Convert bytes to human readable format
-- @tparam number bytes - the number of bytes
-- @treturn string - human readable representation of the given bytes
local bytes_to_size = function(bytes)
  local precision = 2
  local kilobyte = 1024
  local megabyte = kilobyte * 1024
  local gigabyte = megabyte * 1024
  local terabyte = gigabyte * 1024

  if (bytes >= 0) and (bytes < kilobyte) then
    return bytes .. '  B'
  elseif (bytes >= kilobyte) and (bytes < megabyte) then
    return round(bytes / kilobyte, precision) .. ' KB'
  elseif (bytes >= megabyte) and (bytes < gigabyte) then
    return round(bytes / megabyte, precision) .. ' MB'
  elseif (bytes >= gigabyte) and (bytes < terabyte) then
    return round(bytes / gigabyte, precision) .. ' GB'
  elseif bytes >= terabyte then
    return round(bytes / terabyte, precision) .. ' TB'
  else
    return bytes .. '  B'
  end
end

local percent = function(val, of)
  return (of / 100) * val
end

--- Check whether two file ids are equal
-- @tparam {string,string} first - the first id
-- @tparam {string,string} second - the second id
-- @treturn boolean - `true` when both ids are equal, `false` otherwise
local is_id_equal = function(first, second)
  return first[1] == second[1] and first[2] == second[2]
end

--- Run the given function in async context
-- @tparam function fun - the function to run
-- @tparam varargs ... - arguments passed to this function
local async = function(fun, ...)
  local args = { ... }

  run(function()
    fun(unpack(args))
  end)
end

--- Wrap the given function into another function
-- @tparam function fun - the function to wrap
-- @tparam varargs ... - arguments passed to the given function
-- @treturn function - a function which when called will call the given function with the given arguments.
-- Any additional arguments passed to this function will also be passed to the original function.
local wrap = function(fun, ...)
  local a0 = { ... }
  return function(...)
    local a1 = { ... }
    for _, v in ipairs(a1) do
      table.insert(a0, v)
    end

    return fun(unpack(a0))
  end
end

--- Checks whether the given mime type is for a text file.
-- @tparam string mime - mime type to check for
-- @treturn boolean - `true` if the mime type represents a text file, `false` otherwise
local is_text = function(mime)
  local text = 'text'
  return mime:sub(1, #text) == text
end

--- Checks whether a local file is loaded in some buffer.
-- @tparam string path - the file to check for
-- @treturn number|boolean - the `bufnr` if file is loaded, `false` otherwise.
local is_open = function(path)
  local bufs = api.nvim_list_bufs()

  for _, b in ipairs(bufs) do
    local buf_name = api.nvim_buf_get_name(b)
    local buf_is_loaded = api.nvim_buf_is_loaded(b)

    if buf_is_loaded and buf_name == path then
      return b
    end
  end

  return false
end

local get_icon = function(file)
  local icon = file.file_type == 'Dir' and exp_icons.dir or icons.get_icon(file.name)
  local t = Text(icon or exp_icons.default)
  return t
end

return {
  round = round,
  bytes_to_size = bytes_to_size,
  percent = percent,
  is_id_equal = is_id_equal,
  async = async,
  wrap = wrap,
  is_text = is_text,
  is_open = is_open,
  get_icon = get_icon,
}
