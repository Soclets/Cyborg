// trace: D155
import @csup;

use @csup::extern.c as c;
@config(default);

fn.main [] -> void:
    c("hi");
    return void;
end fn.main
