// A test that ran before this block left its mark on it.
// trace: D126, D2

fn eat [$s: Given str] returns bool:
    return 1 below 2;
end fn eat

fn read [$s: str] returns void:
    return void;
end fn read

fn.main [none] returns void:
    let $s: str set_to "a";
    if eat(Give $s):
        return void;
    else:
        read(Lend $s);
    end if
    return void;
end fn.main
