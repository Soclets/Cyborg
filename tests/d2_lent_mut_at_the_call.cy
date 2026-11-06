// trace: D2
fn f [$s: Lent.mut str] -> void:
    return void;
end fn f

fn.main [] -> void:
    let mut $s: str = "a";
    f(Lent.mut $s);
    return void;
end fn.main
