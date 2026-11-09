// trace: D139, D75
@config(default);

fn my::instead [] -> i32:
    return 1;
end fn my::instead

fn.main [] -> void:
    let $a: i32 = instead();
    return void;
end fn.main
