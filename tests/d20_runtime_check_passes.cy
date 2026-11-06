// trace: D20
import std;

fn use_score [$s: i32[min: 0, max: 100]] -> i32:
    return $s + 1;
end fn use_score

fn.main [] -> void:
    let $n: i32 = 100;
    std::println(use_score($n) as str);
    return void;
end fn.main
