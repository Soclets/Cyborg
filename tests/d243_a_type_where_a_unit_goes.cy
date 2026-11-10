// A type parameter goes only where a whole type goes.
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

fn keep<?item: any.type> [$a: f64.stable[unit: ?item]] -> f64.stable:
    return 1.0;
end fn keep

fn.main [] -> void:
    return void;
end fn.main
