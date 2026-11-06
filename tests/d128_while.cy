// trace: D128
import std;

fn.main [] -> void:
    let mut $n: i32 = 3;
    while $n > 0:
        std::println($n as str);
        change $n = $n - 1;
    end while
    while false:
        std::println("never");
    end while
    return void;
end fn.main
