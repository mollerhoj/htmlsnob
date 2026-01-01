use std::io::{self, BufRead};

pub struct TestBlockReader<R: BufRead> {
    reader: io::Lines<R>,
    buffer: String,
}

impl<R: BufRead> Iterator for TestBlockReader<R> {
    type Item = (String, String, String, Vec<(usize, usize)>); // (input, expected, name)

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        let mut found_test_start = false;
        let mut ranges = Vec::new();
        let mut offset = 0;

        // Read until we find a complete test block
        while let Ok(Some(line)) = self.reader.next().transpose() {
            // If line contains only whitespace and dashes, it's an error indicator
            if line.chars().all(|c| c == ' ' || c == '-') {
                ranges = Self::extract_ranges(line, offset);
                continue;
            }

            // Check for test block start
            if !found_test_start && line.contains("<test") {
                found_test_start = true;
            }

            if found_test_start {
                offset = line.chars().take_while(|&c| c == ' ').count();
                self.buffer.push_str(&line);
                self.buffer.push('\n');

                // Check if we have a complete test block
                if line.contains("</test>") {
                    break;
                }
            }
        }

        // If we didn't find a test block, we're done
        if !found_test_start || self.buffer.is_empty() {
            return None;
        }

        // Extract the test name
        let name = self
            .extract_test_name()
            .unwrap_or("unnamed test".to_string());

        // Extract the input and expected values, return if both are found
        let input = self.extract_tag_content("example");
        let expected = self.extract_tag_content("expected");

        if let (Some(input_val), Some(expected_val)) = (input, expected) {
            return Some((name, input_val, expected_val, ranges));
        }

        // Otherwise, extract the test block and return it twice
        let test = self
            .extract_tag_content("test")
            .expect("Failed to extract test block");

        Some((name, test.clone(), test.clone(), ranges))
    }
}

// add  + std::fmt::Debug
impl<R: BufRead> TestBlockReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: reader.lines(),
            buffer: String::new(),
        }
    }

    fn extract_tag_content(&self, tag_name: &str) -> Option<String> {
        let opening_tag_start = self.buffer.find(&format!("<{}", tag_name))?;
        let opening_tag_end = opening_tag_start + self.buffer[opening_tag_start..].find('>')?;
        let tag_start = opening_tag_end + 2;
        let tag_end = self.buffer[tag_start..].find(&format!("</{}>", tag_name))?;
        let tag_content = &self.buffer[tag_start..tag_start + tag_end];

        let indentation = self.buffer[..opening_tag_start]
            .chars()
            .rev()
            .take_while(|&c| c == ' ')
            .count()
            + 2;

        let content = tag_content
            .lines()
            .map(|line| line.chars().skip(indentation).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string();

        Some(content + "\n")
    }

    fn extract_test_name(&self) -> Option<String> {
        // Find the opening test tag
        let test_tag_start = self.buffer.find("<test")?;
        let tag_end = self.buffer[test_tag_start..].find(">")?;
        let test_tag = &self.buffer[test_tag_start..test_tag_start + tag_end];

        // Find the name attribute
        let name_attr = test_tag.find("name=\"")?;
        let name_start = name_attr + "name=\"".len();
        let name_end = test_tag[name_start..].find("\"")?;

        Some(test_tag[name_start..name_start + name_end].to_string())
    }

    fn extract_ranges(line: String, offset: usize) -> Vec<(usize, usize)> {
        // Find the ranges of consecutive '-' characters
        let mut ranges = Vec::new();
        let mut start_index = None;

        for (i, char) in line.char_indices() {
            if char == '-' {
                if start_index.is_none() {
                    start_index = Some(i);
                }
            } else if let Some(start) = start_index {
                ranges.push((start - offset, i - offset));
                start_index = None;
            }
        }

        // Handle case where '-' is at the end of the line
        if let Some(start) = start_index {
            ranges.push((start - offset, line.len() - offset));
        }

        ranges
    }
}
