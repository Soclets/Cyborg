// trace: D91
@config(default);

fn.pure.baked f [$w: i32] -> i32:
    return $w;
end fn.pure.baked f

fn.main [] -> void:
    let $d: i32 = 2;
    let $a: i32 = bake f($d);
    return void;
end fn.main
