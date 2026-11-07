// trace: D158
fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: f64.fast.time.seconds = 0.5 time.seconds;
    return void;
end fn.main
