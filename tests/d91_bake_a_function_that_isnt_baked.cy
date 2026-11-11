// trace: D91

fn.pure area [$w: i32] returns i32:
    return $w;
end fn.pure area

fn.main [none] returns void:
    let $a: i32 set_to bake area(1);
    return void;
end fn.main
