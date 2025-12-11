use html5ever::{
    LocalName, Namespace, QualName,
    interface::QuirksMode,
    serialize::{SerializeOpts, TraversalScope},
    tendril::StrTendril,
};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

/// A node inside a DOM-like tree.
pub struct Node {
    parent: Cell<Option<Weak<Node>>>,
    previous_sibling: Cell<Option<Weak<Node>>>,
    next_sibling: Cell<Option<Rc<Node>>>,
    first_child: Cell<Option<Rc<Node>>>,
    last_child: Cell<Option<Weak<Node>>>,
    data: NodeData,
}
impl std::fmt::Debug for Node {
    #[inline]
    #[allow(clippy::ref_as_ptr)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?} @ {:?}", self.data, self as *const Self)
    }
}

#[derive(Clone, Debug)]
pub struct NodeRef(pub Rc<Node>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attributes(pub Vec<(QualName, StrTendril)>);

/// Node data specific to the node type.
#[derive(Debug)]
pub enum NodeData {
    /// Element node
    Element(ElementData),

    /// Text node
    Text(RefCell<StrTendril>),

    /// Comment node
    Comment(StrTendril),

    /// Processing instruction node
    ProcessingInstruction(StrTendril, StrTendril),

    /// Doctype node
    Doctype {
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    },

    /// Document node
    Document(Cell<QuirksMode>),
}

/// Data specific to element nodes.
pub struct ElementData {
    /// The namespace and local name of the element, such as `ns!(html)` and `body`.
    pub name: QualName,
    /// The attributes of the elements.
    pub attributes: RefCell<Attributes>,
    pub start_offset: Cell<usize>,
    pub end_offset: Cell<usize>,
    pub closed: Cell<bool>,
}
impl std::fmt::Debug for ElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}[{:?}]", self.name, self.attributes)
    }
}

impl Attributes {
    pub fn len(&self) -> usize {
        self.0
            .iter()
            .map(|(k, v)| {
                k.prefix
                    .as_ref()
                    .map(|e| e.len() + ":".len())
                    .unwrap_or_default()
                    + k.local.len()
                    + " =\"\"".len()
                    + escaped_len(v, true)
            })
            .sum()
    }

    pub(crate) fn new_attr(&mut self, key: &str, value: String) {
        let name = QualName::new(None, Namespace::from(""), LocalName::from(key.to_string()));
        for (k, v) in &mut self.0 {
            if *k == name {
                *v = value.into();
                return;
            }
        }
        self.0.push((name, value.into()));
    }
}
impl From<Vec<html5ever::Attribute>> for Attributes {
    fn from(value: Vec<html5ever::Attribute>) -> Self {
        Self(
            value
                .into_iter()
                .map(|html5ever::Attribute { name, value }| (name, value))
                .collect(),
        )
    }
}

