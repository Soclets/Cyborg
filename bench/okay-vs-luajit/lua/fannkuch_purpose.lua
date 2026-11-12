-- Purpose: the cost of array shuffling and branches. n comes from the command line.
local n = tonumber(arg[1])
local perm1, perm, count = {}, {}, {}
for i = 0, n - 1 do perm1[i] = i; perm[i] = 0; count[i] = 0 end
local max_flips, checksum, sign, r = 0, 0, 1, n
while true do
  while r ~= 1 do count[r - 1] = r; r = r - 1 end
  for i = 0, n - 1 do perm[i] = perm1[i] end
  local flips = 0
  local k = perm[0]
  while k ~= 0 do
    local lo, hi = 0, k
    while lo < hi do perm[lo], perm[hi] = perm[hi], perm[lo]; lo = lo + 1; hi = hi - 1 end
    flips = flips + 1
    k = perm[0]
  end
  if flips > max_flips then max_flips = flips end
  checksum = checksum + sign * flips
  sign = -sign
  while true do
    if r == n then
      io.write(checksum, "\nPfannkuchen(", n, ") = ", max_flips, "\n")
      return
    end
    local first = perm1[0]
    for j = 0, r - 1 do perm1[j] = perm1[j + 1] end
    perm1[r] = first
    count[r] = count[r] - 1
    if count[r] > 0 then break end
    r = r + 1
  end
end
