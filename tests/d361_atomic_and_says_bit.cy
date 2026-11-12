// An atomic's bit operations say `.bit`, like the operators.
// trace: D360, D361
import std;
import @concur;

fn.main [none] -> void:
    let $flags: @concur::atomic<u8> = @concur::atomic.new(0);
    $flags.%or(val: 1);
    return void;
end fn.main
