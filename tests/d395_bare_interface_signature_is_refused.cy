// An interface's signature writes its words like any other, since a struct could fill `?self`.
// trace: D395
import std;

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] -> bool;
end interface ordered

fn.main [none] -> void:
    return void;
end fn.main
