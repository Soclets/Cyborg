// trace: D2, D83
@config(default);

const $s: str = "a";

fn f [$s: Lent.mut str] -> void:
    return void;
end fn f

fn.main [] -> void:
    f(Lend.mut $s);
    return void;
end fn.main
