// trace: D5

fn f [$x: str] returns void:
    return void;
end fn f

fn.main [none] returns void:
    let $x: str set_to "x";
    f(%$x);
    return void;
end fn.main
