// trace: D102, D20
import std;
@config(default);

fn.main [] -> void:
    let $d: std::array<u8>[len: 3 items] = all 256;
    return void;
end fn.main
