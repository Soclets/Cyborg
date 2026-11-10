// A unit parameter keeps whatever unit a call fills it with, from a value or from inside a list.
// trace: D243, D241
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn larger<?u: any.unit> [$a: f64.stable.?u, $b: f64.stable.?u] -> f64.stable.?u:
    if $a > $b:
        return $a;
    end if
    return $b;
end fn larger

fn how_many<?u: length> [$xs: std::list<f64.stable.?u>[len.max: 9 items]] -> u64.items:
    return $xs.len;
end fn how_many

fn.main [] -> void:
    let $a: f64.stable.length.meters = 2.5 length.meters;
    let $b: f64.stable.length.meters = 4.0 length.meters;
    let $t: f64.stable.time.seconds = 3.0 time.seconds;
    let $long: f64.stable.length.meters = larger($a, $b);
    let $later: f64.stable.time.seconds = larger($t, 1.5 time.seconds);
    let $feet: std::list<f64.stable.length.feet>[len.max: 9 items] = [1.0 length.feet, 2.0 length.feet];
    std::io::fmt_print("{} {} {}{newln}", $long, $later, how_many(Lend $feet));
    return void;
end fn.main
