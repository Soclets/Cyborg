// trace: D20
import std;
@config(default);

fn use_score [$s: i32[min: 0, max: 100]] -> i32:
    return $s + 1;
end fn use_score

fn.main [] -> void:
    let $n: i32 = 100;
    std::io::fmt_print("{}{newln}", use_score($n));
    return void;
end fn.main
