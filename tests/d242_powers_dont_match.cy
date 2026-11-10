// Storing needs a declared unit whose powers match.
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
    let $speed: f64.motion.speed = $time div $dist;
    return void;
end fn.main
