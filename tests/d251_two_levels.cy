// Two levels can't be added, since in dB that would multiply two powers.
// trace: D251, D249

fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

fn.main [none] -> void:
    let $a: f64.stable[unit: power.dbm.point] = 10.0 power.dbm;
    let $b: f64.stable[unit: power.dbm.point] = 10.0 power.dbm;
    let $c: f64.stable[unit: power.dbm.point] = $a + $b;
    return void;
end fn.main
