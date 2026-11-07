// trace: D158
fn.main [] -> void:
    let $big: f32 = ~300000000000000000000000000000000000000.0;
    let $sum: f32 = $big + $big;
    return void;
end fn.main
