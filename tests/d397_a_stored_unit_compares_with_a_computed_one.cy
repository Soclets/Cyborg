// Two values compare when one could be stored into the other's type, so a computed length per
// time compares with a declared speed, while a different scale still needs an `as`.
// trace: D397, D242
import std;
import os;

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fam.unit motion:
    motion.speed = length.meters per time.seconds;
end fam.unit motion

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $dist: f64.stable[unit: length.meters] = 100.0 length.meters;
    let $time: f64.stable[unit: time.seconds] = 8.0 time.seconds;
    let $speed: f64.stable[unit: motion.speed] = 12.5 motion.speed;
    if $speed == $dist div $time:
        std::io::fmt_print(Lend.mut $out, "same{newln}");
    end if
    if $dist div $time below $speed:
        std::io::fmt_print(Lend.mut $out, "slower{newln}");
    end if
    return void;
end fn.main
