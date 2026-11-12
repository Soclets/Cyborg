// An operator on two `simd` values works on every lane, comparisons give one `bool` per lane,
// and `.%pick` chooses lane by lane.
// trace: D291, D297, D301, D302
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: simd<i32>[lanes: 4] = [1, 8, 3, 6];
    let $b: simd<i32>[lanes: 4] = [5, 5, 5, 5];
    let $c: simd<i32>[lanes: 4] = $a + $b;
    let $m: simd<bool>[lanes: 4] = $a above $b;
    let $big: simd<i32>[lanes: 4] = $m.%pick(if_true: $a, if_false: $b);
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $c.%sum(), $big.%sum(), $a.%max_val(), $a.%min_val());
    if ($a == $b).%any_true():
        std::io::fmt_print(Lend.mut $out, "some lane equal{newln}");
    end if
    if $m.%any_true():
        std::io::fmt_print(Lend.mut $out, "some lane above{newln}");
    end if
    if not $m.%all_true():
        std::io::fmt_print(Lend.mut $out, "not every lane above{newln}");
    end if
    let $z: simd<i32>[lanes: 3] = all 0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $z.%sum());
    return void;
end fn.main
