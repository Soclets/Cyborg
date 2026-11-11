// A type's unit is a `unit:` label in its brackets, a value still writes its unit after the number, and `as` takes the new form.
// trace: D258, D255
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

use std::units::time.seconds as seconds;

fn longer<?u: any.unit> [$a: f64.stable[unit: ?u], $b: f64.stable[unit: ?u]] returns f64.stable[unit: ?u]:
    if $a above $b:
        return $a;
    end if
    return $b;
end fn longer

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $km: f64.stable[unit: length.kilometers] = 2.5 length.kilometers;
    let $m: f64.stable[unit: length.meters] = $km as f64.stable[unit: length.meters];
    let $whole: i32[unit: length.kilometers] = 3 length.kilometers;
    let $wait: i64[unit: seconds.span] = 30 seconds;
    let $xs: std::list<f64.stable[unit: length.meters]>[len.max: 9 items] = [1.0 length.meters, $m];
    let $count: u64[unit: items] = $xs.len;
    std::io::fmt_print(Lend.mut $out, "{$m, unit} {$whole, unit} {$wait, unit} {$count, unit}{newln}");
    std::io::fmt_print(Lend.mut $out, "{}{newln}", longer($m, 4.0 length.meters));
    return void;
end fn.main
