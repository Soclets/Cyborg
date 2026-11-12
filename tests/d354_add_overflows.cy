// A plain `.%add` is checked for overflow like `+`.
// trace: D354, D42
import std;
import @concur;

fn.main [none] -> void:
    let $small: @concur::atomic<u8> = @concur::atomic.new(250);
    $small.%add(val: 10);
    return void;
end fn.main
