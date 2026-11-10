// trace: D164
@config(default);

fn.main [] -> void:
    let $a: f64.stable = 1.5;
    let $b: f64.stable = $a div 2;
    return void;
end fn.main
