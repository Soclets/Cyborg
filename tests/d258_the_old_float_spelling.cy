// The same after a float's kind.
// trace: D258, D257
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [] -> void:
    let $d: f64.stable.length.meters = 5.0 length.meters;
    return void;
end fn.main
