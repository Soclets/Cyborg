// A bound converted to the type's unit is checked there, and the type shows it in that unit.
// trace: D260
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [] -> void:
    let $walked: i32[unit: length.meters] = 1500 length.meters;
    let $short: i32[unit: length.meters, min: 0 length.meters, max: 1 length.kilometers] = $walked;
    std::io::fmt_print("{$short, unit}{newln}");
    return void;
end fn.main
