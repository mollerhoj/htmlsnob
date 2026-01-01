// Run with `cargo run --bin attribute_name_whitelist_from_htmlspec`

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;

#[derive(Debug, Deserialize)]
struct HtmlElement {
    attributes: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    children: Vec<String>,
    #[serde(default)]
    desc: String,
}

#[derive(Debug, Deserialize)]
struct SourceData {
    #[serde(rename = "__META__")]
    meta: serde_json::Value,
    #[serde(flatten)]
    tags: HashMap<String, HtmlElement>,
}

#[derive(Debug, Serialize)]
struct Rules {
    rules: Vec<RuleEntry>,
}

#[derive(Debug, Serialize)]
struct RuleEntry {
    kind: String,
    globals: Vec<String>,
    tags: BTreeMap<String, Vec<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the JSON file
    let json_content = fs::read_to_string("htmlsnob_scripts/data/elements.json")?;
    let data: SourceData = serde_json::from_str(&json_content)?;

    // 2. List Global Attributes
    let global_attributes = vec![
        "accesskey",
        "anchor",
        "autocapitalize",
        "autocorrect",
        "autofocus",
        "class",
        "contenteditable",
        "data-*",
        "dir",
        "draggable",
        "enterkeyhint",
        "exportparts",
        "hidden",
        "id",
        "inert",
        "inputmode",
        "is",
        "itemid",
        "itemprop",
        "itemref",
        "itemscope",
        "itemtype",
        "lang",
        "nonce",
        "part",
        "popover",
        "role",
        "slot",
        "spellcheck",
        "style",
        "tabindex",
        "title",
        "translate",
        "virtualkeyboardpolicy",
        "writingsuggestions",
        "onabort",
        "onauxclick",
        "onbeforeinput",
        "onbeforematch",
        "onbeforetoggle",
        "oncancel",
        "oncanplay",
        "oncanplaythrough",
        "onchange",
        "onclick",
        "onclose",
        "oncommand",
        "oncontextlost",
        "oncontextmenu",
        "oncontextrestored",
        "oncopy",
        "oncuechange",
        "oncut",
        "ondblclick",
        "ondrag",
        "ondragend",
        "ondragenter",
        "ondragleave",
        "ondragover",
        "ondragstart",
        "ondrop",
        "ondurationchange",
        "onemptied",
        "onended",
        "onformdata",
        "oninput",
        "oninvalid",
        "onkeydown",
        "onkeypress",
        "onkeyup",
        "onloadeddata",
        "onloadedmetadata",
        "onloadstart",
        "onmousedown",
        "onmouseenter",
        "onmouseleave",
        "onmousemove",
        "onmouseout",
        "onmouseover",
        "onmouseup",
        "onpaste",
        "onpause",
        "onplay",
        "onplaying",
        "onprogress",
        "onratechange",
        "onreset",
        "onscrollend",
        "onsecuritypolicyviolation",
        "onseeked",
        "onseeking",
        "onselect",
        "onslotchange",
        "onstalled",
        "onsubmit",
        "onsuspend",
        "ontimeupdate",
        "ontoggle",
        "onvolumechange",
        "onwaiting",
        "onwebkitanimationend",
        "onwebkitanimationiteration",
        "onwebkitanimationstart",
        "onwebkittransitionend",
        "onwheel",
    ];

    // 3. Transform data
    let mut tag_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (tag_name, element) in data.tags {
        // Filter out global attributes from the tag-specific list to keep TOML clean
        let mut specific_attrs: Vec<String> = element
            .attributes
            .into_iter()
            .filter(|attr| !global_attributes.contains(&attr.as_str()))
            .collect();

        specific_attrs.sort();

        tag_map.insert(tag_name, specific_attrs);
    }

    let output = Rules {
        rules: vec![RuleEntry {
            kind: "attribute_name_whitelist".to_string(),
            globals: global_attributes.into_iter().map(String::from).collect(),
            tags: tag_map,
        }],
    };

    // 4. Serialize to TOML string
    let toml_string = toml::to_string_pretty(&output)?;
    println!("{}", toml_string);

    // 5. write to a file
    fs::write(
        "htmlsnob_scripts/output/attribute_name_whitelist_from_htmlspec.toml",
        toml_string,
    )?;

    Ok(())
}
