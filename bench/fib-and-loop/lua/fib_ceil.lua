-- Ceil: as fast as LuaJIT can go. fib(35) by fast doubling.
local function pair(n) -- fib(n), fib(n + 1)
  if n == 0 then return 0, 1 end
  local a, b = pair(math.floor(n / 2))
  local c, d = a * (2 * b - a), a * a + b * b
  if n % 2 == 1 then return d, c + d end
  return c, d
end
print(string.format("%d", (pair(35))))
