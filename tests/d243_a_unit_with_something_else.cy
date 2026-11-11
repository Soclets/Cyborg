// A unit parameter needs `any.unit` or one family, and nothing else.
// trace: D243
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] returns bool;
end interface ordered

fn keep<?u: length and ordered> [$a: f64.stable[unit: ?u]] returns f64.stable[unit: ?u]:
    return $a;
end fn keep

fn.main [none] returns void:
    return void;
end fn.main
