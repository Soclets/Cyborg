// A number is only ever copied, so how it's held is no choice, and it takes no word.
// trace: D393, D123
import std;

fn twice [$n: Lent i32] -> i32:
    return $n + $n;
end fn twice

fn.main [none] -> void:
    return void;
end fn.main
