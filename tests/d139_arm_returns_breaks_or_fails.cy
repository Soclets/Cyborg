// trace: D139, D128, D52
@config(default);
import std;

fn f [$which: i32] -> i32 fails [a]:
    if $which > 0:
        fail a;
    end if
    return 1;
end fn f

fn g [] -> i32 fails [b]:
    let $n: i32 = f(1) otherwise:
        case a:
            fail b;
    end otherwise
    return $n;
end fn g

fn h [] -> i32:
    let $n: i32 = f(1) otherwise:
        case a:
            return 7;
    end otherwise
    return $n;
end fn h

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", (g() ?? 5));
    std::io::fmt_print("{}{newln}", h());
    forever:
        let $n: i32 = f(1) otherwise:
            case a:
                std::io::fmt_print("broke out{newln}");
                break;
        end otherwise
        std::io::fmt_print("not reached{newln}");
    end forever
    return void;
end fn.main
