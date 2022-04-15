local id = 0

local handlers = {
  -- The comments below denote the arguments that are passed to handlers
  -- for these events

  -- task_id, files, dest_dir
  copy_all_start = {},
  -- task_id, progress
  copy_all_prog = {},
  -- task_id
  copy_all_end = {},
  -- task_id, files, dest_dir
  mv_all_start = {},
  -- task_id, progress
  mv_all_prog = {},
  -- task_id
  mv_all_end = {},
  -- task_id, files, from_dir
  delete_all_start = {},
  -- task_id, progress
  delete_all_prog = {},
  -- task_id
  delete_all_end = {},
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
