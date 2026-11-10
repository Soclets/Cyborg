// Seconds divided by seconds is a plain number, since units are tracked as powers.
// trace: D164, D242

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] -> void:
    let $a: f64.stable[unit: time.seconds] = 1.5 time.seconds;
    let $b: f64.stable[unit: time.seconds] = 0.5 time.seconds;
    let $c: f64.stable = $a div $b;
    return void;
end fn.main
