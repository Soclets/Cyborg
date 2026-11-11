// trace: D159, D158

fn.main [none] returns void:
    let $reading: f64.ieee set_to 0.5;
    if $reading: f64.stable[min: 0.0, max: 1.0]:
        return void;
    end if
    return void;
end fn.main
