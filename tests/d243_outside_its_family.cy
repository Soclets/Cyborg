// A unit parameter that names a family takes only units in that family.
// trace: D243
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn larger<?u: length> [$a: f64.stable[unit: ?u], $b: f64.stable[unit: ?u]] returns f64.stable[unit: ?u]:
    if $a above $b:
        return $a;
    end if
    return $b;
end fn larger

fn.main [none] returns void:
    let $t: f64.stable[unit: time.seconds] set_to 3.0 time.seconds;
    let $u: f64.stable[unit: time.seconds] set_to larger($t, $t);
    return void;
end fn.main
