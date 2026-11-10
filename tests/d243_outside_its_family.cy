// A unit parameter that names a family takes only units in that family.
// trace: D243
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn larger<?u: length> [$a: f64.?u, $b: f64.?u] -> f64.?u:
    if $a > $b:
        return $a;
    end if
    return $b;
end fn larger

fn.main [] -> void:
    let $t: f64.time.seconds = 3.0 time.seconds;
    let $u: f64.time.seconds = larger($t, $t);
    return void;
end fn.main
