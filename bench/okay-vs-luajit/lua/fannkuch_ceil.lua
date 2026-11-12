-- Ceil.source: as fast as LuaJIT can go. FFI int arrays, the first item kept in a local, and the flip loop unrolled into a swap loop over raw C memory.
local ffi = require("ffi")
local n = 11
local perm1 = ffi.new("int32_t[?]", n + 1)
local perm = ffi.new("int32_t[?]", n + 1)
local count = ffi.new("int32_t[?]", n + 1)
for i = 0, n - 1 do perm1[i] = i end
local max_flips, checksum, sign, r = 0, 0, 1, n
while true do
  while r ~= 1 do count[r - 1] = r; r = r - 1 end
  local k = perm1[0]
  if k ~= 0 then
    ffi.copy(perm, perm1, n * 4)
    local flips = 1
    while true do
      local lo, hi = 1, k - 1
      while lo < hi do perm[lo], perm[hi] = perm[hi], perm[lo]; lo = lo + 1; hi = hi - 1 end
      local nk = perm[k]
      perm[k] = k
      k = nk
      if k == 0 then break end
      flips = flips + 1
    end
    if flips > max_flips then max_flips = flips end
    checksum = checksum + sign * flips
  end
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
