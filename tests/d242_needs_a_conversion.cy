// The same bases in other units would need a derived unit's conversion, which isn't designed.
// trace: D242
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

fn.main [none] returns void:
    let $dist: f64.stable[unit: length.kilometers] = 1.0 length.kilometers;
    let $time: f64.stable[unit: time.seconds] = 8.0 time.seconds;
    let $speed: f64.stable[unit: motion.speed] = $dist div $time;
    return void;
end fn.main
