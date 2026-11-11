// trace: D126, D2

fn eat [$s: Given str] returns bool:
    return 1 below 2;
end fn eat

fn.main [none] returns void:
    let $s: str set_to "a";
    if 1 below 2:
        return void;
    else if eat(Give $s):
        return void;
    end if
    let $t: str set_to Give $s;
    return void;
end fn.main
