// `%` looks up the method on the receiver's type, and this one has none.
// trace: D8, D130
@config(default);

struct point:
    x: i32;
end struct point

fn.main [] -> void:
    let $p: point = point[x = 1];
    $p.%area();
    return void;
end fn.main
