// A combination with no declared unit can't be stored.
// trace: D242
import std;

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
    let $dist: f64.stable[unit: length.meters] set_to 100.0 length.meters;
    let $time: f64.stable[unit: time.seconds] set_to 8.0 time.seconds;
    let $odd: f64.stable set_to $dist x $time;
    return void;
end fn.main
