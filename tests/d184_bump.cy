// trace: D184
import std;

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [none] -> void:
    let mut $count: i32 = 1;
    bump(Lend.mut $count);
    bump(Lend.mut $count);
    std::io::fmt_print("{}{newln}", $count);
    return void;
end fn.main
