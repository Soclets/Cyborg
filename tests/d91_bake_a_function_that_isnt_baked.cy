// trace: D91

fn.pure area [$w: i32] -> i32:
    return $w;
end fn.pure area

fn.main [none] -> void:
    let $a: i32 = bake area(1);
    return void;
end fn.main
