// trace: D127, D75

fn my::true [none] returns i32:
    return 1;
end fn my::true

fn.main [none] returns void:
    let $a: i32 = true();
    return void;
end fn.main
