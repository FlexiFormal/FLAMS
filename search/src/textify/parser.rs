use std::cell::{Cell, RefCell};

#[allow(clippy::wildcard_imports)]
use super::ever::*;
use ftml_ontology::narrative::DocumentRange;
use html5ever::{
    QualName,
    interface::{NodeOrText, TreeSink},
    tendril::{SliceExt, StrTendril, TendrilSink},
};

enum State {
    Msup(String, Option<String>, Option<String>),
    Msub(String, Option<String>, Option<String>),
    Mfrac(String, Option<String>, Option<String>),
}

pub struct HtmlParser {
    pub(crate) document_node: NodeRef,
    pub(crate) body: Cell<(DocumentRange, usize)>,
    pub(crate) out: RefCell<String>,
    states: RefCell<Vec<State>>,
    in_body: Cell<bool>,
}

impl HtmlParser {
    pub fn run(s: &str, inline: bool) -> Result<String, String> {
        let parser = Self {
            document_node: super::ever::NodeRef::new_document(),
            body: Cell::new((DocumentRange::default(), 0)),
            out: RefCell::new(String::new()),
            states: RefCell::new(Vec::new()),
            in_body: Cell::new(inline),
        };
        html5ever::parse_document(parser, html5ever::ParseOpts::default())
            .from_utf8()
            .one(s.as_bytes().to_tendril())
    }

    fn newline(&self) {
        if let Some(State::Msup(out, _, _) | State::Msub(out, _, _) | State::Mfrac(out, _, _)) =
            self.states.borrow_mut().last_mut()
        {
            if !out.is_empty() {
                out.push('\n');
            }
            return;
        }
        let mut out = self.out.borrow_mut();
        if !out.is_empty() {
            out.push('\n');
        }
    }

    fn add(&self, s: &str) {
        if let Some(State::Msup(out, _, _) | State::Msub(out, _, _) | State::Mfrac(out, _, _)) =
            self.states.borrow_mut().last_mut()
        {
            if !out.is_empty() && !out.ends_with([' ', '\n']) {
                out.push(' ');
            }
            out.push_str(s);
            return;
        }
        let mut out = self.out.borrow_mut();
        if !out.is_empty() && !out.ends_with([' ', '\n']) {
            out.push(' ');
        }
        out.push_str(s);
    }

    fn pair(&self, sep: char, a: String, b: String) {
        if a.is_empty() && b.is_empty() {
            return;
        }
        if b.is_empty() {
            self.add(&a);
        }
        self.add(&a);
        self.add(&format!("{sep}{{{b}}}"))
    }
}

impl TreeSink for HtmlParser {
    type Handle = NodeRef;
    type Output = Result<String, String>;
    type ElemName<'a>
        = &'a QualName
    where
        Self: 'a;

    #[allow(clippy::cast_possible_truncation)]
    fn finish(self) -> Self::Output {
        for c in self.document_node.children() {
            self.pop(&c);
        }
        Ok(self.out.into_inner())
    }

    #[inline]
    fn parse_error(&self, _: std::borrow::Cow<'static, str>) {}

    #[inline]
    fn get_document(&self) -> Self::Handle {
        self.document_node.clone()
    }
    #[inline]
    fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
        let NodeData::Document(r) = self.document_node.data() else {
            unreachable!()
        };
        r.set(mode);
    }

