use std::path::Path;

use codepages::tables::CP437_TO_UNICODE;

use crate::{parse_with_parser, parsers, Buffer, BufferFeatures, BufferType, EngineResult, OutputFormat, Position, TagPlacement, TextAttribute, TextPane};

use super::{LoadData, SaveOptions};

pub(crate) const HEX_TABLE: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Default)]
pub struct PCBoard {}

impl OutputFormat for PCBoard {
    fn get_file_extension(&self) -> &str {
        "pcb"
    }

    fn get_name(&self) -> &str {
        "PCBoard"
    }

    fn analyze_features(&self, _features: &BufferFeatures) -> String {
        String::new()
    }

    fn to_bytes(&self, buf: &mut crate::Buffer, options: &SaveOptions) -> EngineResult<Vec<u8>> {
        if buf.palette.len() != 16 {
            return Err(anyhow::anyhow!("Only 16 color palettes are supported by this format."));
        }
        let mut result: Vec<u8> = Vec::new();
        let mut last_attr = TextAttribute::default();
        let mut pos = Position::default();
        let height = buf.get_line_count();
        let mut first_char = true;
        if options.modern_terminal_output {
            // write UTF-8 BOM as unicode indicator.
            result.extend([0xEF, 0xBB, 0xBF]);
        }

        match options.screen_preparation {
            super::ScreenPreperation::None | super::ScreenPreperation::Home => {} // home not supported
            super::ScreenPreperation::ClearScreen => {
                result.extend(b"@CLS@");
            }
        }
        while pos.y < height {
            let line_length = buf.get_line_length(pos.y);

            while pos.x < line_length {
                let mut found_tag = false;
                for tag in &buf.tags {
                    if tag.is_enabled && tag.tag_placement == TagPlacement::InText && tag.position.y == pos.y as i32 && tag.position.x == pos.x as i32 {
                        result.extend(tag.replacement_value.as_bytes());
                        pos.x += (tag.len() as i32).max(1);
                        found_tag = true;
                        break;
                    }
                }
                if found_tag {
                    continue;
                }

                let ch = buf.get_char(pos);

                if first_char || ch.attribute != last_attr {
                    result.extend_from_slice(format!("@X{:02X}", ch.attribute.as_u8(crate::IceMode::Blink)).as_bytes());
                    last_attr = ch.attribute;
                }

                if options.modern_terminal_output {
                    if ch.ch == '\0' {
                        result.push(b' ')
                    } else {
                        let uni_ch = if buf.buffer_type == BufferType::Unicode {
                            ch.ch.to_string()
                        } else {
                            CP437_TO_UNICODE[ch.ch as usize].to_string()
                        };
                        result.extend(uni_ch.as_bytes().to_vec());
                    }
                } else {
                    result.push(if ch.ch == '\0' { b' ' } else { ch.ch as u8 });
                }
                first_char = false;
                pos.x += 1;
            }

            // do not end with eol
            if pos.x < buf.get_width() && pos.y + 1 < height {
                result.push(13);
                result.push(10);
            }

            pos.x = 0;
            pos.y += 1;
        }

        for tag in &buf.tags {
            if tag.is_enabled && tag.tag_placement == crate::TagPlacement::WithGotoXY {
                result.extend(format!("\x1B[{};{}H", tag.position.y + 1, tag.position.x + 1).as_bytes());
                result.extend(tag.replacement_value.as_bytes());
            }
        }

        if options.save_sauce {
            buf.write_sauce_info(icy_sauce::SauceDataType::Character, icy_sauce::char_caps::ContentType::PCBoard, &mut result)?;
        }
        Ok(result)
    }

    fn load_buffer(&self, file_name: &Path, data: &[u8], load_data_opt: Option<LoadData>) -> EngineResult<crate::Buffer> {
        let load_data = load_data_opt.unwrap_or_default();
        let width = load_data.default_terminal_width.unwrap_or(80);
        let mut result: Buffer = Buffer::new((width, 25));

        result.is_terminal_buffer = false;
        result.file_name = Some(file_name.into());
        if let Some(sauce) = load_data.sauce_opt {
            result.load_sauce(sauce);
        }

        /*
                let mut interpreter: Box<dyn BufferParser> = match interpreter {
            CharInterpreter::Ansi => {
                let mut parser = Box::<parsers::ansi::Parser>::default();
                parser.bs_is_ctrl_char = false;
                parser
            }
        };
         */
        let (text, is_unicode) = crate::convert_ansi_to_utf8(data);
        if is_unicode {
            result.buffer_type = crate::BufferType::Unicode;
        }
        parse_with_parser(&mut result, &mut parsers::pcboard::Parser::default(), &text, true)?;
        Ok(result)
    }
}

pub fn get_save_sauce_default_pcb(buf: &Buffer) -> (bool, String) {
    if buf.get_width() != 80 {
        return (true, "width != 80".to_string());
    }

    if buf.has_sauce() {
        return (true, String::new());
    }

    (false, String::new())
}
