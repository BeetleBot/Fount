use std::collections::BTreeMap;
use std::io::Write;

use crate::pdf::{
    Exporter, Screenplay,
    rich_string::RichString,
    screenplay::{DialogueElement, Element},
};

pub struct FdxExporter {
    pub raw_lines: Vec<String>,
    pub include_title_page: bool,
    pub export_sections: bool,
    pub export_synopses: bool,
    pub export_production_tags: bool,
}

impl FdxExporter {
    pub fn new(raw_lines: Vec<String>) -> Self {
        Self {
            raw_lines,
            include_title_page: true,
            export_sections: true,
            export_synopses: true,
            export_production_tags: true,
        }
    }
}

impl Exporter for FdxExporter {
    fn file_extension(&self) -> &'static str {
        "fdx"
    }

    fn export(&self, screenplay: &Screenplay, writer: &mut dyn Write) -> std::io::Result<()> {
        // First Pass: Extract all unique categories and tags across the entire script
        let mut global_categories = BTreeMap::new(); // lowercase_cat -> CAT_ID
        let mut global_tags = BTreeMap::new();       // (lowercase_cat, item) -> TAG_ID

        if self.export_production_tags {
            for span in &screenplay.elements {
                for line_num in span.start_line..=span.end_line {
                    if line_num > 0 && line_num <= self.raw_lines.len() {
                        let line = &self.raw_lines[line_num - 1];
                        let tags = parse_line_tags(line);
                        for t in tags {
                            let cat_id = format!("CAT_{}", make_xml_id(&t.category));
                            let tag_id = format!("TAG_{}_{}", make_xml_id(&t.category), make_xml_id(&t.item));
                            global_categories.insert(t.category.clone(), cat_id);
                            global_tags.insert((t.category.clone(), t.item.clone()), tag_id);
                        }
                    }
                }
            }
        }

        // XML Boilerplate Header
        writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>")?;
        writeln!(writer, "<FinalDraft DocumentType=\"Script\" Version=\"4\">")?;

        // Content Section
        writeln!(writer, "  <Content>")?;

        let mut element_iter = screenplay.elements.iter().peekable();
        let mut line_idx = 0;

        while line_idx < self.raw_lines.len() {
            let current_line_1based = line_idx + 1;

            if element_iter.peek().is_some_and(|span| span.start_line <= current_line_1based) {
                let span = element_iter.next().unwrap();
                match &span.inner {
                        Element::Heading { slug, number } => {
                            let num_attr = number.as_ref().map_or("".to_string(), |num| format!(" Number=\"{}\"", escape_xml(num)));
                            write_paragraph(
                                writer,
                                "Scene Heading",
                                &num_attr,
                                &rich_string_to_fdx(slug),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::Action(rich_string) => {
                            write_paragraph(
                                writer,
                                "Action",
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::Dialogue(dialogue) => {
                            let mut diag_line_idx = span.start_line - 1;
                            
                            // Character Paragraph
                            let mut char_text = dialogue.character.to_plain_string();
                            if let Some(ref ext) = dialogue.extension {
                                char_text.push_str(&format!(" ({})", ext.to_plain_string()));
                            }
                            write_paragraph_for_line(
                                writer,
                                "Character",
                                "",
                                &format!("<Text>{}</Text>", escape_xml(char_text.trim())),
                                diag_line_idx,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                            diag_line_idx += 1;

                            // Dialogue Elements Paragraphs
                            for diag_el in &dialogue.elements {
                                while diag_line_idx < span.end_line && self.raw_lines[diag_line_idx].trim().is_empty() {
                                    diag_line_idx += 1;
                                }
                                if diag_line_idx >= span.end_line {
                                    break;
                                }
                                
                                match diag_el {
                                    DialogueElement::Parenthetical(rich_string) => {
                                        write_paragraph_for_line(
                                            writer,
                                            "Parenthetical",
                                            "",
                                            &rich_string_to_fdx(rich_string),
                                            diag_line_idx,
                                            &self.raw_lines,
                                            &global_tags,
                                        )?;
                                    }
                                    DialogueElement::Line(rich_string) => {
                                        write_paragraph_for_line(
                                            writer,
                                            "Dialogue",
                                            "",
                                            &rich_string_to_fdx(rich_string),
                                            diag_line_idx,
                                            &self.raw_lines,
                                            &global_tags,
                                        )?;
                                    }
                                }
                                diag_line_idx += 1;
                            }
                        }
                        Element::DualDialogue(d1, d2) => {
                            let mut temp_span = span.clone();
                            temp_span.inner = Element::Dialogue(d1.clone());
                            self.export(&Screenplay::new(None, vec![temp_span.clone()]), writer)?;
                            temp_span.inner = Element::Dialogue(d2.clone());
                            self.export(&Screenplay::new(None, vec![temp_span]), writer)?;
                        }
                        Element::Transition(rich_string) => {
                            write_paragraph(
                                writer,
                                "Transition",
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::Shot(rich_string) => {
                            write_paragraph(
                                writer,
                                "Shot",
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::CenteredText(rich_string) => {
                            write_paragraph(
                                writer,
                                "Centered",
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::Synopsis(rich_string) if self.export_synopses => {
                            write_paragraph(
                                writer,
                                "Synopsis",
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::Section(rich_string) if self.export_sections => {
                            let hash_count = if span.start_line > 0 && span.start_line <= self.raw_lines.len() {
                                self.raw_lines[span.start_line - 1].chars().filter(|&c| c == '#').count()
                            } else {
                                1
                            };
                            let outline_type = match hash_count {
                                0 | 1 => "Outline 1",
                                2 => "Outline 2",
                                _ => "Outline 3",
                            };
                            write_paragraph(
                                writer,
                                outline_type,
                                "",
                                &rich_string_to_fdx(rich_string),
                                span.start_line..=span.end_line,
                                &self.raw_lines,
                                &global_tags,
                            )?;
                        }
                        Element::PageBreak => {
                            writeln!(writer, "    <Paragraph Type=\"New Page\"/>")?;
                        }
                        _ => {}
                    }
                    line_idx = span.end_line;
                    continue;
                }

            // Standalone line check (not covered by parsed AST spans)
            let line = &self.raw_lines[line_idx];
            let notes = parse_line_notes(line);
            if !notes.is_empty() {
                write!(writer, "    <Paragraph Type=\"Action\">")?;
                for n in notes {
                    let (color, label, prefix) = get_note_color_label_prefix(&n.key);
                    write!(
                        writer,
                        "<ScriptNote Label=\"{}\" Color=\"{}\" Author=\"Fount\"><Paragraph><Text>{} {}</Text></Paragraph></ScriptNote>",
                        label, color, prefix, escape_xml(&n.content)
                    )?;
                }
                writeln!(writer, "</Paragraph>")?;
            }

            line_idx += 1;
        }

        writeln!(writer, "  </Content>")?;

        // Title Page Section
        if let Some(tp) = screenplay.titlepage.as_ref().filter(|_| self.include_title_page) {
            writeln!(writer, "  <TitlePage>")?;
            write_title_paragraphs(writer, "Title", &tp.title)?;
            write_title_paragraphs(writer, "Credit", &tp.credit)?;
            write_title_paragraphs(writer, "Writer", &tp.authors)?;
            write_title_paragraphs(writer, "Source", &tp.source)?;
            write_title_paragraphs(writer, "Draft Date", &tp.draft_date)?;
            write_title_paragraphs(writer, "Contact", &tp.contact)?;
            write_title_paragraphs(writer, "Notes", &tp.notes)?;
            writeln!(writer, "  </TitlePage>")?;
        }

        // TagData Registry at the end of the file
        if !global_categories.is_empty() {
            writeln!(writer, "  <TagData>")?;
            writeln!(writer, "    <TagCategories>")?;
            for (cat_name, cat_id) in &global_categories {
                let name = map_category_name(cat_name);
                let color = map_category_color(cat_name);
                writeln!(writer, "      <TagCategory Id=\"{}\" Name=\"{}\" Color=\"{}\" Style=\"Bold\"/>", cat_id, name, color)?;
            }
            writeln!(writer, "    </TagCategories>")?;
            writeln!(writer, "    <Tags>")?;
            for ((cat_name, item_name), tag_id) in &global_tags {
                let cat_id = global_categories.get(cat_name).unwrap();
                writeln!(writer, "      <Tag Id=\"{}\" Name=\"{}\" CategoryId=\"{}\"/>", tag_id, escape_xml(item_name), cat_id)?;
            }
            writeln!(writer, "    </Tags>")?;
            writeln!(writer, "  </TagData>")?;
        }

        writeln!(writer, "</FinalDraft>")?;
        Ok(())
    }
}

// XML Helper utilities

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn make_xml_id(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn rich_string_to_fdx(rs: &RichString) -> String {
    let mut cloned = rs.clone();
    if let Some(first) = cloned.elements.first_mut() {
        first.text = first.text.trim_start().to_string();
    }
    if let Some(last) = cloned.elements.last_mut() {
        last.text = last.text.trim_end().to_string();
    }

    let mut out = String::new();
    for element in &cloned.elements {
        if element.text.is_empty() {
            continue;
        }
        let mut styles = Vec::new();
        if element.is_bold() {
            styles.push("Bold");
        }
        if element.is_italic() {
            styles.push("Italic");
        }
        if element.is_underline() {
            styles.push("Underline");
        }
        
        let style_attr = if styles.is_empty() {
            "".to_string()
        } else {
            format!(" Style=\"{}\"", styles.join("+"))
        };
        
        let escaped_text = escape_xml(&element.text);
        out.push_str(&format!("<Text{}>{}</Text>", style_attr, escaped_text));
    }
    out
}

struct TagInfo {
    category: String,
    item: String,
}

fn parse_line_tags(line: &str) -> Vec<TagInfo> {
    let mut tags = Vec::new();
    let mut start_search = 0;
    while let Some(start) = line[start_search..].find("[[") {
        let abs_start = start_search + start;
        if let Some(end) = line[abs_start..].find("]]") {
            let abs_end = abs_start + end;
            let content = &line[abs_start + 2..abs_end];
            if let Some((key, val)) = content.split_once(':') {
                let key = key.trim().to_lowercase();
                if key != "note" && key != "marker" && key != "subtext" && key != "tags" 
                   && key != "scenetype" && key != "scenestatus" && key != "sceneclr" && key != "synopsis" {
                    for v in val.split(',') {
                        let v_trimmed = v.trim();
                        if !v_trimmed.is_empty() {
                            tags.push(TagInfo {
                                category: key.clone(),
                                item: v_trimmed.to_string(),
                            });
                        }
                    }
                }
            }
            start_search = abs_end + 2;
        } else {
            break;
        }
    }
    tags
}

struct NoteInfo {
    key: String,
    content: String,
}

fn parse_line_notes(line: &str) -> Vec<NoteInfo> {
    let mut notes = Vec::new();
    let mut start_search = 0;
    while let Some(start) = line[start_search..].find("[[") {
        let abs_start = start_search + start;
        if let Some(end) = line[abs_start..].find("]]") {
            let abs_end = abs_start + end;
            let content = &line[abs_start + 2..abs_end];
            if let Some((key, val)) = content.split_once(':') {
                let key = key.trim().to_lowercase();
                if key == "note" || key == "marker" || key == "subtext" || key == "tags" {
                    notes.push(NoteInfo {
                        key,
                        content: val.trim().to_string(),
                    });
                }
            }
            start_search = abs_end + 2;
        } else {
            break;
        }
    }
    notes
}

fn map_category_name(cat: &str) -> &'static str {
    match cat {
        "cast" => "Cast Members",
        "props" => "Props",
        "wardrobe" => "Wardrobe",
        "makeup" => "Makeup/Hair",
        "sfx" => "Special Effects",
        "vfx" => "Visual Effects",
        "music" => "Music",
        "dnotes" => "Production Notes",
        "extras" => "Background Actors",
        "stunts" => "Stunts",
        "vehicles" => "Vehicles",
        "animals" => "Animals",
        "setdressing" => "Set Dressing",
        "sound" => "Sound",
        "equipment" => "Special Equipment",
        "security" => "Security",
        "greenery" => "Greenery",
        _ => "General Tag",
    }
}

fn map_category_color(cat: &str) -> &'static str {
    match cat {
        "cast" => "#4080FF",
        "props" => "#FF4040",
        "wardrobe" => "#40FF40",
        "makeup" => "#FF40FF",
        "sfx" => "#FFFF40",
        "vfx" => "#40FFFF",
        "music" => "#FF8000",
        "dnotes" => "#808080",
        "extras" => "#8040FF",
        "stunts" => "#FF4080",
        "vehicles" => "#0080FF",
        "animals" => "#FF8040",
        "setdressing" => "#80FF00",
        "sound" => "#00FF80",
        "equipment" => "#FF00FF",
        "security" => "#FF0000",
        "greenery" => "#00FF00",
        _ => "#C0C0C0",
    }
}

fn get_note_color_label_prefix(key: &str) -> (&'static str, &'static str, &'static str) {
    match key {
        "note" => ("#FFFF80", "Note", "[NOTE]"),
        "marker" => ("#80FF80", "Marker", "[MARKER]"),
        "subtext" => ("#8080FF", "Subtext", "[SUBTEXT]"),
        "tags" => ("#FF80FF", "General Tag", "[TAG]"),
        _ => ("#C0C0C0", "Note", ""),
    }
}

fn write_paragraph(
    writer: &mut dyn Write,
    p_type: &str,
    extra_attrs: &str,
    text_xml: &str,
    lines_range: std::ops::RangeInclusive<usize>,
    raw_lines: &[String],
    global_tags: &BTreeMap<(String, String), String>,
) -> std::io::Result<()> {
    write!(writer, "    <Paragraph Type=\"{}\"{}>", p_type, extra_attrs)?;
    write!(writer, "{}", text_xml)?;

    // Scan the lines of this paragraph for tags & ScriptNotes
    let mut tags_accum = Vec::new();
    let mut notes_accum = Vec::new();
    for line_num in lines_range {
        if line_num > 0 && line_num <= raw_lines.len() {
            let line = &raw_lines[line_num - 1];
            tags_accum.extend(parse_line_tags(line));
            notes_accum.extend(parse_line_notes(line));
        }
    }

    // Write tags
    if !tags_accum.is_empty() {
        write!(writer, "<Tags>")?;
        for t in tags_accum {
            if let Some(tag_id) = global_tags.get(&(t.category, t.item)) {
                write!(writer, "<Tag Id=\"{}\"/>", tag_id)?;
            }
        }
        write!(writer, "</Tags>")?;
    }

    // Write ScriptNotes
    for n in notes_accum {
        let (color, label, prefix) = get_note_color_label_prefix(&n.key);
        write!(
            writer,
            "<ScriptNote Label=\"{}\" Color=\"{}\" Author=\"Fount\"><Paragraph><Text>{} {}</Text></Paragraph></ScriptNote>",
            label, color, prefix, escape_xml(&n.content)
        )?;
    }

    writeln!(writer, "</Paragraph>")?;
    Ok(())
}

fn write_paragraph_for_line(
    writer: &mut dyn Write,
    p_type: &str,
    extra_attrs: &str,
    text_xml: &str,
    line_idx: usize,
    raw_lines: &[String],
    global_tags: &BTreeMap<(String, String), String>,
) -> std::io::Result<()> {
    let range = (line_idx + 1)..=(line_idx + 1);
    write_paragraph(writer, p_type, extra_attrs, text_xml, range, raw_lines, global_tags)
}

fn write_title_paragraphs(
    writer: &mut dyn Write,
    p_type: &str,
    items: &[RichString],
) -> std::io::Result<()> {
    for item in items {
        writeln!(
            writer,
            "    <Paragraph Type=\"{}\">{}</Paragraph>",
            p_type,
            rich_string_to_fdx(item)
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::parse;

    #[test]
    fn test_fdx_exporter() {
        let script = r#"Title: Sample Script
Author: John Doe

# Section 1
## Section 1.1

EXT. PARK - DAY

= Park scene description.

JOHN
(nervous)
Hello there! [[props: Gun]]

[[note: check the dialogue style]]
[[marker: review this part]]
[[subtext: this is subtext]]
[[tags: general info]]
"#;

        let screenplay = parse(script);
        let raw_lines = script.lines().map(|s| s.to_string()).collect::<Vec<_>>();
        let exporter = FdxExporter::new(raw_lines);
        let mut buf = Vec::new();
        exporter.export(&screenplay, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();

        // Verify XML header and doctype
        assert!(output.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>"));
        assert!(output.contains("<FinalDraft DocumentType=\"Script\" Version=\"4\">"));

        // Verify Title Page
        assert!(output.contains("<Paragraph Type=\"Title\"><Text>Sample Script</Text></Paragraph>"));
        assert!(output.contains("<Paragraph Type=\"Writer\"><Text>John Doe</Text></Paragraph>"));

        // Verify Outlines
        assert!(output.contains("<Paragraph Type=\"Outline 1\"><Text>Section 1</Text></Paragraph>"));
        assert!(output.contains("<Paragraph Type=\"Outline 2\"><Text>Section 1.1</Text></Paragraph>"));

        // Verify Heading
        assert!(output.contains("<Paragraph Type=\"Scene Heading\"><Text>EXT. PARK - DAY</Text>"));

        // Verify Synopsis
        assert!(output.contains("<Paragraph Type=\"Synopsis\"><Text>Park scene description.</Text></Paragraph>"));

        // Verify Dialogue & Parenthetical
        assert!(output.contains("<Paragraph Type=\"Character\"><Text>JOHN</Text></Paragraph>"));
        assert!(output.contains("<Paragraph Type=\"Parenthetical\"><Text>(nervous)</Text></Paragraph>"));
        assert!(output.contains("<Paragraph Type=\"Dialogue\"><Text>Hello there!</Text>"));

        // Verify tags are mapped to TagData and referenced in Dialogue
        assert!(output.contains("<Tag Id=\"TAG_PROPS_GUN\" Name=\"Gun\" CategoryId=\"CAT_PROPS\"/>"));
        assert!(output.contains("<TagCategory Id=\"CAT_PROPS\" Name=\"Props\" Color=\"#FF4040\" Style=\"Bold\"/>"));
        assert!(output.contains("<Tag Id=\"TAG_PROPS_GUN\"/>")); // reference inside paragraph

        // Verify ScriptNotes are differentiated with color and label
        assert!(output.contains("<ScriptNote Label=\"Note\" Color=\"#FFFF80\" Author=\"Fount\"><Paragraph><Text>[NOTE] check the dialogue style</Text></Paragraph></ScriptNote>"));
        assert!(output.contains("<ScriptNote Label=\"Marker\" Color=\"#80FF80\" Author=\"Fount\"><Paragraph><Text>[MARKER] review this part</Text></Paragraph></ScriptNote>"));
        assert!(output.contains("<ScriptNote Label=\"Subtext\" Color=\"#8080FF\" Author=\"Fount\"><Paragraph><Text>[SUBTEXT] this is subtext</Text></Paragraph></ScriptNote>"));
        assert!(output.contains("<ScriptNote Label=\"General Tag\" Color=\"#FF80FF\" Author=\"Fount\"><Paragraph><Text>[TAG] general info</Text></Paragraph></ScriptNote>"));
    }

    #[test]
    fn test_fdx_exporter_toggles() {
        let script = r#"Title: Sample Script
Author: John Doe

# Section 1
## Section 1.1

EXT. PARK - DAY

= Park scene description.

JOHN
Hello there! [[props: Gun]]
"#;

        let screenplay = parse(script);
        let raw_lines = script.lines().map(|s| s.to_string()).collect::<Vec<_>>();
        
        let exporter = FdxExporter {
            raw_lines,
            include_title_page: false,
            export_sections: false,
            export_synopses: false,
            export_production_tags: false,
        };
        let mut buf = Vec::new();
        exporter.export(&screenplay, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // 1. Should not contain Title Page
        assert!(!output.contains("<TitlePage>"));
        assert!(!output.contains("Sample Script"));

        // 2. Should not contain Outlines
        assert!(!output.contains("Outline 1"));
        assert!(!output.contains("Outline 2"));

        // 3. Should not contain Synopsis
        assert!(!output.contains("Synopsis"));

        // 4. Should not contain TagData registry or tag references
        assert!(!output.contains("<TagData>"));
        assert!(!output.contains("TAG_PROPS_GUN"));
    }
}
