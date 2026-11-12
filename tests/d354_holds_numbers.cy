// An atomic holds an integer or a `bool`.
// trace: D354
import std;
import @concur;

fn.main [none] -> void:
    let $name: @concur::atomic<str> = @concur::atomic.new("a");
    return void;
end fn.main
