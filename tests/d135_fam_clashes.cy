// trace: D135, D75

fn my::fam [none] -> i32:
    return 1;
end fn my::fam

fn.main [none] -> void:
    let $a: i32 = fam();
    return void;
end fn.main
