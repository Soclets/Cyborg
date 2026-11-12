// Which memory is touched leaks a secret through the cache, so a position stays public.
// trace: D340
import std;

fn.main [none] -> void:
    let $table: std::coll::array<u8>[len: 4 items] = [1, 2, 3, 4];
    let $which: i32.secret = 2;
    let $x: u8 = $table[pos: $which];
    return void;
end fn.main
