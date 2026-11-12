// `.%shuffle` lists where each lane comes from, and `reverse`, `rotate`, `join` and
// `interleave` are its four named forms. `as` turns a `simd` into an array of the same count.
// trace: D304, D305, D307, D298
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $v: simd<i32>[lanes: 4] = [1, 2, 3, 4];
    let $w: simd<i32>[lanes: 2] = [10, 20];
    let $all: std::coll::list<simd<i32>[lanes: 4]>[len.max: 4 items] = [$v.%reverse(), $v.%rotate.left(by: 1), $v.%rotate.right(by: 1), $v.%shuffle(pos: [4, 4, 1, 2])];
    for $each in $all:
        let $a: std::coll::array<i32>[len: 4 items] = $each as std::coll::array<i32>[len: 4 items];
        for $x in $a:
            std::io::fmt_print(Lend.mut $out, "{$x} ");
        end for
        std::io::fmt_print(Lend.mut $out, "{newln}");
    end for
    let $two: simd<i32>[lanes: 4] = $w.%join(then: $w);
    let $mixed: simd<i32>[lanes: 6] = $v.%interleave(with: $w);
    let $a: std::coll::array<i32>[len: 6 items] = $mixed as std::coll::array<i32>[len: 6 items];
    for $x in $a:
        std::io::fmt_print(Lend.mut $out, "{$x} ");
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $two.%sum());
    return void;
end fn.main
