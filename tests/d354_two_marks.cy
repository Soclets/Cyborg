// An atomic is shared with plain `Lend`, so its operations take `.%`.
// trace: D354
import std;
import @concur;

fn.main [none] -> void:
    let $hits: @concur::atomic<u64> = @concur::atomic.new(0);
    $hits.%%add(val: 1);
    return void;
end fn.main
