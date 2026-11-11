// trace: D2

fn f [$s: Given str] returns void:
    return void;
end fn f

fn.main [none] returns void:
    let $s: str = "a";
    f(Given $s);
    return void;
end fn.main