impl NodeRef {
    pub fn string(&self) -> Cow<'_, str> {
        let mut html = Vec::new();
        let _ = html5ever::serialize(
            &mut html,
            self,
            SerializeOpts {
                traversal_scope: TraversalScope::IncludeNode,
                ..Default::default()
            },
        );
        String::from_utf8_lossy(&html).into_owned().into() //from_utf8_lossy_owned(html)
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_wrap)]
    fn len_update(&self, len: isize) {
        if let Some(e) = self.as_element() {
            //assert!((e.end_offset.get() as isize + len) >= e.start_offset.get() as isize);
            e.end_offset
                .set(((e.end_offset.get() as isize) + len) as usize);
        }
        let mut cur = self.clone();
        while let Some(n) = cur.next_sibling() {
            if let Some(e) = n.as_element() {
                //assert!((e.end_offset.get() as isize + len) >= e.start_offset.get() as isize);
                e.start_offset
                    .set(((e.start_offset.get() as isize) + len) as usize);
                e.end_offset
                    .set(((e.end_offset.get() as isize) + len) as usize);
            }
            cur = n;
        }
        if let Some(p) = self.parent() {
            p.len_update(len);
        }
    }

    #[inline]
    pub fn children(&self) -> Siblings {
        match (self.first_child(), self.last_child()) {
            (Some(first_child), Some(last_child)) => Siblings(Some(State {
                next: first_child,
                next_back: last_child,
            })),
            (None, None) => Siblings(None),
            _ => unreachable!(),
        }
    }
    pub fn len(&self) -> usize {
        match &self.data {
            NodeData::Comment(_) => 0, // c.as_bytes().len() + "<!---->".len(),
            NodeData::Text(t) => t.borrow().as_bytes().len(),
            NodeData::Element(e) => e.end_offset.get() - e.start_offset.get(),
            NodeData::Doctype { name, .. } => "<!DOCTYPE >".len() + name.as_bytes().len(),
            NodeData::ProcessingInstruction(target, data) => {
                "<? >".len() + target.as_bytes().len() + data.as_bytes().len()
            }
            NodeData::Document(_) => self.children().map(|c| c.len()).sum(),
        }
    }

    /// Create a new node.
    #[inline]
    pub fn new(data: NodeData) -> Self {
        Self(Rc::new(Node {
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data,
        }))
    }

    pub fn update_len(e: &ElementData) {
        let len = Self::base_len(&e.name) + e.attributes.borrow().len();
        e.end_offset.set(e.start_offset.get() + len);
    }

    fn self_closing(name: &QualName) -> bool {
        use html5ever::{local_name, ns};
        name.ns == ns!(html)
            && matches!(
                name.local,
                local_name!("area")
                    | local_name!("base")
                    | local_name!("basefont")
                    | local_name!("bgsound")
                    | local_name!("br")
                    | local_name!("col")
                    | local_name!("embed")
                    | local_name!("frame")
                    | local_name!("hr")
                    | local_name!("img")
                    | local_name!("input")
                    | local_name!("keygen")
                    | local_name!("link")
                    | local_name!("meta")
                    | local_name!("param")
                    | local_name!("source")
                    | local_name!("track")
                    | local_name!("wbr")
            )
    }

    fn base_len(name: &QualName) -> usize {
        let tag_len = tag_len(name);
        if Self::self_closing(name) {
            tag_len
        } else {
            2 * tag_len + 1
        }
    }

    /// Create a new element node.
    #[inline]
    pub fn new_element(name: QualName, attributes: Attributes) -> Self {
        let attrs_len: usize = attributes.len();
        let base_len = Self::base_len(&name);
        Self::new(NodeData::Element(ElementData {
            name,
            attributes: RefCell::new(attributes),
            start_offset: Cell::new(0),
            end_offset: Cell::new(base_len + attrs_len),
            closed: Cell::new(false),
        }))
    }

    /// Create a new text node.
    #[inline]
    pub fn new_text(value: StrTendril) -> Self {
        Self::new(NodeData::Text(RefCell::new(value)))
    }

    /// Create a new comment node.
    #[inline]
    pub fn new_comment(value: StrTendril) -> Self {
        Self::new(NodeData::Comment(value))
    }

    /// Create a new processing instruction node.
    #[inline]
    pub fn new_processing_instruction(target: StrTendril, data: StrTendril) -> Self {
        Self::new(NodeData::ProcessingInstruction(target, data))
    }

    /// Create a new doctype node.
    #[inline]
    pub fn new_doctype(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Self {
        Self::new(NodeData::Doctype {
            name,
            public_id,
            system_id,
        })
    }

    /// Create a new document node.
    #[inline]
    pub fn new_document() -> Self {
        Self::new(NodeData::Document(Cell::new(QuirksMode::NoQuirks)))
    }

    /// Append a new child to this node, after existing children.
    ///
    /// The new child is detached from its previous position.
    pub fn append(&self, new_child: Self) {
        new_child.detach();
        new_child.parent.replace(Some(Rc::downgrade(&self.0)));
        if let Some(last_child_weak) = self.last_child.replace(Some(Rc::downgrade(&new_child.0)))
            && let Some(last_child) = last_child_weak.upgrade()
        {
            new_child.previous_sibling.replace(Some(last_child_weak));
            debug_assert!(last_child.next_sibling.is_none());
            last_child.next_sibling.replace(Some(new_child.0));
            return;
        }
        debug_assert!(self.first_child.is_none());
        self.first_child.replace(Some(new_child.0));
    }
}

impl Node {
    /// Return a reference to this node’s node-type-specific data.
    #[inline]
    pub const fn data(&self) -> &NodeData {
        &self.data
    }

    /// If this node is an element, return a reference to element-specific data.
    #[inline]
    pub const fn as_element(&self) -> Option<&ElementData> {
        match self.data {
            NodeData::Element(ref value) => Some(value),
            _ => None,
        }
    }

    /// If this node is a document, return a reference to element-specific data.
    #[inline]
    pub const fn as_document(&self) -> Option<&Cell<QuirksMode>> {
        match self.data {
            NodeData::Document(ref value) => Some(value),
            _ => None,
        }
    }

    /// If this node is a text node, return a reference to its contents.
    #[inline]
    pub const fn as_text(&self) -> Option<&RefCell<StrTendril>> {
        match self.data {
            NodeData::Text(ref value) => Some(value),
            _ => None,
        }
    }

