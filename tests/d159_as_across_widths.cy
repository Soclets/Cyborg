// trace: D159

fn.main [none] returns void:
    let $reading: f32.ieee set_to 0.5;
    let $clean: f64.stable set_to $reading as f64.stable;
    return void;
end fn.main
