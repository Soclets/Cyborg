// A direct call to `@csup::extern.c` is NotDesignedYet too, even inside a shield.
// trace: D225
@config(default);
import @csup;

fn.main [] -> void:
    shield talk:
        reason "calls C"
        @csup::extern.c("hi");
    end shield talk
    return void;
end fn.main
