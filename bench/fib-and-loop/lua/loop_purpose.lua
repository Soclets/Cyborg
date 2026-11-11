-- Purpose: a loop with an add, with the count from the command line.
local n = tonumber(arg[1] or "100000000")
local sum = 0
for i = 1, n do sum = sum + i end
print(string.format("%d", sum))
