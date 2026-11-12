// A test that ran before this block left its mark on it.
// trace: D126, D2

fn eat [$s: Given str[len.max: 32 bytes]] -> bool:
    return 1 below 2;
end fn eat

fn read [$s: Lent str[len.max: 32 bytes]] -> void:
    return void;
end fn read

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    if eat(Give $s):
        return void;
    else:
        read(Lend $s);
    end if
    return void;
end fn.main
