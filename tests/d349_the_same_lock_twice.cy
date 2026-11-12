// Naming one lock twice in one block is refused, since a task would wait on itself.
// trace: D349
import std;
import @concur;

fn twice [$a: Lent @concur::lock<i64>] -> void:
    @concur::locked $a, $a as $x, $y:
        change $x = 1;
    end @concur::locked
    return void;
end fn twice

fn.main [none] -> void:
    return void;
end fn.main
