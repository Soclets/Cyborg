// A unit parameter keeps whatever unit a call fills it with, from a value or from inside a list.
// trace: D243, D241
import std;
import os;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn larger<?u: any.unit> [$a: f64.stable[unit: ?u], $b: f64.stable[unit: ?u]] -> f64.stable[unit: ?u]:
    if $a above $b:
        return $a;
    end if
    return $b;
end fn larger

fn how_many<?u: length> [$xs: Lent std::coll::list<f64.stable[unit: ?u]>[len.max: 9 items]] -> u64[unit: items]:
    return $xs.len;
end fn how_many

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable[unit: length.meters] = 2.5 length.meters;
    let $b: f64.stable[unit: length.meters] = 4.0 length.meters;
    let $t: f64.stable[unit: time.seconds] = 3.0 time.seconds;
    let $long: f64.stable[unit: length.meters] = larger($a, $b);
    let $later: f64.stable[unit: time.seconds] = larger($t, 1.5 time.seconds);
    let $feet: std::coll::list<f64.stable[unit: length.feet]>[len.max: 9 items] = [1.0 length.feet, 2.0 length.feet];
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $long, $later, how_many(Lend $feet));
    return void;
end fn.main
