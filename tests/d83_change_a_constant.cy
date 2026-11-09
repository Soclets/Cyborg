// trace: D83, D3
@config(default);

const $a: i32 = 1;

fn.main [] -> void:
    change $a = 2;
    return void;
end fn.main
