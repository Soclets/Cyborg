// How an arm reads a failure's data isn't decided, so this only shows it raised.
// trace: D52, D47
@config(default);
import std;

fn half [$n: i32] -> i32 fails [negative[value: i32]]:
    if $n < 0:
        fail negative[value = $n];
    end if
    return $n;
end fn half

fn.main [] -> void:
    let $n: i32 = half(-3);
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
