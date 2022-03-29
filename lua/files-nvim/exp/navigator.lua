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

  if h.index == 0 and not h.dirs[h.index] then
    new_index = 0
  else
    new_index = h.index + 1
  end

  h.dirs[new_index] = dir

  local dir, files = self:_nav(new_index)
  h.dirs[new_index + 1] = nil

  return dir, files
end

function Navigator:next()
  local h = self.history
  local new_index = h.index + 1

  if not h.dirs[new_index] then
    return
  end

  return self:_nav(new_index)
end

function Navigator:prev()
  local h = self.history
  local new_index = h.index - 1

  if not h.dirs[new_index] then
    return
  end

  return self:_nav(new_index)
end

function Navigator:up()
  local h = self.history
  local dir = h.dirs[h.index]

  if dir.parent_id == vim.NIL then
    return
  end

  local err, dir = self.client:get_meta(dir.parent_id)
  assert(not err, err)

  return self:nav(dir)
end

function Navigator:_nav(new_index)
  local h = self.history
  local dir = h.dirs[new_index]

  local err, files = self.client:list_meta(dir.id)
  assert(not err, err)

  h.index = new_index

  return dir, files
end

return Navigator
