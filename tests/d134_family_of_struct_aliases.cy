// trace: D134, D57
import std;
@config(default);

struct point:
    x: i32;
end struct point

fam.type.alias shapes:
    shapes.dot = point;
end fam.type.alias shapes

fn.main [] -> void:
    let $d: shapes.dot = shapes.dot[x = 4];
    std::io::fmt_print("{}{newln}", $d.x);
    return void;
end fn.main
