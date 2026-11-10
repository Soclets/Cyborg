// trace: D159

fn take [$x: f64.stable] -> void:
    return void;
end fn take

fn.main [none] -> void:
    let $reading: f64.ieee = 1.5;
    take($reading);
    return void;
end fn.main
