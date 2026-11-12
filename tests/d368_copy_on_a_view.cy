// `Copy` on a view hides no mistake, so it's refused with a fix that drops it.
// trace: D368, D182
import std;

fn.main [none] -> void:
    let $a: str = "first";
    let $b: str = Copy $a;
    return void;
end fn.main
