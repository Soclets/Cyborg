// trace: D126, D2

fn eat [$s: Given str[len.max: 32 bytes]] -> bool:
    return 1 below 2;
end fn eat

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    if 1 below 2:
        return void;
    else if eat(Give $s):
        return void;
    end if
    let $t: str[len.max: 32 bytes] = Give $s;
    return void;
end fn.main
