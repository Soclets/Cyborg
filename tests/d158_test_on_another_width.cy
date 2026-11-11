// trace: D158

fn.main [none] returns void:
    let $x: f32.stable set_to 0.5;
    if $x: f64.stable[min: 0.0, max: 1.0]:
        return void;
    end if
    return void;
end fn.main
