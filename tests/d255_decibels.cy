// A level in dBm converts to milliwatts and back, so two levels are added by way of watts.
// trace: D255, D251
@config(default);
import std;

fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

fn.main [] -> void:
    let $a: f64.stable.power.dbm.point = 10.0 power.dbm;
    let $b: f64.stable.power.dbm.point = 20.0 power.dbm;
    let $a_mw: f64.stable.power.milliwatts.point = $a as f64.stable.power.milliwatts.point;
    let $b_mw: f64.stable.power.milliwatts.point = $b as f64.stable.power.milliwatts.point;
    std::io::fmt_print("{$a_mw, unit} {$b_mw, unit}{newln}");
    return void;
end fn.main
