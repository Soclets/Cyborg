// Seconds divided by seconds is a plain number, since units are tracked as powers.
// trace: D164, D242

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let $a: f64.stable[unit: time.seconds] set_to 1.5 time.seconds;
    let $b: f64.stable[unit: time.seconds] set_to 0.5 time.seconds;
    let $c: f64.stable set_to $a div $b;
    return void;
end fn.main
