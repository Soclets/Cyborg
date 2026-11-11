// A prefix family is declared once, and a unit opts in with `with`, which makes its prefixed units.
// trace: D250
import std;
import os;

fam.prefix metric:
    metric.kilo = 1000;
    metric.milli = 1 per 1000;
end fam.prefix metric

fam.unit length:
    length.meters: base with metric;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $road: f64.stable[unit: length.kilometers] = 2.5 length.kilometers;
    let $hair: f64.stable[unit: length.millimeters] = 0.5 length.millimeters;
    std::io::fmt_print(Lend.mut $out, "{$road, unit} {$hair, unit}{newln}");
    return void;
end fn.main
