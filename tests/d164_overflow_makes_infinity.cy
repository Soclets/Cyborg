// trace: D164, D158

fn.main [none] -> void:
    let $big: f32.stable = ~300000000000000000000000000000000000000.0;
    let $a: f32.stable = $big div 0.25;
    return void;
end fn.main
