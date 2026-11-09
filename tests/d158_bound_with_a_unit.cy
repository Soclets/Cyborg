// trace: D158
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $t: f64.time.seconds[min: 0.0, max: 1.0] = 0.5 time.seconds;
    return void;
end fn.main
