// trace: D164, D158

fn.main [none] returns void:
    let $big: f32.stable set_to ~300000000000000000000000000000000000000.0;
    let $a: f32.stable set_to $big div 0.25;
    return void;
end fn.main
