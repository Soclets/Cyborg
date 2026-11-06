// trace: D123
fn f [$x: f64] -> f64:
    return $x;
end fn f

fn.main [] -> void:
    let $a: f64 = 0.5;
    let $b: f64 = f(Lend $a);
    return void;
end fn.main
