// Locking inside a locked block is refused, so deadlocks can't happen.
// trace: D348
import std;
import @concur;

fn move [$a: @concur::lock<i64>, $b: @concur::lock<i64>] -> void:
    @concur::locked $a as $x:
        @concur::locked $b as $y:
            change $y = $x;
        end @concur::locked
    end @concur::locked
    return void;
end fn move

fn.main [none] -> void:
    return void;
end fn.main
