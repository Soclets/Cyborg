// trace: D127, D75

fn my::true [none] -> i32:
    return 1;
end fn my::true

fn.main [none] -> void:
    let $a: i32 = true();
    return void;
end fn.main
