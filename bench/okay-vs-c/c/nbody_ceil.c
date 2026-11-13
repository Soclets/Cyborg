// n-body, Ceil.source: hand-tuned, single-threaded, after the Benchmarks Game's fastest C ideas:
// fields in their own arrays, the ten pairs' differences worked out together so the compiler can
// vectorize them, and each pair's mass times dt over distance cubed computed once.
#include <math.h>
#include <stdio.h>

#define PI 3.141592653589793
#define SOLAR_MASS (4 * PI * PI)
#define DAYS 365.24
#define N 5
#define PAIRS 10

static double x[N] = {0, 4.84143144246472090e+00, 8.34336671824457987e+00, 1.28943695621391310e+01, 1.53796971148509165e+01};
static double y[N] = {0, -1.16032004402742839e+00, 4.12479856412430479e+00, -1.51111514016986312e+01, -2.59193146099879641e+01};
static double z[N] = {0, -1.03622044471123109e-01, -4.03523417114321381e-01, -2.23307578892655734e-01, 1.79258772950371181e-01};
static double vx[N] = {0, 1.66007664274403694e-03 * DAYS, -2.76742510726862411e-03 * DAYS, 2.96460137564761618e-03 * DAYS, 2.68067772490389322e-03 * DAYS};
static double vy[N] = {0, 7.69901118419740425e-03 * DAYS, 4.99852801234917238e-03 * DAYS, 2.37847173959480950e-03 * DAYS, 1.62824170038242295e-03 * DAYS};
static double vz[N] = {0, -6.90460016972063023e-05 * DAYS, 2.30417297573763929e-05 * DAYS, -2.96589568540237556e-05 * DAYS, -9.51592254519715870e-05 * DAYS};
static double m[N] = {SOLAR_MASS, 9.54791938424326609e-04 * SOLAR_MASS, 2.85885980666130812e-04 * SOLAR_MASS, 4.36624404335156298e-05 * SOLAR_MASS, 5.15138902046611451e-05 * SOLAR_MASS};
static int pi_[PAIRS], pj_[PAIRS];

static double energy(void) {
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
    int k = 0;
    for (int i = 0; i < N; i++) for (int j = i + 1; j < N; j++) { pi_[k] = i; pj_[k] = j; k++; }
    double px = 0, py = 0, pz = 0;
    for (int i = 0; i < N; i++) { px += vx[i] * m[i]; py += vy[i] * m[i]; pz += vz[i] * m[i]; }
    vx[0] = -px / SOLAR_MASS; vy[0] = -py / SOLAR_MASS; vz[0] = -pz / SOLAR_MASS;
    printf("%.9f\n", energy());
    const double dt = 0.01;
    double dx[PAIRS], dy[PAIRS], dz[PAIRS], mag[PAIRS];
    for (long step = 0; step < 1000000; step++) {
        for (int p = 0; p < PAIRS; p++) {
            dx[p] = x[pi_[p]] - x[pj_[p]]; dy[p] = y[pi_[p]] - y[pj_[p]]; dz[p] = z[pi_[p]] - z[pj_[p]];
        }
        for (int p = 0; p < PAIRS; p++) {
            double d2 = dx[p] * dx[p] + dy[p] * dy[p] + dz[p] * dz[p];
            mag[p] = dt / (d2 * sqrt(d2));
        }
        for (int p = 0; p < PAIRS; p++) {
            int i = pi_[p], j = pj_[p];
            double mi = m[i] * mag[p], mj = m[j] * mag[p];
            vx[i] -= dx[p] * mj; vy[i] -= dy[p] * mj; vz[i] -= dz[p] * mj;
            vx[j] += dx[p] * mi; vy[j] += dy[p] * mi; vz[j] += dz[p] * mi;
        }
        for (int i = 0; i < N; i++) { x[i] += dt * vx[i]; y[i] += dt * vy[i]; z[i] += dt * vz[i]; }
    }
    printf("%.9f\n", energy());
    return 0;
}
