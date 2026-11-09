// trace: D133
@config(default);

fn.main [] -> void:
    let $x: i32 = 1;
    let $n: i32 = $x[pos: 1];
    return void;
end fn.main
