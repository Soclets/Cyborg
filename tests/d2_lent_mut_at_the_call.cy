// trace: D2

fn f [$s: Lent.mut str] returns void:
    return void;
end fn f

fn.main [none] returns void:
    let mut $s: str = "a";
    f(Lent.mut $s);
    return void;
end fn.main
