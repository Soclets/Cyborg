// An unbuffered writer writes each print straight away, so it has nothing to flush.
// trace: D273
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    $out.%%flush();
    return void;
end fn.main
