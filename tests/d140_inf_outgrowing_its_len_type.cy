// `inf` always fits as a limit, but what happens when the count can't be stored isn't stated.
// trace: D140, D96
import std;

fn.main [none] -> void:
    let mut $xs: std::list<i32>[len.max: inf items, len.type: u8] = empty;
    for $i in range[from: 1, to: 255]:
        $xs.std::%%append(val: $i);
    end for
    std::io::fmt_print("{}{newln}", $xs.len);
    $xs.std::%%append(val: 256);
    std::io::fmt_print("after{newln}");
    return void;
end fn.main
