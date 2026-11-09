// trace: D126, D2
@config(default);

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $s: str = "a";
    if 1 < 2:
        eat(Give $s);
    else:
        eat(Give $s);
    end if
    eat(Give $s);
    return void;
end fn.main
