// trace: D159, D158
@config(default);

fn.main [] -> void:
    let $reading: f64.ieee = 0.5;
    if $reading: f64[min: 0.0, max: 1.0]:
        return void;
    end if
    return void;
end fn.main
