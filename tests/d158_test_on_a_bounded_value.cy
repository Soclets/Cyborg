// As with integers, the test is only on a plain value so far.
// trace: D158, D20
@config(default);

fn.main [] -> void:
    let $t: f64[min: -40.0, max: 125.0] = 21.5;
    if $t: f64[min: 0.0, max: 25.0]:
        return void;
    end if
    return void;
end fn.main
