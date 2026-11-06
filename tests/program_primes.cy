// A real program, now that loops exist: primes by trial division.
// trace: D126, D127, D128
import std;

fn is_prime [$n: i32] -> bool:
    if $n < 2:
        return false;
    end if
    let mut $d: i32 = 2;
    while $d x $d at_most $n:
        if $n mod.trunc $d == 0:
            return false;
        end if
        change $d = $d + 1;
    end while
    return true;
end fn is_prime

fn.main [] -> void:
    let mut $found: std::list<i32>[len.max: 8 items] = empty;
    for $n in range[from: 1, to: 30]:
        if is_prime($n):
            $found.std::%append(val: $n) ?? void;
        end if
    end for
    for $p in $found:
        std::println($p as str);
    end for
    std::println("kept ", $found.len as str);
    return void;
end fn.main
