// trace: D134, D57
import std;

struct point:
    x: i32;
end struct point

fam.type.alias shapes:
    shapes.dot = point;
end fam.type.alias shapes

fn.main [] -> void:
    let $d: shapes.dot = shapes.dot[x = 4];
    std::println($d.x as str);
    return void;
end fn.main
