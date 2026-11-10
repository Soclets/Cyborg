// trace: D165, D158
@config(default);

fn.main [] -> void:
    let $a: f32.stable = exponent ~3.5e38;
    return void;
end fn.main
