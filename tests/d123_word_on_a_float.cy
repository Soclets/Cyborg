// trace: D123

fn f [$x: f64.stable] returns f64.stable:
    return $x;
end fn f

fn.main [none] returns void:
    let $a: f64.stable = 0.5;
    let $b: f64.stable = f(Lend $a);
    return void;
end fn.main
