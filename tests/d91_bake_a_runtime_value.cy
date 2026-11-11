// trace: D91

fn.pure.baked f [$w: i32] returns i32:
    return $w;
end fn.pure.baked f

fn.main [none] returns void:
    let $d: i32 = 2;
    let $a: i32 = bake f($d);
    return void;
end fn.main
