// n-body, Ceil.source: hand-tuned and single-threaded. Each field is in its own local array, so
// the compiler can keep every body in registers, every loop has a fixed count so all ten pairs
// unroll, and each pair's dt over distance cubed is worked out once.
#include <math.h>
#include <stdio.h>

#define PI 3.141592653589793
#define SOLAR_MASS (4 * PI * PI)
#define DAYS 365.24
#define N 5

static const double init_x[N] = {0, 4.84143144246472090e+00, 8.34336671824457987e+00, 1.28943695621391310e+01, 1.53796971148509165e+01};
static const double init_y[N] = {0, -1.16032004402742839e+00, 4.12479856412430479e+00, -1.51111514016986312e+01, -2.59193146099879641e+01};
static const double init_z[N] = {0, -1.03622044471123109e-01, -4.03523417114321381e-01, -2.23307578892655734e-01, 1.79258772950371181e-01};
static const double vinit_x[N] = {0, 1.66007664274403694e-03 * DAYS, -2.76742510726862411e-03 * DAYS, 2.96460137564761618e-03 * DAYS, 2.68067772490389322e-03 * DAYS};
static const double vinit_y[N] = {0, 7.69901118419740425e-03 * DAYS, 4.99852801234917238e-03 * DAYS, 2.37847173959480950e-03 * DAYS, 1.62824170038242295e-03 * DAYS};
static const double vinit_z[N] = {0, -6.90460016972063023e-05 * DAYS, 2.30417297573763929e-05 * DAYS, -2.96589568540237556e-05 * DAYS, -9.51592254519715870e-05 * DAYS};
static const double m[N] = {SOLAR_MASS, 9.54791938424326609e-04 * SOLAR_MASS, 2.85885980666130812e-04 * SOLAR_MASS, 4.36624404335156298e-05 * SOLAR_MASS, 5.15138902046611451e-05 * SOLAR_MASS};

static double energy(const double *x, const double *y, const double *z, const double *vx, const double *vy, const double *vz) {
    double e = 0;
    for (int i = 0; i < N; i++) {
        e += 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        for (int j = i + 1; j < N; j++) {
            double dx = x[i] - x[j], dy = y[i] - y[j], dz = z[i] - z[j];
            e -= m[i] * m[j] / sqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    return e;
}

int main(void) {
    double x[N], y[N], z[N], vx[N], vy[N], vz[N];
    for (int i = 0; i < N; i++) { x[i] = init_x[i]; y[i] = init_y[i]; z[i] = init_z[i]; vx[i] = vinit_x[i]; vy[i] = vinit_y[i]; vz[i] = vinit_z[i]; }
    double px = 0, py = 0, pz = 0;
    for (int i = 0; i < N; i++) { px += vx[i] * m[i]; py += vy[i] * m[i]; pz += vz[i] * m[i]; }
    vx[0] = -px / SOLAR_MASS; vy[0] = -py / SOLAR_MASS; vz[0] = -pz / SOLAR_MASS;
    printf("%.9f\n", energy(x, y, z, vx, vy, vz));
    const double dt = 0.01;
    for (long step = 0; step < 1000000; step++) {
        for (int i = 0; i < N; i++) {
            for (int j = i + 1; j < N; j++) {
                double dx = x[i] - x[j], dy = y[i] - y[j], dz = z[i] - z[j];
                double d2 = dx * dx + dy * dy + dz * dz;
                double mag = dt / (d2 * sqrt(d2));
                double mi = m[i] * mag, mj = m[j] * mag;
                vx[i] -= dx * mj; vy[i] -= dy * mj; vz[i] -= dz * mj;
                vx[j] += dx * mi; vy[j] += dy * mi; vz[j] += dz * mi;
            }
        }
        for (int i = 0; i < N; i++) { x[i] += dt * vx[i]; y[i] += dt * vy[i]; z[i] += dt * vz[i]; }
    }
    printf("%.9f\n", energy(x, y, z, vx, vy, vz));
    return 0;
}
