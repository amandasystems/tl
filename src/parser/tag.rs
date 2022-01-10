use crate::{
    inline::{hashmap::InlineHashMap, vec::InlineVec},
    Bytes,
};
use std::borrow::Cow;

use super::{handle::NodeHandle, Parser};

const INLINED_ATTRIBUTES: usize = 4;
const INLINED_SUBNODES: usize = 4;

/// The type of map for "raw" attributes
pub type RawAttributesMap<'a> = InlineHashMap<Bytes<'a>, Option<Bytes<'a>>, INLINED_ATTRIBUTES>;

/// Stores all attributes of an HTML tag, as well as additional metadata such as `id` and `class`
#[derive(Debug, Clone)]
pub struct Attributes<'a> {
    /// Raw attributes (maps attribute key to attribute value)
    pub(crate) raw: RawAttributesMap<'a>,
    /// The ID of this HTML element, if present
    pub(crate) id: Option<Bytes<'a>>,
    /// A list of class names of this HTML element, if present
    pub(crate) class: Option<Bytes<'a>>,
}

impl<'a> Attributes<'a> {
    /// Creates a new `Attributes
    pub(crate) fn new() -> Self {
        Self {
            raw: InlineHashMap::new(),
            id: None,
            class: None,
        }
    }

    /// Counts the number of attributes
    pub fn len(&self) -> usize {
        let mut raw = self.raw.len();
        if self.id.is_some() {
            raw += 1;
        }
        if self.class.is_some() {
            raw += 1;
        }
        raw
    }

    /// Checks whether this collection of attributes is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks whether a given string is in the class names list
    pub fn is_class_member<B: AsRef<[u8]>>(&self, member: B) -> bool {
        self.class_iter()
            .map_or(false, |mut i| i.any(|s| s.as_bytes() == member.as_ref()))
    }

    /// Checks whether this attributes collection contains a given key and returns its value
    ///
    /// Attributes that exist in this tag but have no value set will have their inner Option set to None
    pub fn get<B>(&self, key: B) -> Option<Option<Bytes<'a>>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.clone().map(Some),
            b"class" => self.class.clone().map(Some),
            _ => self.raw.get(&key).cloned(),
        }
    }

    /// Checks whether this attributes collection contains a given key and returns its value
    pub fn get_mut<B>(&mut self, key: B) -> Option<Option<&mut Bytes<'a>>>
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();

        match key.as_bytes() {
            b"id" => self.id.as_mut().map(Some),
            b"class" => self.class.as_mut().map(Some),
            _ => self.raw.get_mut(&key).map(Option::as_mut),
        }
    }

    /// Inserts a new attribute into this attributes collection
    pub fn insert<B>(&mut self, key: B, value: Option<B>)
    where
        B: Into<Bytes<'a>>,
    {
        let key: Bytes = key.into();
        let value = value.map(Into::into);

        match key.as_bytes() {
            b"id" => self.id = value,
            b"class" => self.class = value,
            _ => self.raw.insert(key, value),
        };
    }

    /// Returns an iterator `(attribute_key, attribute_value)` over the attributes of this `HTMLTag`
    pub fn iter(&self) -> impl Iterator<Item = (Cow<str>, Option<Cow<str>>)> + '_ {
        self.raw
            .iter()
            .map(|(k, v)| {
                let k = k.as_utf8_str();
                let v = v.as_ref().map(|x| x.as_utf8_str());

                (Some(k), v)
            })
            .chain([
                (
                    self.id.is_some().then(|| Cow::Borrowed("id")),
                    self.id.as_ref().map(|x| x.as_utf8_str()),
                ),
                (
                    self.class.is_some().then(|| Cow::Borrowed("class")),
                    self.class.as_ref().map(|x| x.as_utf8_str()),
                ),
            ])
            .flat_map(|(k, v)| k.map(|k| (k, v)))
    }

    /// Returns the `id` attribute of this HTML tag, if present
    pub fn id(&self) -> Option<&Bytes<'a>> {
        self.id.as_ref()
    }

    /// Returns the `class` attribute of this HTML tag, if present
    pub fn class(&self) -> Option<&Bytes<'a>> {
        self.id.as_ref()
    }

    /// Returns an iterator over all of the class members
    pub fn class_iter(&self) -> Option<impl Iterator<Item = &'_ str> + '_> {
        self.class
            .as_ref()
            .and_then(Bytes::try_as_utf8_str)
            .map(str::split_ascii_whitespace)
    }

    /// Returns the underlying raw map for attributes
    ///
    /// ## A note on stability
    /// It is not guaranteed for the returned map to include all attributes.
    /// Some attributes may be stored in `Attributes` itself and not in the raw map.
    /// For that reason you should prefer to call methods on `Attributes` directly,
    /// i.e. `Attributes::get()` to lookup an attribute by its key.
    pub fn unstable_raw(&self) -> &RawAttributesMap<'a> {
        &self.raw
    }
}

