// With `else:`, nothing runs past a chain whose blocks all leave.
// trace: D126, D11
import std;
@config(default);

fn sign [$n: i32] -> i32:
    if $n > 0:
        return 1;
    else if $n < 0:
        return -1;
    else:
        return 0;
    end if
end fn sign

fn.main [] -> void:
    std::println(sign(5) as str);
    std::println(sign(-5) as str);
    std::println(sign(0) as str);
    return void;
end fn.main
