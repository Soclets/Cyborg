// The kind goes straight after the float type, before a unit or bounds.
// trace: D257, D158
@config(default);
import std;

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [] -> void:
    let $d: f64.stable[unit: length.meters] = 2.5 length.meters;
    let $t: f64.stable[min: -40.0, max: 125.0] = 21.5;
    let $w: f64.ieee[unit: length.meters] = 1.5 length.meters;
    std::io::fmt_print("{$d, unit} {$t} {$w, unit}{newln}");
    return void;
end fn.main
