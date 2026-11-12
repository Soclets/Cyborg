// `.%lookup` takes positions worked out while running, and one past the end is a bounds
// failure that says which lane.
// trace: D306
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $table: simd<u8>[lanes: 4] = [10, 20, 30, 40];
    let $which: simd<u8>[lanes: 3] = [4, 1, 2];
    let $got: simd<u8>[lanes: 3] = $table.%lookup(pos: $which);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $got.%sum());
    let $bad: simd<u8>[lanes: 3] = [1, 5, 2];
    let $boom: simd<u8>[lanes: 3] = $table.%lookup(pos: $bad);
    return void;
end fn.main
