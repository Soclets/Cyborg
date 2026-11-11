// trace: D91

fn.pure area [$w: i32] returns i32:
    return $w;
end fn.pure area

fn.pure.baked f [$w: i32] returns i32:
    return area($w);
end fn.pure.baked f

fn.main [none] returns void:
    return void;
end fn.main
