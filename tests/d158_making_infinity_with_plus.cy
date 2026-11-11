// trace: D158

fn.main [none] returns void:
    let $big: f32.stable set_to ~300000000000000000000000000000000000000.0;
    let $sum: f32.stable set_to $big + $big;
    return void;
end fn.main
