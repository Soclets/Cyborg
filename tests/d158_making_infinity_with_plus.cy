// trace: D158
@config(default);

fn.main [] -> void:
    let $big: f32.stable = ~300000000000000000000000000000000000000.0;
    let $sum: f32.stable = $big + $big;
    return void;
end fn.main
