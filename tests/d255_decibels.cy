// A level in dBm converts to milliwatts and back, so two levels are added by way of watts.
// trace: D255, D251
import std;
import os;

fam.unit power:
    power.milliwatts: base;
    power.dbm set_to decibels of 1 power.milliwatts;
end fam.unit power

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: f64.stable[unit: power.dbm.point] set_to 10.0 power.dbm;
    let $b: f64.stable[unit: power.dbm.point] set_to 20.0 power.dbm;
    let $a_mw: f64.stable[unit: power.milliwatts.point] set_to $a as f64.stable[unit: power.milliwatts.point];
    let $b_mw: f64.stable[unit: power.milliwatts.point] set_to $b as f64.stable[unit: power.milliwatts.point];
    std::io::fmt_print(Lend.mut $out, "{$a_mw, unit} {$b_mw, unit}{newln}");
    return void;
end fn.main
