// A unit parameter goes only where a unit goes.
// trace: D243
import std;

fam.unit length:
    length.meters: base;
    length.feet: base;
end fam.unit length

fam.unit time:
    time.seconds: base;
end fam.unit time

fn keep<?u: any.unit> [$a: ?u] -> f64.stable:
    return 1.0;
end fn keep

fn.main [none] -> void:
    return void;
end fn.main