/// Represents a single HTML element
#[derive(Debug, Clone)]
pub struct HTMLTag<'a> {
    pub(crate) _name: Bytes<'a>,
    pub(crate) _attributes: Attributes<'a>,
    pub(crate) _children: InlineVec<NodeHandle, INLINED_SUBNODES>,
    pub(crate) _raw: Bytes<'a>,
}

impl<'a> HTMLTag<'a> {
    /// Creates a new HTMLTag
    #[inline(always)]
    pub(crate) fn new(
        name: Bytes<'a>,
        attr: Attributes<'a>,
        children: InlineVec<NodeHandle, INLINED_SUBNODES>,
        raw: Bytes<'a>,
    ) -> Self {
        Self {
            _name: name,
            _attributes: attr,
            _children: children,
            _raw: raw,
        }
    }

    /// Returns an iterator over subnodes ("children") of this HTML tag
    #[inline]
    pub fn children(&self) -> impl Iterator<Item = NodeHandle> + '_ {
        self._children.iter().copied()
    }

    /// Returns the name of this HTML tag
    #[inline]
    pub fn name(&self) -> Bytes<'a> {
        self._name.clone()
    }

    /// Returns a mutable reference to the name of this HTML tag
    #[inline]
    pub fn name_mut(&mut self) -> &mut Bytes<'a> {
        &mut self._name
    }

    /// Returns attributes of this HTML tag
    #[inline]
    pub fn attributes(&self) -> &Attributes<'a> {
        &self._attributes
    }

    /// Returns a mutable reference to the attributes of this HTML tag
    #[inline]
    pub fn attributes_mut(&mut self) -> &mut Attributes<'a> {
        &mut self._attributes
    }

    /// Returns the contained markup
    ///
    /// ## Limitations
    /// - The order of tag attributes is not guaranteed
    /// - Spaces within the tag are not guaranteed to be preserved (i.e. `<img      src="">` may become `<img src="">`)
    ///
    /// Equivalent to [Element#innerHTML](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerHTML) in browsers)
    pub fn inner_html<'p>(&'p self, parser: &'p Parser<'a>) -> String {
        let mut inner_html = format!("<{}", self._name.as_utf8_str());

        #[inline]
        fn write_attribute(dest: &mut String, k: Cow<str>, v: Option<Cow<str>>) {
            dest.push(' ');

            dest.push_str(&k);

            if let Some(value) = v {
                dest.push_str("=\"");
                dest.push_str(&value);
                dest.push('"');
            }
        }

        let attr = self.attributes();

        for (k, v) in attr.iter() {
            write_attribute(&mut inner_html, k, v);
        }

        inner_html.push('>');

        for handle in self.children() {
            let node = handle.get(parser).unwrap();
            inner_html.push_str(&node.inner_html(parser));
        }

        inner_html.push_str("</");
        inner_html.push_str(&self._name.as_utf8_str());
        inner_html.push('>');

        inner_html
    }

    /// Returns the raw HTML of this tag.
    /// This is a cheaper version of `HTMLTag::inner_html` if you never mutate any nodes.
    ///
    /// **Note:** Mutating this tag does *not* re-compute the HTML representation of this tag.
    /// This simply returns a reference to the substring.
    pub fn raw(&self) -> &Bytes<'a> {
        &self._raw
    }

    /// Returns the contained text of this element, excluding any markup
    /// Equivalent to [Element#innerText](https://developer.mozilla.org/en-US/docs/Web/API/Element/innerText) in browsers)
    /// This function may not allocate memory for a new string as it can just return the part of the tag that doesn't have markup
    /// For tags that *do* have more than one subnode, this will allocate memory
    pub fn inner_text<'p>(&self, parser: &'p Parser<'a>) -> Cow<'p, str> {
        let len = self._children.len();

        if len == 0 {
            // If there are no subnodes, we can just return a static, empty, string slice
            return Cow::Borrowed("");
        }

        let first = self._children[0].get(parser).unwrap();

        if len == 1 {
            match &first {
                Node::Tag(t) => return t.inner_text(parser),
                Node::Raw(e) => return e.as_utf8_str(),
                Node::Comment(_) => return Cow::Borrowed(""),
            }
        }

        // If there are >1 nodes, we need to allocate a new string and push each inner_text in it
        // TODO: check if String::with_capacity() is worth it
        let mut s = String::from(first.inner_text(parser));

        for &id in self._children.iter().skip(1) {
            let node = id.get(parser).unwrap();

            match &node {
                Node::Tag(t) => s.push_str(&t.inner_text(parser)),
                Node::Raw(e) => s.push_str(&e.as_utf8_str()),
                Node::Comment(_) => { /* no op */ }
            }
        }

        Cow::Owned(s)
    }

    /// Calls the given closure with each tag as parameter
    ///
    /// The closure must return a boolean, indicating whether it should stop iterating
    /// Returning `true` will break the loop
    pub fn find_node<F>(&self, parser: &Parser<'a>, f: &mut F) -> Option<NodeHandle>
    where
        F: FnMut(&Node<'a>) -> bool,
    {
        for &id in self._children.iter() {
            let node = id.get(parser).unwrap();

            if f(node) {
                return Some(id);
            }
        }
        None
    }
}

/// An HTML Node
#[derive(Debug, Clone)]
pub enum Node<'a> {
    /// A regular HTML element/tag
    Tag(HTMLTag<'a>),
    /// Raw text (no particular HTML element)
    Raw(Bytes<'a>),
    /// Comment (<!-- -->)
    Comment(Bytes<'a>),
}

impl<'a> Node<'a> {
    /// Returns the inner text of this node
    pub fn inner_text<'s, 'p: 's>(&'s self, parser: &'p Parser<'a>) -> Cow<'s, str> {
        match self {
            Node::Comment(_) => Cow::Borrowed(""),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => t.inner_text(parser),
        }
    }

    /// Returns the inner HTML of this node
    pub fn inner_html<'s>(&'s self, parser: &Parser<'a>) -> Cow<'s, str> {
        match self {
            Node::Comment(c) => c.as_utf8_str(),
            Node::Raw(r) => r.as_utf8_str(),
            Node::Tag(t) => Cow::Owned(t.inner_html(parser)),
        }
    }

    /// Returns an iterator over subnodes ("children") of this HTML tag, if this is a tag
    pub fn children(&self) -> Option<impl Iterator<Item = NodeHandle> + '_> {
        match self {
            Node::Tag(t) => Some(t.children()),
            _ => None,
        }
    }

    /// Calls the given closure with each tag as parameter
    ///
    /// The closure must return a boolean, indicating whether it should stop iterating
    /// Returning `true` will break the loop and return a handle to the node
    pub fn find_node<F>(&self, parser: &Parser<'a>, f: &mut F) -> Option<NodeHandle>
    where
        F: FnMut(&Node<'a>) -> bool,
    {
        if let Some(children) = self.children() {
            for id in children {
                let node = id.get(parser).unwrap();

                if f(node) {
                    return Some(id);
                }

                let subnode = node.find_node(parser, f);
                if subnode.is_some() {
                    return subnode;
                }
            }
        }
        None
    }

    /// Tries to coerce this node into a `HTMLTag` variant
    pub fn as_tag(&self) -> Option<&HTMLTag<'a>> {
        match self {
            Self::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    /// Tries to coerce this node into a `HTMLTag` variant
    pub fn as_tag_mut(&mut self) -> Option<&mut HTMLTag<'a>> {
        match self {
            Self::Tag(tag) => Some(tag),
            _ => None,
        }
    }

    /// Tries to coerce this node into a comment, returning the text
    pub fn as_comment(&self) -> Option<&Bytes<'a>> {
        match self {
            Self::Comment(c) => Some(c),
            _ => None,
        }
    }

    /// Tries to coerce this node into a comment, returning the text
    pub fn as_comment_mut(&mut self) -> Option<&mut Bytes<'a>> {
        match self {
            Self::Comment(c) => Some(c),
            _ => None,
        }
    }

    /// Tries to coerce this node into a raw text node, returning the text
    ///
    /// "Raw text nodes" are nodes that are not HTML tags, but just text
    pub fn as_raw(&self) -> Option<&Bytes<'a>> {
        match self {
            Self::Raw(r) => Some(r),
            _ => None,
        }
    }

    /// Tries to coerce this node into a mutable raw text node, returning the text
    ///
    /// "Raw text nodes" are nodes that are not HTML tags, but just text
    pub fn as_raw_mut(&mut self) -> Option<&mut Bytes<'a>> {
        match self {
            Self::Raw(r) => Some(r),
            _ => None,
        }
    }
}
