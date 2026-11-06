// A test that ran before this block left its mark on it.
// trace: D126, D2
fn eat [$s: Given str] -> bool:
    return 1 < 2;
end fn eat

fn read [$s: str] -> void:
    return void;
end fn read

fn.main [] -> void:
    let $s: str = "a";
    if eat(Give $s):
        return void;
    else:
        read(Lend $s);
    end if
    return void;
end fn.main
