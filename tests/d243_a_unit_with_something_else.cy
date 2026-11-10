// A unit parameter needs `any.unit` or one family, and nothing else.
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

interface ordered<?self>:
    op < [$a: ?self, $b: ?self] -> bool;
end interface ordered

fn keep<?u: length and ordered> [$a: f64.?u] -> f64.?u:
    return $a;
end fn keep

fn.main [] -> void:
    return void;
end fn.main
