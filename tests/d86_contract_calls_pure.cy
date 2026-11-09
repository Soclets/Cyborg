// trace: D86
import std;
@config(default);

fn.pure positive [$n: i32] -> bool:
    return $n at_least 0;
end fn.pure positive

fn f [$n: i32] -> i32:
    requires positive($n);

    return $n;
end fn f

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", f(1));
    return void;
end fn.main
