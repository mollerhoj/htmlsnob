use htmlsnob::ast::{CloseTag, Node, OpenTag};
use htmlsnob::dynamic_format::dynamic_format;
use htmlsnob::parser::ParseState;
use htmlsnob::rule_trait::RuleTrait;
use htmlsnob::warning::Warning;
use htmlsnob::WarningSeverity;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

// The following tags have a content model with special cases, and are ignored by this rule:
// `audio`, `button`, `colgroup`, `datalist`, `details`, `div`, `dl`, `fieldset`, `figure`,
// `head`, `hgroup`, `html`, `legend`, `optgroup`, `option`, `picture`, `ruby`, `select`,
// `span`, `table`, `template`, `time`, `video`,

/// Enforces that a tag is allowed within its ancestor according to content model categories.
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    name: String,
    kind: String,
    #[serde(default)]
    severity: WarningSeverity,
    #[serde(default = "default_error_message")]
    pub error_message: String,
    /// A map of tag names to their allowed content model categories and whitelists.
    tags: HashMap<String, TagContentModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagContentModel {
    categories: HashSet<String>,
    #[serde(default)]
    is_special_case: bool,
    #[serde(default)]
    is_transparent: bool,
    #[serde(default)]
    child_category_whitelist: HashSet<String>,
    #[serde(default)]
    child_tag_whitelist: HashSet<String>,
}

fn default_error_message() -> String {
    "Tag `{tag}` is not allowed within `{ancestor}`".to_string()
}

