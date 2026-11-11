// trace: D2, D83

const $s: str = "a";

fn f [$s: Lent.mut str] returns void:
    return void;
end fn f

fn.main [none] returns void:
    f(Lend.mut $s);
    return void;
end fn.main
