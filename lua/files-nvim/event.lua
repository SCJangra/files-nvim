local id = 0

local handlers = {
  copy_start = {},
  copy_prog = {},
  copy_end = {},
  move_start = {},
  move_prog = {},
  move_end = {},
  delete_start = {},
  delete_prog = {},
  delete_end = {},
  renamed = {},
  created_file = {},
  created_dir = {},
}

local Event = {}

function Event:on(event, handler)
  id = id + 1
  handlers[event][id] = handler
  return id
end

function Event:off(event, id)
  handlers[event][id] = nil
end

function Event:broadcast(event, ...)
  local h = handlers[event]

  for _, f in pairs(h) do
    f(...)
  end
end

return Event