    /// Return a reference to the parent node, unless this node is the root of the tree.
    #[inline]
    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.upgrade().map(NodeRef)
    }

    /// Return a reference to the first child of this node, unless it has no child.
    #[inline]
    pub fn first_child(&self) -> Option<NodeRef> {
        self.first_child.clone_inner().map(NodeRef)
    }

    /// Return a reference to the last child of this node, unless it has no child.
    #[inline]
    pub fn last_child(&self) -> Option<NodeRef> {
        self.last_child.upgrade().map(NodeRef)
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    #[inline]
    pub fn previous_sibling(&self) -> Option<NodeRef> {
        self.previous_sibling.upgrade().map(NodeRef)
    }

    /// Return a reference to the next sibling of this node, unless it is a last child.
    #[inline]
    pub fn next_sibling(&self) -> Option<NodeRef> {
        self.next_sibling.clone_inner().map(NodeRef)
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    ///
    /// To remove a node and its descendants, detach it and drop any strong reference to it.
    pub fn detach(&self) {
        let parent_weak = self.parent.take();
        let previous_sibling_weak = self.previous_sibling.take();
        let next_sibling_strong = self.next_sibling.take();

        let previous_sibling_opt = previous_sibling_weak.as_ref().and_then(Weak::upgrade);

        if let Some(next_sibling_ref) = next_sibling_strong.as_ref() {
            next_sibling_ref
                .previous_sibling
                .replace(previous_sibling_weak);
        } else if let Some(parent_ref) = parent_weak.as_ref()
            && let Some(parent_strong) = parent_ref.upgrade()
        {
            parent_strong.last_child.replace(previous_sibling_weak);
        }

        if let Some(previous_sibling_strong) = previous_sibling_opt {
            previous_sibling_strong
                .next_sibling
                .replace(next_sibling_strong);
        } else if let Some(parent_ref) = parent_weak.as_ref()
            && let Some(parent_strong) = parent_ref.upgrade()
        {
            parent_strong.first_child.replace(next_sibling_strong);
        }
    }
}

impl std::ops::Deref for NodeRef {
    type Target = Node;
    #[inline]
    fn deref(&self) -> &Node {
        &self.0
    }
}

impl Eq for NodeRef {}
impl PartialEq for NodeRef {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let a: *const Node = &raw const *self.0;
        let b: *const Node = &raw const *other.0;
        a == b
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        fn non_recursive_drop_unique_rc(mut rc: Rc<Node>, stack: &mut Vec<Rc<Node>>) {
            loop {
                if let Some(child) = rc.first_child.take_if_unique_strong() {
                    stack.push(rc);
                    rc = child;
                    continue;
                }
                if let Some(sibling) = rc.next_sibling.take_if_unique_strong() {
                    // The previous value of `rc: Rc<Node>` is dropped here.
                    // Since it was unique, the corresponding `Node` is dropped as well.
                    // `<Node as Drop>::drop` does not call `drop_rc`
                    // as both the first child and next sibling were already taken.
                    // Weak reference counts decremented here for `Cell`s that are `Some`:
                    // * `rc.parent`: still has a strong reference in `stack` or elsewhere
                    // * `rc.last_child`: this is the last weak ref. Deallocated now.
                    // * `rc.previous_sibling`: this is the last weak ref. Deallocated now.
                    rc = sibling;
                    continue;
                }
                if let Some(parent) = stack.pop() {
                    // Same as in the above comment.
                    rc = parent;
                    continue;
                }
                return;
            }
        }
        // `.take_if_unique_strong()` temporarily leaves the tree in an inconsistent state,
        // as the corresponding `Weak` reference in the other direction is not removed.
        // It is important that all `Some(_)` strong references it returns
        // are dropped by the end of this `drop` call,
        // and that no user code is invoked in-between.

        // Sharing `stack` between these two calls is not necessary,
        // but it allows re-using memory allocations.
        let mut stack = Vec::new();
        if let Some(rc) = self.first_child.take_if_unique_strong() {
            non_recursive_drop_unique_rc(rc, &mut stack);
        }
        if let Some(rc) = self.next_sibling.take_if_unique_strong() {
            non_recursive_drop_unique_rc(rc, &mut stack);
        }
    }
}

#[inline]
pub fn tag_len(name: &QualName) -> usize {
    name.prefix
        .as_ref()
        .map(|e| e.len() + 1 /* ':' */)
        .unwrap_or_default()
        + name.local.len()
        + "<>".len()
}

