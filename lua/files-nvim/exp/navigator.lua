local Navigator = {}

function Navigator:new(client)
  local n = {
    client = client,
    history = {
      index = 0,
      dirs = {},
    },
  }
  return setmetatable(n, { __index = self })
end

function Navigator:nav(dir)
  local h = self.history
  local new_index

  if h.index == 0 then
    new_index = 0
  else
    new_index = h.index + 1
  end

  h.dirs[new_index] = dir

  return self:_nav(new_index)
end

function Navigator:_nav(new_index)
  local h = self.history
  local dir = h.dirs[new_index]

  local err, files = self.client:list_meta(dir.id)
  assert(not err, vim.inspect(err))

  h.index = new_index

  return dir, files
end

return Navigator
