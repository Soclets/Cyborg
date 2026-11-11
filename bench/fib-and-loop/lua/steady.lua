-- Steady state, not a whole program: each benchmark run many times inside one LuaJIT, after
-- warm-up, timed with os.clock. Prints the answers, then the mean, standard deviation and min.
local function fib(n)
  if n < 2 then return n end
  return fib(n - 1) + fib(n - 2)
end
local function loop(n)
  local sum = 0
  for i = 1, n do sum = sum + i end
  return sum
end
local function measure(name, work, input, warm, runs)
  local answer
  for _ = 1, warm do answer = work(input) end
  local times = {}
  for k = 1, runs do
    local start = os.clock()
    answer = work(input)
    times[k] = os.clock() - start
  end
  local total, least = 0, math.huge
  for _, t in ipairs(times) do total = total + t; least = math.min(least, t) end
  local mean = total / runs
  local spread = 0
  for _, t in ipairs(times) do spread = spread + (t - mean) ^ 2 end
  print(string.format("%s %d mean %.4f sd %.4f min %.4f", name, answer, mean,
    math.sqrt(spread / (runs - 1)), least))
end
measure("fib", fib, tonumber(arg[1] or "35"), 5, 20)
measure("loop", loop, tonumber(arg[2] or "100000000"), 5, 20)
