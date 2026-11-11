// trace: D158

fn.main [none] -> void:
    let $big: f32.stable = ~300000000000000000000000000000000000000.0;
    let $sum: f32.stable = $big + $big;
    return void;
end fn.main
