// A derived unit can't be made of itself.
// trace: D242, D109
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

fam.unit odd:
    odd.loop = odd.loop per time.seconds;
end fam.unit odd

fn.main [none] -> void:
    return void;
end fn.main
