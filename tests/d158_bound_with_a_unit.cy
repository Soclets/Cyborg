// trace: D158, D260

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $t: f64.stable[unit: time.seconds, min: 0.0, max: 1.0] = 0.5 time.seconds;
    return void;
end fn.main
