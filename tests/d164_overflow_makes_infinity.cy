// trace: D164, D158
@config(default);

fn.main [] -> void:
    let $big: f32 = ~300000000000000000000000000000000000000.0;
    let $a: f32 = $big div 0.25;
    return void;
end fn.main
