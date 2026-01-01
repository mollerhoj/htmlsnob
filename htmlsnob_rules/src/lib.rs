use htmlsnob::registry::Registry;

pub mod attributes {
    /// Enforces that an element does not have any of the specified attributes.
    pub mod attribute_name_blacklist;
    /// Enforces that all attribute names match a specified casing style.
    pub mod attribute_name_casing_style;
    /// Enforces that no attributes names are empty. (e.g. <div ="foo">)
    pub mod attribute_name_missing;
    /// Enforces that all attribute names match a specified regular expression.
    pub mod attribute_name_regexp;
    /// Enforces that an element has all of the specified attributes.
    pub mod attribute_name_requirement;
    /// Enforces that an element only has the specified attributes.
    pub mod attribute_name_whitelist;
    /// Enforces that all attribute values match a specified casing style.
    pub mod attribute_value_casing_style;
    /// Enforces that all attribute values use the same quote style.
    pub mod attribute_value_quote_style;
    /// Enforces that all attribute values match a specified regular expression.
    pub mod attribute_value_regexp;
    /// Enforces that an elements specfied attribute has one of the specified values.
    pub mod attribute_value_whitelist;
    /// Enforces that all attributes are in a specified order.
    pub mod attributes_order;
    /// Enforces that boolean attributes use a specified style.
    pub mod boolean_attribute_style;
    /// Enforces that no attributes appear more than once in the same element.
    pub mod duplicate_attribute_names_disallowed;

    // Idea: attributes should have a "data type": string, boolean, number, url, enum, etc.
    // Potentially parse webref IDL files for this info: https://github.com/w3c/webref/blob/64eef22d0a356a378344a340f2ff7d965512349c/ed/idl/html.idl
    //
    // Potential new rules:
    // - attribute_value_blacklist
    // - boolean_attribute_style (e.g. checked vs checked="checked" vs checked="") [none, empty_string, same_as_name]
    // - attribute_value_missing (attribute must have value) -> class, id should have values. Also,
    // don't allow them just be empty strings e.g. <div class="" id="">
    // Perhaps only some attributes should be allowed to be valueless (e.g. disabled, checked,
    // readonly, etc.) aka. boolean attributes. (See DOM interface here:
    // http://html.spec.whatwg.org/multipage/forms.html#the-form-element)
}

pub mod class_and_id {
    /// Enforces that all class names match a specified casing style.
    pub mod class_name_casing_style;
    /// Enforces that all class names are in a specified order.
    pub mod class_order;
    /// Enforces that no element has the same class name more than once.
    pub mod duplicate_classes_disallowed;
    /// Enforces that the value of all ids match a specified casing style.
    pub mod id_casing_style;
    /// Enforces that all ids are unique within the document.
    pub mod id_unique;

    // Potential new rules:
    // /// Enforces that an element does not have any of the specified class names.
    // pub mod class_blacklist;
    // /// Enforces that all ids match a specified regular expression.
    // pub mod id_regexp;
    // /// Enforces that an element has one of the specified class names.
    // pub mod class_requirement;
    // /// Enforces that an element only has the specified class names.
    // pub mod class_whitelist;
    // /// Enforces that all class names match a specified regular expression.
    // pub mod class_name_regexp;
}

pub mod content {
    /// Enforces that an element does not have any text content.
    pub mod text_disallowed;
    /// Enforces that an elements text content matches a specified regular expression.
    pub mod text_regexp;
    /// Enforces that an element has text content.
    pub mod text_requirement;
}

//
pub mod structure {
    /// Enforces that an element does not have any of the specified ancestor elements at any level.
    pub mod ancestor_blacklist;
    /// Enforces that an element has one of the specified ancestor elements at some level.
    pub mod ancestor_requirement;
    //
    //    /// Enforces that an element does not have any of the specified direct child elements.
    //    pub mod child_blacklist;
    //    /// Enforces that an element has all of the specified direct child elements.
    pub mod child_requirement;
    /// Enforces that an element only has the specified direct child elements.
    //    pub mod child_whitelist;
    //
    /// Enforces that an element contains all of the specified descendant elements at some depth.
    pub mod descendant_requirement;
    //
    /// Enforces that the specified elements appear only once in the entire document.
    pub mod duplicate_elements_blacklist;
    //
    //    /// Enforces that an element has one of the specified direct parent elements.
    //    pub mod parent_whitelist;

    // Ideas for potential new rules:
    //"ancestor_whitelist", # Enforces that an element only has the specified ancestors elements at any level.
    //"descendant_blacklist" # Enforces that an element does not contain any of the specified descendant elements at any depth.
    //"descendant_whitelist" # Enforces that an element only contains the specified descendant elements at any depth.
    //"parent_blacklist" # Enforces that an element does not have any of the specified direct parent elements.

