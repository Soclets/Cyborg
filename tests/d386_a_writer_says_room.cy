// A buffered writer says `room:`, the space for what it holds, as an arena does.
// trace: D386, D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.buffered = os::stdout.buffered(size: 1024 bytes);
    os::done(Give $out);
    return void;
end fn.main
