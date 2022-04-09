local run = require('plenary.async').run
local icons = require 'nvim-web-devicons'
local Text = require 'nui.text'

local exp_icons = require('files-nvim.config').get_config().exp.icons

local api = vim.api

local round = function(num, idp)
  return tonumber(string.format('%.' .. (idp or 0) .. 'f', num))
end

-- Convert bytes to human readable format
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

local is_id_equal = function(id1, id2)
  return id1[1] == id2[1] and id1[2] == id2[2]
end

local async = function(fun, ...)
  local args = { ... }

  run(function()
    fun(unpack(args))
  end)
end

local wrap = function(fun, ...)
  local a0 = { ... }
  return function(...)
    local a1 = { ... }
    for _, v in ipairs(a1) do
      table.insert(a0, v)
    end

    fun(unpack(a0))
  end
end

local async_wrap = function(fun, ...)
  local a0 = { ... }
  return function(...)
    local a1 = { ... }
    for _, v in ipairs(a1) do
      table.insert(a0, v)
    end

    run(function()
      fun(unpack(a0))
    end)
  end
end

--- Checks whether the given mime type is for a text file.
local is_text = function(mime)
  local text = 'text'
  return mime:sub(1, #text) == text
end

--- Checks whether a local file is loaded in some buffer.
-- @return the `bufnr` if file is loaded, `false` otherwise.
local is_open = function(name)
  local bufs = api.nvim_list_bufs()

  for _, b in ipairs(bufs) do
    local buf_name = api.nvim_buf_get_name(b)
    local buf_is_loaded = api.nvim_buf_is_loaded(b)

    if buf_is_loaded and buf_name == name then
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
  async_wrap = async_wrap,
  is_text = is_text,
  is_open = is_open,
  get_icon = get_icon,
}
