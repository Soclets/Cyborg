-- Purpose: the cost of a function call, with n from the command line.
local function fib(n)
  if n < 2 then return n end
  return fib(n - 1) + fib(n - 2)
end
print(string.format("%d", fib(tonumber(arg[1] or "35"))))
