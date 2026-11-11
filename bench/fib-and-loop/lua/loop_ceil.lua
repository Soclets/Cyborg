-- Ceil: as fast as LuaJIT can go. The sum of 1 to n is n x (n + 1) / 2.
local n = 100000000
print(string.format("%d", n * (n + 1) / 2))
