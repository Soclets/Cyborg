// A derived unit can't be made of itself.
// trace: D242, D109
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

fam.unit odd:
    odd.loop set_to odd.loop per time.seconds;
end fam.unit odd

fn.main [none] returns void:
    return void;
end fn.main
