// trace: D161
@config(default);

fam.failure file_failed:
    file_failed.disk.full;
end fam.failure file_failed

fn.main [] -> void:
    return void;
end fn.main
