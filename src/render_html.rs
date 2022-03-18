/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::path::Path;

use syntect::highlighting::{Color, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub trait GenerateHTML {
    fn generate(&mut self);
}

#[allow(dead_code)]
pub const STYLE: &str = "
    ";

thread_local! {
    pub(crate) static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
}

pub struct SourcegraphQuery<'a> {
    pub filepath: &'a str,
    pub code: &'a str,
}

impl<'a> SourcegraphQuery<'a> {
    pub fn syntax_highlight(&self) -> String {
        //    let ss = SYNTAX_SET;
        let ts = ThemeSet::load_defaults();

        let theme = &ts.themes["InspiredGitHub"];
        let c = theme.settings.background.unwrap_or(Color::WHITE);
        let mut num = 1;
        let mut output = format!(
            "<style>
        .gist_file {{
            background-color:#{:02x}{:02x}{:02x};
        }}</style>",
            c.r, c.g, c.b
        );

        // highlighted_html_for_string(&q.code, syntax_set, syntax_def, theme),
        let html = SYNTAX_SET.with(|ss| {
            let language = self.determine_language(ss);
            highlighted_html_for_string(self.code, ss, &language, theme)
        });
        for (line_num, line) in html.lines().enumerate() {
            if !line.trim().is_empty() {
                if line_num == 0 {
                    //|| line_num == total_lines - 1 {
                    output.push_str(line);
                } else {
                    output.push_str(&format!("<div id=\"line-{num}\"class=\"line\"><a href=\"#line-{num}\"<span class=\"line-number\">{num}</span></a>{line}</div>"
                    ));
                    num += 1;
                }
            }
        }
        output
    }

    // adopted from
    // https://github.com/sourcegraph/sourcegraph/blob/9fe138ae75fd64dce06b621572b252a9c9c8da70/docker-images/syntax-highlighter/crates/sg-syntax/src/lib.rs#L81
    // with minimum modifications. Crate was MIT licensed at the time(2022-03-12 11:11)
    fn determine_language(&self, syntax_set: &SyntaxSet) -> SyntaxReference {
        if self.filepath.is_empty() {
            // Legacy codepath, kept for backwards-compatability with old clients.
            match syntax_set.find_syntax_by_first_line(self.code) {
                Some(v) => {
                    return v.to_owned();
                }
                None => unimplemented!(), //Err(json!({"error": "invalid extension"})),
            };
        }

        // Split the input path ("foo/myfile.go") into file name
        // ("myfile.go") and extension ("go").
        let path = Path::new(&self.filepath);
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|x| x.to_str()).unwrap_or("");

        // Override syntect's language detection for conflicting file extensions because
        // it's impossible to express this logic in a syntax definition.
        struct Override {
            extension: &'static str,
            prefix_langs: Vec<(&'static str, &'static str)>,
            default: &'static str,
        }
        let overrides = vec![Override {
            extension: "cls",
            prefix_langs: vec![("%", "TeX"), ("\\", "TeX")],
            default: "Apex",
        }];

        if let Some(Override {
            prefix_langs,
            default,
            ..
        }) = overrides.iter().find(|o| o.extension == extension)
        {
            let name = match prefix_langs
                .iter()
                .find(|(prefix, _)| self.code.starts_with(prefix))
            {
                Some((_, lang)) => lang,
                None => default,
            };
            return syntax_set
                .find_syntax_by_name(name)
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
                .to_owned();
        }

        syntax_set
            // First try to find a syntax whose "extension" matches our file
            // name. This is done due to some syntaxes matching an "extension"
            // that is actually a whole file name (e.g. "Dockerfile" or "CMakeLists.txt")
            // see https://github.com/trishume/syntect/pull/170
            .find_syntax_by_extension(file_name)
            .or_else(|| syntax_set.find_syntax_by_extension(extension))
            .or_else(|| syntax_set.find_syntax_by_first_line(self.code))
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::SourcegraphQuery;

    use syntect::parsing::SyntaxSet;

    #[test]
    fn cls_tex() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let query = SourcegraphQuery {
            filepath: "foo.cls",
            code: "%",
        };
        let result = query.determine_language(&syntax_set);
        assert_eq!(result.name, "TeX");
        let _result = query.syntax_highlight();
    }

    //#[test]
    //fn cls_apex() {
    //    let syntax_set = SyntaxSet::load_defaults_newlines();
    //    let query = SourcegraphQuery {
    //        filepath: "foo.cls".to_string(),
    //        code: "/**".to_string(),
    //        extension: String::new(),
    //    };
    //    let result = determine_language(&query, &syntax_set);
    //    assert_eq!(result.unwrap().name, "Apex");
    //}
}
