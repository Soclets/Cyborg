// The full list of `as` narrowings isn't decided, so one on anything but a change of unit isn't either.
// trace: D255
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
    length.inches: base;
    length.yards = 36 length.inches;
    length.feet = 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $n: i32 = 7;
    let $s: str = $n as.trunc str;
    return void;
end fn.main
