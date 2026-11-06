// How an offset is written isn't decided.
// trace: D110
narrowings.unit heat:
    heat.kelvin: base;
    heat.celsius = 1 heat.kelvin + 273;
end narrowings.unit heat

fn.main [] -> void:
    return void;
end fn.main
