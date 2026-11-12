// A struct could fill `?item`, so its parameter says how it's held.
// trace: D394
import std;

interface ordered<?self>:
    op below [$a: ?self, $b: ?self] -> bool;
end interface ordered

fn smallest<?item: ordered> [$a: ?item, $b: ?item] -> ?item:
    if $a below $b:
        return Copy $a;
    end if
    return Copy $b;
end fn smallest

fn.main [none] -> void:
    return void;
end fn.main
