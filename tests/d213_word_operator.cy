// A word operator has one space on each side too.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $a: bool = true;
    let $b: bool = $a and  $a;
    return void;
end fn.main
