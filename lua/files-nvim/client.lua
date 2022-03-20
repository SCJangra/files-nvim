local a = require 'plenary.async'
local channel = require('plenary.async.control').channel
local uv = a.uv
local loop = vim.loop
local json = vim.json

local Client = {}

function Client:new()
  local c = {
    sock = '/tmp/files',
    jrpc_ver = '2.0',
    pipe = nil,
    res = {},
    nt = {},
    mid = 0,
  }
  return setmetatable(c, { __index = self })
end

function Client:start()
  if self.pipe then
    return
  end

  self.pipe = loop.new_pipe(false)

  local err = uv.pipe_connect(self.pipe, self.sock)
  assert(not err, err)

  self:_read_start()
end

function Client:_read_start()
  local line = nil

  self.pipe:read_start(function(err, data)
    assert(not err, err)

    if not data then
      self:stop()
      return
    end

    if line then
      table.insert(line, data)
    else
      line = { data }
    end

    local msg = nil
    if vim.endswith(data, '\n') then
      msg = table.concat(line)
      line = nil
    else
      return
    end

    local msgs = vim.split(msg, '\n', { trimempty = true })

    a.run(function()
      for _, m in ipairs(msgs) do
        self:_handle_msg(json.decode(m))
      end
    end)
  end)
end

function Client:_handle_msg(msg)
  if msg.id then
    self.res[msg.id](msg.error, msg.result)
    self.res[msg.id] = nil
    return
  end

  if msg.params then
    local params = msg.params
    local nt = self.nt
    local sub_id = params.subscription

    nt[sub_id].s(params.error, params.result)
  end
end

function Client:stop()
  if not self.pipe then
    return
  end

  uv.close(self.pipe)
  self.pipe = nil
end

function Client:request(method, params)
  self.mid = self.mid + 1

  local s, r = channel.oneshot()

  self.res[self.mid] = s

  local err = uv.write(
    self.pipe,
    json.encode {
      jsonrpc = self.jrpc_ver,
      method = method,
      params = params,
      id = self.mid,
    }
  )

  if err then
    self.res[self.mid] = nil
    assert(false, err)
  end

  return r()
end

function Client:subscribe(method, params)
  local err, sub_id = self:request(method, params)

  if err then
    return err
  end

  local s, r = channel.oneshot()

  local client = self

  local sr = {
    s = s,
    r = r,
  }

  function sr:recv()
    local err, res = self.r()

    if res == vim.NIL then
      client.nt[sub_id] = nil
      return err, res
    end

    local s, r = channel.oneshot()

    self.s = s
    self.r = r

    return err, res
  end

  function sr:for_each(cb)
    while true do
      local err, prog = self:recv()

      if prog == vim.NIL then
        return
      end

      cb(err, prog)
    end
  end

  self.nt[sub_id] = sr

  return err, sr
end

return Client
