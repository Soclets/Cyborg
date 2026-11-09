// trace: D135, D75
@config(default);

fn fam [] -> i32:
    return 1;
end fn fam

fn.main [] -> void:
    let $a: i32 = fam();
    return void;
end fn.main
