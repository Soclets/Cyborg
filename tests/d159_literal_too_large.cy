// trace: D159, D158
@config(default);

fn.main [] -> void:
    let $x: f32.ieee = ~1000000000000000000000000000000000000000000.0;
    return void;
end fn.main
