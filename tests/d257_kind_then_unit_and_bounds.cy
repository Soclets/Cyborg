// The kind goes straight after the float type, before a unit or bounds.
// trace: D257, D158
import std;
import os;

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $d: f64.stable[unit: length.meters] set_to 2.5 length.meters;
    let $t: f64.stable[min: -40.0, max: 125.0] set_to 21.5;
    let $w: f64.ieee[unit: length.meters] set_to 1.5 length.meters;
    std::io::fmt_print(Lend.mut $out, "{$d, unit} {$t} {$w, unit}{newln}");
    return void;
end fn.main
