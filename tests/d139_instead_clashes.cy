// trace: D139, D75
@config(default);

fn instead [] -> i32:
    return 1;
end fn instead

fn.main [] -> void:
    let $a: i32 = instead();
    return void;
end fn.main
