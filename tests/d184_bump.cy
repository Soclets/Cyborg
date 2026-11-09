// trace: D184
import std;
@config(default);

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    let mut $count: i32 = 1;
    bump(Lend.mut $count);
    bump(Lend.mut $count);
    std::println($count as str);
    return void;
end fn.main
