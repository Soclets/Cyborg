// `flush` changes its writer, so it's called with `.%%`.
// trace: D273, D211
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    $out.%flush();
    os::done(Give $out);
    return void;
end fn.main
