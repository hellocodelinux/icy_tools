use super::*;
use crate::parsers::create_buffer;

#[test]
fn test_rip_text_window() {
    test_roundtrip("|w00001B0M10");
}

#[test]
fn test_rip_viewport() {
    test_roundtrip("|v00002E1M");
}

#[test]
fn test_reset_windows() {
    test_roundtrip("|*");
}

#[test]
fn test_erase_window() {
    test_roundtrip("|e");
}

#[test]
fn test_erase_view() {
    test_roundtrip("|E");
}

#[test]
fn test_gotoxy() {
    test_roundtrip("|g0509");
}

#[test]
fn test_home() {
    test_roundtrip("|H");
}

#[test]
fn test_erase_eol() {
    test_roundtrip("|>");
}

#[test]
fn test_color() {
    test_roundtrip("|c0A");
}

#[test]
fn test_set_palette() {
    test_roundtrip("|Q000102030405060708090A0B0C0D0E0F");
}

#[test]
fn test_one_palette() {
    test_roundtrip("|a051B");
}

#[test]
fn test_write_mode() {
    test_roundtrip("|W00");
}

#[test]
fn test_move() {
    test_roundtrip("|m0509");
}

#[test]
fn test_text() {
    test_roundtrip("|Thello world");
}

#[test]
fn test_text_xy() {
    test_roundtrip("|@0011hello world");
}

#[test]
fn test_font_style() {
    test_roundtrip("|Y01000400");
}

#[test]
fn test_pixel() {
    test_roundtrip("|X1122");
}

#[test]
fn test_line() {
    test_roundtrip("|L00010A0E");
}

#[test]
fn test_rectangle() {
    test_roundtrip("|R00010A0E");
}

#[test]
fn test_bar() {
    test_roundtrip("|B00010A0E");
}

#[test]
fn test_circle() {
    test_roundtrip("|C1E180M");
}

#[test]
fn test_oval() {
    test_roundtrip("|O1E1A18003G15");
}

#[test]
fn test_filled_oval() {
    test_roundtrip("|o1G2B0M0G");
}

#[test]
fn test_arc() {
    test_roundtrip("|A1E18003G15");
}

#[test]
fn test_oval_arc() {
    test_roundtrip("|V1E18003G151Q");
}

#[test]
fn test_pie_slice() {
    test_roundtrip("|I1E18003G15");
}

#[test]
fn test_oval_pie_slice() {
    test_roundtrip("|i1E18003G151Q");
}

#[test]
fn test_bezier() {
    test_roundtrip("|Z0A0B0C0D0E0F0G0H1G");
}

#[test]
fn test_polygon() {
    test_roundtrip("|P03010105090905");
}

#[test]
fn test_fill_polygon() {
    test_roundtrip("|p03010105050909");
}

#[test]
fn test_polyline() {
    test_roundtrip("|l03010105050909");
}

#[test]
fn test_fill() {
    test_roundtrip("|F25090F");
}

#[test]
fn test_line_style() {
    test_roundtrip("|=01000001");
}

#[test]
fn test_fill_style() {
    test_roundtrip("|S050F");
}

#[test]
fn test_fill_pattern() {
    test_roundtrip("|s11223344556677880F");
}

#[test]
fn test_mouse() {
    test_roundtrip("|1M00001122331100000host command^M");
}

#[test]
fn test_kill_mouse_fields() {
    test_roundtrip("|1K");
}

#[test]
fn test_begin_text() {
    test_roundtrip("|1T0011001100");
}

#[test]
fn test_region_text() {
    test_roundtrip("|1t1This is a text line to be justified");
}

#[test]
fn test_end_text() {
    test_roundtrip("|1K");
}

#[test]
fn test_get_image() {
    test_roundtrip("|1C001122330");
}

#[test]
fn test_put_image() {
    test_roundtrip("|1P0011010");
}

#[test]
fn test_write_icon() {
    test_roundtrip("|1W0filename.icn");
}

/*
#[test]
fn test_load_icon() {
    test_roundtrip("|1I001101010button.icn");
}*/

#[test]
fn test_button_style() {
    test_roundtrip("|1B0A0A010274030F080F080700010E07000000");
}
/*
#[test]
fn test_button() {
    test_roundtrip("|1U010100003200iconfile<>Label<>HostCmd^m");
}*/

#[test]
fn test_define() {
    test_roundtrip("|1D00700text_var,60:?question?default data");
}

#[test]
fn test_query() {
    test_roundtrip("|1\x1B0000this is a query $COMMAND$^m");
}

#[test]
fn test_copy_region() {
    test_roundtrip("|1G080G140M0005");
}

#[test]
fn test_read_scene() {
    test_roundtrip("|1R00000000testfile.rip");
}

#[test]
fn test_enter_block_mode() {
    test_roundtrip("|9\x1B00010000ICONFILE.ICN<>");
}

fn test_roundtrip(arg: &str) {
    let mut parser = Parser::new(Box::default(), PathBuf::new());
    parser.record_rip_commands = true;
    create_buffer(&mut parser, ("!".to_string() + arg + "|").as_bytes());

    assert!(parser.command.is_none());
    assert_eq!(parser.rip_commands.len(), 1);
    assert_eq!(parser.rip_commands[0].to_rip_string(), arg);
}