pub fn escaped_len(str: &str, attr_mode: bool) -> usize {
    str.chars()
        .map(|b| match b {
            '&' => "&amp;".len(),
            '\u{00A0}' => "&nbsp;".len(),
            '"' if attr_mode => "&quot;".len(),
            '<' if !attr_mode => "&lt;".len(),
            '>' if !attr_mode => "&gt;".len(),
            c => c.len_utf8(),
        })
        .sum()
}

#[derive(Debug, Clone)]
struct State<T> {
    next: T,
    next_back: T,
}
/// A double-ended iterator of sibling nodes.
#[derive(Debug, Clone)]
pub struct Siblings(Option<State<NodeRef>>);

macro_rules! siblings_next {
    ($next: ident, $next_back: ident, $next_sibling: ident) => {
        fn $next(&mut self) -> Option<NodeRef> {
            #![allow(non_shorthand_field_patterns)]
            self.0.take().map(
                |State {
                     $next: next,
                     $next_back: next_back,
                 }| {
                    if let Some(sibling) = next.$next_sibling() {
                        if next != next_back {
                            self.0 = Some(State {
                                $next: sibling,
                                $next_back: next_back,
                            })
                        }
                    }
                    next
                },
            )
        }
    };
}

impl Iterator for Siblings {
    type Item = NodeRef;
    siblings_next!(next, next_back, next_sibling);
}

impl DoubleEndedIterator for Siblings {
    siblings_next!(next_back, next, previous_sibling);
}

impl html5ever::serialize::Serialize for NodeRef {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope,
    ) -> std::io::Result<()>
    where
        S: html5ever::serialize::Serializer,
    {
        match (traversal_scope, self.data()) {
            (ref scope, NodeData::Element(element)) => {
                if *scope == TraversalScope::IncludeNode {
                    let attrs = element.attributes.borrow();

                    serializer.start_elem(
                        element.name.clone(),
                        attrs.0.iter().map(|(name, value)| (name, &**value)),
                    )?;
                }
                let children = self.children();

                for child in children {
                    html5ever::serialize::Serialize::serialize(
                        &child,
                        serializer,
                        TraversalScope::IncludeNode,
                    )?;
                }

                if *scope == TraversalScope::IncludeNode {
                    serializer.end_elem(element.name.clone())?;
                }
                Ok(())
            }

            (_, &NodeData::Document(_)) => {
                for child in self.children() {
                    html5ever::serialize::Serialize::serialize(
                        &child,
                        serializer,
                        TraversalScope::IncludeNode,
                    )?;
                }
                Ok(())
            }

            (TraversalScope::ChildrenOnly(_), _) => Ok(()),

            (TraversalScope::IncludeNode, NodeData::Doctype { name, .. }) => {
                serializer.write_doctype(name)
            }
            (TraversalScope::IncludeNode, NodeData::Text(text)) => {
                serializer.write_text(&text.borrow())
            }
            (TraversalScope::IncludeNode, NodeData::Comment(_text)) => Ok(()), //serializer.write_comment(text),
            (TraversalScope::IncludeNode, NodeData::ProcessingInstruction(target, data)) => {
                serializer.write_processing_instruction(target, data)
            }
        }
    }
}

trait CellOption {
    fn is_none(&self) -> bool;
}

impl<T> CellOption for Cell<Option<T>> {
    #[inline]
    fn is_none(&self) -> bool {
        unsafe { (*self.as_ptr()).is_none() }
    }
}

trait CellOptionWeak<T> {
    fn upgrade(&self) -> Option<Rc<T>>;
}

impl<T> CellOptionWeak<T> for Cell<Option<Weak<T>>> {
    #[inline]
    fn upgrade(&self) -> Option<Rc<T>> {
        unsafe { (*self.as_ptr()).as_ref().and_then(Weak::upgrade) }
    }
}

trait CellOptionRc<T> {
    /// Return `Some` if this `Rc` is the only strong reference count,
    /// even if there are weak references.
    fn take_if_unique_strong(&self) -> Option<Rc<T>>;
    fn clone_inner(&self) -> Option<Rc<T>>;
}

impl<T> CellOptionRc<T> for Cell<Option<Rc<T>>> {
    #[inline]
    fn take_if_unique_strong(&self) -> Option<Rc<T>> {
        unsafe {
            match *self.as_ptr() {
                None => None,
                Some(ref rc) if Rc::strong_count(rc) > 1 => None,
                // Not borrowing the `Rc<T>` here
                // as we would be invalidating that borrow while it is outstanding:
                Some(_) => self.take(),
            }
        }
    }

    #[inline]
    fn clone_inner(&self) -> Option<Rc<T>> {
        unsafe { (*self.as_ptr()).clone() }
    }
}
