// trace: D2
@config(default);

fn f [$s: Given str] -> void:
    return void;
end fn f

fn.main [] -> void:
    let $s: str = "a";
    f(Given $s);
    return void;
end fn.main
