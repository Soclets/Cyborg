// trace: D158
@config(default);

fn.main [] -> void:
    let $a: f64.fast = 1.5;
    let $b: f64.stable = $a as f64.stable;
    return void;
end fn.main
