// Two narrowings on one struct, and their order, are listed as undecided (12.2).
// trace: D319
import std;
import os;

struct.packed.must_Give Pair:
    $a: u8,
    $b: u32,
end struct.packed.must_Give

fn.main [none] -> void:
    return void;
end fn.main
