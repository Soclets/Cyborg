// trace: D139, D128, D52
import std;
import os;

fn f [$which: i32] returns i32 fails [a]:
    if $which above 0:
        fail a;
    end if
    return 1;
end fn f

fn g [none] returns i32 fails [b]:
    let $n: i32 set_to f(1) otherwise:
        case a:
            fail b;
    end otherwise
    return $n;
end fn g

fn h [none] returns i32:
    let $n: i32 set_to f(1) otherwise:
        case a:
            return 7;
    end otherwise
    return $n;
end fn h

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (g() ?? 5));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", h());
    forever:
        let $n: i32 set_to f(1) otherwise:
            case a:
                std::io::fmt_print(Lend.mut $out, "broke out{newln}");
                break;
        end otherwise
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end forever
    return void;
end fn.main
