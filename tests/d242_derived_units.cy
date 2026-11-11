// A unit made of others is declared once with `per` and `x`, and maths makes any combination.
// trace: D242
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit motion:
    motion.speed set_to length.meters per time.seconds;
    motion.accel set_to motion.speed per time.seconds;
    motion.area set_to length.meters x length.meters;
end fam.unit motion

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $dist: f64.stable[unit: length.meters] set_to 100.0 length.meters;
    let $time: f64.stable[unit: time.seconds] set_to 8.0 time.seconds;
    let $speed: f64.stable[unit: motion.speed] set_to $dist div $time;
    let $accel: f64.stable[unit: motion.accel] set_to $speed div $time;
    let $area: f64.stable[unit: motion.area] set_to $dist x $dist;
    let $back: f64.stable[unit: length.meters] set_to $speed x $time;
    let $ratio: f64.stable set_to $time div $time;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {} {}{newln}", $speed, $accel, $area, $back, $ratio);
    return void;
end fn.main
