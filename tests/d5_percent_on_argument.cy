// trace: D5

fn f [$x: str] -> void:
    return void;
end fn f

fn.main [none] -> void:
    let $x: str = "x";
    f(%$x);
    return void;
end fn.main
