// trace: D139, D44
@config(default);
import std;

fn inner [] -> i32:
    fails [lost[where: i32]];

    fail lost[where = 9];
end fn inner

fn outer [] -> i32:
    fails [lost[where: i32]];

    return inner()?;
end fn outer

fn.main [] -> void:
    let $n: i32 = outer() otherwise:
        case lost $l:
            instead $l.where;
    end otherwise
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
