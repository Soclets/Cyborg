// trace: D5
@config(default);

fn f [$x: str] -> void:
    return void;
end fn f

fn.main [] -> void:
    let $x: str = "x";
    f(%$x);
    return void;
end fn.main
