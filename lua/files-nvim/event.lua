local get_event = function()
  return {
    id = 0,
    handlers = {},
  }
end

local mt = {
  __index = {
    add = function(self, handler)
      self.id = self.id + 1
      self.handlers[self.id] = handler
      return self.id
    end,
    remove = function(self, id)
      self.handlers[id] = nil
    end,
    broadcast = function(self, ...)
      for _, f in pairs(self.handlers) do
        f(...)
      end
    end,
  },
}

local event = {
  -- dir_id
  dir_modified = setmetatable(get_event(), mt),
  -- exp
  exp_closed = setmetatable(get_event(), mt),
}

return event
