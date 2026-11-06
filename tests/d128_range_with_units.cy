// trace: D128, D100
narrowings.unit time:
    time.seconds: base;
end narrowings.unit time

fn.main [] -> void:
    for $i in range[from: 1 time.seconds, to: 5 time.seconds]:
        return void;
    end for
    return void;
end fn.main
