import std;
import os;
use std::io::fmt_print as fmt_print;

// Ceil.source: n-body as fast as today's Cyborg can go. Cyborg has no square root
// yet, so each pair's Newton's method starts from that pair's root in the step
// before, since planets barely move in 0.01 days, and converges in a few steps.

struct body:
    x: f64.stable;
    y: f64.stable;
    z: f64.stable;
    vx: f64.stable;
    vy: f64.stable;
    vz: f64.stable;
    mass: f64.stable;
end struct body

fn root_from [$v: f64.stable, $start: f64.stable] -> f64.stable:
    let mut $g: f64.stable = ($start + $v div $start) div 2.0;
    let mut $next: f64.stable = ($g + $v div $g) div 2.0;
    while $next below $g:
        change $g = $next;
        change $next = ($g + $v div $g) div 2.0;
    end while
    return $g;
end fn root_from

fn root [$v: f64.stable] -> f64.stable:
    let mut $g: f64.stable = 1.0;
    if $v above 1.0:
        change $g = $v;
    end if
    return root_from($v, $g);
end fn root

fn energy [$bs: std::array<body>[len: 5 items]] -> f64.stable:
    let mut $e: f64.stable = 0.0;
    let mut $i: i64 = 1;
    while $i at_most 5:
        let $b: body = Copy $bs[pos: $i];
        change $e = $e + 0.5 x $b.mass x ($b.vx x $b.vx + $b.vy x $b.vy + $b.vz x $b.vz);
        let mut $j: i64 = $i + 1;
        while $j at_most 5:
            let $c: body = Copy $bs[pos: $j];
            let $dx: f64.stable = $b.x - $c.x;
            let $dy: f64.stable = $b.y - $c.y;
            let $dz: f64.stable = $b.z - $c.z;
            change $e = $e - $b.mass x $c.mass div root($dx x $dx + $dy x $dy + $dz x $dz);
            change $j = $j + 1;
        end while
        change $i = $i + 1;
    end while
    return $e;
end fn energy

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    let $pi: f64.stable = ~3.141592653589793;
    let $solar: f64.stable = 4.0 x $pi x $pi;
    let $days: f64.stable = ~365.24;
    let mut $bs: std::array<body>[len: 5 items] = [
        body[x = 0.0, y = 0.0, z = 0.0, vx = 0.0, vy = 0.0, vz = 0.0, mass = $solar],
        body[x = ~4.84143144246472090, y = -~1.16032004402742839, z = -~0.103622044471123109,
            vx = ~0.00166007664274403694 x $days, vy = ~0.00769901118419740425 x $days,
            vz = -~0.0000690460016972063023 x $days, mass = ~0.000954791938424326609 x $solar],
        body[x = ~8.34336671824457987, y = ~4.12479856412430479, z = -~0.403523417114321381,
            vx = -~0.00276742510726862411 x $days, vy = ~0.00499852801234917238 x $days,
            vz = ~0.0000230417297573763929 x $days, mass = ~0.000285885980666130812 x $solar],
        body[x = ~12.8943695621391310, y = -~15.1111514016986312, z = -~0.223307578892655734,
            vx = ~0.00296460137564761618 x $days, vy = ~0.00237847173959480950 x $days,
            vz = -~0.0000296589568540237556 x $days, mass = ~0.0000436624404335156298 x $solar],
        body[x = ~15.3796971148509165, y = -~25.9193146099879641, z = ~0.179258772950371181,
            vx = ~0.00268067772490389322 x $days, vy = ~0.00162824170038242295 x $days,
            vz = -~0.0000951592254519715870 x $days, mass = ~0.0000515138902046611451 x $solar]
    ];
    let mut $px: f64.stable = 0.0;
    let mut $py: f64.stable = 0.0;
    let mut $pz: f64.stable = 0.0;
    let mut $k: i64 = 1;
    while $k at_most 5:
        change $px = $px + $bs[pos: $k].vx x $bs[pos: $k].mass;
        change $py = $py + $bs[pos: $k].vy x $bs[pos: $k].mass;
        change $pz = $pz + $bs[pos: $k].vz x $bs[pos: $k].mass;
        change $k = $k + 1;
    end while
    change $bs[pos: 1].vx = 0.0 - $px div $solar;
    change $bs[pos: 1].vy = 0.0 - $py div $solar;
    change $bs[pos: 1].vz = 0.0 - $pz div $solar;
    let $before: f64.stable = energy(Lend $bs);
    fmt_print(Lend.mut $out, "{$before, places: 9}{newln}");
    let $dt: f64.stable = ~0.01;
    let mut $last: std::array<f64.stable>[len: 25 items] = all 1.0;
    let mut $step: i64 = 1;
    while $step at_most 1_000_000:
        let mut $i: i64 = 1;
        while $i at_most 5:
            let mut $j: i64 = $i + 1;
            while $j at_most 5:
                let $dx: f64.stable = $bs[pos: $i].x - $bs[pos: $j].x;
                let $dy: f64.stable = $bs[pos: $i].y - $bs[pos: $j].y;
                let $dz: f64.stable = $bs[pos: $i].z - $bs[pos: $j].z;
                let $d2: f64.stable = $dx x $dx + $dy x $dy + $dz x $dz;
                let $slot: i64 = ($i - 1) x 5 + $j;
                let $r: f64.stable = root_from($d2, $last[pos: $slot]);
                change $last[pos: $slot] = $r;
                let $mag: f64.stable = $dt div ($d2 x $r);
                let $mi: f64.stable = $bs[pos: $i].mass x $mag;
                let $mj: f64.stable = $bs[pos: $j].mass x $mag;
                change $bs[pos: $i].vx = $bs[pos: $i].vx - $dx x $mj;
                change $bs[pos: $i].vy = $bs[pos: $i].vy - $dy x $mj;
                change $bs[pos: $i].vz = $bs[pos: $i].vz - $dz x $mj;
                change $bs[pos: $j].vx = $bs[pos: $j].vx + $dx x $mi;
                change $bs[pos: $j].vy = $bs[pos: $j].vy + $dy x $mi;
                change $bs[pos: $j].vz = $bs[pos: $j].vz + $dz x $mi;
                change $j = $j + 1;
            end while
            change $i = $i + 1;
        end while
        let mut $m: i64 = 1;
        while $m at_most 5:
            change $bs[pos: $m].x = $bs[pos: $m].x + $dt x $bs[pos: $m].vx;
            change $bs[pos: $m].y = $bs[pos: $m].y + $dt x $bs[pos: $m].vy;
            change $bs[pos: $m].z = $bs[pos: $m].z + $dt x $bs[pos: $m].vz;
            change $m = $m + 1;
        end while
        change $step = $step + 1;
    end while
    let $after: f64.stable = energy(Lend $bs);
    fmt_print(Lend.mut $out, "{$after, places: 9}{newln}");
    os::done(Give $out);
    return void;
end fn.main
