-- Normal: adding 1 to 100,000,000 in a loop, written plainly.
local sum = 0
for i = 1, 100000000 do sum = sum + i end
print(string.format("%d", sum))
