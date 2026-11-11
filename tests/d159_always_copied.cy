// trace: D159, D123

fn.main [none] returns void:
    let $reading: f64.ieee set_to 0.5;
    let $again: f64.ieee set_to Copy $reading;
    return void;
end fn.main
