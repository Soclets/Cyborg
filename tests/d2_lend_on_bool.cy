// trace: D2

fn f [$b: bool] -> bool:
    return $b;
end fn f

fn.main [none] -> void:
    let $b: bool = 1 below 2;
    let $c: bool = f(Lend $b);
    return void;
end fn.main
