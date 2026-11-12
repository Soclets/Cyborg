// `arena` and `allocator` live in `std::mem`.
// trace: D378
import std;

fn.main [none] -> void:
    let $a: std::arena = std::arena.new(room: 64 bytes);
    return void;
end fn.main
