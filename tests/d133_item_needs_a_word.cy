// An item that isn't a plain number shows its hand-off like anything else.
// trace: D133, D3
import std;

fn.main [none] -> void:
    let $a: std::coll::array<str[len.max: 32 bytes]>[len: 1 items] = ["hi"];
    let $s: str[len.max: 32 bytes] = $a[pos: 1];
    return void;
end fn.main
