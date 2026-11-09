// An endless loop in compile-time code reaches the step limit.
// trace: D128, D89
@config(default);

fn.pure.baked spin [] -> i32:
    forever:
    end forever
end fn.pure.baked spin

const $a: i32 = spin();

fn.main [] -> void:
    return void;
end fn.main
