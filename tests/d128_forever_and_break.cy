// trace: D128
import std;
@config(default);

fn.main [] -> void:
    let mut $n: i32 = 0;
    forever:
        change $n = $n + 1;
        if $n == 3:
            break;
        end if
    end forever
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
