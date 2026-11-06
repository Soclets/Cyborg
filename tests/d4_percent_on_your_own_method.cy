// How a method is declared isn't decided.
// trace: D8
struct point:
    x: i32;
end struct point

fn.main [] -> void:
    let $p: point = point[x = 1];
    $p.%area();
    return void;
end fn.main
