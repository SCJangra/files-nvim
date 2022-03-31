local a = require 'plenary.async'
local run = require('plenary.async').run

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

return {
  round = round,
  bytes_to_size = bytes_to_size,
  percent = percent,
  is_id_equal = is_id_equal,
  async = async,
  wrap = wrap,
  async_wrap = async_wrap,
}
