// The old chained spelling is refused, with the bracket form as its fix.
// trace: D258
import std;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fn.main [none] returns void:
    let $d: i32.length.meters set_to 5 length.meters;
    return void;
end fn.main
