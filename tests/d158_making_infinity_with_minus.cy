// trace: D158
@config(default);

fn.main [] -> void:
    let $big: f32 = ~300000000000000000000000000000000000000.0;
    let $down: f32 = -$big - $big;
    return void;
end fn.main
