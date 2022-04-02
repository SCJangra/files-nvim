local id = 0

local handlers = {
  -- The comments below denote the arguments that are passed to handlers
  -- for these events

  -- task_id, files, dest_dir
  copy_start = {},
  -- task_id, progress
  copy_prog = {},
  -- task_id
  copy_end = {},
  -- task_id, files, dest_dir
  move_start = {},
  -- task_id, progress
  move_prog = {},
  -- task_id
  move_end = {},
  -- task_id, files, from_dir
  delete_start = {},
  -- task_id, progress
  delete_prog = {},
  -- task_id
  delete_end = {},
  -- file_old, file_new, dir
  renamed = {},
  -- type, file, dir
  created = {},
  -- exp
  exp_closed = {},
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
