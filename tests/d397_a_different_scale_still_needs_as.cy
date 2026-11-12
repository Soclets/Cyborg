// What would need an `as` to store needs the same `as` to compare.
// trace: D397, D100
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [none] -> void:
    let $a: f64.stable[unit: length.meters] = 100.0 length.meters;
    let $b: f64.stable[unit: length.kilometers] = 1.0 length.kilometers;
    if $a below $b:
    end if
    return void;
end fn.main
