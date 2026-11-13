import std;
import os;
use std::io::fmt_print as fmt_print;

// n-body, from the Computer Language Benchmarks Game: the sun and four planets,
// 1,000,000 steps of 0.01 days. Ceil.source: each field in its own array, so the
// compiler keeps them apart and can work on several bodies at once, and each pair's
// distance cubed is worked out once.

fn energy [$x: Lent std::coll::array<f64.stable>[len: 5 items], $y: Lent std::coll::array<f64.stable>[len: 5 items], $z: Lent std::coll::array<f64.stable>[len: 5 items], $vx: Lent std::coll::array<f64.stable>[len: 5 items], $vy: Lent std::coll::array<f64.stable>[len: 5 items], $vz: Lent std::coll::array<f64.stable>[len: 5 items], $m: Lent std::coll::array<f64.stable>[len: 5 items]] -> f64.stable:
    let mut $e: f64.stable = 0.0;
    for $i in range[from: 1, to: 5]:
        change $e = $e + 0.5 x $m[pos: $i] x ($vx[pos: $i] x $vx[pos: $i] + $vy[pos: $i] x $vy[pos: $i] + $vz[pos: $i] x $vz[pos: $i]);
        for $j in range[from: $i + 1, to: 5]:
            let $dx: f64.stable = $x[pos: $i] - $x[pos: $j];
            let $dy: f64.stable = $y[pos: $i] - $y[pos: $j];
            let $dz: f64.stable = $z[pos: $i] - $z[pos: $j];
            change $e = $e - $m[pos: $i] x $m[pos: $j] div std::math::sqrt($dx x $dx + $dy x $dy + $dz x $dz);
        end for
    end for
    return $e;
end fn energy

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(room: 1024 bytes);
    let $pi: f64.stable = ~3.141592653589793;
    let $solar: f64.stable = 4.0 x $pi x $pi;
    let $days: f64.stable = ~365.24;
    let mut $x: std::coll::array<f64.stable>[len: 5 items] = [0.0, ~4.84143144246472090, ~8.34336671824457987, ~12.8943695621391310, ~15.3796971148509165];
    let mut $y: std::coll::array<f64.stable>[len: 5 items] = [0.0, -~1.16032004402742839, ~4.12479856412430479, -~15.1111514016986312, -~25.9193146099879641];
    let mut $z: std::coll::array<f64.stable>[len: 5 items] = [0.0, -~0.103622044471123109, -~0.403523417114321381, -~0.223307578892655734, ~0.179258772950371181];
    let mut $vx: std::coll::array<f64.stable>[len: 5 items] = [0.0, ~0.00166007664274403694 x $days, -~0.00276742510726862411 x $days, ~0.00296460137564761618 x $days, ~0.00268067772490389322 x $days];
    let mut $vy: std::coll::array<f64.stable>[len: 5 items] = [0.0, ~0.00769901118419740425 x $days, ~0.00499852801234917238 x $days, ~0.00237847173959480950 x $days, ~0.00162824170038242295 x $days];
    let mut $vz: std::coll::array<f64.stable>[len: 5 items] = [0.0, -~0.0000690460016972063023 x $days, ~0.0000230417297573763929 x $days, -~0.0000296589568540237556 x $days, -~0.0000951592254519715870 x $days];
    let $m: std::coll::array<f64.stable>[len: 5 items] = [$solar, ~0.000954791938424326609 x $solar, ~0.000285885980666130812 x $solar, ~0.0000436624404335156298 x $solar, ~0.0000515138902046611451 x $solar];
    let mut $px: f64.stable = 0.0;
    let mut $py: f64.stable = 0.0;
    let mut $pz: f64.stable = 0.0;
    for $k in range[from: 1, to: 5]:
        change $px = $px + $vx[pos: $k] x $m[pos: $k];
        change $py = $py + $vy[pos: $k] x $m[pos: $k];
        change $pz = $pz + $vz[pos: $k] x $m[pos: $k];
    end for
    change $vx[pos: 1] = 0.0 - $px div $solar;
    change $vy[pos: 1] = 0.0 - $py div $solar;
    change $vz[pos: 1] = 0.0 - $pz div $solar;
    let $before: f64.stable = energy(Lend $x, Lend $y, Lend $z, Lend $vx, Lend $vy, Lend $vz, Lend $m);
    fmt_print(Lend.mut $out, "{$before, places: 9}{newln}");
    let $dt: f64.stable = ~0.01;
    for $step in range[from: 1, to: 1_000_000]:
        for $i in range[from: 1, to: 4]:
            for $j in range[from: $i + 1, to: 5]:
                let $dx: f64.stable = $x[pos: $i] - $x[pos: $j];
                let $dy: f64.stable = $y[pos: $i] - $y[pos: $j];
                let $dz: f64.stable = $z[pos: $i] - $z[pos: $j];
                let $d2: f64.stable = $dx x $dx + $dy x $dy + $dz x $dz;
                let $mag: f64.stable = $dt div ($d2 x std::math::sqrt($d2));
                let $mi: f64.stable = $m[pos: $i] x $mag;
                let $mj: f64.stable = $m[pos: $j] x $mag;
                change $vx[pos: $i] = $vx[pos: $i] - $dx x $mj;
                change $vy[pos: $i] = $vy[pos: $i] - $dy x $mj;
                change $vz[pos: $i] = $vz[pos: $i] - $dz x $mj;
                change $vx[pos: $j] = $vx[pos: $j] + $dx x $mi;
                change $vy[pos: $j] = $vy[pos: $j] + $dy x $mi;
                change $vz[pos: $j] = $vz[pos: $j] + $dz x $mi;
            end for
        end for
        for $k in range[from: 1, to: 5]:
            change $x[pos: $k] = $x[pos: $k] + $dt x $vx[pos: $k];
            change $y[pos: $k] = $y[pos: $k] + $dt x $vy[pos: $k];
            change $z[pos: $k] = $z[pos: $k] + $dt x $vz[pos: $k];
        end for
    end for
    let $after: f64.stable = energy(Lend $x, Lend $y, Lend $z, Lend $vx, Lend $vy, Lend $vz, Lend $m);
    fmt_print(Lend.mut $out, "{$after, places: 9}{newln}");
    os::done(Give $out);
    return void;
end fn.main
