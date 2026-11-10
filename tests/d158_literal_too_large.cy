// trace: D158, D20
@config(default);

fn.main [] -> void:
    let $x: f32.stable = ~1000000000000000000000000000000000000000000.0;
    return void;
end fn.main
