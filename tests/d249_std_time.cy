// `std`'s time has points and spans too: a moment is a point, a length of time a span.
// trace: D249, D199
import std;

fam.unit temperature:
    temperature.kelvin: base;
    temperature.celsius = temperature.kelvin offset ~-273.15;
end fam.unit temperature

use my::temperature.celsius as celsius;

use std::units::time.seconds as seconds;

fn.main [none] -> void:
    let $start: i64[unit: seconds.point] = 1000 seconds;
    let $wait: i64[unit: seconds.span] = 30 seconds;
    let $end: i64[unit: seconds.point] = $start + $wait;
    std::io::fmt_print("{$end, unit}{newln}");
    return void;
end fn.main
