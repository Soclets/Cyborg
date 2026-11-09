// trace: D91
@config(default);

fn.pure area [$w: i32] -> i32:
    return $w;
end fn.pure area

fn.pure.baked f [$w: i32] -> i32:
    return area($w);
end fn.pure.baked f

fn.main [] -> void:
    return void;
end fn.main
