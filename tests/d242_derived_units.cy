// A unit made of others is declared once with `per` and `x`, and maths makes any combination.
// trace: D242
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit motion:
    motion.speed = length.meters per time.seconds;
    motion.accel = motion.speed per time.seconds;
    motion.area = length.meters x length.meters;
end fam.unit motion

fn.main [] -> void:
    let $dist: f64.length.meters = 100.0 length.meters;
    let $time: f64.time.seconds = 8.0 time.seconds;
    let $speed: f64.motion.speed = $dist div $time;
    let $accel: f64.motion.accel = $speed div $time;
    let $area: f64.motion.area = $dist x $dist;
    let $back: f64.length.meters = $speed x $time;
    let $ratio: f64 = $time div $time;
    std::io::fmt_print("{} {} {} {} {}{newln}", $speed, $accel, $area, $back, $ratio);
    return void;
end fn.main
