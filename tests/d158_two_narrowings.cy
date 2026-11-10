// A kind and then a unit is how a float type is written.
// trace: D158, D257
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: f64.fast[unit: time.seconds] = 0.5 time.seconds;
    return void;
end fn.main
