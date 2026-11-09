// trace: D57, D83
@config(default);

const $a: i32 = 1;

fn.main [] -> void:
    let $a: i32 = 2;
    return void;
end fn.main
