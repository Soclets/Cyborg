// Seconds divided by seconds is a plain number, since units are tracked as powers.
// trace: D164, D242
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: f64.time.seconds = 1.5 time.seconds;
    let $b: f64.time.seconds = 0.5 time.seconds;
    let $c: f64 = $a div $b;
    return void;
end fn.main
