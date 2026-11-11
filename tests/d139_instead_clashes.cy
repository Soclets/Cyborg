// trace: D139, D75

fn my::instead [none] -> i32:
    return 1;
end fn my::instead

fn.main [none] -> void:
    let $a: i32 = instead();
    return void;
end fn.main
