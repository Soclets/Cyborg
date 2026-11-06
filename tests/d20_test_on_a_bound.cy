// The test is how to handle a bad value yourself.
// trace: D20, D9
import std;

fn use_score [$s: i32[min: 0, max: 100]] -> void:
    std::println("score ", $s as str);
    return void;
end fn use_score

fn try [$n: i32] -> void:
    if $n: i32[min: 0, max: 100]:
        use_score($n);
    end if
    return void;
end fn try

fn.main [] -> void:
    try(50);
    try(150);
    return void;
end fn.main
