// trace: D159
@config(default);

fn take [$x: f64] -> void:
    return void;
end fn take

fn.main [] -> void:
    let $reading: f64.ieee = 1.5;
    take($reading);
    return void;
end fn.main
