// `.%` says what it is, so a method's name doesn't clash.
// trace: D130, D75
import std;
@config(default);

struct point:
    x: i32;
end struct point

fn point.%x [$p: point] -> i32:
    return $p.x;
end fn point.%x

fn.main [] -> void:
    let $p: point = point[x = 3];
    std::println(($p.%x() x 2) as str);
    return void;
end fn.main
