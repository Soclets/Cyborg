// Two levels can't be added, since in dB that would multiply two powers.
// trace: D251, D249
@config(default);

fam.unit power:
    power.milliwatts: base;
    power.dbm = decibels of 1 power.milliwatts;
end fam.unit power

fn.main [] -> void:
    let $a: f64.stable.power.dbm.point = 10.0 power.dbm;
    let $b: f64.stable.power.dbm.point = 10.0 power.dbm;
    let $c: f64.stable.power.dbm.point = $a + $b;
    return void;
end fn.main
