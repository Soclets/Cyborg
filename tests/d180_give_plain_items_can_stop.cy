// Numbers carry no duty, so stopping early leaves nothing owed.
// trace: D180, D128
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: inf items] = [1, 2, 3];
    for $n in Give $xs:
        if $n == 2:
            break;
        end if
        std::println($n as str);
    end for
    return void;
end fn.main
