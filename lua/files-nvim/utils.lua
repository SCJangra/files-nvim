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
    return bytes, ' B'
  elseif (bytes >= kilobyte) and (bytes < megabyte) then
    return round(bytes / kilobyte, precision), 'KB'
  elseif (bytes >= megabyte) and (bytes < gigabyte) then
    return round(bytes / megabyte, precision), 'MB'
  elseif (bytes >= gigabyte) and (bytes < terabyte) then
    return round(bytes / gigabyte, precision), 'GB'
  elseif bytes >= terabyte then
    return round(bytes / terabyte, precision), 'TB'
  else
    return bytes, ' B'
  end
end

return {
  round = round,
  bytes_to_size = bytes_to_size,
}