impl RuleTrait for Rule {
    fn apply_tag(
        &self,
        open_tag: Option<&OpenTag>,
        close_tag: Option<&CloseTag>,
        parse_state: &ParseState,
    ) -> Option<Warning> {
        let tag_name = match (open_tag, close_tag) {
            (Some(open_tag), Some(_)) => open_tag.name.as_str(),
            (Some(open_tag), None) => open_tag.name.as_str(),
            (None, Some(close_tag)) => close_tag.name.as_str(),
            _ => panic!("Expected either an open tag or a close tag"),
        };
        let open_tag_names = parse_state.open_tag_names();

        // TODO: Handle "TRANSPARENT" category properly:
        // If parent_tag.child_category_whitelist is "TRANSPARENT", then we need to look at the
        // grandparent tag. (Iterate up the stack until we find a non-TRANSPARENT category
        // whitelist.)

        let mut index = open_tag_names.len();
        let mut parent_tag_name;
        loop {
            if index == 0 {
                return None;
            }
            parent_tag_name = &open_tag_names[index - 1];
            let is_transparent = self.tags.get(parent_tag_name)?.is_transparent;
            if !is_transparent {
                break;
            }
            index -= 1;
        }

        // Skip if parent_tag is a special case
        if self.tags.get(parent_tag_name)?.is_special_case {
            return None;
        }

        let categories: &HashSet<String> = &self.tags.get(tag_name)?.categories;
        let category_whitelist: &HashSet<String> =
            &self.tags.get(parent_tag_name)?.child_category_whitelist;
        let child_tag_whitelist: &HashSet<String> =
            &self.tags.get(parent_tag_name)?.child_tag_whitelist;

        // Check that `category_whitelist` does not contain any item from `categories`
        if categories.is_disjoint(category_whitelist) && !child_tag_whitelist.contains(tag_name) {
            let message = dynamic_format(
                &self.error_message,
                &[
                    ("tag", tag_name.to_string()),
                    ("ancestor", parent_tag_name.to_string()),
                ],
            );

            let mut areas = Vec::new();
            if let Some(open_tag) = open_tag {
                areas.push(open_tag.area.clone());
            }
            if let Some(close_tag) = close_tag {
                areas.push(close_tag.area.clone());
            }

            return Some(Warning::from_areas(
                &self.name,
                &self.kind,
                &areas,
                &message,
                self.severity.clone(),
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::registry;
    use htmlsnob::test_utils::tests::test_case;

    const CONFIG: &str = r#"
        [[rules]]
        kind = "content_model_categories"
        [rules.tags]
        a.categories = ["flow", "phrasing", ]
        a.is_transparent = true
        abbr.categories = ["flow", "phrasing", ]
        abbr.child_category_whitelist = ["phrasing", ]
        address.categories = ["flow", ]
        address.child_category_whitelist = ["flow", ]
        area.categories = ["flow", "phrasing", ]
        area.child_category_whitelist = []
        article.categories = ["flow", "sectioning", ]
        article.child_category_whitelist = ["flow", "sectioning", ]
        aside.categories = ["flow", "sectioning", ]
        aside.child_category_whitelist = ["flow", ]
        audio.categories = ["flow", "phrasing", ]
        audio.is_special_case = true
        b.categories = ["flow", "phrasing", ]
        b.child_category_whitelist = ["phrasing", ]
        base.categories = ["metadata", ]
        base.child_category_whitelist = []
        bdi.categories = ["flow", "phrasing", ]
        bdi.child_category_whitelist = ["phrasing", ]
        bdo.categories = ["flow", "phrasing", ]
        bdo.child_category_whitelist = ["phrasing", ]
        blockquote.categories = ["flow", ]
        blockquote.child_category_whitelist = ["flow", ]
        body.categories = []
        body.child_category_whitelist = ["flow", ]
        br.categories = ["flow", "phrasing", ]
        br.child_category_whitelist = []
        button.categories = ["flow", "phrasing", "interactive", ]
        button.is_special_case = true
        canvas.categories = ["flow", "phrasing", ]
        canvas.is_transparent = true
        canvas.child_tag_whitelist = ["a", "img", "button", "input", "select", ]
        caption.categories = []
        caption.child_category_whitelist = ["flow", ]
        cite.categories = ["flow", "phrasing", ]
        cite.child_category_whitelist = ["phrasing", ]
        code.categories = ["flow", "phrasing", ]
        code.child_category_whitelist = ["phrasing", ]
        col.categories = []
        col.child_category_whitelist = []
        colgroup.categories = []
        colgroup.is_special_case = true
        data.categories = ["flow", "phrasing", ]
        data.child_category_whitelist = ["phrasing", ]
        datalist.categories = ["flow", "phrasing", ]
        datalist.is_special_case = true
        dd.categories = []
        dd.child_category_whitelist = ["flow", ]
        del.categories = []
        del.child_category_whitelist = ["flow", ]
        details.categories = ["flow", "interactive", ]
        details.is_special_case = true
        dfn.categories = ["flow", "phrasing", ]
        dfn.child_category_whitelist = ["phrasing", ]
        dialog.categories = ["flow", ]
        dialog.child_category_whitelist = ["flow", ]
        div.categories = ["flow", ]
        div.is_special_case = true
        dl.categories = ["flow", ]
        dl.is_special_case = true
        dt.categories = []
        dt.child_category_whitelist = ["flow", ]
        em.categories = ["flow", "phrasing", ]
        em.child_category_whitelist = ["phrasing", ]
        embed.categories = ["flow", "phrasing", "interactive", ]
        embed.child_category_whitelist = []
        fencedframe.categories = ["flow", "phrasing", "interactive", ]
        fencedframe.child_category_whitelist = []
        fieldset.categories = ["flow", ]
        fieldset.is_special_case = true
        figcaption.categories = []
        figcaption.child_category_whitelist = ["flow", ]
        figure.categories = ["flow", ]
        figure.is_special_case = true
        footer.categories = ["flow", ]
        footer.child_category_whitelist = ["flow", ]
        form.categories = ["flow", ]
        form.child_category_whitelist = ["flow", ]
        h1.categories = ["flow", "heading", ]
        h1.child_category_whitelist = ["phrasing", ]
        h2.categories = ["flow", "heading", ]
        h2.child_category_whitelist = ["phrasing", ]
        h3.categories = ["flow", "heading", ]
        h3.child_category_whitelist = ["phrasing", ]
        h4.categories = ["flow", "heading", ]
        h4.child_category_whitelist = ["phrasing", ]
        h5.categories = ["flow", "heading", ]
        h5.child_category_whitelist = ["phrasing", ]
        h6.categories = ["flow", "heading", ]
        h6.child_category_whitelist = ["phrasing", ]
        head.categories = []
        head.is_special_case = true
        header.categories = ["flow", ]
        header.child_category_whitelist = ["flow", ]
        hgroup.categories = ["flow", "heading", ]
        hgroup.is_special_case = true
        hr.categories = ["flow", ]
        hr.child_category_whitelist = []
        html.categories = []
        html.is_special_case = true
        i.categories = ["flow", "phrasing", ]
        i.child_category_whitelist = ["phrasing", ]
        iframe.categories = ["flow", "phrasing", "interactive", ]
        iframe.child_category_whitelist = []
        img.categories = ["flow", "phrasing", ]
        img.child_category_whitelist = []
        input.categories = ["flow", "phrasing", ]
        input.child_category_whitelist = []
        ins.categories = []
        ins.child_category_whitelist = ["flow", ]
        kbd.categories = ["flow", "phrasing", ]
        kbd.child_category_whitelist = ["phrasing", ]
        label.categories = ["flow", "phrasing", "interactive", ]
        label.child_category_whitelist = ["phrasing", ]
        legend.categories = []
        legend.is_special_case = true
        li.categories = []
        li.child_category_whitelist = ["flow", ]
        link.categories = ["metadata", ]
        link.child_category_whitelist = []
        main.categories = ["flow", ]
        main.child_category_whitelist = ["flow", ]
        map.categories = ["flow", "phrasing", ]
        map.is_transparent = true
        math.categories = ["flow", ]
        math.child_category_whitelist = []
        mark.categories = ["flow", "phrasing", ]
        mark.child_category_whitelist = ["phrasing", ]
        menu.categories = ["flow", ]
        menu.child_category_whitelist = []
        menu.child_tag_whitelist = ["li", "script", "template", ]
        meta.categories = ["metadata", ]
        meta.child_category_whitelist = []
        meter.categories = ["flow", "phrasing", ]
        meter.child_category_whitelist = ["phrasing", ]
        nav.categories = ["flow", "sectioning", ]
        nav.child_category_whitelist = ["flow", ]
        noscript.categories = ["metadata", "flow", "phrasing", ]
        noscript.is_transparent = true
        object.categories = ["flow", "phrasing", ]
        object.is_transparent = true
        ol.categories = ["flow", ]
        ol.child_category_whitelist = []
        ol.child_tag_whitelist = ["li", "script", "template", ]
        optgroup.categories = ["flow", "phrasing", ]
        optgroup.is_special_case = true
        option.categories = []
        option.is_special_case = true
        output.categories = ["flow", "phrasing", ]
        output.child_category_whitelist = ["phrasing", ]
        p.categories = ["flow", ]
        p.child_category_whitelist = ["phrasing", ]
        picture.categories = ["flow", "phrasing", ]
        picture.is_special_case = true
        pre.categories = ["flow", ]
        pre.child_category_whitelist = ["phrasing", ]
        progress.categories = ["flow", "phrasing", ]
        progress.child_category_whitelist = ["phrasing", ]
        q.categories = ["flow", "phrasing", ]
        q.child_category_whitelist = ["phrasing", ]
        rp.categories = []
        rp.child_category_whitelist = []
        rp.child_tag_whitelist = ["text", ]
        rt.categories = []
        rt.child_category_whitelist = ["phrasing", ]
        ruby.categories = ["flow", "phrasing", ]
        ruby.is_special_case = true
        s.categories = ["flow", "phrasing", ]
        s.child_category_whitelist = ["phrasing", ]
        samp.categories = ["flow", "phrasing", ]
        samp.child_category_whitelist = ["phrasing", ]
        script.categories = ["metadata", "flow", "phrasing", ]
        script.child_category_whitelist = ["text", ]
        search.categories = ["flow", ]
        search.child_category_whitelist = ["flow", ]
        section.categories = ["flow", ]
        section.child_category_whitelist = ["flow", ]
        select.categories = ["flow", "phrasing", "interactive", ]
        select.is_special_case = true
        selectedcontent.categories = []
        selectedcontent.child_category_whitelist = []
        slot.categories = ["flow", "phrasing", ]
        slot.is_transparent = true
        small.categories = ["flow", "phrasing", ]
        small.child_category_whitelist = ["phrasing", ]
        source.categories = []
        source.child_category_whitelist = []
        span.categories = ["flow", "phrasing", ]
        span.is_special_case = true
        strong.categories = ["flow", "phrasing", ]
        strong.child_category_whitelist = ["phrasing", ]
        style.categories = ["metadata", ]
        style.child_category_whitelist = ["text", ]
        sub.categories = ["flow", "phrasing", ]
        sub.child_category_whitelist = ["phrasing", ]
        summary.categories = []
        summary.child_category_whitelist = ["phrasing", ]
        summary.child_tag_whitelist = ["h1", "h2", "h3", "h4", "h5", "h6", "hgroup", ]
        sup.categories = ["flow", "phrasing", ]
        sup.child_category_whitelist = ["phrasing", ]
        svg.categories = ["flow", ]
        svg.child_category_whitelist = []
        table.categories = ["flow", ]
        table.is_special_case = true
        tbody.categories = []
        tbody.child_category_whitelist = []
        tbody.child_tag_whitelist = ["tr", "script", "template", ]
        td.categories = []
        td.child_category_whitelist = ["flow", ]
        template.categories = ["metadata", "flow", "phrasing", ]
        template.is_special_case = true
        textarea.categories = ["flow", "phrasing", "interactive", ]
        textarea.child_category_whitelist = ["text", ]
        tfoot.categories = []
        tfoot.child_category_whitelist = []
        tfoot.child_tag_whitelist = ["tr", "script", "template", ]
        th.categories = []
        th.child_category_whitelist = ["flow", ]
        thead.categories = []
        thead.child_category_whitelist = []
        thead.child_tag_whitelist = ["tr", "script", "template", ]
        time.categories = ["flow", "phrasing", ]
        time.is_special_case = true
        title.categories = ["metadata", ]
        title.child_category_whitelist = ["text", ]
        tr.categories = []
        tr.child_category_whitelist = []
        tr.child_tag_whitelist = ["td", "th", "script", "template", ]
        track.categories = []
        track.child_category_whitelist = []
        track.child_tag_whitelist = ["td", "th", "script", "template", ]
        u.categories = ["flow", "phrasing", ]
        u.child_category_whitelist = ["phrasing", ]
        ul.categories = ["flow", ]
        ul.child_category_whitelist = []
        ul.child_tag_whitelist = ["li", "script", "template", ]
        var.categories = ["flow", "phrasing", ]
        var.child_category_whitelist = ["phrasing", ]
        video.categories = ["flow", "phrasing", ]
        video.is_special_case = true
        wbr.categories = ["flow", "phrasing", ]
        wbr.child_category_whitelist = []
    "#;

    #[test]
    fn good_case() {
        test_case("<abbr><a></a></abbr>", CONFIG, &registry())
    }

    #[test]
    fn good_case_child_tag_whitelist() {
        test_case("<ul><li>Hello</li></ul>", CONFIG, &registry())
    }

    #[test]
    fn good_case_special_case() {
        test_case("<table><li></li></table>", CONFIG, &registry())
    }

    #[test]
    fn bad_case_category() {
        test_case(
            r#"
               <abbr><address>Hello</address></abbr>
                     ---------     ----------
                     content_model_categories: Tag `address` is not allowed within `abbr`
            "#,
            CONFIG,
            &registry(),
        )
    }

    #[test]
    fn bad_case_transparent() {
        test_case(
            r#"
               <abbr><a><address>Hello</address></a></abbr>
                        ---------     ----------
                        content_model_categories: Tag `address` is not allowed within `abbr`
            "#,
            CONFIG,
            &registry(),
        )
    }
}
