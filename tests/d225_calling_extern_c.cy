// `@csup::extern.c` was never defined, and how C functions come through `@csup` isn't designed.
// trace: D225, D155
@config(default);
import @csup;

use @csup::extern.c as c;

fn.main [] -> void:
    c("hi");
    return void;
end fn.main
