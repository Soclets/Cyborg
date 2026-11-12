-- Ceil.source: as fast as LuaJIT can go. Bodies in an FFI array of C structs, math.sqrt, everything kept in locals.
local ffi = require("ffi")
ffi.cdef[[ typedef struct { double x, y, z, vx, vy, vz, mass; } body; ]]
local sqrt = math.sqrt
local PI = 3.141592653589793
local SOLAR_MASS = 4 * PI * PI
local DAYS = 365.24
local n = 5
local b = ffi.new("body[5]")
local init = {
  {0,0,0,0,0,0,SOLAR_MASS},
  {4.84143144246472090,-1.16032004402742839,-1.03622044471123109e-01,1.66007664274403694e-03*DAYS,7.69901118419740425e-03*DAYS,-6.90460016972063023e-05*DAYS,9.54791938424326609e-04*SOLAR_MASS},
  {8.34336671824457987,4.12479856412430479,-4.03523417114321381e-01,-2.76742510726862411e-03*DAYS,4.99852801234917238e-03*DAYS,2.30417297573763929e-05*DAYS,2.85885980666130812e-04*SOLAR_MASS},
  {1.28943695621391310e+01,-1.51111514016986312e+01,-2.23307578892655734e-01,2.96460137564761618e-03*DAYS,2.37847173959480950e-03*DAYS,-2.96589568540237556e-05*DAYS,4.36624404335156298e-05*SOLAR_MASS},
  {1.53796971148509165e+01,-2.59193146099879641e+01,1.79258772950371181e-01,2.68067772490389322e-03*DAYS,1.62824170038242295e-03*DAYS,-9.51592254519715870e-05*DAYS,5.15138902046611451e-05*SOLAR_MASS},
}
for i = 0, n - 1 do
  local v = init[i + 1]
  b[i].x, b[i].y, b[i].z, b[i].vx, b[i].vy, b[i].vz, b[i].mass = v[1], v[2], v[3], v[4], v[5], v[6], v[7]
end
local function energy()
  local e = 0
  for i = 0, n - 1 do
    local p = b[i]
    e = e + 0.5 * p.mass * (p.vx*p.vx + p.vy*p.vy + p.vz*p.vz)
    for j = i + 1, n - 1 do
      local q = b[j]
      local dx, dy, dz = p.x - q.x, p.y - q.y, p.z - q.z
      e = e - p.mass * q.mass / sqrt(dx*dx + dy*dy + dz*dz)
    end
  end
  return e
end
local px, py, pz = 0, 0, 0
for i = 0, n - 1 do px, py, pz = px + b[i].vx*b[i].mass, py + b[i].vy*b[i].mass, pz + b[i].vz*b[i].mass end
b[0].vx, b[0].vy, b[0].vz = -px/SOLAR_MASS, -py/SOLAR_MASS, -pz/SOLAR_MASS
io.write(string.format("%.9f\n", energy()))
local dt = 0.01
for _ = 1, 1000000 do
  for i = 0, n - 1 do
    local p = b[i]
    local bx, by, bz, bm, bvx, bvy, bvz = p.x, p.y, p.z, p.mass, p.vx, p.vy, p.vz
    for j = i + 1, n - 1 do
      local q = b[j]
      local dx, dy, dz = bx - q.x, by - q.y, bz - q.z
      local d2 = dx*dx + dy*dy + dz*dz
      local mag = dt / (d2 * sqrt(d2))
      local qm, pm = q.mass * mag, bm * mag
      bvx, bvy, bvz = bvx - dx*qm, bvy - dy*qm, bvz - dz*qm
      q.vx, q.vy, q.vz = q.vx + dx*pm, q.vy + dy*pm, q.vz + dz*pm
    end
    p.vx, p.vy, p.vz = bvx, bvy, bvz
  end
  for i = 0, n - 1 do
    local p = b[i]
    p.x, p.y, p.z = p.x + dt*p.vx, p.y + dt*p.vy, p.z + dt*p.vz
  end
end
io.write(string.format("%.9f\n", energy()))
