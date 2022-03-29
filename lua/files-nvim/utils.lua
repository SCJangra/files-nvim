local a = require 'plenary.async'

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

local call = function(obj, fun, ...)
  fun(obj, ...)
end

local call_wrap = function(obj, fun, ...)
  local params = { ... }

  return function()
    fun(obj, unpack(params))
  end
end

local call_async = function(obj, fun, ...)
  local params = { ... }

  a.run(function()
    fun(obj, unpack(params))
  end)
end

local call_wrap_async = function(obj, fun, ...)
  local params = { ... }

  return function()
    a.run(function()
      fun(obj, unpack(params))
    end)
  end
end

local percent = function(val, of)
  return (of / 100) * val
end

return {
  round = round,
  bytes_to_size = bytes_to_size,
  call = call,
  call_async = call_async,
  call_wrap = call_wrap,
  call_wrap_async = call_wrap_async,
  percent = percent,
}
