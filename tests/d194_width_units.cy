// `é` is one column and two bytes, `日` two columns, and the flag one grapheme of two characters.
// trace: D194
@config(default);
import std;

fn.main [] -> void:
    std::io::fmt_print("[{align: left, width: 4 cols}]{newln}", "é");
    std::io::fmt_print("[{align: left, width: 4 bytes}]{newln}", "é");
    std::io::fmt_print("[{align: left, width: 4 cols}]{newln}", "日");
    std::io::fmt_print("[{align: left, width: 4 graphemes}]{newln}", "🇹🇭");
    std::io::fmt_print("[{align: left, width: 4 cols.east_asian}]{newln}", "±");
    std::io::fmt_print("[{align: left, width: 4 cols}]{newln}", "±");
    return void;
end fn.main