    #[inline]
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    #[inline]
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        &target.as_element().unwrap_or_else(|| unreachable!()).name
    }

    #[inline]
    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        _: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        NodeRef::new_element(name, attrs.into())
    }
    #[inline]
    fn create_comment(&self, text: StrTendril) -> NodeRef {
        NodeRef::new_comment(text)
    }
    #[inline]
    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        NodeRef::new_processing_instruction(target, data)
    }

    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::too_many_lines)]
    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        if let Some(e) = parent.last_child() {
            self.pop(&e);
        }
        {
            if let Some(p) = parent.as_element() {
                let mut sts = self.states.borrow_mut();
                let last = sts.last_mut();
                if &*p.name.local == "msup"
                    && let Some(State::Msup(s, first, second)) = last
                {
                    if first.is_none() || second.is_none() {
                        *second = first.take();
                        *first = Some(std::mem::take(s));
                    }
                } else if &*p.name.local == "msub"
                    && let Some(State::Msub(s, first, second)) = last
                {
                    if first.is_none() || second.is_none() {
                        *second = first.take();
                        *first = Some(std::mem::take(s));
                    }
                } else if &*p.name.local == "mfrac"
                    && let Some(State::Mfrac(s, first, second)) = last
                {
                    if first.is_none() || second.is_none() {
                        *second = first.take();
                        *first = Some(std::mem::take(s));
                    }
                }
            }
        }
        match child {
            NodeOrText::AppendNode(child) => {
                if parent.as_document().is_some() {
                    if let Some(child_elem) = child.as_element() {
                        let new_start = parent.len();
                        let len = child.len();
                        child_elem.start_offset.set(new_start);
                        child_elem.end_offset.set(new_start + len);
                    }
                }
                if let Some(e) = child.as_element() {
                    let attrs = e.attributes.borrow();
                    for a in &attrs.0 {
                        if &*a.0.local == ftml_parser::FtmlKey::Definition.attr_name() {
                            self.newline();
                            self.add("DEFINITION: ");
                        } else if &*a.0.local == ftml_parser::FtmlKey::Assertion.attr_name() {
                            self.newline();
                            self.add("ASSERTION: ");
                        } else if &*a.0.local == ftml_parser::FtmlKey::Example.attr_name() {
                            self.newline();
                            self.add("EXAMPLE: ");
                        } else if &*a.0.local == ftml_parser::FtmlKey::Problem.attr_name()
                            || &*a.0.local == ftml_parser::FtmlKey::SubProblem.attr_name()
                        {
                            self.newline();
                            self.add("PROBLEM: ");
                        }
                    }
                    drop(attrs);
                    if &*e.name.local == "msup" {
                        self.states
                            .borrow_mut()
                            .push(State::Msup(String::new(), None, None));
                    } else if &*e.name.local == "msub" {
                        self.states
                            .borrow_mut()
                            .push(State::Msub(String::new(), None, None));
                    } else if &*e.name.local == "mfrac" {
                        self.states
                            .borrow_mut()
                            .push(State::Mfrac(String::new(), None, None));
                    } else if &*e.name.local == "tr" {
                        self.newline();
                    } else if &*e.name.local == "td" {
                        self.add("|");
                    } else if &*e.name.local == "body" {
                        self.in_body.set(true);
                    }
                }
                parent.append(child);
            }
            NodeOrText::AppendText(text) => {
                if let Some(elem) = parent.as_element() {
                    let len = if matches!(
                        &*elem.name.local,
                        "style"
                            | "script"
                            | "xmp"
                            | "iframe"
                            | "noembed"
                            | "noframes"
                            | "plaintext"
                            | "noscript"
                    ) {
                        text.as_bytes().len()
                    } else {
                        escaped_len(&text, false)
                    };
                    prolong(parent, len as isize);
                    if self.in_body.get() {
                        let txt = text.trim();
                        if !txt.is_empty() {
                            self.add(txt);
                        }
                    }
                }
                if let Some(last_child) = parent.last_child()
                    && let Some(existing) = last_child.as_text()
                {
                    existing.borrow_mut().extend(text.chars());
                    return;
                }
                parent.append(NodeRef::new_text(text));
            }
        }
    }

    #[inline]
    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        self.document_node
            .append(NodeRef::new_doctype(name, public_id, system_id));
    }

    #[inline]
    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if element.parent().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn pop(&self, node: &Self::Handle) {
        let Some(elem) = node.as_element() else {
            return;
        };
        if elem.closed.get() {
            return;
        }
        elem.closed.set(true);
        for c in node.children() {
            self.pop(&c);
        }
        let attrs = elem.attributes.borrow();
        if attrs.0.iter().any(|a| {
            [
                ftml_parser::FtmlKey::Definition.attr_name(),
                ftml_parser::FtmlKey::Assertion.attr_name(),
                ftml_parser::FtmlKey::Example.attr_name(),
                ftml_parser::FtmlKey::Problem.attr_name(),
                ftml_parser::FtmlKey::SubProblem.attr_name(),
            ]
            .contains(&&*a.0.local)
        }) {
            self.newline();
        }
        drop(attrs);

        let mut sts = self.states.borrow_mut();
        if &elem.name.local == "body" {
            let range = DocumentRange {
                start: elem.start_offset.get(),
                end: elem.end_offset.get(),
            };
            let off = elem.attributes.borrow().len();
            self.body.set((range, "<body>".len() + off));
            self.in_body.set(false);
        } else if &elem.name.local == "msup"
            && let Some(State::Msup(s, Some(a), _)) = sts.pop()
        {
            drop(sts);
            self.pair('^', a, s);
        } else if &elem.name.local == "msub"
            && let Some(State::Msub(s, Some(a), _)) = sts.pop()
        {
            drop(sts);
            self.pair('_', a, s);
        } else if &elem.name.local == "mfrac"
            && let Some(State::Mfrac(s, Some(a), _)) = sts.pop()
        {
            drop(sts);
            self.add(&format!("{{{a}}}/{{{s}}}"));
        }
    }

    #[inline]
    fn append_before_sibling(&self, _sibling: &Self::Handle, _child: NodeOrText<Self::Handle>) {
        unreachable!()
    }

    #[inline]
    fn remove_from_parent(&self, _target: &Self::Handle) {
        unreachable!()
    }
    #[inline]
    fn reparent_children(&self, _node: &Self::Handle, _new_parent: &Self::Handle) {
        unreachable!()
    }
    #[inline]
    fn mark_script_already_started(&self, _node: &Self::Handle) {
        unreachable!()
    }
    fn get_template_contents(&self, _target: &Self::Handle) -> Self::Handle {
        unreachable!()
    }
    #[inline]
    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        if let Some(e) = target.as_element() {
            let mut ats = e.attributes.borrow_mut();
            for a in attrs {
                if let Some(att) = ats.0.iter_mut().find(|att| att.0 == a.name) {
                    *att = (a.name, a.value);
                } else {
                    ats.0.push((a.name, a.value));
                }
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
fn prolong(parent: &NodeRef, len: isize) {
    if let Some(elem) = parent.as_element() {
        let end = elem.end_offset.get();
        elem.end_offset.set(((end as isize) + len) as usize);
        if let Some(p) = parent.parent() {
            prolong(&p, len);
        }
    }
}
