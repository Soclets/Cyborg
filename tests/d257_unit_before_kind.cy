// A unit comes after the kind, never before it.
// trace: D257
import std;

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [none] returns void:
    let $d: f64.length.meters.stable set_to 2.5 length.meters;
    return void;
end fn.main
