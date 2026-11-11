// Two levels can't be added, since in dB that would multiply two powers.
// trace: D251, D249

fam.unit power:
    power.milliwatts: base;
    power.dbm set_to decibels of 1 power.milliwatts;
end fam.unit power

fn.main [none] returns void:
    let $a: f64.stable[unit: power.dbm.point] set_to 10.0 power.dbm;
    let $b: f64.stable[unit: power.dbm.point] set_to 10.0 power.dbm;
    let $c: f64.stable[unit: power.dbm.point] set_to $a + $b;
    return void;
end fn.main
