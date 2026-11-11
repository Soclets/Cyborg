// A unit made of others is declared once with `per` and `x`, and maths makes any combination.
// trace: D242
import std;
import os;

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

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $dist: f64.stable[unit: length.meters] = 100.0 length.meters;
    let $time: f64.stable[unit: time.seconds] = 8.0 time.seconds;
    let $speed: f64.stable[unit: motion.speed] = $dist div $time;
    let $accel: f64.stable[unit: motion.accel] = $speed div $time;
    let $area: f64.stable[unit: motion.area] = $dist x $dist;
    let $back: f64.stable[unit: length.meters] = $speed x $time;
    let $ratio: f64.stable = $time div $time;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {} {}{newln}", $speed, $accel, $area, $back, $ratio);
    return void;
end fn.main
