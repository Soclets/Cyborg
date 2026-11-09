// trace: D139, D128, D52
import std;
@config(default);

fn f [$which: i32] -> i32:
    fails [a];

    if $which > 0:
        fail a;
    end if
    return 1;
end fn f

fn g [] -> i32:
    fails [b];

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
    std::println((g() ?? 5) as str);
    std::println(h() as str);
    forever:
        let $n: i32 = f(1) otherwise:
            case a:
                std::println("broke out");
                break;
        end otherwise
        std::println("not reached");
    end forever
    return void;
end fn.main
