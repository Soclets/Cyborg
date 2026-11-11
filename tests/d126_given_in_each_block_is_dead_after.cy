// trace: D126, D2

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $s: str set_to "a";
    if 1 below 2:
        eat(Give $s);
    else:
        eat(Give $s);
    end if
    eat(Give $s);
    return void;
end fn.main
