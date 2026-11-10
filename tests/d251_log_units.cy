// A log unit has points and spans: a level plus a gain is a level, and gains add.
// trace: D251, D249
@config(default);
import std;

fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

fn.main [] -> void:
    let $input: f64.power.dbm.point = 10.0 power.dbm;
    let $gain: f64.power.dbm.span = 3.0 power.dbm;
    let $output: f64.power.dbm.point = $input + $gain;
    std::io::fmt_print("{$output, unit}{newln}");
    return void;
end fn.main
