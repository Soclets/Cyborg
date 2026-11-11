// trace: D139, D44
import std;
import os;

fn inner [none] returns i32 fails [lost[where: i32]]:
    fail lost[where = 9];
end fn inner

fn outer [none] returns i32 fails [lost[where: i32]]:
    return inner()?;
end fn outer

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = outer() otherwise:
        case lost $l:
            instead $l.where;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
