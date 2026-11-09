// trace: D158
@config(default);

fn.main [] -> void:
    let $x: f32 = 0.5;
    if $x: f64[min: 0.0, max: 1.0]:
        return void;
    end if
    return void;
end fn.main
