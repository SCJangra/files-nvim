local a = require 'plenary.async'
local channel = require('plenary.async.control').channel
local uv = a.uv
local loop = vim.loop
local json = vim.json
local fn = vim.fn

local uconf = require('files-nvim.config').get_config()

local Client = {}

function Client:new()
  local c = {
    jrpc_ver = '2.0',
    pipe = loop.new_pipe(false),
    job_id = nil,
    res = {},
    nt = {},
    mid = 0,
  }
  return setmetatable(c, { __index = self })
end

function Client:start()
  if self.job_id then
    return
  end

  local sock = os.tmpname()

  local s, r = channel.oneshot()
  local started = false

  self.job_id = fn.jobstart({ fn.stdpath 'data' .. '/files-ipc', sock }, {
    on_stdout = function()
      if started then
        return
      end

      started = true
      a.run(function()
        s()
      end)
    end,
  })
  assert(self.job_id ~= 0, 'Invalid arguments to ipc server')
  assert(self.job_id ~= -1, 'server binary is not executable')

  r()

  local err = uv.pipe_connect(self.pipe, sock)
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
    local err = msg.error

    err = err and string.format('%s. %s', err.message, err.data)

    self.res[msg.id](err, msg.result)
    self.res[msg.id] = nil
    return
  end

  if msg.params then
    local params = msg.params
    local nt = self.nt
    local sub_id = params.subscription

    local err = params.error

    err = err and string.format('%s. %s', err.message, err.data)

    nt[sub_id](err, params.result)
  end
end

function Client:stop()
  if not self.job_id then
    return
  end

  fn.jobstop(self.job_id)
  self.job_id = nil
  self.pipe:close()
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

function Client:subscribe(method, params, on_prog)
  local err, sub_id = self:request(method, params)

  if err then
    return err
  end

  local s, r = channel.oneshot()

  local cancel = function()
    local err, res = self:request(method .. '_c', { sub_id })

    self.nt[sub_id] = nil
    s()

    return err, res
  end

  self.nt[sub_id] = function(err, res)
    if res == vim.NIL then
      s()
    else
      on_prog(err, res)
    end
  end

  return err, sub_id, cancel, r
end

function Client:get_meta(id)
  return self:request('get_meta', { id })
end

function Client:list_meta(dir_id)
  return self:request('list_meta', { dir_id })
end

function Client:create_file(name, dir_id)
  return self:request('create_file', { name, dir_id })
end

function Client:create_dir(name, dir_id)
  return self:request('create_dir', { name, dir_id })
end

function Client:delete_file(file_id)
  return self:request('delete_file', { file_id })
end

function Client:delete_dir(dir_id)
  return self:request('delete_dir', { dir_id })
end

function Client:rename(id, new_name)
  return self:request('rename', { id, new_name })
end

function Client:move_file(id, dest_id)
  return self:request('move_file', { id, dest_id })
end

function Client:get_mime(id)
  return self:request('get_mime', { id })
end

function Client:copy(files, dst, prog_interval, on_prog)
  return self:subscribe('copy', { files, dst, prog_interval or uconf.task.cp_interval }, on_prog)
end

function Client:move(files, dst, on_prog)
  return self:subscribe('move', { files, dst }, on_prog)
end

function Client:delete(files, on_prog)
  return self:subscribe('delete', { files }, on_prog)
end

return Client
