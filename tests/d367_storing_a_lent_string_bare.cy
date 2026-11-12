// Storing a view of another string writes its source in the type.
// trace: D367, D2
import std;

fn.main [none] -> void:
    let $owned: str[len.max: 16 bytes] = "attic";
    let $v: str = Lend $owned;
    return void;
end fn.main
