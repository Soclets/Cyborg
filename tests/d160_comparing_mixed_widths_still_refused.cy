// trace: D160, D7
@config(default);

fn.main [] -> void:
    let $a: f64.stable = 0.5;
    let $b: f32.stable = 0.5;
    if $a == $b:
        return void;
    end if
    return void;
end fn.main
