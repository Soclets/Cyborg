// `@tighten_config` is gone too, wherever it's written.
// trace: D265
import std;
@tighten_config(var.mut.never_changed);

fn.main [none] returns void:
    return void;
end fn.main
