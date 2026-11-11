// trace: D2

fn f [$b: bool] returns bool:
    return $b;
end fn f

fn.main [none] returns void:
    let $b: bool set_to 1 below 2;
    let $c: bool set_to f(Lend $b);
    return void;
end fn.main
