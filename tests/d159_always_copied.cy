// trace: D159, D123
@config(default);

fn.main [] -> void:
    let $reading: f64.ieee = 0.5;
    let $again: f64.ieee = Copy $reading;
    return void;
end fn.main
