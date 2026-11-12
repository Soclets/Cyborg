// A parameter holding a struct chooses how it's held, so it says `Lent`, `Lent.mut` or
// `Given`, and a bare one is refused. This reverses D182.
// trace: D393
import std;

struct reading:
    celsius: i32;
end struct reading

fn show [$r: reading] -> i32:
    return $r.celsius;
end fn show

fn.main [none] -> void:
    return void;
end fn.main
