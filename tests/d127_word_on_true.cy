// A fresh value takes no word, and neither does a `bool`.
// trace: D127, D2
@config(default);

fn f [$b: bool] -> bool:
    return $b;
end fn f

fn.main [] -> void:
    let $a: bool = f(Lend true);
    return void;
end fn.main
