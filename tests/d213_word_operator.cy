// A word operator has one space on each side too.
// trace: D213
import std;

fn.main [none] -> void:
    let $a: bool = true;
    let $b: bool = $a and  $a;
    return void;
end fn.main
