// A log unit has points and spans: a level plus a gain is a level, and gains add.
// trace: D251, D249
import std;
import os;

fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $input: f64.stable[unit: power.dbm.point] = 10.0 power.dbm;
    let $gain: f64.stable[unit: power.dbm.span] = 3.0 power.dbm;
    let $output: f64.stable[unit: power.dbm.point] = $input + $gain;
    std::io::fmt_print(Lend.mut $out, "{$output, unit}{newln}");
    return void;
end fn.main
