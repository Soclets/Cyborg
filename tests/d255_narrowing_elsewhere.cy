// The full list of `as` narrowings isn't decided, so one on anything but a change of unit isn't either.
// trace: D255
import std;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let $n: i32 set_to 7;
    let $s: str set_to $n as.trunc str;
    return void;
end fn.main
