// With `else:`, nothing runs past a chain whose blocks all leave.
// trace: D126, D11
import std;

fn sign [$n: i32] -> i32:
    if $n > 0:
        return 1;
    else if $n < 0:
        return -1;
    else:
        return 0;
    end if
end fn sign

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", sign(5));
    std::io::fmt_print("{}{newln}", sign(-5));
    std::io::fmt_print("{}{newln}", sign(0));
    return void;
end fn.main
