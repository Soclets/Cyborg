// trace: D140
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = [1, 2, 3];
    let $n: u64.items = $xs.len;
    std::println($n as str);
    $xs.std::%append(val: 4);
    std::println($xs.len as str);
    std::println($n as str);
    let $more: u64.items = $xs.len + 2 items;
    std::println($more as str);
    if $more > $n:
        std::println("more");
    end if
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $m: u64.items = $a.len;
    std::println($m as str);
    if $a.len < $xs.len:
        std::println("fewer");
    end if
    return void;
end fn.main
