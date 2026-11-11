// trace: D159

fn take [$x: f64.stable] returns void:
    return void;
end fn take

fn.main [none] returns void:
    let $reading: f64.ieee set_to 1.5;
    take($reading);
    return void;
end fn.main
