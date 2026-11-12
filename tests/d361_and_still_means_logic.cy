// Plain `and`, `or` and `not` stay `bool` logic, next to their `.bit` forms.
// trace: D361
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 12;
    if ($a and.bit 4) == 4 and not ($a or.bit 1) == 12:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", not.bit $a);
    end if
    return void;
end fn.main
