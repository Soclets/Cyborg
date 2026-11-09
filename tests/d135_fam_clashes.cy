// trace: D135, D75
@config(default);

fn my::fam [] -> i32:
    return 1;
end fn my::fam

fn.main [] -> void:
    let $a: i32 = fam();
    return void;
end fn.main
