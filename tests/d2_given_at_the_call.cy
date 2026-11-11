// trace: D2

fn f [$s: Given str] -> void:
    return void;
end fn f

fn.main [none] -> void:
    let $s: str = "a";
    f(Given $s);
    return void;
end fn.main
