// A combination with no declared unit can't be stored.
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

fn.main [none] -> void:
    let $dist: f64.stable[unit: length.meters] = 100.0 length.meters;
    let $time: f64.stable[unit: time.seconds] = 8.0 time.seconds;
    let $odd: f64.stable = $dist x $time;
    return void;
end fn.main
