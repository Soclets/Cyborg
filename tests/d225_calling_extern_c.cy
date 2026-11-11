// `@csup::extern.c` was never defined, and how C functions come through `@csup` isn't designed.
// trace: D225, D155
import @csup;

use @csup::extern.c as c;

fn.main [none] returns void:
    c("hi");
    return void;
end fn.main
