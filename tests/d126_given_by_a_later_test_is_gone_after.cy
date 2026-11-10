// trace: D126, D2

fn eat [$s: Given str] -> bool:
    return 1 < 2;
end fn eat

fn.main [none] -> void:
    let $s: str = "a";
    if 1 < 2:
        return void;
    else if eat(Give $s):
        return void;
    end if
    let $t: str = Give $s;
    return void;
end fn.main