    pub mod maximum_nesting_depth;
}

pub mod tags {
    /// Enforces that all closing tags have a matching opening tag.
    pub mod missing_close_tag_disallowed;
    /// Enforces that all tags have their end brackets present.
    pub mod missing_end_bracket_disallowed;
    /// Enforces that all closing tags have a matching opening tag.
    pub mod missing_open_tag_disallowed;
    /// Enforces that self-closing tags are with a specified style.
    pub mod self_closing_tag_style;
    /// Enforces that the document does not use any of the specified tags.
    pub mod tag_name_blacklist;
    /// Enforces that the tag name matches a specified casing style.
    pub mod tag_name_casing;
    /// Enforces that the tag name matches a specified regular expression.
    pub mod tag_name_regexp;
    /// Enforces that the document only use the specified tags.
    pub mod tag_name_whitelist;
}

pub fn registry() -> Registry {
    Registry::new()
        // ---- Attributes -------------------------------------------------------------------------
        .register_rule::<attributes::attribute_name_missing::Rule>("attribute_name_missing")
        .register_rule::<attributes::attribute_name_regexp::Rule>("attribute_name_regexp")
        .register_rule::<attributes::attribute_name_requirement::Rule>("attribute_name_requirement")
        .register_rule::<attributes::attribute_name_blacklist::Rule>("attribute_name_blacklist")
        .register_rule::<attributes::attribute_value_whitelist::Rule>("attribute_value_whitelist")
        .register_rule::<attributes::attribute_value_casing_style::Rule>(
            "attribute_value_casing_style",
        )
        .register_rule::<attributes::attribute_value_regexp::Rule>("attribute_value_regexp")
        .register_rule::<attributes::attribute_value_whitelist::Rule>("attribute_value_whitelist")
        .register_rule::<attributes::attributes_order::Rule>("attributes_order")
        .register_rule::<attributes::duplicate_attribute_names_disallowed::Rule>(
            "duplicate_attribute_names_disallowed",
        )
        .register_rule::<attributes::attribute_name_whitelist::Rule>("attribute_name_whitelist")
        .register_rule::<attributes::boolean_attribute_style::Rule>("boolean_attribute_style")
        .register_rule::<attributes::attribute_value_quote_style::Rule>(
            "attribute_value_quote_style",
        )
        .register_rule::<attributes::attribute_name_casing_style::Rule>(
            "attribute_name_casing_style",
        )
        // ---- Class and Id -----------------------------------------------------------------------
        .register_rule::<class_and_id::class_name_casing_style::Rule>("class_name_casing_style")
        .register_rule::<class_and_id::duplicate_classes_disallowed::Rule>(
            "duplicate_classes_disallowed",
        )
        .register_rule::<class_and_id::id_casing_style::Rule>("id_casing_style")
        .register_rule::<class_and_id::class_order::Rule>("class_order")
        .register_rule::<class_and_id::id_unique::Rule>("id_unique")
        // ---- Content ----------------------------------------------------------------------------
        .register_rule::<content::text_disallowed::Rule>("text_disallowed")
        .register_rule::<content::text_requirement::Rule>("text_requirement")
        .register_rule::<content::text_regexp::Rule>("text_regexp")
        // ---- Structure --------------------------------------------------------------------------
        .register_rule::<structure::ancestor_blacklist::Rule>("ancestor_blacklist")
        .register_rule::<structure::duplicate_elements_blacklist::Rule>(
            "duplicate_elements_blacklist",
        )
        .register_rule::<structure::ancestor_requirement::Rule>("ancestor_requirement")
        .register_rule::<structure::child_requirement::Rule>("child_requirement")
        .register_rule::<structure::descendant_requirement::Rule>("descendant_requirement")
        .register_rule::<structure::maximum_nesting_depth::Rule>("maximum_nesting_depth")
        // ---- Tags  ------------------------------------------------------------------------------
        .register_rule::<tags::self_closing_tag_style::Rule>("self_closing_tag_style")
        .register_rule::<tags::tag_name_blacklist::Rule>("tag_name_blacklist")
        .register_rule::<tags::tag_name_casing::Rule>("tag_name_casing")
        .register_rule::<tags::tag_name_regexp::Rule>("tag_name_regexp")
        .register_rule::<tags::tag_name_whitelist::Rule>("tag_name_whitelist")
        .register_rule::<tags::missing_end_bracket_disallowed::Rule>(
            "missing_end_bracket_disallowed",
        )
        .register_rule::<tags::missing_close_tag_disallowed::Rule>("missing_close_tag_disallowed")
        .register_rule::<tags::missing_open_tag_disallowed::Rule>("missing_open_tag_disallowed")
}
