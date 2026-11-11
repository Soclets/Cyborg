// In the order written, the first `x` makes infinity. A compiler that merges `10.0 x ~0.1` carries
// on, and that isn't a bug.
// trace: D163

fn.main [none] returns void:
    let $big: f32.fast set_to ~300000000000000000000000000000000000000.0;
    let $y: f32.fast set_to $big x 10.0 x ~0.1;
    return void;
end fn.main
