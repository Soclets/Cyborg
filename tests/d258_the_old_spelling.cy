// The old chained spelling is refused, with the bracket form as its fix.
// trace: D258
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
end fam.unit length

fn.main [] -> void:
    let $d: i32.length.meters = 5 length.meters;
    return void;
end fn.main
