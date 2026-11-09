// trace: D159
@config(default);

fn.main [] -> void:
    let $reading: f32.ieee = 0.5;
    let $clean: f64 = $reading as f64;
    return void;
end fn.main
