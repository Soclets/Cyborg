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
    std::println(f(1) as str);
    return void;
end fn.main
