// A literal lands in `items` next to one, so the item's in `items` too. Reported as a reading.
// trace: D142, D101
import std;

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 3 items] set_to [4, 5, 6];
    for $i in range[from: 1, to: $xs.len]:
        let $n: u64 set_to $i;
    end for
    return void;
end fn.main
