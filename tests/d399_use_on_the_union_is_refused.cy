// C's union joins the base `union` family, and `use` never works on a base family.
// trace: D399, D232
import std;
import @csup;
use union.@csup::untagged as untagged;

fn.main [none] -> void:
    return void;
end fn.main
