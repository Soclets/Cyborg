// A unit comes after the kind, never before it.
// trace: D257
@config(default);
import std;

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [] -> void:
    let $d: f64.length.meters.stable = 2.5 length.meters;
    return void;
end fn.main
